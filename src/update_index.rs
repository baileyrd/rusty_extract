//! Update-index fetch/diff logic: ports the plaintext remote-index parsing
//! and local-vs-remote comparison that decide whether a program update is
//! available, from `_UpdateGetIndex`, `_UpdateGetSize`, `_UpdateFileCompare`,
//! and `CheckUpdateHelpers` (UniExtract.au3:5448-5477,5580-5637).
//!
//! This capability covers only the parsing and decision logic. Fetching the
//! index over the network (`_INetGetSource`) and the actual file download
//! loop (`_UpdateHelpers`) are the deferred network-updater capability
//! (D003) — real I/O the caller performs, then hands the results (a
//! response body, a file size, a hash) to the pure functions here.

/// A single row of the update index: `path,size,md5` — the wire format is
/// undocumented in the source but reconstructed from `_UpdateGetIndex`'s
/// parsing (`StringSplit($return, @LF, 2)` then, per row,
/// `StringSplit($aReturn[$i], ",", 2)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub path: String,
    pub size: i64,
    pub md5: String,
}

/// Ports `_UpdateGetIndex`'s two-stage split of the raw index response:
/// rows are LF-separated, columns comma-separated. Neither AutoIt's
/// `StringSplit` calls trim whitespace or a trailing `\r`, so this doesn't
/// either — a server that sends CRLF-terminated lines leaves a `\r` on the
/// last column of every row, exactly as it would in the original.
pub fn split_index_response(raw: &str) -> Vec<Vec<String>> {
    raw.split('\n')
        .map(|line| line.split(',').map(str::to_string).collect())
        .collect()
}

/// Builds an [`IndexEntry`] from one already-split row. Returns `None` when
/// the row doesn't have the three expected columns, or its size column
/// isn't a valid integer — `_UpdateGetIndex` has no such guard (it only
/// checks `@error` from the `StringSplit` calls themselves, not the
/// resulting column count), so a malformed row there would go on to a
/// crashing out-of-bounds array access at `$a[1]`/`$a[2]` in `_UpdateGetSize`
/// or `_UpdateFileCompare`. This port fails closed instead.
pub fn parse_index_entry(row: &[String]) -> Option<IndexEntry> {
    let [path, size, md5] = row else { return None };
    Some(IndexEntry {
        path: path.clone(),
        size: size.parse().ok()?,
        md5: md5.clone(),
    })
}

/// Parses a full index response in one call: splits into rows/columns and
/// discards malformed rows (see [`parse_index_entry`]).
pub fn parse_index(raw: &str) -> Vec<IndexEntry> {
    split_index_response(raw)
        .iter()
        .filter_map(|row| parse_index_entry(row))
        .collect()
}

/// Ports `StringRight($a[0], 1) = "/"`: a trailing `/` marks an index row as
/// a subdirectory rather than a file.
pub fn is_directory_entry(path: &str) -> bool {
    path.ends_with('/')
}

/// Which set of plugin-binary exclusions applies to a `_UpdateGetSize` call,
/// based on which bin-subdirectory `sPath` names (UniExtract.au3:5606-5616).
/// AutoIt's `=` here is its default case-insensitive comparison, not `==`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinDirRole {
    /// `bindir\x86\` or `bindir\x64\` — exclude only that arch's `ffmpeg.exe`.
    ArchSubdir,
    /// `bindir` itself — exclude the full hardcoded plugin-binary list.
    Root,
    /// Any other directory — no exclusions apply.
    Other,
}

pub fn resolve_bindir_role(path: &str, bindir: &str) -> BinDirRole {
    let x86 = format!("{bindir}x86\\");
    let x64 = format!("{bindir}x64\\");
    if path.eq_ignore_ascii_case(&x86) || path.eq_ignore_ascii_case(&x64) {
        BinDirRole::ArchSubdir
    } else if path.eq_ignore_ascii_case(bindir) {
        BinDirRole::Root
    } else {
        BinDirRole::Other
    }
}

/// The hardcoded plugin-binary exclusion list from `_UpdateGetSize`
/// (UniExtract.au3:5609-5611), relative to `bindir`. **Verified quirk,
/// preserved rather than "fixed"**: this list must be updated by hand
/// whenever a new plugin binary is added elsewhere in the codebase, or a
/// plugin's size silently starts counting toward the update-size
/// comparison — there is no mechanism that keeps it in sync automatically.
pub const ROOT_EXCLUDED_PLUGIN_FILES: &[&str] = &[
    "x86\\ffmpeg.exe",
    "x64\\ffmpeg.exe",
    "arc_conv.exe",
    "Extractor.exe",
    "iscab.exe",
    "ISTools.dll",
    "umodel.exe",
    "SDL2.dll",
    "dcp_unpacker.exe",
    "ci-extractor.exe",
    "gea.dll",
    "gentee.dll",
    "dgcac.exe",
    "bootimg.exe",
    "I5comp.exe",
    "ZD50149.DLL",
    "ZD51145.DLL",
];

/// The exclusion filenames (relative to the directory being sized) for a
/// given [`BinDirRole`].
pub fn excluded_files_for_role(role: BinDirRole) -> &'static [&'static str] {
    match role {
        BinDirRole::ArchSubdir => &["ffmpeg.exe"],
        BinDirRole::Root => ROOT_EXCLUDED_PLUGIN_FILES,
        BinDirRole::Other => &[],
    }
}

/// Ports `_UpdateGetSize`'s plugin-exclusion subtraction
/// (UniExtract.au3:5605-5616). `file_size` is the caller's real
/// `FileGetSize` equivalent for one excluded filename.
///
/// **Verified quirk, preserved rather than "fixed"**: AutoIt's
/// `FileGetSize` returns `-1` for a file that doesn't exist, and the source
/// subtracts that return value directly (`$iSize -= FileGetSize(...)`)
/// with no existence check first. So when an excluded plugin binary is
/// *missing* locally, the subtraction of `-1` actually *adds* one byte to
/// the effective directory size instead of leaving it unchanged — a real,
/// easy-to-miss inversion this port preserves by using signed sizes and a
/// caller-supplied lookup with the same `-1`-for-missing contract, rather
/// than defensively treating a missing file as size `0`.
pub fn apply_size_exclusions(
    raw_size: i64,
    role: BinDirRole,
    file_size: impl Fn(&str) -> i64,
) -> i64 {
    excluded_files_for_role(role)
        .iter()
        .fold(raw_size, |acc, name| acc - file_size(name))
}

/// Ports `_UpdateFileCompare` (UniExtract.au3:5621-5637): decides whether a
/// local file/directory differs from its index entry. **Verified quirk,
/// preserved rather than "fixed"**: a directory is compared by size only —
/// its hash is never computed or checked, so two directories of identical
/// total size but different contents are silently reported as up to date.
/// A file's hash is only computed (by the caller, lazily) when its size
/// already matches — an optimization, not a correctness issue, reflected
/// here by `local_md5` being an `Option` the caller need not populate when
/// sizes already differ.
pub fn decide_file_needs_update(
    is_directory: bool,
    local_size: i64,
    index_size: i64,
    local_md5: Option<&str>,
    index_md5: &str,
) -> bool {
    if is_directory {
        return local_size != index_size;
    }
    if local_size != index_size {
        return true;
    }
    match local_md5 {
        Some(hash) => hash != index_md5,
        None => true,
    }
}

/// The three-way branch `CheckUpdateHelpers` takes for one index entry once
/// `_UpdateFileCompare` has run (UniExtract.au3:5460-5471).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperCheckStep {
    /// The entry matches the index; move on to the next one.
    Skip,
    /// A file differs, or a directory entry is missing locally —
    /// `CheckUpdateHelpers` returns `True` immediately in either case.
    UpdateFound,
    /// A directory entry differs but exists locally: fetch its own index
    /// (`_UpdateGetIndex`) and append the expanded rows for further
    /// comparison, rather than reporting an update directly.
    RecurseIntoDirectory,
}

/// Ports the decision tree at UniExtract.au3:5460-5469.
pub fn decide_helper_check_step(
    entry_is_directory: bool,
    needs_update: bool,
    local_dir_exists: bool,
) -> HelperCheckStep {
    if !needs_update {
        HelperCheckStep::Skip
    } else if !entry_is_directory || !local_dir_exists {
        HelperCheckStep::UpdateFound
    } else {
        HelperCheckStep::RecurseIntoDirectory
    }
}

/// Ports `CheckUpdateHelpers`'s progress-bar math
/// (UniExtract.au3:5455): `($i / _Max($iSize, 200)) * 100`. **Verified
/// quirk, preserved rather than "fixed"**: the denominator is floored at
/// 200, so with fewer than 200 total index entries the progress bar never
/// visually reaches 100% — cosmetic, not a correctness bug.
pub fn update_progress_percent(i: usize, total: usize) -> f64 {
    (i as f64 / total.max(200) as f64) * 100.0
}

/// Ports the self-file skip check repeated in both `CheckUpdateHelpers` and
/// `_UpdateHelpers` (`If $sPath == @ScriptFullPath Then ContinueLoop`) —
/// AutoIt's `==` here is case-sensitive, unlike most other path comparisons
/// in this file.
pub fn is_self_path(candidate: &str, script_full_path: &str) -> bool {
    candidate == script_full_path
}

#[cfg(test)]
mod tests {
    use super::{
        apply_size_exclusions, decide_file_needs_update, decide_helper_check_step,
        excluded_files_for_role, is_directory_entry, is_self_path, parse_index, parse_index_entry,
        resolve_bindir_role, split_index_response, update_progress_percent, BinDirRole,
        HelperCheckStep, IndexEntry,
    };

    #[test]
    fn splits_rows_by_lf_and_columns_by_comma() {
        let raw = "foo.txt,123,abc123\nbar/,456,def456";
        let rows = split_index_response(raw);
        assert_eq!(
            rows,
            vec![
                vec![
                    "foo.txt".to_string(),
                    "123".to_string(),
                    "abc123".to_string()
                ],
                vec!["bar/".to_string(), "456".to_string(), "def456".to_string()],
            ]
        );
    }

    /// A CRLF-terminated server response leaves a trailing `\r` on the last
    /// column of every row, exactly as AutoIt's `@LF`-only split would.
    #[test]
    fn crlf_terminated_rows_retain_trailing_carriage_return() {
        let raw = "foo.txt,123,abc123\r\nbar.txt,456,def456\r\n";
        let rows = split_index_response(raw);
        assert_eq!(rows[0][2], "abc123\r");
        assert_eq!(rows[1][2], "def456\r");
        // The final empty line after the trailing LF becomes its own row.
        assert_eq!(rows[2], vec![""]);
    }

    #[test]
    fn parses_well_formed_row_into_index_entry() {
        let row = vec![
            "sub/file.txt".to_string(),
            "789".to_string(),
            "hash".to_string(),
        ];
        assert_eq!(
            parse_index_entry(&row),
            Some(IndexEntry {
                path: "sub/file.txt".to_string(),
                size: 789,
                md5: "hash".to_string(),
            })
        );
    }

    /// A malformed row (wrong column count, or a non-numeric size) is
    /// dropped rather than causing a crashing out-of-bounds access, the way
    /// the untouched AutoIt source would eventually hit downstream.
    #[test]
    fn malformed_rows_are_rejected() {
        assert_eq!(parse_index_entry(&["only_one_column".to_string()]), None);
        assert_eq!(
            parse_index_entry(&[
                "a.txt".to_string(),
                "not_a_number".to_string(),
                "hash".to_string()
            ]),
            None
        );
    }

    #[test]
    fn parse_index_discards_malformed_rows_and_keeps_valid_ones() {
        let raw = "good.txt,10,aaa\nbroken\nalso_good/,20,bbb";
        let entries = parse_index(raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "good.txt");
        assert_eq!(entries[1].path, "also_good/");
    }

    #[test]
    fn directory_entries_are_identified_by_trailing_slash() {
        assert!(is_directory_entry("sub/"));
        assert!(!is_directory_entry("file.txt"));
    }

    #[test]
    fn bindir_role_matches_case_insensitively() {
        let bindir = r"C:\Program Files\UniExtract\";
        assert_eq!(
            resolve_bindir_role(r"C:\Program Files\UniExtract\x86\", bindir),
            BinDirRole::ArchSubdir
        );
        assert_eq!(
            resolve_bindir_role(r"C:\PROGRAM FILES\UNIEXTRACT\X64\", bindir),
            BinDirRole::ArchSubdir
        );
        assert_eq!(resolve_bindir_role(bindir, bindir), BinDirRole::Root);
        assert_eq!(
            resolve_bindir_role(r"C:\Program Files\UniExtract\Lang\", bindir),
            BinDirRole::Other
        );
    }

    #[test]
    fn root_exclusion_list_has_seventeen_entries() {
        assert_eq!(excluded_files_for_role(BinDirRole::Root).len(), 17);
        assert_eq!(
            excluded_files_for_role(BinDirRole::ArchSubdir),
            ["ffmpeg.exe"]
        );
        assert!(excluded_files_for_role(BinDirRole::Other).is_empty());
    }

    #[test]
    fn size_exclusions_subtract_each_excluded_files_size() {
        let sizes = [("x86\\ffmpeg.exe", 100i64), ("x64\\ffmpeg.exe", 200)];
        let lookup = |name: &str| {
            sizes
                .iter()
                .find(|(n, _)| *n == name)
                .map_or(0, |(_, s)| *s)
        };
        // Root scope subtracts every listed file it can find a size for;
        // unlisted files contribute 0.
        assert_eq!(
            apply_size_exclusions(10_000, BinDirRole::Root, lookup),
            10_000 - 300
        );
        assert_eq!(apply_size_exclusions(500, BinDirRole::Other, lookup), 500);
    }

    /// The verified quirk: a missing excluded file (lookup returns `-1`,
    /// mirroring AutoIt's `FileGetSize` on a nonexistent path) makes the
    /// effective size *larger* by one byte, not smaller.
    #[test]
    fn missing_excluded_file_inflates_size_by_one() {
        let missing = |_: &str| -1i64;
        assert_eq!(
            apply_size_exclusions(1000, BinDirRole::ArchSubdir, missing),
            1001
        );
    }

    #[test]
    fn file_compare_checks_hash_only_when_sizes_match() {
        assert!(decide_file_needs_update(
            false,
            100,
            200,
            None,
            "irrelevant"
        ));
        assert!(!decide_file_needs_update(
            false,
            100,
            100,
            Some("abc"),
            "abc"
        ));
        assert!(decide_file_needs_update(
            false,
            100,
            100,
            Some("abc"),
            "xyz"
        ));
        // Sizes match but no hash was computed: treated as needing update.
        assert!(decide_file_needs_update(false, 100, 100, None, "abc"));
    }

    /// The verified quirk: directories are compared by size alone, so a
    /// same-size, different-content directory is reported as up to date.
    #[test]
    fn directory_compare_ignores_hash_entirely() {
        assert!(!decide_file_needs_update(true, 500, 500, None, "unused"));
        assert!(decide_file_needs_update(true, 500, 600, None, "unused"));
    }

    #[test]
    fn helper_check_step_skips_when_no_update_needed() {
        assert_eq!(
            decide_helper_check_step(false, false, true),
            HelperCheckStep::Skip
        );
        assert_eq!(
            decide_helper_check_step(true, false, true),
            HelperCheckStep::Skip
        );
    }

    #[test]
    fn helper_check_step_reports_update_for_a_differing_file() {
        assert_eq!(
            decide_helper_check_step(false, true, true),
            HelperCheckStep::UpdateFound
        );
    }

    #[test]
    fn helper_check_step_reports_update_for_a_missing_directory() {
        assert_eq!(
            decide_helper_check_step(true, true, false),
            HelperCheckStep::UpdateFound
        );
    }

    #[test]
    fn helper_check_step_recurses_into_an_existing_differing_directory() {
        assert_eq!(
            decide_helper_check_step(true, true, true),
            HelperCheckStep::RecurseIntoDirectory
        );
    }

    #[test]
    fn progress_percent_floors_denominator_at_two_hundred() {
        // With only 10 total entries, the source still divides by 200 —
        // it never reaches 100% until `i` itself reaches 200.
        assert!((update_progress_percent(10, 10) - 5.0).abs() < f64::EPSILON);
        // With 400 total entries, the real total is used.
        assert!((update_progress_percent(200, 400) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn self_path_check_is_case_sensitive() {
        assert!(is_self_path(r"C:\App\app.exe", r"C:\App\app.exe"));
        assert!(!is_self_path(r"C:\App\APP.EXE", r"C:\App\app.exe"));
    }
}
