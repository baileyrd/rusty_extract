//! Top-level detection cascade order (`StartExtraction`,
//! UniExtract.au3:376-463, comment 424-430) — capability C037.
//!
//! ```autoit
//! ; If an extractor is specified via command line parameter, we simply use that without scanning
//! If $sArcTypeOverride Then Return extract($sArcTypeOverride, $sArcTypeOverride & " " & t('TERM_FILE'))
//!
//! ; UniExtract uses four methods of detection (in order):
//! ; 1. File extensions for special cases
//! ; 2. Binary file analysis of files using TrID if file extension is not .exe
//! ; 3. Binary file analysis of PE (executable) files using Exeinfo PE
//! ; 4. Extra analysis using PeID if executable is not recognized by Exeinfo PE
//! ; 5. Binary file analysis of files using TrID
//! ; 6. File extensions
//!
//! ; First, check for file extensions that require special actions
//! InitialCheckExt()
//!
//! ; If file is an .exe, scan with Exeinfo PE and PEiD
//! If $fileext = "exe" Or $fileext = "dll" Then IsExe()
//!
//! ; Scan file with TrID, if file is not an .exe
//! FileScan_Trid($extract)
//!
//! ; ExeInfo PE supports non-executables as well
//! If Not $exefailed Then FileScan_ExeInfo()
//!
//! ; Display file information and terminate if scan only mode
//! If Not $extract Then
//!     FileScan_MediaInfo()
//!     terminate($STATUS_FILEINFO, $filenamefull, $fileext)
//! EndIf
//!
//! ; Else perform additional extraction methods
//! CheckIso()
//! CheckGame()
//! CheckTotalObserver()
//!
//! ; Use file extension if signature not recognized
//! CheckExt()
//!
//! check7z()
//!
//! ; Cannot determine filetype, all checks failed - abort
//! terminate($STATUS_UNKNOWNEXT, $file, $fileext & "; " & StringLeft($aFiletype[0][1], 45))
//! ```
//!
//! **The `$sArcTypeOverride` early return** is C006's own routing
//! decision (`type_override::parse_type_override`) — not duplicated
//! here; a caller checks that first and only reaches [`steps`] once no
//! override is in effect. **`InitialCheckExt()`** is C046
//! (`detection::initial_ext_check`), already covered and always the
//! first step, so it isn't repeated in [`steps`]'s output either — this
//! module picks up right after it. Every other named step
//! (`IsExe`/`FileScan_Trid`/`FileScan_ExeInfo`/`FileScan_MediaInfo`/
//! `CheckIso`/`CheckGame`/`CheckTotalObserver`/`CheckExt`/`check7z`) is
//! its own separately-tracked capability (C038-C045, C077, C047, C048)
//! — this capability is purely the *order and gating* between them,
//! matching the source comment's framing ("order itself is
//! behavior-significant").
//!
//! **`fileext` is assumed already-lowercased** upstream, the same
//! assumption `detection::initial_ext_check` documents (citing
//! C175/C176) — the source's own comparison here is a single `=`
//! (case-insensitive per this script's default `StringCompareMode`
//! anyway), so this doesn't change behavior either way.
//!
//! **A genuinely surprising finding, not obvious from reading
//! `StartExtraction` in isolation**: `IsExe()` (UniExtract.au3:466-496)
//! ends every one of its own paths in `extract` mode by calling
//! `terminate(...)` itself — either indirectly (a nested `extract(...)`
//! call that matched a type) or via its own unconditional
//! `terminate($STATUS_UNKNOWNEXE, ...)` at the very end. So whenever
//! `$fileext` is `"exe"`/`"dll"` **and** `$extract` is true, control
//! never returns to `StartExtraction()` at all — none of
//! `FileScan_Trid`/`FileScan_ExeInfo`/`CheckIso`/`CheckGame`/
//! `CheckTotalObserver`/`CheckExt`/`check7z` ever run for such a file;
//! [`Step::IsExe`] is the only step [`steps`] reports. `IsExe()` only
//! returns control to this cascade in scan-only mode (`$extract` is
//! false), because of its own early `If Not $extract Then Return` gate
//! — reached only after it has already set `$exefailed = True`, which
//! is why the following `If Not $exefailed Then FileScan_ExeInfo()`
//! never re-runs the Exeinfo PE scan for a file `IsExe()` already
//! covered.

/// One named step `StartExtraction()` can take after `InitialCheckExt()`,
/// each already covered by its own capability — see the module doc
/// comment for which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `IsExe()` — UniExtract.au3:436. In extract mode this is always
    /// the *last* step reported, since `IsExe()` never returns control
    /// in that mode (see module doc comment).
    IsExe,
    /// `FileScan_Trid($extract)` — UniExtract.au3:439.
    TridScan,
    /// `FileScan_ExeInfo()` — UniExtract.au3:442, gated on `Not
    /// $exefailed`.
    ExeInfoScan,
    /// `FileScan_MediaInfo()` then `terminate($STATUS_FILEINFO, ...)` —
    /// UniExtract.au3:445-447. Always the last step in scan-only mode.
    MediaInfoThenTerminateFileInfo,
    /// `CheckIso()` — UniExtract.au3:451.
    CheckIso,
    /// `CheckGame()` — UniExtract.au3:452.
    CheckGame,
    /// `CheckTotalObserver()` — UniExtract.au3:453.
    CheckTotalObserver,
    /// `CheckExt()` — UniExtract.au3:456.
    CheckExt,
    /// `check7z()` — UniExtract.au3:458.
    Check7z,
    /// `terminate($STATUS_UNKNOWNEXT, ...)` — UniExtract.au3:462. Only
    /// reached if every step before it failed to dispatch.
    TerminateUnknownExt,
}

/// Ports `StartExtraction()`'s step order from right after
/// `InitialCheckExt()` (UniExtract.au3:436-462), given only the two
/// externally-visible facts that drive its branching: the (assumed
/// already-lowercased) file extension and whether this is extract mode
/// (`$extract`) or scan-only.
pub fn steps(fileext: &str, extract: bool) -> Vec<Step> {
    let is_exe_ext = fileext == "exe" || fileext == "dll";

    if is_exe_ext && extract {
        return vec![Step::IsExe];
    }

    let mut out = Vec::new();
    if is_exe_ext {
        out.push(Step::IsExe);
    }
    out.push(Step::TridScan);
    if !is_exe_ext {
        out.push(Step::ExeInfoScan);
    }

    if !extract {
        out.push(Step::MediaInfoThenTerminateFileInfo);
        return out;
    }

    out.extend([
        Step::CheckIso,
        Step::CheckGame,
        Step::CheckTotalObserver,
        Step::CheckExt,
        Step::Check7z,
        Step::TerminateUnknownExt,
    ]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C037: an exe/dll file in extract mode
    /// is entirely delegated to `IsExe()` — nothing else in
    /// `StartExtraction()` ever runs.
    #[test]
    fn exe_extension_in_extract_mode_delegates_entirely_to_is_exe() {
        assert_eq!(steps("exe", true), vec![Step::IsExe]);
        assert_eq!(steps("dll", true), vec![Step::IsExe]);
    }

    /// Parity test for capability C037: an exe/dll file in scan-only
    /// mode returns from `IsExe()` with `$exefailed` already set, so
    /// `FileScan_ExeInfo()` is skipped the second time around.
    #[test]
    fn exe_extension_in_scan_only_mode_skips_the_redundant_exeinfo_scan() {
        assert_eq!(
            steps("exe", false),
            vec![
                Step::IsExe,
                Step::TridScan,
                Step::MediaInfoThenTerminateFileInfo
            ]
        );
    }

    /// Parity test for capability C037: a non-exe file in scan-only mode
    /// runs the Exeinfo scan directly (no `IsExe()` step at all) then
    /// always terminates on the file-info display.
    #[test]
    fn non_exe_extension_in_scan_only_mode_runs_exeinfo_then_terminates() {
        assert_eq!(
            steps("zip", false),
            vec![
                Step::TridScan,
                Step::ExeInfoScan,
                Step::MediaInfoThenTerminateFileInfo
            ]
        );
    }

    /// Parity test for capability C037: a non-exe file in extract mode
    /// is the only combination that reaches the full remaining cascade.
    #[test]
    fn non_exe_extension_in_extract_mode_runs_the_full_cascade() {
        assert_eq!(
            steps("zip", true),
            vec![
                Step::TridScan,
                Step::ExeInfoScan,
                Step::CheckIso,
                Step::CheckGame,
                Step::CheckTotalObserver,
                Step::CheckExt,
                Step::Check7z,
                Step::TerminateUnknownExt,
            ]
        );
    }
}
