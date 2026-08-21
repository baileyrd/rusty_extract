//! Teelog dual-output mechanism (`_Run()`, UniExtract.au3:4880-5008) —
//! capability C166. When capturing a helper binary's live output is
//! enabled (`$bUseTee`), its stdout/stderr are piped through a
//! tee-like helper into a fixed log file, polled incrementally while
//! the process runs (progress display, and the "needs user input"
//! detection already ported as [`batch::needs_user_input`], C149),
//! then folded into the run log and deleted once the process exits.
//! When it's disabled, a different, simpler path just polls the output
//! directory's growing size instead, revealing the previously-hidden
//! window once if that size stays flat for too long.
//!
//! ```autoit
//! Func _Run($f, $sWorkingDir = $outdir, $show_flag = @SW_MINIMIZE, $bUseCmd = True, $bUseTee = True, $bPatternSearch = True, $bInitialShow = True)
//!     Local Const $LogFile = $logdir & "teelog.txt"
//!     $f = _MakeCommand($f, $bUseCmd) & ($bUseTee? ' 2>&1 | ' & $tee & ' "' & $LogFile & '"': '')
//!
//!     If $bUseTee Then
//!         ; ... spawn, poll the live log for progress/prompts (C149) ...
//!         Local $hFile = FileOpen($LogFile)
//!         While ProcessExists($run)
//!             ; ...
//!         WEnd
//!         ; Write tee log to UniExtract log file
//!         FileSetPos($hFile, 0, $FILE_BEGIN)
//!         $return = FileRead($hFile)
//!         If Not StringIsSpace($return) Then Cout("Teelog:" & @CRLF & $return)
//!         FileClose($hFile)
//!         FileDelete($LogFile)
//!         EvaluateLog($return)
//!     Else
//!         ; ... spawn, poll only $outdir's growing size ...
//!         While ProcessExists($run)
//!             If $size > 0 And $bPatternSearch > -1 Then
//!                 If $TBgui Then _SetTrayMessageBoxText($size & " MB")
//!             Else
//!                 If $TimerStart And TimerDiff($TimerStart) > 60000 Then
//!                     WinSetState($runtitle, "", @SW_SHOW)
//!                     WinActivate($runtitle)
//!                     Sleep(5000)
//!                     $TimerStart = 0
//!                 EndIf
//!             EndIf
//!             Sleep(100)
//!         WEnd
//!     EndIf
//! EndFunc
//! ```
//!
//! **Already ported elsewhere, not duplicated here**: `EvaluateLog()`
//! (`log_eval::evaluate_log`, C167/C144/C162/etc.) is what the tee
//! branch's captured output feeds into once read; the live "needs user
//! input" text scan inside the tee-branch polling loop is
//! `batch::needs_user_input` (C149). This module covers the pieces
//! around those: composing the tee command line, deciding whether the
//! captured output is worth logging at all, and the no-tee branch's own
//! entirely separate "reveal the window once" heuristic.
//!
//! **`_MakeCommand`'s own bindir-prefixing isn't modeled** — the same
//! scope note already made in `extract::iscab`/`extract::expand`;
//! [`build_run_command`] takes its result as an opaque, already-built
//! base command string.
//!
//! **Not modeled**: spawning the process, all `Win*`/`Process*`/
//! `Timer*`/`Sleep` calls, and the teelog file's own
//! open/read/close/delete — all real I/O and GUI automation.

/// Ports the final command-line composition (UniExtract.au3:4885):
/// appends the tee pipe suffix to `base_command` only when `use_tee` is
/// set; otherwise `base_command` passes through unchanged. `base_command`
/// is `_MakeCommand`'s own result — see the module doc comment.
pub fn build_run_command(
    base_command: &str,
    use_tee: bool,
    tee_program: &str,
    log_file: &str,
) -> String {
    if use_tee {
        format!("{base_command} 2>&1 | {tee_program} \"{log_file}\"")
    } else {
        base_command.to_string()
    }
}

/// Ports `If Not StringIsSpace($return) Then Cout("Teelog:" & @CRLF & $return)`
/// (UniExtract.au3:4963): the captured tee-log content is only worth
/// folding into the run log if it isn't empty/all-whitespace.
pub fn should_log_teelog_output(captured_output: &str) -> bool {
    !captured_output.trim().is_empty()
}

/// What the no-tee branch's size-polling loop does on each iteration
/// (UniExtract.au3:4991-4999).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizePollAction {
    /// Update the tray status text with the current size, in MB.
    UpdateTrayProgress,
    /// The output size has stayed flat for more than 60 seconds while
    /// the reveal timer is still active: show and activate the
    /// previously-hidden window. The caller must then disable the
    /// timer (matching `$TimerStart = 0`) so this only ever fires
    /// once per run — see [`decide_size_poll_action`]'s own doc
    /// comment.
    RevealWindow,
    /// Neither condition applies this iteration.
    NoAction,
}

/// Ports the no-tee branch's per-iteration decision
/// (UniExtract.au3:4991-4999). `pattern_search` is compared against
/// `-1` exactly as the source does (`$bPatternSearch > -1`) rather than
/// treated as a plain boolean — an explicit `-1` is the only value that
/// disables this check; both `0` (`False`) and `1` (`True`) satisfy it
/// the same way, a real quirk worth preserving rather than "cleaning up"
/// into `pattern_search: bool`.
///
/// **The reveal-once quirk**: `RevealWindow` is only reachable while
/// `timer_active` is `true`. The source disables further reveals by
/// zeroing `$TimerStart` right after showing the window
/// (UniExtract.au3:4998) — a caller must do the same (pass
/// `timer_active: false` on every subsequent call this run) to match;
/// this function has no memory of its own across calls.
pub fn decide_size_poll_action(
    size_mb: f64,
    pattern_search: i32,
    tray_gui_enabled: bool,
    timer_active: bool,
    elapsed_ms: u64,
) -> SizePollAction {
    if size_mb > 0.0 && pattern_search > -1 {
        if tray_gui_enabled {
            SizePollAction::UpdateTrayProgress
        } else {
            SizePollAction::NoAction
        }
    } else if timer_active && elapsed_ms > 60_000 {
        SizePollAction::RevealWindow
    } else {
        SizePollAction::NoAction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_run_command_appends_tee_pipe_when_enabled() {
        assert_eq!(
            build_run_command(
                "7z.exe x archive.7z",
                true,
                "tee.exe",
                r"C:\logs\teelog.txt"
            ),
            r#"7z.exe x archive.7z 2>&1 | tee.exe "C:\logs\teelog.txt""#
        );
    }

    #[test]
    fn build_run_command_passes_through_unchanged_when_disabled() {
        assert_eq!(
            build_run_command(
                "7z.exe x archive.7z",
                false,
                "tee.exe",
                r"C:\logs\teelog.txt"
            ),
            "7z.exe x archive.7z"
        );
    }

    #[test]
    fn should_log_teelog_output_requires_non_whitespace_content() {
        assert!(should_log_teelog_output(
            "Extracting archive.7z\r\nEverything is Ok"
        ));
        assert!(!should_log_teelog_output(""));
        assert!(!should_log_teelog_output("   \r\n  "));
    }

    #[test]
    fn size_poll_updates_tray_when_growing_and_pattern_search_active() {
        assert_eq!(
            decide_size_poll_action(1.5, 1, true, true, 0),
            SizePollAction::UpdateTrayProgress
        );
    }

    /// Parity test for capability C166: `$TBgui` gates the actual tray
    /// update even when the outer size/pattern-search condition holds
    /// -- no action at all, not a fallback to the reveal-window branch.
    #[test]
    fn size_poll_takes_no_action_when_tray_gui_disabled() {
        assert_eq!(
            decide_size_poll_action(1.5, 1, false, true, 70_000),
            SizePollAction::NoAction
        );
    }

    /// Parity test for capability C166: `pattern_search` is compared
    /// against `-1`, not treated as a plain boolean -- `0` still
    /// satisfies the outer condition the same as `1` does.
    #[test]
    fn size_poll_pattern_search_only_disabled_by_exact_negative_one() {
        assert_eq!(
            decide_size_poll_action(1.5, 0, true, true, 0),
            SizePollAction::UpdateTrayProgress
        );
        assert_eq!(
            decide_size_poll_action(1.5, -1, true, true, 70_000),
            SizePollAction::RevealWindow
        );
    }

    #[test]
    fn size_poll_reveals_window_after_60_seconds_of_no_growth() {
        assert_eq!(
            decide_size_poll_action(0.0, 1, true, true, 60_001),
            SizePollAction::RevealWindow
        );
        assert_eq!(
            decide_size_poll_action(0.0, 1, true, true, 59_999),
            SizePollAction::NoAction
        );
    }

    /// Parity test for capability C166: once the timer is disabled
    /// (the caller's own responsibility after a reveal), no further
    /// reveal happens even past the 60-second mark.
    #[test]
    fn size_poll_does_not_reveal_again_once_timer_disabled() {
        assert_eq!(
            decide_size_poll_action(0.0, 1, true, false, 120_000),
            SizePollAction::NoAction
        );
    }
}
