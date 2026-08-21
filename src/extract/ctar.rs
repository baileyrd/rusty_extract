//! CTAR (`$TYPE_CTAR`): decompress with 7-Zip, then probe every
//! newly-created file for nested archives and extract those too — a
//! same-tool "unwrap the payload" loop, not `extract()` recursion.
//!
//! ```autoit
//! Case $TYPE_CTAR
//!     $oldfiles = ReturnFiles($outdir)
//!
//!     ; Decompress archive with 7-zip
//!     _Run($7z & ' x "' & $file & '"', $outdir)
//!
//!     ; Check for new files
//!     Local $aFiles = _FileListToArray($outdir, "*", $FLTA_FILES)
//!     If @error Then Local $aFiles[1]
//!
//!     For $i = 1 To $aFiles[0]
//!         Local $fname = $aFiles[$i]
//!         If StringInStr($oldfiles, $fname) Then ContinueLoop
//!
//!         ; Check for supported archive format
//!         Local $return = FetchStdout($7z & ' l "' & $outdir & '\' & $fname & '"', $outdir, @SW_HIDE)
//!         If Not StringInStr($return, "Listing archive:", 0) Then ContinueLoop
//!
//!         _Run($7z & ' x "' & $outdir & '\' & $fname & '"', $outdir, @SW_HIDE)
//!         FileDelete($outdir & '\' & $fname)
//!     Next
//! ```
//!
//! **Not `extract()` recursion.** Unlike C054's six call sites (see
//! `extract::completion`), this loop never calls `extract()` again — it
//! re-invokes `7z` directly on each newly-discovered file, the same
//! tool, not a dispatched type. `extract::completion` doesn't apply
//! here at all; this is its own, simpler probe-then-classify shape,
//! matching `detection::sevenzip_probe`'s.
//!
//! **Preserved quirk — the old-files check is a raw substring match,
//! not an exact-name comparison.** `ReturnFiles` returns a
//! pipe-delimited string (e.g. `"a.txt|b.zip"`), and
//! `StringInStr($oldfiles, $fname)` (bare, case-insensitive) checks
//! whether `$fname` appears *anywhere* in that string — not whether
//! it's one of the delimited tokens. A newly-extracted file whose name
//! happens to be a substring of an old file's name (e.g. old file
//! `notes.txt.bak`, new file `notes.txt`) is incorrectly treated as
//! "already existed" and skipped. [`is_newly_created`] reproduces this
//! exactly.
//!
//! **Scope — invocations and classification only.** `ReturnFiles`,
//! `_FileListToArray`, and `FileDelete` are real filesystem I/O, left
//! to the caller.

use super::{Invocation, WindowMode};

/// Builds the initial decompression invocation (UniExtract.au3:2481):
/// `<7z> x "<file>"`, run in `outdir`. No `$show_flag` argument is
/// passed, so `_Run`'s own default (`@SW_MINIMIZE`) applies — the same
/// convention `extract::raiu`/`extract::helpdeco` already document for
/// their own bare `_Run($cmd, $dir)` calls.
pub fn initial_extract_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Ports the "is this a file we should even look at" gate
/// (UniExtract.au3:2488): `Not StringInStr($oldfiles, $fname)`, bare
/// (case-insensitive). See the module doc comment's preserved-quirk
/// note — `oldfiles` is a raw pipe-delimited string, and this is a
/// substring search, not an exact-token match.
pub fn is_newly_created(oldfiles: &str, fname: &str) -> bool {
    !oldfiles.to_lowercase().contains(&fname.to_lowercase())
}

/// Builds the listing-probe invocation (UniExtract.au3:2491): `<7z> l
/// "<outdir>\<fname>"`, run in `outdir` with the window hidden.
pub fn listing_probe_invocation(program: &str, outdir: &str, fname: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["l".to_string(), format!("{outdir}\\{fname}")],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports the classification of the listing probe's captured output
/// (UniExtract.au3:2492): `StringInStr($return, "Listing archive:", 0)`
/// — case-insensitive: AutoIt's `casesense` parameter treats `0` the
/// same as omitting it (`0` is the documented default), so this is no
/// different from every other bare `StringInStr` call in this port —
/// whether 7-Zip recognized the file as a listable archive at all.
pub fn is_nested_archive(listing_output: &str) -> bool {
    listing_output.to_lowercase().contains("listing archive:")
}

/// Builds the nested-archive extraction invocation (UniExtract.au3:2494):
/// `<7z> x "<outdir>\<fname>"`, run in `outdir` with the window hidden.
/// The source deletes `<outdir>\<fname>` afterward (`FileDelete`) — real
/// filesystem I/O, out of scope here.
pub fn nested_extract_invocation(program: &str, outdir: &str, fname: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), format!("{outdir}\\{fname}")],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_extract_invocation_matches_source() {
        let inv = initial_extract_invocation(
            r"C:\bin\7z.exe",
            r"C:\downloads\game.ctar",
            r"C:\downloads\unpacked",
        );
        assert_eq!(inv.program, r"C:\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\downloads\game.ctar".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C181: a genuinely new filename, absent
    /// from the old-files snapshot, is recognized as new.
    #[test]
    fn is_newly_created_true_for_absent_filename() {
        assert!(is_newly_created("a.txt|b.zip", "c.dat"));
    }

    /// Parity test for capability C181: a filename present in the
    /// snapshot is not new.
    #[test]
    fn is_newly_created_false_for_present_filename() {
        assert!(!is_newly_created("a.txt|b.zip", "b.zip"));
    }

    /// Parity test for capability C181: the preserved substring-match
    /// quirk — a new file whose name is a substring of an old file's
    /// name is incorrectly treated as not-new.
    #[test]
    fn is_newly_created_false_when_name_is_substring_of_old_entry() {
        assert!(!is_newly_created("notes.txt.bak", "notes.txt"));
    }

    #[test]
    fn is_newly_created_is_case_insensitive() {
        assert!(!is_newly_created("A.TXT|B.ZIP", "a.txt"));
    }

    #[test]
    fn listing_probe_invocation_matches_source() {
        let inv = listing_probe_invocation(r"C:\bin\7z.exe", r"C:\downloads\unpacked", "data1.bin");
        assert_eq!(
            inv.args,
            vec![
                "l".to_string(),
                r"C:\downloads\unpacked\data1.bin".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C181: 7-Zip's listing output is
    /// classified by the literal "Listing archive:", matched
    /// case-insensitively — AutoIt's explicit `0` casesense argument is
    /// the same as its default.
    #[test]
    fn is_nested_archive_matches_marker_case_insensitively() {
        assert!(is_nested_archive(
            "7-Zip [64] ...\nListing archive: data1.bin\n\n..."
        ));
        assert!(is_nested_archive("listing archive: data1.bin"));
        assert!(!is_nested_archive("Error: cannot open file as archive"));
    }

    #[test]
    fn nested_extract_invocation_matches_source() {
        let inv =
            nested_extract_invocation(r"C:\bin\7z.exe", r"C:\downloads\unpacked", "data1.bin");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\downloads\unpacked\data1.bin".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
