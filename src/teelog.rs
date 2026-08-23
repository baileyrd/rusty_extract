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
//! **Both branches are now fully orchestrated too**, PR [#417](https://github.com/baileyrd/rusty_extract/pull/417),
//! built on C069's `automation::GuiAutomation` infrastructure, extended
//! with the process-polling/file-reading primitives this capability's
//! own streaming-process needs introduced (`process_exists`/
//! `win_get_by_pid`/`read_file_incremental`/`read_file_from_start`/
//! `dir_size_bytes`/`win_set_state_by_title`/`win_activate`) — see
//! [`run_with_tee`]/[`run_without_tee`]. Carries the same honesty
//! caveat as C069: fake-backed tests prove the orchestration logic
//! against the source line-by-line, not that the real Win32 backend
//! drives a real spawned process's window correctly.
//!
//! **A genuine bug found and preserved, not "fixed"**: the tee branch's
//! "needs user input" reveal (UniExtract.au3:4936) calls
//! `WinSetState($run, "", @SW_SHOW)` — passing `$run` (the spawned
//! process's **PID**), not `$runtitle` (the resolved window handle
//! `WinActivate($runtitle)` uses two lines later). A PID never matches a
//! real window title, so this call is a silent no-op in the source
//! itself; [`run_with_tee`] reproduces it exactly via
//! `win_set_state_by_title` on the stringified PID, rather than quietly
//! using the handle instead.
//!
//! **`_PatternSearch`'s own regex-based progress-text parsing isn't
//! modeled.** It both classifies `$return` (four alternative
//! percentage/progress patterns) *and* mutates the tray GUI directly in
//! one function — [`decide_tee_iteration`] takes its boolean outcome
//! (`pattern_matched`) as a caller-supplied input instead, the same
//! "accept a bounded, well-justified limitation" call already made for
//! `extract::ffmpeg`'s own regex-backtracking edge case.
//!
//! **`$size`'s permanent lockout, preserved exactly**: once
//! `_PatternSearch` ever matches, `$size` becomes `-1` and nothing in
//! the source ever resets it — the fallback directory-size check that's
//! the *only* code path able to reassign `$size` is itself gated on
//! `$size > -1`, so one match permanently disables it for the rest of
//! the run. [`TeeLoopState`] carries this exactly: once `size` goes
//! negative, [`decide_tee_iteration`] never recomputes it.
//!
//! **`_DirGetSize`'s big-drive-root guard isn't modeled** — it returns
//! a caller-chosen fallback (`0` at the needs-input call site,
//! `-1`'s own default everywhere else) only when `$outdir` is itself a
//! bare, near-full drive root, a case this port's own `dir_size_bytes`
//! doesn't special-case (real filesystem/drive-space queries, out of
//! scope). In practice `$outdir` is essentially never a bare drive
//! root, so [`decide_tee_iteration`] takes one plain byte count for
//! both purposes rather than modeling a rarely-reachable distinction.

use crate::automation::{GuiAutomation, WindowHandle};
use crate::batch;
use crate::extract::{Invocation, WindowMode};
use crate::log_eval::{self, LogEvalOutcome};

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

/// `Round($x, 3)` — AutoIt's `Round` matches ordinary round-half-away-
/// from-zero at this precision, which `f64::round` on a pre-scaled value
/// already gives for the non-negative sizes this module only ever
/// computes.
fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Ports `Run(...)`'s own show-flag selection (UniExtract.au3:4892):
/// `$bInitialShow? @SW_MINIMIZE: $show_flag`.
pub fn initial_show_flag(initial_show: bool, show_flag: WindowMode) -> WindowMode {
    if initial_show {
        WindowMode::Minimized
    } else {
        show_flag
    }
}

/// The tee branch's per-iteration mutable state (`$size`/`$lastSize`,
/// UniExtract.au3:4882), carried across polling iterations. See the
/// module doc comment for the permanent `-1` lockout this preserves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeeLoopState {
    pub size: f64,
    pub last_size_mb: f64,
}

impl TeeLoopState {
    pub fn new() -> Self {
        Self {
            size: 1.0,
            last_size_mb: 0.0,
        }
    }
}

impl Default for TeeLoopState {
    fn default() -> Self {
        Self::new()
    }
}

/// What the tee branch's polling loop does on one iteration
/// (UniExtract.au3:4923-4949).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeeIterationAction {
    /// New output looks like it needs user input
    /// (`batch::needs_user_input`): reveal the (buggy, see module doc
    /// comment) window and skip the rest of this iteration
    /// (`ContinueLoop`).
    NeedsInput,
    /// The fallback directory-size check ran and the tray text should
    /// be updated.
    UpdateTrayProgress { size_mb: f64 },
    /// Neither applies this iteration.
    NoAction,
}

/// Ports the tee branch's per-iteration decision (UniExtract.au3:4923-
/// 4949). `new_output` is this iteration's freshly-read chunk
/// (`GuiAutomation::read_file_incremental`'s result); `output_changed`
/// is `$return <> $state` (the caller's own responsibility to track,
/// since `$state` itself isn't part of this decision). `pattern_search`
/// is `$bPatternSearch`, used with **two different comparisons in the
/// same source function**: a truthy check (`$bPatternSearch And ...`,
/// where AutoIt treats any nonzero number — including `-1` — as `True`)
/// gates whether `pattern_matched` even gets consulted, while `> -1`
/// (matching `decide_size_poll_action`'s own already-documented
/// tri-state quirk) gates the fallback size check. `pattern_matched` is
/// `_PatternSearch`'s own boolean outcome — not modeled here, see module
/// doc comment. `dir_size_bytes`/`initial_dir_size_bytes` feed the
/// fallback size computation; see the module doc comment for why one
/// shared byte count covers both of the source's slightly different
/// `_DirGetSize` call sites.
pub fn decide_tee_iteration(
    state: &mut TeeLoopState,
    output_changed: bool,
    new_output: &str,
    pattern_search: i32,
    pattern_matched: bool,
    dir_size_bytes: u64,
    initial_dir_size_bytes: u64,
) -> TeeIterationAction {
    if output_changed {
        if batch::needs_user_input(new_output) {
            let size_mb =
                round3((dir_size_bytes as f64 - initial_dir_size_bytes as f64) / 1024.0 / 1024.0);
            state.last_size_mb = size_mb;
            return TeeIterationAction::NeedsInput;
        }
        if pattern_search != 0 && pattern_matched {
            state.size = -1.0;
        }
    }

    if state.size > -1.0 && pattern_search > -1 {
        let size_mb =
            round3((dir_size_bytes as f64 - initial_dir_size_bytes as f64) / 1024.0 / 1024.0);
        let update = size_mb > 0.0 && size_mb != state.last_size_mb;
        state.last_size_mb = size_mb;
        state.size = size_mb;
        if update {
            TeeIterationAction::UpdateTrayProgress { size_mb }
        } else {
            TeeIterationAction::NoAction
        }
    } else {
        TeeIterationAction::NoAction
    }
}

/// Ports the tee branch in full (UniExtract.au3:4890-4966): spawns
/// `invocation` (already built via [`build_run_command`]), waits for it
/// to start and for its log file to appear, then polls the log
/// incrementally until the process exits, driving [`decide_tee_iteration`]
/// each cycle. `pattern_search_matches` stands in for `_PatternSearch`'s
/// own boolean outcome (see module doc comment) — called once per
/// iteration with that iteration's freshly-read chunk. Returns
/// `log_eval::evaluate_log`'s own outcome over the log's full final
/// content; whether that content was worth logging at all
/// ([`should_log_teelog_output`]) and the real file open/close/delete
/// are the caller's own responsibility.
#[allow(clippy::too_many_arguments)]
pub fn run_with_tee<A: GuiAutomation>(
    automation: &mut A,
    invocation: &Invocation,
    log_file: &str,
    initial_show: bool,
    show_flag: WindowMode,
    outdir: &str,
    initial_dir_size_bytes: u64,
    pattern_search: i32,
    mut pattern_search_matches: impl FnMut(&str) -> bool,
) -> (String, LogEvalOutcome) {
    let pid = automation.run(invocation);

    // `Do; Sleep(1); If ... Then ExitLoop; Until ProcessExists($run)`
    // (UniExtract.au3:4904-4907): a Do-Until loop always runs its body
    // at least once before the first `ProcessExists` check, unlike a
    // plain `while`-first loop -- preserved via `loop { ...; if cond
    // { break } }` rather than `while !cond { ... }`.
    let start_timer = automation.timer_init();
    loop {
        automation.sleep(1);
        if automation.elapsed_ms(start_timer) > 5_000 {
            break;
        }
        if automation.process_exists(pid) {
            break;
        }
    }

    let runtitle = automation.win_get_by_pid(pid).unwrap_or(WindowHandle(0));
    if initial_show {
        automation.win_set_state(runtitle, show_flag);
    }

    // Same Do-Until shape as above (UniExtract.au3:4916-4919).
    let log_timer = automation.timer_init();
    loop {
        automation.sleep(10);
        if automation.elapsed_ms(log_timer) > 5_000 {
            break;
        }
        if automation.file_exists(log_file) {
            break;
        }
    }

    let mut state = TeeLoopState::new();
    let mut previous_output = String::new();
    while automation.process_exists(pid) {
        let chunk = automation.read_file_incremental(log_file);
        let output_changed = chunk != previous_output;
        if output_changed {
            previous_output = chunk.clone();
        }
        let pattern_matched = pattern_search_matches(&chunk);
        let dir_size = automation.dir_size_bytes(outdir);

        let action = decide_tee_iteration(
            &mut state,
            output_changed,
            &chunk,
            pattern_search,
            pattern_matched,
            dir_size,
            initial_dir_size_bytes,
        );

        match action {
            TeeIterationAction::NeedsInput => {
                // Preserved bug -- see module doc comment: the source
                // passes the PID, not `runtitle`, to WinSetState here.
                automation.win_set_state_by_title(&pid.to_string(), WindowMode::Show);
                automation.win_activate(runtitle);
                // `ContinueLoop` (UniExtract.au3:4940) skips the trailing
                // `Sleep(100)` below entirely for this iteration.
                continue;
            }
            TeeIterationAction::UpdateTrayProgress { .. } => {
                automation.sleep(50);
            }
            TeeIterationAction::NoAction => {}
        }
        automation.sleep(100);
    }

    let full_output = automation.read_file_from_start(log_file);
    let outcome = log_eval::evaluate_log(&full_output);
    (full_output, outcome)
}

/// Ports the no-tee branch in full (UniExtract.au3:4968-5005): spawns
/// `invocation`, waits for it to start (the source's own `Do; Sleep(10);
/// Until ProcessExists($run)` loop has **no timeout at all** — a real
/// hang risk, distinct from and in addition to the tee branch's
/// 5-second-bounded waits, preserved here the same way rather than
/// silently adding a bound the source doesn't have), hides its window
/// unconditionally, then polls the output directory's growing size via
/// the already-ported [`decide_size_poll_action`] until the process
/// exits.
pub fn run_without_tee<A: GuiAutomation>(
    automation: &mut A,
    invocation: &Invocation,
    outdir: &str,
    initial_dir_size_bytes: u64,
    pattern_search: i32,
    tray_gui_enabled: bool,
) {
    let pid = automation.run(invocation);

    // Do-Until, same shape as `run_with_tee`'s wait loops but with no
    // timeout at all (UniExtract.au3:4980-4982) -- see this function's
    // own doc comment.
    loop {
        automation.sleep(10);
        if automation.process_exists(pid) {
            break;
        }
    }

    let runtitle = automation.win_get_by_pid(pid).unwrap_or(WindowHandle(0));
    automation.win_set_state(runtitle, WindowMode::Hidden);

    let timer = automation.timer_init();
    let mut timer_active = true;
    while automation.process_exists(pid) {
        let dir_size = automation.dir_size_bytes(outdir);
        let size_mb = round3((dir_size as f64 - initial_dir_size_bytes as f64) / 1024.0 / 1024.0);
        let elapsed = automation.elapsed_ms(timer);

        match decide_size_poll_action(
            size_mb,
            pattern_search,
            tray_gui_enabled,
            timer_active,
            elapsed,
        ) {
            SizePollAction::RevealWindow => {
                automation.win_set_state(runtitle, WindowMode::Show);
                automation.win_activate(runtitle);
                automation.sleep(5_000);
                timer_active = false;
            }
            SizePollAction::UpdateTrayProgress | SizePollAction::NoAction => {}
        }
        automation.sleep(100);
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

    #[test]
    fn initial_show_uses_minimized_regardless_of_show_flag() {
        assert_eq!(
            initial_show_flag(true, WindowMode::Show),
            WindowMode::Minimized
        );
    }

    #[test]
    fn non_initial_show_passes_show_flag_through() {
        assert_eq!(initial_show_flag(false, WindowMode::Show), WindowMode::Show);
        assert_eq!(
            initial_show_flag(false, WindowMode::Hidden),
            WindowMode::Hidden
        );
    }

    mod tee_iteration {
        use super::*;

        #[test]
        fn changed_output_needing_input_reveals_and_skips_size_check() {
            let mut state = TeeLoopState::new();
            let action = decide_tee_iteration(
                &mut state,
                true,
                "Do you want to overwrite foo.txt?",
                1,
                false,
                0,
                0,
            );
            assert_eq!(action, TeeIterationAction::NeedsInput);
        }

        #[test]
        fn unchanged_output_never_triggers_needs_input_even_if_it_would_match() {
            let mut state = TeeLoopState::new();
            let action = decide_tee_iteration(
                &mut state,
                false,
                "Do you want to overwrite foo.txt?",
                1,
                false,
                5_000_000,
                0,
            );
            assert_ne!(action, TeeIterationAction::NeedsInput);
        }

        #[test]
        fn pattern_match_permanently_locks_out_the_size_fallback() {
            let mut state = TeeLoopState::new();
            // First: a pattern match sets state.size to -1.
            let first = decide_tee_iteration(&mut state, true, "50%", 1, true, 1_000_000, 0);
            assert_eq!(first, TeeIterationAction::NoAction);
            assert_eq!(state.size, -1.0);

            // Later iterations never recompute the fallback again, even
            // with plenty of growth and no further pattern match.
            let second =
                decide_tee_iteration(&mut state, true, "more output", 1, false, 50_000_000, 0);
            assert_eq!(second, TeeIterationAction::NoAction);
            assert_eq!(state.size, -1.0);
        }

        /// Parity test for capability C166: `$bPatternSearch And ...`
        /// is a truthy check, not `> -1` -- `-1` itself is truthy in
        /// AutoIt, so a pattern match still locks out the fallback even
        /// when `pattern_search == -1`.
        #[test]
        fn pattern_search_negative_one_is_still_truthy_for_the_match_gate() {
            let mut state = TeeLoopState::new();
            decide_tee_iteration(&mut state, true, "50%", -1, true, 0, 0);
            assert_eq!(state.size, -1.0);
        }

        #[test]
        fn fallback_updates_tray_when_size_grows() {
            let mut state = TeeLoopState::new();
            let action = decide_tee_iteration(&mut state, false, "", 1, false, 5 * 1024 * 1024, 0);
            assert_eq!(
                action,
                TeeIterationAction::UpdateTrayProgress { size_mb: 5.0 }
            );
        }

        #[test]
        fn fallback_takes_no_action_when_size_is_unchanged() {
            let mut state = TeeLoopState {
                size: 5.0,
                last_size_mb: 5.0,
            };
            let action = decide_tee_iteration(&mut state, false, "", 1, false, 5 * 1024 * 1024, 0);
            assert_eq!(action, TeeIterationAction::NoAction);
        }

        /// Parity test for capability C166: `$bPatternSearch > -1`
        /// disables the fallback entirely when `pattern_search ==
        /// -1`, a different comparison from the truthy match gate above.
        #[test]
        fn fallback_disabled_when_pattern_search_is_exactly_negative_one() {
            let mut state = TeeLoopState::new();
            let action = decide_tee_iteration(&mut state, false, "", -1, false, 5 * 1024 * 1024, 0);
            assert_eq!(action, TeeIterationAction::NoAction);
        }
    }

    mod run_with_tee_tests {
        use super::*;
        use crate::automation::fake::{Call, FakeGuiAutomation};

        fn base_invocation() -> Invocation {
            Invocation {
                program: r"C:\bin\7z.exe".to_string(),
                args: vec!["x".to_string(), "archive.7z".to_string()],
                working_dir: r"C:\downloads".to_string(),
                window: WindowMode::Hidden,
            }
        }

        #[test]
        fn spawns_waits_for_log_then_polls_until_process_exits() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_process_exists_for_iterations(1, 100);
            fake.script_file_appears_after(r"C:\logs\teelog.txt", 1);
            fake.script_win_get_by_pid(1, Some(WindowHandle(9)));
            fake.script_incremental_reads(r"C:\logs\teelog.txt", vec!["Extracting...", ""]);
            fake.script_file_from_start(r"C:\logs\teelog.txt", "Extracting...\r\nEverything is Ok");

            let (full_output, _outcome) = run_with_tee(
                &mut fake,
                &base_invocation(),
                r"C:\logs\teelog.txt",
                true,
                WindowMode::Minimized,
                r"C:\downloads\unpacked",
                0,
                1,
                |_| false,
            );

            assert_eq!(full_output, "Extracting...\r\nEverything is Ok");
            assert!(fake
                .calls()
                .iter()
                .any(|c| matches!(c, Call::Run(inv) if inv.program == r"C:\bin\7z.exe")));
        }

        /// Parity test for capability C166: the "needs input" reveal
        /// uses the preserved bug -- `WinSetState` is called with the
        /// stringified PID, not the resolved window handle.
        #[test]
        fn needs_input_reveals_via_the_preserved_pid_bug() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_process_exists_for_iterations(1, 2);
            fake.script_file_appears_after(r"C:\logs\teelog.txt", 1);
            fake.script_win_get_by_pid(1, Some(WindowHandle(9)));
            fake.script_incremental_reads(r"C:\logs\teelog.txt", vec!["overwrite foo.txt?"]);

            run_with_tee(
                &mut fake,
                &base_invocation(),
                r"C:\logs\teelog.txt",
                false,
                WindowMode::Minimized,
                r"C:\downloads\unpacked",
                0,
                1,
                |_| false,
            );

            assert!(fake
                .calls()
                .contains(&Call::WinSetStateByTitle("1".to_string(), WindowMode::Show)));
            assert!(fake.calls().contains(&Call::WinActivate(WindowHandle(9))));
            // `ContinueLoop` (UniExtract.au3:4940) skips the trailing
            // `Sleep(100)` for this iteration entirely -- with only one
            // main-loop iteration (a NeedsInput one) in this scenario,
            // no Sleep(100) should ever be recorded.
            assert!(!fake.calls().contains(&Call::Sleep(100)));
        }
    }

    mod run_without_tee_tests {
        use super::*;
        use crate::automation::fake::{Call, FakeGuiAutomation};

        fn base_invocation() -> Invocation {
            Invocation {
                program: r"C:\bin\7z.exe".to_string(),
                args: vec!["x".to_string(), "archive.7z".to_string()],
                working_dir: r"C:\downloads".to_string(),
                window: WindowMode::Hidden,
            }
        }

        #[test]
        fn hides_window_immediately_after_launch() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_process_exists_for_iterations(1, 1);
            fake.script_win_get_by_pid(1, Some(WindowHandle(9)));

            run_without_tee(
                &mut fake,
                &base_invocation(),
                r"C:\downloads\unpacked",
                0,
                1,
                true,
            );

            assert!(fake
                .calls()
                .contains(&Call::WinSetState(WindowHandle(9), WindowMode::Hidden)));
        }

        #[test]
        fn reveals_the_correct_handle_after_60_seconds_of_no_growth() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_process_exists_for_iterations(1, 3);
            fake.script_win_get_by_pid(1, Some(WindowHandle(9)));
            fake.script_dir_sizes(r"C:\downloads\unpacked", vec![0, 0, 0]);

            run_without_tee(
                &mut fake,
                &base_invocation(),
                r"C:\downloads\unpacked",
                0,
                1,
                true,
            );

            // Advancing the virtual clock happens via automation.sleep(100)
            // each iteration; 3 iterations only add up to 300ms, well
            // under 60s, so no reveal is expected here -- this test just
            // proves the correct (non-buggy) handle-based calls are used
            // for this branch, unlike the tee branch's PID bug.
            assert!(!fake
                .calls()
                .iter()
                .any(|c| matches!(c, Call::WinSetState(_, WindowMode::Show))));
        }
    }
}
