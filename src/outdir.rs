//! Output-directory resolution: ports `ValidateOutputDirectory()`
//! (UniExtract.au3:526-544), `GetLastOutdir()`
//! (UniExtract.au3:872-878), `CreateOutdir()`
//! (UniExtract.au3:3968-3978), the empty-outdir cleanup check inside
//! `terminate()` (UniExtract.au3:4224), and `$initoutdir`'s computation
//! inside `FilenameParse()` (UniExtract.au3:500-518).

use crate::status::Status;

/// C138: ports `$initoutdir`'s computation inside `FilenameParse()`
/// (UniExtract.au3:500-518) — the default `/sub` destination (C004).
///
/// `stem` is `$filename` with its final `.`-delimited extension already
/// trimmed off when `has_extension` is `true` (e.g. `"archive.tar"` for
/// `"archive.tar.gz"`, since only the *last* extension is stripped), or
/// the whole filename unchanged when `has_extension` is `false`.
///
/// With an extension, the result is `filedir\<stem>` — *unless* `stem`
/// itself still contains a `.` (a multi-extension name, e.g.
/// `"archive.tar"`) **and** a plain file (not a directory) already
/// exists at that exact path (`initoutdir_collision`, standing in for
/// that filesystem check), in which case the source falls back to
/// `filedir\<stem with every '.' replaced by '_'>`. This collision check
/// is narrowly scoped to multi-extension names — a single-extension stem
/// (no embedded `.`) never triggers it, matching the source exactly.
///
/// Without an extension, the result is `filedir\<stem>_<unpacked_suffix>`
/// — the source's `t('TERM_UNPACKED')` translated term; this port's
/// localization is out of scope, so the caller supplies the literal
/// suffix text.
pub fn default_output_subfolder(
    filedir: &str,
    stem: &str,
    has_extension: bool,
    initoutdir_collision: bool,
    unpacked_suffix: &str,
) -> String {
    if has_extension {
        if stem.contains('.') && initoutdir_collision {
            format!("{filedir}\\{}", stem.replace('.', "_"))
        } else {
            format!("{filedir}\\{stem}")
        }
    } else {
        format!("{filedir}\\{stem}_{unpacked_suffix}")
    }
}

/// C005: ports `GetLastOutdir()` (UniExtract.au3:872-878) — the most
/// recently used output directory is the `"Directory History"` section's
/// ini key `"0"` (the newest slot, in `prefs::push_history`'s convention,
/// C021). `None` here represents the source's failure path: no history
/// yet, which shows a `MsgBox` (out of scope, deferred GUI subsystem)
/// and calls `terminate($STATUS_SILENT)` (exit 0, C016) — the source
/// never returns a directory at all in that case, so a caller reaching
/// [`resolve_output_directory`]'s `/last` branch must already have
/// handled this `None` case the same way (i.e. it can't happen in a
/// faithful driver of this function).
pub fn get_last_outdir(history_dir_slot0: Option<&str>) -> Option<String> {
    history_dir_slot0.map(str::to_string)
}

fn is_drive_absolute(path: &str) -> bool {
    path.as_bytes().get(1) == Some(&b':')
}

/// C004, C139, C140: ports `ValidateOutputDirectory()`
/// (UniExtract.au3:526-544).
///
/// - **`/sub` (C004):** resolves to `initoutdir` (a subdirectory named
///   after the archive, computed by the caller — UniExtract.au3:645).
/// - **`/last` (C005):** resolves to `last_outdir`, the already-resolved
///   result of [`get_last_outdir`] — see that function's doc comment for
///   why this parameter is a plain `Option<&str>` rather than something
///   this function computes itself.
/// - **Drive-absolute (`X:...`) or UNC (`\\...`) paths:** returned
///   unchanged.
/// - **A single leading backslash, not a UNC double backslash (C139):**
///   inherits the input file's drive letter (`filedir`'s first two
///   characters) rather than being treated as relative.
/// - **Anything else (C139):** resolved against `filedir` by
///   concatenation, mirroring `$filedir & '\' & $outdir`.
/// - **Trailing slash (C140):** a trailing `/` is stripped first, then a
///   trailing `\` is unconditionally appended if not already present —
///   applied regardless of which branch above produced the path.
///
/// **Not modeled — `_PathFull`'s own segment normalization.** The
/// relative-path branch mirrors the source's exact string concatenation
/// but doesn't reproduce whatever `.`/`..`-collapsing AutoIt's
/// `_PathFull(path)` (the single-argument form the source calls here,
/// UniExtract.au3:535) performs internally — that UDF isn't defined
/// anywhere in this port's source checkout, the same gap already noted
/// for the two-argument `_PathFull` behind `prefs::resolve_batchqueue_path`
/// (C018) and `prefs::resolve_filescanlogfile_path` (C019).
pub fn resolve_output_directory(
    outdir: &str,
    initoutdir: &str,
    filedir: &str,
    last_outdir: Option<&str>,
) -> String {
    let mut resolved = if outdir.eq_ignore_ascii_case("/sub") {
        initoutdir.to_string()
    } else if outdir.eq_ignore_ascii_case("/last") {
        last_outdir.unwrap_or_default().to_string()
    } else if is_drive_absolute(outdir) {
        outdir.to_string()
    } else if outdir.starts_with('\\') && !outdir.starts_with(r"\\") {
        let drive_letter = &filedir[..filedir.len().min(2)];
        format!("{drive_letter}{outdir}")
    } else if !outdir.starts_with(r"\\") {
        format!("{filedir}\\{outdir}")
    } else {
        outdir.to_string()
    };

    if resolved.ends_with('/') {
        resolved.pop();
    }
    if !resolved.ends_with('\\') {
        resolved.push('\\');
    }
    resolved
}

/// C140 (continued): ports the trailing-backslash strip at the start of
/// `extract()` (UniExtract.au3:2278: `If StringRight($outdir, 1) = "\"
/// Then $outdir = StringTrimRight($outdir, 1)`).
/// [`resolve_output_directory`] always leaves a trailing backslash, but
/// `extract()` strips it again immediately — so every extraction routine
/// that runs in between operates on an outdir with *no* trailing slash.
/// This inconsistency is documented in the source's own `todo.txt` (line
/// 35) as a known rough edge, preserved here rather than "fixed" into a
/// single normalized representation used throughout.
///
/// **C141 — drive-root consequence:** this is a plain string operation
/// with no drive-root special case, so a drive-root outdir like `C:\`
/// strips down to `C:` — which Windows treats as "current directory on
/// that drive" (a drive-relative reference), not the drive's root. That
/// ambiguity is exactly what produces `todo.txt`'s documented
/// "Extracting to C:/ creates file in @ScriptDir" bug: a spawned
/// extractor given `C:` as its working directory resolves relative
/// output paths against whatever the process's per-drive current
/// directory happens to be (often `@ScriptDir`, if the launching process
/// last set it there), not `C:\`. Preserved here rather than special-cased
/// away, matching the "known quirk, verify still present" framing of
/// C141's own manifest description.
pub fn strip_trailing_backslash_for_extraction(outdir: &str) -> String {
    outdir.strip_suffix('\\').unwrap_or(outdir).to_string()
}

/// C140 (continued): ports the trailing-backslash re-append at the end
/// of `extract()` (UniExtract.au3:3413: `$outdir &= "\"`) — restores the
/// trailing slash [`strip_trailing_backslash_for_extraction`] removed,
/// unconditionally (the source's `&=` doesn't check whether one is
/// already present, so calling this on an outdir that already ends in
/// `\` produces a doubled backslash — not reachable in the source's own
/// control flow, since every caller reaches this point via the stripped
/// value, but reproduced here rather than silently guarded against).
pub fn reappend_trailing_backslash_after_extraction(outdir: &str) -> String {
    format!("{outdir}\\")
}

/// C142: the result of `CreateOutdir()`'s decision tree
/// (UniExtract.au3:3968-3978) — an already-existing outdir that's a
/// writable directory needs no action; anything else either creates the
/// directory or terminates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutdirOutcome {
    /// `$outdir` already existed as a writable directory — nothing to do.
    AlreadyValid,
    /// `$outdir` didn't exist and `DirCreate` succeeded — the source sets
    /// `$createdir = True` here (UniExtract.au3:3976), tracked so a later
    /// cleanup step can remove it if the extraction that follows fails
    /// and leaves it empty (capability C157, not ported by this
    /// function).
    Created,
    /// `terminate($STATUS_INVALIDDIR, $outdir, "")` (UniExtract.au3:3970):
    /// `$outdir` exists but isn't a directory.
    ExistsButNotADirectory,
    /// `terminate($STATUS_INVALIDDIR, $outdir)` (UniExtract.au3:3973):
    /// `$outdir` exists, is a directory, but isn't writable
    /// (`CanAccess` failed).
    ExistsButNotAccessible,
    /// `terminate($STATUS_INVALIDDIR, $outdir)` (UniExtract.au3:3975):
    /// `$outdir` didn't exist and `DirCreate` failed.
    CreateFailed,
}

impl OutdirOutcome {
    /// Whether this outcome corresponds to one of `CreateOutdir()`'s three
    /// `terminate($STATUS_INVALIDDIR, ...)` calls — exit code 5, per
    /// `status::exit_code(status::Status::InvalidDir)` (C016).
    pub fn is_fatal(self) -> bool {
        !matches!(self, OutdirOutcome::AlreadyValid | OutdirOutcome::Created)
    }
}

/// C142: ports `CreateOutdir()` (UniExtract.au3:3968-3978) as a pure
/// decision over already-known filesystem facts — the actual
/// `FileExists`/`_IsDirectory`/`CanAccess`/`DirCreate` calls are the
/// caller's job; this function only reproduces the branching UniExtract2
/// does once those facts are known. `dir_create_succeeded` is ignored
/// when `exists` is `true` (the source never calls `DirCreate` in that
/// case).
pub fn decide_outdir_outcome(
    exists: bool,
    is_directory: bool,
    can_access: bool,
    dir_create_succeeded: bool,
) -> OutdirOutcome {
    if exists {
        if !is_directory {
            OutdirOutcome::ExistsButNotADirectory
        } else if !can_access {
            OutdirOutcome::ExistsButNotAccessible
        } else {
            OutdirOutcome::AlreadyValid
        }
    } else if dir_create_succeeded {
        OutdirOutcome::Created
    } else {
        OutdirOutcome::CreateFailed
    }
}

/// C157: ports the empty-output-directory cleanup check inside
/// `terminate()` (UniExtract.au3:4224: `If $createdir And $status <>
/// $STATUS_SUCCESS And DirGetSize($outdir) = 0 Then DirRemove($outdir,
/// 1)`). `created_dir` is whether *this run* created the directory
/// (C142's `OutdirOutcome::Created`, not an outdir that already
/// existed) — only a directory this run itself brought into being gets
/// cleaned up. A failed run whose output directory is non-empty is left
/// alone; only a still-empty one (nothing was ever written, or
/// everything written was itself removed) qualifies.
pub fn should_remove_empty_created_outdir(
    created_dir: bool,
    status: Status,
    dir_is_empty: bool,
) -> bool {
    created_dir && status != Status::Success && dir_is_empty
}

#[cfg(test)]
mod tests {
    use super::{
        decide_outdir_outcome, default_output_subfolder, get_last_outdir,
        reappend_trailing_backslash_after_extraction, resolve_output_directory,
        should_remove_empty_created_outdir, strip_trailing_backslash_for_extraction, OutdirOutcome,
    };
    use crate::status::Status;

    /// Parity test for capability C005: a present history slot 0 is
    /// returned as the resolved directory; a missing one maps to `None`,
    /// standing in for the source's `terminate($STATUS_SILENT)` path.
    #[test]
    fn get_last_outdir_matches_source() {
        assert_eq!(
            get_last_outdir(Some(r"C:\downloads\unpacked")),
            Some(r"C:\downloads\unpacked".to_string())
        );
        assert_eq!(get_last_outdir(None), None);
    }

    /// Parity test for capability C004: `/sub` resolves to `initoutdir`.
    #[test]
    fn sub_token_resolves_to_initoutdir() {
        assert_eq!(
            resolve_output_directory("/sub", r"C:\downloads\archive", r"C:\downloads", None),
            r"C:\downloads\archive\"
        );
        // Case-insensitive, matching AutoIt's default `=` comparison.
        assert_eq!(
            resolve_output_directory("/SUB", r"C:\downloads\archive", r"C:\downloads", None),
            r"C:\downloads\archive\"
        );
    }

    /// Parity test for capability C005 (as consumed by
    /// `resolve_output_directory`): `/last` resolves to the pre-resolved
    /// `last_outdir`.
    #[test]
    fn last_token_resolves_to_last_outdir() {
        assert_eq!(
            resolve_output_directory(
                "/last",
                r"C:\downloads\archive",
                r"C:\downloads",
                Some(r"D:\history\unpacked")
            ),
            r"D:\history\unpacked\"
        );
    }

    /// Parity test for capability C139: a drive-absolute or UNC outdir
    /// passes through unchanged (aside from trailing-slash handling).
    #[test]
    fn drive_absolute_and_unc_paths_pass_through() {
        assert_eq!(
            resolve_output_directory(r"D:\custom", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"\\server\share\out", "", r"C:\downloads", None),
            r"\\server\share\out\"
        );
    }

    /// Parity test for capability C139: a single leading backslash
    /// inherits the input file's drive letter rather than being resolved
    /// relative to `filedir`.
    #[test]
    fn single_leading_backslash_inherits_drive_letter() {
        assert_eq!(
            resolve_output_directory(r"\output", "", r"D:\downloads\sub", None),
            r"D:\output\"
        );
    }

    /// Parity test for capability C139: a plain relative outdir resolves
    /// against `filedir` by concatenation.
    #[test]
    fn relative_path_resolves_against_filedir() {
        assert_eq!(
            resolve_output_directory("unpacked", "", r"C:\downloads", None),
            r"C:\downloads\unpacked\"
        );
    }

    /// Parity test for capability C140: a trailing `/` is stripped, then
    /// a trailing `\` is always appended.
    #[test]
    fn trailing_slash_normalized_to_backslash() {
        assert_eq!(
            resolve_output_directory(r"D:\custom/", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"D:\custom\", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"D:\custom", "", r"C:\downloads", None),
            r"D:\custom\"
        );
    }

    /// Parity test for capability C140 (continued): `extract()` strips
    /// `ValidateOutputDirectory`'s trailing backslash immediately
    /// (UniExtract.au3:2278) — an outdir with no trailing backslash is
    /// left unchanged.
    #[test]
    fn strip_trailing_backslash_matches_extract_start() {
        assert_eq!(
            strip_trailing_backslash_for_extraction(r"C:\downloads\unpacked\"),
            r"C:\downloads\unpacked"
        );
        assert_eq!(
            strip_trailing_backslash_for_extraction(r"C:\downloads\unpacked"),
            r"C:\downloads\unpacked"
        );
    }

    /// Parity test for capability C140 (continued): `extract()`
    /// re-appends the trailing backslash only at the very end
    /// (UniExtract.au3:3413), unconditionally.
    #[test]
    fn reappend_trailing_backslash_matches_extract_end() {
        assert_eq!(
            reappend_trailing_backslash_after_extraction(r"C:\downloads\unpacked"),
            r"C:\downloads\unpacked\"
        );
    }

    /// Parity test for capability C141: stripping the trailing backslash
    /// from a drive-root outdir (`C:\`) produces the ambiguous
    /// drive-relative reference `C:`, not the drive root — the string-level
    /// cause of `todo.txt`'s documented "Extracting to C:/ creates file in
    /// @ScriptDir" bug. `strip_trailing_backslash_for_extraction` has no
    /// drive-root special case, so it reproduces this exactly.
    #[test]
    fn strip_trailing_backslash_reproduces_drive_root_ambiguity() {
        assert_eq!(strip_trailing_backslash_for_extraction(r"C:\"), "C:");
        // A non-root drive path is unaffected: only the drive-root case
        // collapses to the ambiguous two-character form.
        assert_eq!(
            strip_trailing_backslash_for_extraction(r"C:\downloads\"),
            r"C:\downloads"
        );
    }

    /// Parity test for capability C142: an already-existing, writable
    /// directory needs no action.
    #[test]
    fn existing_writable_directory_is_already_valid() {
        assert_eq!(
            decide_outdir_outcome(true, true, true, false),
            OutdirOutcome::AlreadyValid
        );
        assert!(!OutdirOutcome::AlreadyValid.is_fatal());
    }

    /// Parity test for capability C142: a missing outdir that `DirCreate`
    /// successfully creates is `Created`, not fatal.
    #[test]
    fn missing_directory_created_successfully() {
        assert_eq!(
            decide_outdir_outcome(false, false, false, true),
            OutdirOutcome::Created
        );
        assert!(!OutdirOutcome::Created.is_fatal());
    }

    /// Parity test for capability C142: the three
    /// `terminate($STATUS_INVALIDDIR, ...)` cases (UniExtract.au3:3970,
    /// 3973, 3975) are each distinguished and all fatal.
    #[test]
    fn invalid_directory_cases_are_all_fatal() {
        assert_eq!(
            decide_outdir_outcome(true, false, true, false),
            OutdirOutcome::ExistsButNotADirectory
        );
        assert_eq!(
            decide_outdir_outcome(true, true, false, false),
            OutdirOutcome::ExistsButNotAccessible
        );
        assert_eq!(
            decide_outdir_outcome(false, false, false, false),
            OutdirOutcome::CreateFailed
        );
        for outcome in [
            OutdirOutcome::ExistsButNotADirectory,
            OutdirOutcome::ExistsButNotAccessible,
            OutdirOutcome::CreateFailed,
        ] {
            assert!(outcome.is_fatal());
        }
    }

    /// Parity test for capability C157: a directory this run created,
    /// still empty, on a failed run gets removed.
    #[test]
    fn empty_created_outdir_removed_on_failure() {
        assert!(should_remove_empty_created_outdir(
            true,
            Status::Failed,
            true
        ));
    }

    /// Parity test for capability C157: a non-empty failed output
    /// directory is left in place, even if this run created it.
    #[test]
    fn nonempty_created_outdir_not_removed_on_failure() {
        assert!(!should_remove_empty_created_outdir(
            true,
            Status::Failed,
            false
        ));
    }

    /// Parity test for capability C157: an outdir that already existed
    /// before this run (not created by it) is never removed, empty or
    /// not.
    #[test]
    fn preexisting_outdir_never_removed() {
        assert!(!should_remove_empty_created_outdir(
            false,
            Status::Failed,
            true
        ));
    }

    /// Parity test for capability C157: a successful run never removes
    /// the output directory, even if this run created it and it's empty.
    #[test]
    fn successful_run_never_removes_outdir() {
        assert!(!should_remove_empty_created_outdir(
            true,
            Status::Success,
            true
        ));
    }

    /// Parity test for capability C138: a single-extension file (no
    /// embedded dot in the stem) resolves to a same-name subfolder,
    /// regardless of collision — the collision check never triggers for
    /// a single-extension stem.
    #[test]
    fn default_output_subfolder_single_extension() {
        assert_eq!(
            default_output_subfolder(r"C:\downloads", "archive", true, false, "unpacked"),
            r"C:\downloads\archive"
        );
        assert_eq!(
            default_output_subfolder(r"C:\downloads", "archive", true, true, "unpacked"),
            r"C:\downloads\archive"
        );
    }

    /// Parity test for capability C138: a multi-extension stem (e.g.
    /// "archive.tar" from "archive.tar.gz") resolves to a same-name
    /// subfolder when there's no collision.
    #[test]
    fn default_output_subfolder_multi_extension_no_collision() {
        assert_eq!(
            default_output_subfolder(r"C:\downloads", "archive.tar", true, false, "unpacked"),
            r"C:\downloads\archive.tar"
        );
    }

    /// Parity test for capability C138: a multi-extension stem that
    /// collides with an existing file falls back to an underscore-replaced
    /// name.
    #[test]
    fn default_output_subfolder_multi_extension_collision_falls_back() {
        assert_eq!(
            default_output_subfolder(r"C:\downloads", "archive.tar", true, true, "unpacked"),
            r"C:\downloads\archive_tar"
        );
    }

    /// Parity test for capability C138: an extensionless input file gets
    /// the `_unpacked`-style suffix.
    #[test]
    fn default_output_subfolder_no_extension_gets_suffix() {
        assert_eq!(
            default_output_subfolder(r"C:\downloads", "archive", false, false, "unpacked"),
            r"C:\downloads\archive_unpacked"
        );
    }
}
