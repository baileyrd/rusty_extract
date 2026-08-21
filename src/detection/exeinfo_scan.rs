//! Exeinfo PE detection, run before TrID for `.exe`/`.dll` files
//! (`FileScan_ExeInfo`, UniExtract.au3:1096-1281) — the scan-orchestration
//! half of capability C042. The dispatch table this scan feeds into is
//! already ported separately as [`detection::exeinfo_dispatch::classify`]
//! (C043, UniExtract.au3:1141-1278) — this module covers everything
//! *around* it: building the scan invocation, the corrupted/too-big/
//! not-an-exe early exits, and the scan-only-mode short-circuit.
//!
//! ```autoit
//! Func FileScan_ExeInfo($bUseCmd = $extract)
//!     Local $sFileType = ""
//!     If $bUseCmd Then ; Use log command line for best speed
//!         Local Const $LogFile = $logdir & "exeinfo.log"
//!         RunWait($exeinfope & ' "' & $file & '*" /sx /log:"' & $LogFile & '"', $bindir, @SW_HIDE)
//!         $sFileType = _FileRead($LogFile, True)
//!         If StringInStr($sFileType, "File corrupted or Buffer Error") Or StringIsSpace($sFileType) Then Return FileScan_ExeInfo(False)
//!     Else ; In scan only mode run and read GUI fields to get additional information on how to extract
//!         $aReturn = OpenExeInfo()
//!         ; ... ControlGetText(...) polling loop ...
//!         CloseExeInfo($aReturn)
//!     EndIf
//!
//!     If StringInStr($sFileType, $filenamefull) Then $sFileType = StringTrimLeft(StringStripWS(StringReplace($sFileType, $filenamefull, ""), 1), 2)
//!
//!     ; Return if file is too big
//!     If StringInStr($sFileType, "Skipped") Then Return
//!
//!     ; Do not display 'unknown file type' scan result in scan only mode
//!     If Not $extract And StringInStr($sFileType, "file is not EXE or DLL") Then Return
//!
//!     _FiletypeAdd("Exeinfo PE", $sFileType)
//!
//!     ; Return filetype without matching if specified
//!     If Not $extract Then Return $sFileType
//!
//!     Select ; ... already ported as detection::exeinfo_dispatch::classify (C043) ...
//!     EndSelect
//! EndFunc
//! ```
//!
//! **A real, non-obvious finding**: `$bUseCmd` defaults to `$extract`,
//! so in *extract mode* this scan is a plain command-line invocation
//! (`RunWait` + reading a log file) — **not** GUI automation. Only the
//! scan-only-mode path (`Else`), and the corrupted/buffer-error retry
//! (`Return FileScan_ExeInfo(False)`), drive PEiD-style GUI automation
//! (`OpenExeInfo`/`ControlGetText`/`CloseExeInfo`) — the same deferred
//! GUI subsystem blocker as elsewhere in this port (manifest row D001).
//! [`scan_invocation`] covers the portable command-line path only; the
//! GUI path and the retry-on-corruption fallback into it are the reason
//! manifest row C042 stays `REQUIRED` (partial coverage, same shape as
//! C044).
//!
//! `StringInStr` calls here are all bare (no explicit casesense
//! argument) — case-insensitive, the documented default. `StringReplace`
//! without an explicit `casesense` argument shares that same default in
//! AutoIt, so [`strip_filename_prefix`]'s containment check and
//! replacement are both modeled case-insensitively too.

use crate::extract::{Invocation, WindowMode};

/// Builds the command-line scan invocation `FileScan_ExeInfo` makes in
/// extract mode (UniExtract.au3:1104): `<exeinfope> "<file>*" /sx
/// /log:"<log_file>"`, run in `bindir` with the window hidden. The
/// literal trailing `*` appended to `file` inside its own quoted token
/// is exactly what the source concatenates — not a typo to "fix".
/// `_FileRead($LogFile, True)` (reading the resulting log) is real
/// filesystem I/O, left to the caller.
pub fn scan_invocation(exeinfope: &str, file: &str, bindir: &str, log_file: &str) -> Invocation {
    Invocation {
        program: exeinfope.to_string(),
        args: vec![
            format!("{file}*"),
            "/sx".to_string(),
            format!("/log:\"{log_file}\""),
        ],
        working_dir: bindir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports the retry condition (UniExtract.au3:1105): the command-line log
/// read either explicitly reports corruption, or came back empty/
/// all-whitespace (`StringIsSpace`, which AutoIt also satisfies for a
/// zero-length string). Either way the source retries via
/// `FileScan_ExeInfo(False)` — the GUI-automation path, out of scope
/// here (see module doc comment).
pub fn should_retry_as_gui(log_output: &str) -> bool {
    log_output
        .to_lowercase()
        .contains("file corrupted or buffer error")
        || log_output.trim().is_empty()
}

/// Ports the filename-echo strip (UniExtract.au3:1119): when the scanned
/// text contains `filenamefull`, remove that substring, strip leading
/// whitespace from what's left, then drop 2 more characters (the
/// source's own `StringTrimLeft(..., 2)`, preserved exactly even though
/// it isn't obviously a specific separator). If `filenamefull` isn't
/// present at all, the text passes through unchanged.
pub fn strip_filename_prefix(raw: &str, filenamefull: &str) -> String {
    if filenamefull.is_empty() || !raw.to_lowercase().contains(&filenamefull.to_lowercase()) {
        return raw.to_string();
    }
    let idx = raw
        .to_lowercase()
        .find(&filenamefull.to_lowercase())
        .unwrap();
    let mut replaced = String::with_capacity(raw.len());
    replaced.push_str(&raw[..idx]);
    replaced.push_str(&raw[idx + filenamefull.len()..]);
    let leading_stripped = replaced.trim_start();
    leading_stripped.chars().skip(2).collect()
}

/// What `FileScan_ExeInfo` does once it has a cleaned scan-result string
/// in hand (UniExtract.au3:1122-1140).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanResult {
    /// `"Skipped"` — the file was too big for Exeinfo PE to analyze; the
    /// source returns with no further action.
    Skipped,
    /// Scan-only mode and the text says `"file is not EXE or DLL"` —
    /// the source deliberately suppresses this particular "unknown"
    /// result from scan-only display.
    ScanOnlySuppressed,
    /// Scan-only mode, not suppressed: the source records the result
    /// (`_FiletypeAdd`) and returns the raw text, without reaching the
    /// dispatch `Select`.
    ScanOnlyRecorded,
    /// Extract mode: the source records the result and proceeds to the
    /// dispatch `Select` — `detection::exeinfo_dispatch::classify`
    /// (C043).
    ProceedToDispatch,
}

/// Ports `FileScan_ExeInfo`'s post-cleanup branch (UniExtract.au3:1122-
/// 1140). `cleaned` is [`strip_filename_prefix`]'s result.
pub fn classify_scan_result(cleaned: &str, extract: bool) -> ScanResult {
    let s = cleaned.to_lowercase();
    if s.contains("skipped") {
        ScanResult::Skipped
    } else if !extract && s.contains("file is not exe or dll") {
        ScanResult::ScanOnlySuppressed
    } else if !extract {
        ScanResult::ScanOnlyRecorded
    } else {
        ScanResult::ProceedToDispatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_invocation_matches_source_shape() {
        assert_eq!(
            scan_invocation(
                r"C:\bin\exeinfope.exe",
                r"C:\downloads\setup.exe",
                r"C:\bin",
                r"C:\logs\exeinfo.log"
            ),
            Invocation {
                program: r"C:\bin\exeinfope.exe".to_string(),
                args: vec![
                    r"C:\downloads\setup.exe*".to_string(),
                    "/sx".to_string(),
                    r#"/log:"C:\logs\exeinfo.log""#.to_string(),
                ],
                working_dir: r"C:\bin".to_string(),
                window: WindowMode::Hidden,
            }
        );
    }

    #[test]
    fn corrupted_or_empty_log_output_triggers_gui_retry() {
        assert!(should_retry_as_gui("File corrupted or Buffer Error"));
        assert!(should_retry_as_gui(""));
        assert!(should_retry_as_gui("   \r\n  "));
        assert!(!should_retry_as_gui("Inno Setup installer detected"));
    }

    #[test]
    fn retry_check_is_case_insensitive() {
        assert!(should_retry_as_gui("FILE CORRUPTED OR BUFFER ERROR"));
    }

    #[test]
    fn strip_filename_prefix_removes_echo_and_two_trailing_chars() {
        // "setup.exe" removed, leading whitespace trimmed, then 2 more
        // chars dropped (source's own StringTrimLeft(..., 2)).
        assert_eq!(
            strip_filename_prefix("setup.exe  : Inno Setup installer", "setup.exe"),
            "Inno Setup installer"
        );
    }

    #[test]
    fn strip_filename_prefix_passes_through_when_absent() {
        assert_eq!(
            strip_filename_prefix("Inno Setup installer", "setup.exe"),
            "Inno Setup installer"
        );
    }

    #[test]
    fn strip_filename_prefix_is_case_insensitive() {
        assert_eq!(
            strip_filename_prefix("SETUP.EXE  : Inno Setup installer", "setup.exe"),
            "Inno Setup installer"
        );
    }

    #[test]
    fn skipped_result_takes_no_action() {
        assert_eq!(
            classify_scan_result("Skipped (file too big)", true),
            ScanResult::Skipped
        );
        assert_eq!(
            classify_scan_result("Skipped (file too big)", false),
            ScanResult::Skipped
        );
    }

    #[test]
    fn scan_only_mode_suppresses_the_not_exe_or_dll_message() {
        assert_eq!(
            classify_scan_result("file is not EXE or DLL", false),
            ScanResult::ScanOnlySuppressed
        );
        // In extract mode the same text is not suppressed -- it just
        // proceeds to the dispatch table like anything else.
        assert_eq!(
            classify_scan_result("file is not EXE or DLL", true),
            ScanResult::ProceedToDispatch
        );
    }

    #[test]
    fn ordinary_results_split_on_extract_mode() {
        assert_eq!(
            classify_scan_result("Inno Setup installer", false),
            ScanResult::ScanOnlyRecorded
        );
        assert_eq!(
            classify_scan_result("Inno Setup installer", true),
            ScanResult::ProceedToDispatch
        );
    }
}
