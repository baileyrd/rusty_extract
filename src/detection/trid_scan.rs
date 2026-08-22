//! TrID signature-based detection (`FileScan_Trid`, UniExtract.au3:901-
//! 941) — the scan-orchestration half of capability C038. The dispatch
//! table this scan feeds into is already ported separately as
//! [`detection::trid_dispatch::classify`] (C039, UniExtract.au3:1490-
//! 1801) — this module covers the decisions around it: which of TrID's
//! candidate results get passed to it, and how the scan-only-mode
//! command-line invocation and output filtering work.
//!
//! ```autoit
//! Func FileScan_Trid($analyze = 1)
//!     If $tridfailed Then Return
//!
//!     If $extract Then
//!         Local $iResults = TridLib_Analyse($file)
//!         If $iResults = 0 Then
//!             Cout("Unknown filetype!")
//!         Else
//!             For $i = 1 To $iResults
//!                 Local $sType = TridLib_GetType($i)
//!                 _FiletypeAdd("TrID", $sType)
//!                 If $appendext And $i == 1 Then RenameWithTridExtension()
//!                 If $analyze And $i < 4 Then tridcompare($sType)
//!             Next
//!         EndIf
//!     Else ; Run TrID and fetch output to include additional information about the file type
//!         Local $aReturn = StringSplit(FetchStdout($trid & ' "' & $file & '"' & ($analyze? "": " -v"), $filedir, @SW_HIDE, 0, True, False), @CRLF)
//!         If $appendext Then RenameWithTridExtension($file, True)
//!
//!         Local $sFileType = ""
//!         For $i = 1 To UBound($aReturn) - 1
//!             If StringInStr($aReturn[$i], "%") Or (Not $analyze And (StringInStr($aReturn[$i], "Related URL") Or StringInStr($aReturn[$i], "Remarks"))) Then _
//!                 $sFileType &= $aReturn[$i] & @CRLF
//!         Next
//!
//!         If $sFileType <> "" Then
//!             _FiletypeAdd("TrID", $sFileType)
//!             If $analyze Then tridcompare($sFileType)
//!         EndIf
//!     EndIf
//!
//!     FileScan_UnixFile()
//!     $tridfailed = True
//! EndFunc
//! ```
//!
//! **`TridLib_Analyse`/`TridLib_GetType`'s `DllCall`s into `TrIDLib.dll`
//! are now ported too**, PR [#416](https://github.com/baileyrd/rusty_extract/pull/416):
//! `dlllib::tridlib_load`/`tridlib_analyse`/`TridLibrary::result_type`
//! (built on `dlllib`'s new DLL-calling infrastructure, the same
//! trait/fake/real-Win32 split `automation::GuiAutomation` already
//! established for window automation). `FetchStdout` itself (scan-only
//! mode's `trid.exe` process spawn) stays out of scope, the same
//! external-process boundary documented throughout this port.
//!
//! **A genuinely surprising reversal from C042's split**: there, extract
//! mode was the portable command-line path and scan-only mode was
//! GUI-blocked. Here it's the opposite — extract mode calls the blocked
//! `TrIDLib.dll` functions directly, while scan-only mode shells out to
//! `trid.exe` via `FetchStdout`, a real process invocation whose
//! *command-line construction* ([`scan_invocation`]) is fully portable.
//!
//! **`$tridfailed`/`FileScan_UnixFile()`/setting `$tridfailed = True`**
//! are, respectively: a plain re-entrancy guard (trivial, not modeled as
//! its own decision function); the unconditional call already documented
//! as part of C040's own module doc comment; and a state update with no
//! further decision attached. None of the three need a dedicated
//! function here.
//!
//! **Capability C178 — verified still present.** `TridLib_Load`'s
//! `TrID_LoadDefsPack` call (UniExtract.au3:949,
//! `DllCall($hTridDll, "int", "TrID_LoadDefsPack", "str", $bindir)`)
//! marshals its directory argument as `"str"` — AutoIt's ANSI
//! (single-byte, current-codepage) string type, not `"wstr"` (UTF-16).
//! `TridLib_Analyse`'s own `TrID_SubmitFileA` call (UniExtract.au3:964
//! — the "A" suffix is itself the Win32 ANSI-variant naming
//! convention) marshals the *scanned file's own path* the same way.
//! Every other string parameter into `TrIDLib.dll` across this whole
//! wrapper (`TridLib_GetType`/`TridLib_GetExtension`'s output buffers
//! included) is `"str"` too — there is no `"wstr"` call site anywhere
//! in the wrapper. This is consistent with the documented UNC-path
//! TrID-detection-failure report: a UNC path, or any path containing a
//! character outside the process's current ANSI codepage, can be
//! silently corrupted by this narrowing conversion before `TrIDLib.dll`
//! ever sees it. [`trid_dll_string_marshalling`] makes the finding
//! explicit and testable — the real `DllCall`s themselves stay out of
//! scope (missing-FFI blocker, same as the rest of this module).

use crate::extract::{Invocation, WindowMode};

/// AutoIt `DllCall` string-parameter marshalling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringMarshalling {
    /// `"str"` — ANSI, single-byte, current codepage.
    Ansi,
    /// `"wstr"` — wide, UTF-16.
    Wide,
}

/// Capability C178: every string parameter `TrIDLib.dll`'s wrapper
/// functions pass — the directory in `TrID_LoadDefsPack`
/// (UniExtract.au3:949) and the scanned file's own path in
/// `TrID_SubmitFileA` (UniExtract.au3:964) alike — is marshalled as
/// [`StringMarshalling::Ansi`]. See the module doc comment for why
/// this is the documented root cause of TrID's UNC-path detection
/// unreliability.
pub fn trid_dll_string_marshalling() -> StringMarshalling {
    StringMarshalling::Ansi
}

/// Builds the scan-only-mode invocation `FileScan_Trid` makes
/// (UniExtract.au3:922): `<trid> "<file>"`, with a trailing `-v` token
/// appended only when `analyze` is `false`. Run in `file_dir` with the
/// window hidden.
pub fn scan_invocation(
    trid_program: &str,
    file: &str,
    file_dir: &str,
    analyze: bool,
) -> Invocation {
    let mut args = vec![file.to_string()];
    if !analyze {
        args.push("-v".to_string());
    }
    Invocation {
        program: trid_program.to_string(),
        args,
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// What extract mode does for the `i`-th (1-based) of TrID's candidate
/// results (UniExtract.au3:916-917). `total_results` isn't needed here
/// — the loop bound (`$iResults`) only decides *how many* indices exist,
/// not what each one does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractResultAction {
    /// `$appendext And $i == 1` — rename the file to TrID's suggested
    /// extension. Only ever true for the very first result.
    pub rename_with_trid_extension: bool,
    /// `$analyze And $i < 4` — dispatch this result to `tridcompare`
    /// (C039). Only the first three candidate results are ever
    /// compared, regardless of how many TrID actually returned.
    pub dispatch_to_tridcompare: bool,
}

/// Ports the per-result decision inside `FileScan_Trid`'s extract-mode
/// loop (UniExtract.au3:913-918). `index` is 1-based, matching the
/// source's own `For $i = 1 To $iResults`.
pub fn extract_result_action(index: u32, appendext: bool, analyze: bool) -> ExtractResultAction {
    ExtractResultAction {
        rename_with_trid_extension: appendext && index == 1,
        dispatch_to_tridcompare: analyze && index < 4,
    }
}

/// Ports the scan-only-mode output-line filter (UniExtract.au3:927): a
/// line survives if it contains `"%"`, or — only when `analyze` is
/// `false` — if it mentions `"Related URL"` or `"Remarks"`. Every
/// `StringInStr` here is bare (case-insensitive, the documented
/// default).
pub fn should_keep_scan_only_line(line: &str, analyze: bool) -> bool {
    let s = line.to_lowercase();
    s.contains('%') || (!analyze && (s.contains("related url") || s.contains("remarks")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_invocation_omits_verbose_flag_when_analyzing() {
        assert_eq!(
            scan_invocation("trid.exe", r"C:\file.bin", r"C:\dir", true),
            Invocation {
                program: "trid.exe".to_string(),
                args: vec![r"C:\file.bin".to_string()],
                working_dir: r"C:\dir".to_string(),
                window: WindowMode::Hidden,
            }
        );
    }

    #[test]
    fn scan_invocation_appends_verbose_flag_when_not_analyzing() {
        assert_eq!(
            scan_invocation("trid.exe", r"C:\file.bin", r"C:\dir", false),
            Invocation {
                program: "trid.exe".to_string(),
                args: vec![r"C:\file.bin".to_string(), "-v".to_string()],
                working_dir: r"C:\dir".to_string(),
                window: WindowMode::Hidden,
            }
        );
    }

    /// Parity test for capability C038: only the first result ever
    /// triggers the rename.
    #[test]
    fn only_first_result_renames_with_trid_extension() {
        assert!(extract_result_action(1, true, true).rename_with_trid_extension);
        assert!(!extract_result_action(2, true, true).rename_with_trid_extension);
        assert!(!extract_result_action(1, false, true).rename_with_trid_extension);
    }

    /// Parity test for capability C038: only the first three results
    /// (index 1-3) ever dispatch to `tridcompare`, regardless of how
    /// many total results TrID returned.
    #[test]
    fn only_first_three_results_dispatch_to_tridcompare() {
        for i in 1..=3 {
            assert!(extract_result_action(i, false, true).dispatch_to_tridcompare);
        }
        assert!(!extract_result_action(4, false, true).dispatch_to_tridcompare);
        assert!(!extract_result_action(10, false, true).dispatch_to_tridcompare);
        assert!(!extract_result_action(1, false, false).dispatch_to_tridcompare);
    }

    #[test]
    fn scan_only_line_kept_when_it_mentions_a_percentage() {
        assert!(should_keep_scan_only_line(
            "File type identified: 85.0% (.exe)",
            true
        ));
        assert!(should_keep_scan_only_line(
            "File type identified: 85.0% (.exe)",
            false
        ));
    }

    /// Parity test for capability C038: "Related URL"/"Remarks" lines
    /// are only kept in non-analyze (verbose, `-v`) mode.
    #[test]
    fn related_url_and_remarks_lines_only_kept_when_not_analyzing() {
        assert!(should_keep_scan_only_line(
            "Related URL: https://example.com",
            false
        ));
        assert!(should_keep_scan_only_line("Remarks: none", false));
        assert!(!should_keep_scan_only_line(
            "Related URL: https://example.com",
            true
        ));
        assert!(!should_keep_scan_only_line("Remarks: none", true));
    }

    #[test]
    fn unrelated_lines_are_dropped() {
        assert!(!should_keep_scan_only_line(
            "TrID/32 - File Identifier",
            true
        ));
        assert!(!should_keep_scan_only_line(
            "TrID/32 - File Identifier",
            false
        ));
    }

    #[test]
    fn line_filter_is_case_insensitive() {
        assert!(should_keep_scan_only_line("RELATED URL: x", false));
        assert!(should_keep_scan_only_line("REMARKS: x", false));
    }

    /// Parity test for capability C178: verified still present -- the
    /// TrIDLib.dll wrapper marshals every string argument as ANSI
    /// ("str"), never wide ("wstr"), consistent with the documented
    /// UNC-path detection-failure report.
    #[test]
    fn trid_dll_calls_use_ansi_not_wide_string_marshalling() {
        assert_eq!(trid_dll_string_marshalling(), StringMarshalling::Ansi);
    }
}
