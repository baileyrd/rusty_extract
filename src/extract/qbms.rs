//! QuickBMS (`$TYPE_QBMS`) + WCX plugin fan-out: three probe-then-classify
//! detectors (InstallExplorer, ISO/CD-DVD image, TotalObserver), each
//! wrapping a different WCX plugin around the same `quickbms.exe`, plus
//! the shared extraction case they (and `BmsExtract`'s game-specific
//! `.bms` scripts) all funnel into.
//!
//! ```autoit
//! ; Determine if InstallExplorer can extract the file
//! Func checkIE()
//!     If $iefailed Then Return False
//!     Local $return = FetchStdout($quickbms & ' -l "' & $bindir & $ie & '" "' & $file & '"', $filedir, @SW_HIDE)
//!     If StringInStr($return, "Target directory:", 0) Or StringInStr($return, "0 files found", 0) Or StringInStr($return, "Error", 0) _
//!     Or StringInStr($return, "exception occured", 0) Or StringInStr($return, "not supported", 0) Or StringInStr($return, "crash occurred", 0) _
//!     Or $return == "" Then
//!         $iefailed = True
//!         Return False
//!     EndIf
//!     ; ... extract($TYPE_QBMS, ..., $ie, ...) on success ...
//! EndFunc
//!
//! ; Determine if file is CD/DVD image
//! Func CheckIso($returnSuccess = False, $returnFail = False)
//!     If $isofailed Then Return False
//!     Local $return = FetchStdout($quickbms & ' -l "' & $bindir & $iso & '" "' & $file & '"', $filedir, @SW_HIDE)
//!     If StringInStr($return, "Target directory:") Or StringInStr($return, "0 files found") Or $return == "" _
//!     Or StringInStr($return, "exception occured") Or StringInStr($return, "not supported by this WCX plugin") Then
//!         $isofailed = True
//!         Return False
//!     EndIf
//!     Return extract($TYPE_QBMS, t('TERM_DISK_IMAGE'), $iso, $returnSuccess, $returnFail)
//! EndFunc
//!
//! ; Determine if file can be extracted by TotalObserver
//! Func CheckTotalObserver($arcdisp = 0)
//!     If $observerfailed Then Return False
//!     Local $return = FetchStdout($quickbms & ' -l "' & $bindir & $observer & '" "' & $file & '"', $filedir, @SW_HIDE)
//!     If StringInStr($return, "not supported by this WCX plugin") Or StringInStr($return, "0 files found") Or _
//!        StringInStr($return, "exception occured") Or StringInStr($return, "EXCEPTION HANDLER") Then
//!        $observerfailed = True
//!        Return False
//!     EndIf
//!     extract($TYPE_QBMS, $arcdisp, $observer)
//! EndFunc
//!
//! Case $TYPE_QBMS
//!     Local $sPlugin = $additionalParameters? $bindir & $additionalParameters: $bms
//!     _Run($quickbms & ' -K "' & $sPlugin & '" "' & $file & '" "' & $outdir & '"', $outdir, @SW_MINIMIZE, True, False)
//!     If FileExists($bms) Then FileDelete($bms)
//!
//!     If $additionalParameters == $ie Then
//!         Local $aCleanup[] = ["[NSIS].nsi", "[LICENSE].*", "$PLUGINSDIR", "$TEMP", "uninstall.exe", "[LICENSE]"]
//!         Cleanup($aCleanup)
//!     EndIf
//! ```
//!
//! **Scope — three detectors and the shared extraction case; not
//! `BmsExtract` or `CheckGame`'s game-database lookup.** `BmsExtract`
//! (UniExtract.au3:3544) loads a game-specific `.bms` script via
//! `_SQLite_GetTable` — the same SQLite array-indexing semantics already
//! found ambiguous for C055 (`CheckGame`'s `BMS.db` lookup, UniExtract.au3
//! :2007 among this capability's own citations) — and isn't modeled here
//! for the same reason: guessing at `_SQLite_GetTable`'s exact
//! array-shape contract risks silently-wrong parity, not a call to make
//! without independent verification this port doesn't have. This
//! module's own detectors and the shared `Case $TYPE_QBMS` invocation
//! are fully independent of that SQLite path and don't need it to be
//! useful on their own.
//!
//! **`$additionalParameters` doubles as a plugin-selector string.**
//! Every caller of `extract($TYPE_QBMS, ...)` passes its own WCX plugin
//! filename (or a `.bms` script name) as the 3rd positional argument —
//! `checkIE` passes `$ie`, `CheckIso` passes `$iso`, `CheckTotalObserver`
//! passes `$observer`, `CheckGame`'s GAUP probe (C055/C180, not modeled
//! here) passes `$gaup`. [`resolve_plugin_path`] ports the resulting
//! `Case $TYPE_QBMS` selection: a non-empty selector resolves to
//! `<bindir><selector>`; an empty one falls back to `$bms`, the
//! dynamically-written game-script path `BmsExtract` would have written.

use crate::extract::{Invocation, WindowMode};

/// Builds the InstallExplorer listing-probe invocation
/// (UniExtract.au3:2072): `<quickbms> -l "<bindir><ie_plugin>" "<file>"`,
/// run in `filedir` with the window hidden.
pub fn ie_probe_invocation(
    quickbms: &str,
    bindir: &str,
    ie_plugin: &str,
    file: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: quickbms.to_string(),
        args: vec![
            "-l".to_string(),
            format!("{bindir}{ie_plugin}"),
            file.to_string(),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports `checkIE`'s failure classification (UniExtract.au3:2074-2076):
/// case-insensitive on every marker (explicit `0` third argument, same
/// default AutoIt's bare `StringInStr` already uses).
pub fn is_ie_probe_failure(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.is_empty()
        || lower.contains("target directory:")
        || lower.contains("0 files found")
        || lower.contains("error")
        || lower.contains("exception occured")
        || lower.contains("not supported")
        || lower.contains("crash occurred")
}

/// Builds `CheckGame`'s GAUP listing-probe invocation
/// (UniExtract.au3:2007), capabilities C055/C180: `<quickbms> -l
/// "<bindir><gaup_plugin>" "<file>"`, run in `filedir` with the window
/// hidden. This is the exact call site C180 investigated for its "hang
/// risk": `FetchStdout`'s own polling loop (UniExtract.au3:5075-5098)
/// already bounds it by `$Timeout`, but an unset/first-run `$Timeout`
/// preference resolves to ~16.7 hours of busy-polling
/// (`prefs::tests::missing_preference_key_reproduces_the_sixty_million_millisecond_quirk`,
/// C026) — this crate's own runner carries no timeout modeling for any
/// call site regardless (C150, PR #380), so there's nothing further to
/// port for the "hang" itself.
pub fn gaup_probe_invocation(
    quickbms: &str,
    bindir: &str,
    gaup_plugin: &str,
    file: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: quickbms.to_string(),
        args: vec![
            "-l".to_string(),
            format!("{bindir}{gaup_plugin}"),
            file.to_string(),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports `CheckGame`'s GAUP-probe failure classification
/// (UniExtract.au3:2009-2010): bare `StringInStr`/explicit `0`
/// (case-insensitive). **`output` must already be the tail this call
/// site's `FetchStdout(..., -1)` extracts** — everything from the
/// *second*-to-last `@CRLF` onward, the same `_StringGetLine` idiom
/// `password_search::nth_line_from_end`'s own doc comment documents in
/// full — not the probe's entire captured output, unlike
/// `is_ie_probe_failure`/`is_iso_probe_failure`/
/// `is_observer_probe_failure` (none of which request a specific line).
pub fn is_gaup_probe_failure(output: &str) -> bool {
    let lower = output.to_lowercase();
    output.is_empty()
        || lower.contains("target directory:")
        || lower.contains("0 files found")
        || lower.contains("error")
        || lower.contains("exception occured")
        || lower.contains("not supported")
}

/// Builds the ISO/CD-DVD-image listing-probe invocation
/// (UniExtract.au3:2121): `<quickbms> -l "<bindir><iso_plugin>"
/// "<file>"`, run in `filedir` with the window hidden.
pub fn iso_probe_invocation(
    quickbms: &str,
    bindir: &str,
    iso_plugin: &str,
    file: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: quickbms.to_string(),
        args: vec![
            "-l".to_string(),
            format!("{bindir}{iso_plugin}"),
            file.to_string(),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports `CheckIso`'s failure classification (UniExtract.au3:2123-2124):
/// bare `StringInStr`, case-insensitive.
pub fn is_iso_probe_failure(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.is_empty()
        || lower.contains("target directory:")
        || lower.contains("0 files found")
        || lower.contains("exception occured")
        || lower.contains("not supported by this wcx plugin")
}

/// Builds the TotalObserver listing-probe invocation
/// (UniExtract.au3:2162): `<quickbms> -l "<bindir><observer_plugin>"
/// "<file>"`, run in `filedir` with the window hidden.
pub fn observer_probe_invocation(
    quickbms: &str,
    bindir: &str,
    observer_plugin: &str,
    file: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: quickbms.to_string(),
        args: vec![
            "-l".to_string(),
            format!("{bindir}{observer_plugin}"),
            file.to_string(),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports `CheckTotalObserver`'s failure classification
/// (UniExtract.au3:2164-2165): bare `StringInStr`, case-insensitive.
pub fn is_observer_probe_failure(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("not supported by this wcx plugin")
        || lower.contains("0 files found")
        || lower.contains("exception occured")
        || lower.contains("exception handler")
}

/// Ports `Case $TYPE_QBMS`'s plugin-path selection (UniExtract.au3:2985):
/// `$additionalParameters? $bindir & $additionalParameters: $bms` — a
/// non-empty selector resolves against `bindir`; an empty one (AutoIt's
/// falsy-empty-string ternary condition) falls back to `default_bms`,
/// the dynamically-written game-script path `BmsExtract` would have
/// written (not modeled here — see module doc comment).
pub fn resolve_plugin_path(additional_parameters: &str, bindir: &str, default_bms: &str) -> String {
    if additional_parameters.is_empty() {
        default_bms.to_string()
    } else {
        format!("{bindir}{additional_parameters}")
    }
}

/// Builds the shared extraction invocation (UniExtract.au3:2986):
/// `<quickbms> -K "<plugin_path>" "<file>" "<outdir>"`, run in `outdir`
/// with the window minimized.
pub fn qbms_invocation(quickbms: &str, plugin_path: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: quickbms.to_string(),
        args: vec![
            "-K".to_string(),
            plugin_path.to_string(),
            file.to_string(),
            outdir.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Ports `If $additionalParameters == $ie Then` (UniExtract.au3:2989):
/// case-sensitive `==` — whether this was an InstallExplorer extraction,
/// which gets its own extra cleanup pass.
pub fn is_installexplorer_plugin(additional_parameters: &str, ie_plugin: &str) -> bool {
    additional_parameters == ie_plugin
}

/// The InstallExplorer-specific cleanup targets (UniExtract.au3:2990) —
/// real `Cleanup(...)` execution stays out of scope, matching this
/// crate's usual split.
pub const INSTALLEXPLORER_CLEANUP_TARGETS: &[&str] = &[
    "[NSIS].nsi",
    "[LICENSE].*",
    "$PLUGINSDIR",
    "$TEMP",
    "uninstall.exe",
    "[LICENSE]",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaup_probe_invocation_matches_source() {
        let inv = gaup_probe_invocation(
            r"C:\bin\quickbms.exe",
            r"C:\bin\",
            "gaup.wcx",
            r"C:\downloads\game.dat",
            r"C:\downloads",
        );
        assert_eq!(
            inv.args,
            vec![
                "-l".to_string(),
                r"C:\bin\gaup.wcx".to_string(),
                r"C:\downloads\game.dat".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn gaup_probe_failure_detects_every_marker_case_insensitively() {
        assert!(is_gaup_probe_failure(""));
        assert!(is_gaup_probe_failure("Target directory: C:\\out"));
        assert!(is_gaup_probe_failure("0 FILES FOUND"));
        assert!(is_gaup_probe_failure("ERROR: bad script"));
        assert!(is_gaup_probe_failure("Exception Occured"));
        assert!(is_gaup_probe_failure("Not Supported"));
        assert!(!is_gaup_probe_failure("Offset  Filename\n0  file.dat"));
    }

    /// Parity test for capabilities C055/C180: unlike the IE/ISO/observer
    /// probes, GAUP's marker set has no "crash occurred"/"by this WCX
    /// plugin" phrasing — it's a distinct, shorter list, not a copy of
    /// any of the other three.
    #[test]
    fn gaup_probe_failure_does_not_check_ie_specific_markers() {
        assert!(!is_gaup_probe_failure("crash occurred while scanning"));
    }

    #[test]
    fn ie_probe_invocation_matches_source() {
        let inv = ie_probe_invocation(
            r"C:\bin\quickbms.exe",
            r"C:\bin\",
            "InstExpl.wcx",
            r"C:\downloads\installer.exe",
            r"C:\downloads",
        );
        assert_eq!(
            inv.args,
            vec![
                "-l".to_string(),
                r"C:\bin\InstExpl.wcx".to_string(),
                r"C:\downloads\installer.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn ie_probe_failure_detects_every_marker_case_insensitively() {
        assert!(is_ie_probe_failure(""));
        assert!(is_ie_probe_failure("Target directory: C:\\out"));
        assert!(is_ie_probe_failure("0 FILES FOUND"));
        assert!(is_ie_probe_failure("Error opening archive"));
        assert!(is_ie_probe_failure("an exception occured here"));
        assert!(is_ie_probe_failure("format not supported"));
        assert!(is_ie_probe_failure("a crash occurred"));
        assert!(!is_ie_probe_failure("Extracting 12 files..."));
    }

    #[test]
    fn iso_probe_failure_detects_every_marker() {
        assert!(is_iso_probe_failure(""));
        assert!(is_iso_probe_failure("target directory: x"));
        assert!(is_iso_probe_failure("0 files found"));
        assert!(is_iso_probe_failure("exception occured"));
        assert!(is_iso_probe_failure("not supported by this WCX plugin"));
        assert!(!is_iso_probe_failure("Extracting ISO9660 image..."));
    }

    #[test]
    fn observer_probe_failure_detects_every_marker() {
        assert!(is_observer_probe_failure(
            "not supported by this WCX plugin"
        ));
        assert!(is_observer_probe_failure("0 files found"));
        assert!(is_observer_probe_failure("exception occured"));
        assert!(is_observer_probe_failure("EXCEPTION HANDLER triggered"));
        assert!(!is_observer_probe_failure("Extracting archive..."));
    }

    /// Parity test for capability C077: a real plugin/script selector
    /// resolves against `bindir`.
    #[test]
    fn resolve_plugin_path_uses_selector_when_present() {
        assert_eq!(
            resolve_plugin_path("InstExpl.wcx", r"C:\bin\", r"C:\bms\game.bms"),
            r"C:\bin\InstExpl.wcx"
        );
    }

    /// Parity test for capability C077: an empty selector falls back to
    /// the default `.bms` path.
    #[test]
    fn resolve_plugin_path_falls_back_to_default_bms_when_empty() {
        assert_eq!(
            resolve_plugin_path("", r"C:\bin\", r"C:\bms\game.bms"),
            r"C:\bms\game.bms"
        );
    }

    #[test]
    fn qbms_invocation_matches_source() {
        let inv = qbms_invocation(
            r"C:\bin\quickbms.exe",
            r"C:\bin\InstExpl.wcx",
            r"C:\downloads\installer.exe",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "-K".to_string(),
                r"C:\bin\InstExpl.wcx".to_string(),
                r"C:\downloads\installer.exe".to_string(),
                r"C:\downloads\unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C077: the InstallExplorer plugin match
    /// is case-sensitive (`==`), unlike the surrounding `StringInStr`
    /// probes.
    #[test]
    fn is_installexplorer_plugin_is_case_sensitive() {
        assert!(is_installexplorer_plugin("InstExpl.wcx", "InstExpl.wcx"));
        assert!(!is_installexplorer_plugin("instexpl.wcx", "InstExpl.wcx"));
        assert!(!is_installexplorer_plugin("Iso.wcx", "InstExpl.wcx"));
    }

    #[test]
    fn installexplorer_cleanup_targets_match_source() {
        assert_eq!(
            INSTALLEXPLORER_CLEANUP_TARGETS,
            &[
                "[NSIS].nsi",
                "[LICENSE].*",
                "$PLUGINSDIR",
                "$TEMP",
                "uninstall.exe",
                "[LICENSE]",
            ]
        );
    }
}
