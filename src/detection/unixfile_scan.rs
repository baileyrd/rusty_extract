//! Unix `file` tool as secondary/second-opinion detector
//! (`FileScan_UnixFile`, UniExtract.au3:1033-1051), run automatically
//! right after TrID (UniExtract.au3:938) — capability C040.
//!
//! ```autoit
//! Func FileScan_UnixFile()
//!     Local $sFileType = FetchStdout($filetool & ' "' & $file & '"', $filedir, @SW_HIDE)
//!     $sFileType = StringReplace(StringReplace($sFileType, $file & ": ", ""), @CRLF, "")
//!
//!     If $sFileType And $sFileType <> "data" Then _FiletypeAdd("Unix File Tool", $sFileType)
//!
//!     If Not $extract Then
//!         ; Text files are often misdetected, renaming them is not a good idea
//!         If $appendext And (StringInStr($sFileType, "text", 0) Or StringInStr($sFileType, "ASCII", 0)) Then $appendext = False
//!         Return
//!     EndIf
//!
//!     filecompare($sFileType)
//! EndFunc
//! ```
//!
//! **Unconditional after TrID**: `FileScan_Trid` (C038) calls
//! `FileScan_UnixFile()` directly at line 938, regardless of whether
//! TrID itself found anything — the "run automatically after TrID" half
//! of this capability's description is that unconditional call, not a
//! decision this module makes.
//!
//! **What's out of scope**: `FetchStdout($filetool & ' "' & $file &
//! '"', ...)` is real process I/O, the same missing-FFI/external-process
//! boundary already documented elsewhere in this port. [`clean_output`]
//! takes the raw stdout text as a caller-supplied string instead.
//!
//! **In extract mode, this hands off entirely to `filecompare`** —
//! already ported as [`detection::file_dispatch::classify`] (C041), not
//! duplicated here. [`Outcome::Dispatch`] just signals that a caller
//! should call it with the cleaned output.
//!
//! **`$sFileType <> "data"` only gates whether the result is logged**
//! (`_FiletypeAdd`, informational bookkeeping) — it doesn't change
//! control flow, so it isn't modeled as a decision here.
//!
//! `StringInStr(..., "text", 0)`/`StringInStr(..., "ASCII", 0)` are
//! case-insensitive (explicit `0`, the documented default) — matching
//! the same rule already verified for C039/C041/C043.

/// Ports the `StringReplace`/`StringReplace` cleanup
/// (UniExtract.au3:1038): strips a leading `"<file>: "` prefix (the
/// `file` tool's own filename echo) and all `\r\n` sequences from the
/// raw stdout text.
pub fn clean_output(raw_stdout: &str, file_path: &str) -> String {
    raw_stdout
        .replace(&format!("{file_path}: "), "")
        .replace("\r\n", "")
}

/// What `FileScan_UnixFile` does with the cleaned output once scanning
/// is done (UniExtract.au3:1044-1051).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Scan-only mode (`$extract` is `false`): no dispatch happens.
    /// `disable_appendext` mirrors the source clearing `$appendext`
    /// when the text looks like a text file — renaming a possibly
    /// misdetected text file is deliberately avoided.
    ScanOnly { disable_appendext: bool },
    /// Extract mode: hand off to `filecompare` —
    /// [`detection::file_dispatch::classify`] (C041), called with this
    /// same cleaned text.
    Dispatch,
}

/// Ports `FileScan_UnixFile`'s post-scan branch (UniExtract.au3:1044-
/// 1051). `cleaned_file_type` is [`clean_output`]'s result;
/// `appendext_enabled` is the caller's current `$appendext` state,
/// consulted only in scan-only mode.
pub fn scan_outcome(cleaned_file_type: &str, extract: bool, appendext_enabled: bool) -> Outcome {
    if extract {
        return Outcome::Dispatch;
    }
    let looks_like_text = {
        let s = cleaned_file_type.to_lowercase();
        s.contains("text") || s.contains("ascii")
    };
    Outcome::ScanOnly {
        disable_appendext: appendext_enabled && looks_like_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_output_strips_filename_prefix_and_crlf() {
        assert_eq!(
            clean_output("C:\\archive.zip: Zip archive data\r\n", "C:\\archive.zip"),
            "Zip archive data"
        );
    }

    #[test]
    fn clean_output_handles_multiple_crlf_occurrences() {
        assert_eq!(
            clean_output("file.bin: data\r\nmore\r\n", "file.bin"),
            "datamore"
        );
    }

    #[test]
    fn extract_mode_always_dispatches_regardless_of_content() {
        assert_eq!(
            scan_outcome("Zip archive data", true, true),
            Outcome::Dispatch
        );
        assert_eq!(scan_outcome("data", true, false), Outcome::Dispatch);
    }

    /// Parity test for capability C040: in scan-only mode, a text-like
    /// result disables `$appendext` -- but only if it was already
    /// enabled.
    #[test]
    fn scan_only_mode_disables_appendext_for_text_like_results() {
        assert_eq!(
            scan_outcome("ASCII text, with CRLF", false, true),
            Outcome::ScanOnly {
                disable_appendext: true
            }
        );
        assert_eq!(
            scan_outcome("some text document", false, true),
            Outcome::ScanOnly {
                disable_appendext: true
            }
        );
        assert_eq!(
            scan_outcome("ASCII text, with CRLF", false, false),
            Outcome::ScanOnly {
                disable_appendext: false
            }
        );
    }

    #[test]
    fn scan_only_mode_leaves_appendext_alone_for_non_text_results() {
        assert_eq!(
            scan_outcome("Zip archive data", false, true),
            Outcome::ScanOnly {
                disable_appendext: false
            }
        );
    }

    #[test]
    fn text_detection_is_case_insensitive() {
        assert_eq!(
            scan_outcome("ascii TEXT file", false, true),
            Outcome::ScanOnly {
                disable_appendext: true
            }
        );
    }
}
