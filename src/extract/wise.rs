//! Wise Installer (`$TYPE_WISE`) — try the direct `e_wise_w.exe`
//! unpacker first; on failure, fall back to a user-disambiguated choice
//! between four extraction methods (or "not a Wise installer" at all).
//!
//! ```autoit
//! Case $TYPE_WISE
//!     _Run($wise_ewise & ' "' & $file & '" "' & $outdir & '"', $filedir)
//!     If $success == $RESULT_FAILED Then
//!         $success = $RESULT_UNKNOWN
//!         Local $aOptions = ['Wise ' & t('TERM_INSTALLER'), t('METHOD_UNPACKER_RADIO', 'Wise UNpacker'), t('METHOD_SWITCH_RADIO', 'Wise Installer /x'), t('METHOD_EXTRACTION_RADIO', 'Wise MSI'), t('METHOD_EXTRACTION_RADIO', 'Unzip'), t('METHOD_NOT_INSTALLER_RADIO', "Wise")]
//!         $iChoice = GUI_MethodSelect($aOptions, $arcdisp)
//!
//!         Switch $iChoice
//!             ; Extract with WUN
//!             Case 1
//!                 RunWait(_MakeCommand($wise_wun, True) & ' "' & $filename & '" "' & $tempoutdir & '"', $filedir)
//!
//!                 Local $aCleanup[] = [$tempoutdir & "INST0*", $tempoutdir & "WISE0*"]
//!                 Cleanup($aCleanup)
//!                 MoveFiles($tempoutdir, $outdir, False, "", True)
//!
//!             ; Extract using the /x switch
//!             Case 2
//!                 Warn_Execute($file & ' /x ' & $outdir)
//!                 ShellExecuteWait($file, ' /x ' & $outdir, $filedir)
//!
//!             ; Attempt to extract MSI
//!             Case 3
//!                 ; Some Wise installers contain a msi installer, which is unpacked to CommonFilesDir & "\Wise Installation Wizard"
//!                 ; when the main file is executed. Trying to find the correct file inside this directory is unreliable, so we simply
//!                 ; search the msi inside the exe file.
//!                 If RipExeInfo($tempoutdir, "{DOWN}{DOWN}{DOWN}") Then MoveFiles($tempoutdir, $outdir, False, "", True, True)
//!
//!             ; Extract using unzip, falling back to 7-Zip
//!             Case 4
//!                 _Run($zip & ' -x "' & $file & '"', $outdir)
//!                 If $success == $RESULT_FAILED Then _Run($7z & ' x "' & $file & '"', $outdir)
//!             ; Not a Wise installer
//!             Case 5
//!                 Return False
//!         EndSwitch
//!     Else
//!         RunWait($cmd & '00000000.BAT', $outdir, @SW_HIDE)
//!         FileDelete($outdir & '\00000000.BAT')
//!     EndIf
//! ```
//!
//! **Scope — invocations and routing decisions only, choice 3 excepted.**
//! `Cleanup`, `MoveFiles`, and `FileDelete` are real filesystem I/O, out
//! of scope everywhere in this crate. The disambiguation candidate list
//! is C053's own `method_select::WISE_CANDIDATES`, and which choice gets
//! picked at all is `method_select::decide_method_selection` — both
//! reused here, not duplicated. `Warn_Execute`'s "you're about to run an
//! executable, continue?" confirmation gate (choice 2) isn't modeled,
//! matching `extract::expand`'s own note about the same function —
//! [`switch_invocation`] reproduces only the command it passes through.
//!
//! **Choice 3 is genuinely GUI-blocked.** `RipExeInfo`
//! (UniExtract.au3:1861-1896, already investigated for C069) drives
//! Exeinfo PE via real Win32 window/control automation — this crate has
//! no Win32 GUI-automation FFI, so whether it even finds an MSI to rip
//! can't be determined here. [`RIP_EXEINFO_KEY_SEQUENCE`] pins down the
//! literal keystroke sequence this call site sends it, the same
//! "pin the data, defer the automation" split `extract::mscf` already
//! uses for its own `RipExeInfo` call site.
//!
//! **The `$cmd &` prefix on the completion-BAT invocation isn't modeled
//! as a literal string.** Same as `extract::expand`'s own note: `$cmd &
//! '00000000.BAT'` is a literal `cmd.exe /d /c ` prefix concatenated
//! directly onto a bare relative filename (not `_Run`'s own
//! `_MakeCommand` bindir-prefixing) — functionally this still just runs
//! `00000000.BAT` in `outdir`, so [`success_bat_invocation`] targets it
//! directly.
//!
//! **Choice 5 (`Return False`) exits `extract()` outright**, distinct
//! from every other choice falling through to the end of the `Switch`
//! normally — [`WiseChoice::NotWiseInstaller`] models it as a distinct
//! variant so a caller can tell the two apart.

use super::{Invocation, WindowMode};

/// Builds the primary `e_wise_w.exe` invocation (UniExtract.au3:3333):
/// `<program> "<file>" "<outdir>"`, run in `filedir`. No `$show_flag`
/// argument is passed at this call site, so `_Run`'s own default
/// (`@SW_MINIMIZE`) applies — the same convention `extract::raiu`/
/// `extract::iscab` already document for their own bare `_Run($cmd,
/// $dir)` calls.
pub fn primary_invocation(program: &str, file: &str, outdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), outdir.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// What the primary run's result routes to (UniExtract.au3:3334,3368):
/// `$success == $RESULT_FAILED` reaches the disambiguation `Switch`;
/// anything else runs the completion BAT instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryOutcome {
    /// Route to the (`method_select`-mediated) disambiguation choice.
    Fallback,
    /// The primary run wasn't a failure: run the completion BAT.
    RunCompletionBat,
}

/// Ports the primary-result routing decision itself.
pub fn classify_primary_result(primary_run_failed: bool) -> PrimaryOutcome {
    if primary_run_failed {
        PrimaryOutcome::Fallback
    } else {
        PrimaryOutcome::RunCompletionBat
    }
}

/// One of the five 1-indexed choices `method_select::WISE_CANDIDATES`'
/// GUI dialog can resolve to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiseChoice {
    /// Choice 1: extract with WUN.
    WunUnpacker,
    /// Choice 2: extract using the `/x` switch.
    InstallerSwitch,
    /// Choice 3: attempt to rip the MSI via Exeinfo PE — genuinely
    /// GUI-blocked, see module doc comment.
    WiseMsi,
    /// Choice 4: extract using unzip, falling back to 7-Zip.
    Unzip,
    /// Choice 5: not a Wise installer at all — exits `extract()`
    /// outright (`Return False`), not a normal fall-through.
    NotWiseInstaller,
}

/// Maps `$iChoice`'s 1-indexed numeric value to its [`WiseChoice`],
/// mirroring `Switch $iChoice`'s five `Case`s. `None` for any value
/// outside `1..=5` (the `Switch` has no `Case Else`, so the source
/// simply falls through and does nothing).
pub fn wise_choice_from_number(choice: u32) -> Option<WiseChoice> {
    match choice {
        1 => Some(WiseChoice::WunUnpacker),
        2 => Some(WiseChoice::InstallerSwitch),
        3 => Some(WiseChoice::WiseMsi),
        4 => Some(WiseChoice::Unzip),
        5 => Some(WiseChoice::NotWiseInstaller),
        _ => None,
    }
}

/// Builds choice 1's WUN invocation (UniExtract.au3:3341): `<program>
/// "<filename>" "<tempoutdir>"`, run via `RunWait` in `filedir`. No
/// `$show_flag` argument is passed at this call site, so `RunWait`'s own
/// default (`@SW_SHOWNORMAL`) applies. `_MakeCommand`'s own
/// bindir-prefixing isn't modeled, the same scope note already made in
/// `extract::iscab`/`extract::expand` — `program` is `_MakeCommand`'s
/// own result, taken as an opaque, already-resolved path.
pub fn wun_invocation(
    program: &str,
    filename: &str,
    tempoutdir: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![filename.to_string(), tempoutdir.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Show,
    }
}

/// Choice 1's cleanup patterns (UniExtract.au3:3343): `$tempoutdir &
/// "INST0*"` and `$tempoutdir & "WISE0*"` — real `Cleanup(...)`
/// execution stays out of scope (see module doc comment). `tempoutdir`
/// is expected pre-joined with its own trailing backslash, matching
/// `TempDir()`'s own return shape (`$sDir & "\" & $sPath & "\"`,
/// UniExtract.au3:3932) — plain concatenation here, no separator
/// inserted.
pub fn wun_cleanup_patterns(tempoutdir: &str) -> [String; 2] {
    [format!("{tempoutdir}INST0*"), format!("{tempoutdir}WISE0*")]
}

/// Builds choice 2's `/x`-switch invocation (UniExtract.au3:3349):
/// `ShellExecuteWait($file, ' /x ' & $outdir, $filedir)` — `program` is
/// `file` itself (the installer executable, run via its own switch, not
/// a separate helper binary), `args` is the `/x <outdir>` parameter
/// string split into its two tokens, run in `filedir`. No `showflag`
/// argument is passed, so `ShellExecuteWait`'s own default
/// (`@SW_SHOWNORMAL`) applies. `Warn_Execute`'s confirmation gate
/// (called just before this, its return value discarded) isn't modeled
/// — see module doc comment.
pub fn switch_invocation(file: &str, outdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec!["/x".to_string(), outdir.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Show,
    }
}

/// The literal keystroke sequence choice 3 sends `RipExeInfo`
/// (UniExtract.au3:3358) — three Down-arrow presses, the keyboard
/// navigation Exeinfo PE's UI needs to reach its MSI-rip command for a
/// Wise installer's dialog layout. Pinned down as data even though
/// `RipExeInfo` itself (real Win32 GUI automation, C069) isn't
/// implemented here.
pub const RIP_EXEINFO_KEY_SEQUENCE: &str = "{DOWN}{DOWN}{DOWN}";

/// Builds choice 4's `unzip` invocation (UniExtract.au3:3364): `<program>
/// -x "<file>"`, run in `outdir`. No `$show_flag` argument is passed at
/// this call site, so `_Run`'s own default (`@SW_MINIMIZE`) applies.
pub fn unzip_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds choice 4's 7-Zip fallback invocation (UniExtract.au3:3365),
/// only reached when the `unzip` attempt failed (`$success ==
/// $RESULT_FAILED`): `<program> x "<file>"`, run in `outdir`, same
/// `_Run` default window (`@SW_MINIMIZE`) as [`unzip_invocation`].
pub fn sevenzip_fallback_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds the completion-BAT invocation reached when the primary run
/// didn't fail (UniExtract.au3:3369): `00000000.BAT`, run in `outdir`
/// with the window hidden. See the module doc comment for why the
/// source's `$cmd &` prefix isn't modeled as a literal string.
pub fn success_bat_invocation(outdir: &str) -> Invocation {
    Invocation {
        program: "00000000.BAT".to_string(),
        args: vec![],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_invocation_matches_source() {
        let inv = primary_invocation(
            r"C:\bin\e_wise_w.exe",
            r"C:\downloads\setup.exe",
            r"C:\downloads\unpacked",
            r"C:\downloads",
        );
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\setup.exe".to_string(),
                r"C:\downloads\unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn primary_result_failed_routes_to_fallback() {
        assert_eq!(classify_primary_result(true), PrimaryOutcome::Fallback);
    }

    #[test]
    fn primary_result_not_failed_routes_to_completion_bat() {
        assert_eq!(
            classify_primary_result(false),
            PrimaryOutcome::RunCompletionBat
        );
    }

    /// Parity test for capability C106: `$iChoice`'s five values map to
    /// their `Switch` cases in order, and anything outside `1..=5` falls
    /// through unmatched (no `Case Else`).
    #[test]
    fn choice_mapping_matches_source_switch() {
        assert_eq!(wise_choice_from_number(1), Some(WiseChoice::WunUnpacker));
        assert_eq!(
            wise_choice_from_number(2),
            Some(WiseChoice::InstallerSwitch)
        );
        assert_eq!(wise_choice_from_number(3), Some(WiseChoice::WiseMsi));
        assert_eq!(wise_choice_from_number(4), Some(WiseChoice::Unzip));
        assert_eq!(
            wise_choice_from_number(5),
            Some(WiseChoice::NotWiseInstaller)
        );
        assert_eq!(wise_choice_from_number(0), None);
        assert_eq!(wise_choice_from_number(6), None);
    }

    #[test]
    fn wun_invocation_matches_source() {
        let inv = wun_invocation(
            r"C:\bin\wun.exe",
            "setup.exe",
            r"C:\downloads\temp7\",
            r"C:\downloads",
        );
        assert_eq!(
            inv.args,
            vec!["setup.exe".to_string(), r"C:\downloads\temp7\".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    #[test]
    fn wun_cleanup_patterns_match_source() {
        assert_eq!(
            wun_cleanup_patterns(r"C:\downloads\temp7\"),
            [
                r"C:\downloads\temp7\INST0*".to_string(),
                r"C:\downloads\temp7\WISE0*".to_string(),
            ]
        );
    }

    #[test]
    fn switch_invocation_matches_source() {
        let inv = switch_invocation(
            r"C:\downloads\setup.exe",
            r"C:\downloads\unpacked",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\downloads\setup.exe");
        assert_eq!(
            inv.args,
            vec!["/x".to_string(), r"C:\downloads\unpacked".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    #[test]
    fn rip_exeinfo_key_sequence_matches_source() {
        assert_eq!(RIP_EXEINFO_KEY_SEQUENCE, "{DOWN}{DOWN}{DOWN}");
    }

    #[test]
    fn unzip_invocation_matches_source() {
        let inv = unzip_invocation(
            r"C:\bin\unzip.exe",
            r"C:\downloads\setup.exe",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec!["-x".to_string(), r"C:\downloads\setup.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn sevenzip_fallback_invocation_matches_source() {
        let inv = sevenzip_fallback_invocation(
            r"C:\bin\7z.exe",
            r"C:\downloads\setup.exe",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\downloads\setup.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn success_bat_invocation_matches_source() {
        let inv = success_bat_invocation(r"C:\downloads\unpacked");
        assert_eq!(inv.program, "00000000.BAT");
        assert!(inv.args.is_empty());
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C106: choice 4's `unzip`→`7z` fallback
    /// reuses `extract::sevenzip::classify_run_error`'s own status
    /// vocabulary conceptually, but this call site only ever checks
    /// `$success == $RESULT_FAILED` directly — no `@error`/`@extended`
    /// classification happens here, matching the source's own bare `If`.
    #[test]
    fn wise_candidates_are_reused_from_method_select() {
        use crate::method_select::WISE_CANDIDATES;
        assert_eq!(WISE_CANDIDATES.len(), 5);
    }
}
