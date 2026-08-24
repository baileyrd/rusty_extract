//! Update-failure/restart/relaunch mechanics: ports `KillHelper`,
//! `Restart`, `RestartWithoutAdminRights`, `RepairProgramFiles`,
//! `_UpdateCheckFailed`, and the custom progress-dialog primitives
//! (`_ProgressOn`/`_ProgressSet`/`_ProgressOff`) the updater uses
//! (UniExtract.au3:5316-5347,5640-5670,5782-5791).
//!
//! This capability covers the pure decision gates and the one real command
//! string construction ([`build_restart_without_admin_command`]) in these
//! functions. The real actions themselves — `StdioClose`, `WinActivate`/
//! `Send`/`WinClose`, `ProcessClose`, `Run`, `terminate`, `MsgBox`, and the
//! actual GUI dialog creation — are I/O the caller performs at the points
//! these functions return a decision. `_DeleteFromArchDir` is already
//! ported (as a private helper) by [`crate::update_migration`]'s
//! `post_update_actions`; it isn't re-derived here.

/// Ports `KillHelper`'s initial guard (UniExtract.au3:5317): nothing to do
/// if there's no tracked running process.
pub fn should_kill_helper(has_run_handle: bool) -> bool {
    has_run_handle
}

/// Ports `If Not @error And Not StringIsSpace($runtitle) Then ...`
/// (UniExtract.au3:5321): attempt the graceful console-window shutdown
/// sequence (activate, `^c`, close) only if `StdioClose` itself didn't
/// error and a window title was actually recorded.
pub fn should_attempt_console_shutdown(
    stdio_close_succeeded: bool,
    runtitle_is_blank: bool,
) -> bool {
    stdio_close_succeeded && !runtitle_is_blank
}

/// Ports `If WinActive($runtitle) Then Send("^c")` (UniExtract.au3:5325):
/// only send the interrupt if the window actually became active.
pub fn should_send_ctrl_c(window_is_active: bool) -> bool {
    window_is_active
}

/// Ports `If ProcessExists($run) Then ProcessClose($run)`
/// (UniExtract.au3:5331): force-terminate only if the graceful shutdown
/// attempts above didn't already end the process.
pub fn should_force_kill_process(process_still_exists: bool) -> bool {
    process_still_exists
}

/// Ports `RestartWithoutAdminRights`'s `runas` command construction
/// (UniExtract.au3:5342): `$cmd & 'runas /trustlevel:0x20000 "' &
/// @ScriptFullPath & $sParameters & '"'`. `cmd` is the caller's resolved
/// shell prefix (`$cmd`, UniExtract.au3:96 — already ends with a trailing
/// space, e.g. `"cmd.exe /d /c "`).
///
/// **Verified bug, preserved rather than "fixed"**: `parameters` is spliced
/// directly inside the closing quote with no escaping of its own — a `"`
/// in `parameters` breaks out of the quoted path segment. It's also glued
/// onto the path with no separating space, so a caller must remember to
/// include a leading space in `parameters` themselves for it to read as a
/// separate argument rather than as part of the path. In practice, every
/// call site in the source passes an empty `parameters`, so this bug is
/// latent rather than triggered — but the function itself doesn't guard
/// against it.
pub fn build_restart_without_admin_command(
    cmd: &str,
    script_full_path: &str,
    parameters: &str,
) -> String {
    format!("{cmd}runas /trustlevel:0x20000 \"{script_full_path}{parameters}\"")
}

/// Ports `_UpdateCheckFailed`'s dialog gate (UniExtract.au3:5641):
/// `If Not $bSilent Then MsgBox(...)`. The function itself always returns
/// `False` regardless of this gate — a "no update found" sentinel, not a
/// decision — so only the dialog-visibility gate is modeled here.
pub fn should_show_update_check_failed_dialog(silent: bool) -> bool {
    !silent
}

/// Ports `RepairProgramFiles`'s confirmation gate (UniExtract.au3:5783):
/// proceeding with the repair (a silent helper-only `CheckUpdate` followed
/// by relaunching the app) requires the user to have clicked "Yes".
pub fn should_repair_program_files(user_confirmed_yes: bool) -> bool {
    user_confirmed_yes
}

/// The outcome of calling `_ProgressOn` (UniExtract.au3:5646-5654) against
/// [`ProgressState`]'s single tracked slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressOnOutcome {
    /// No progress dialog was already open; this call opens the only one.
    Opened,
    /// **Verified architectural constraint, preserved rather than
    /// "fixed"**: a progress dialog was already open when this call
    /// happened. `$hProgress`/`$idProgress` are bare globals with no
    /// stacking or nesting support — `_ProgressOn` overwrites them with
    /// the new dialog's handles, and the *previous* dialog window is
    /// never closed. It's simply orphaned: `_ProgressOff` can from then
    /// on only ever close the most recently opened one.
    ReplacedWithoutClosingPrevious,
}

/// A minimal state machine mirroring the single global progress-dialog
/// slot `_ProgressOn`/`_ProgressOff` share. Doesn't render anything — it
/// only tracks whether a dialog is considered "open", so a caller wiring
/// this up to a real window can detect (and decide how to handle) the
/// orphaning scenario the original code allows silently.
#[derive(Debug, Default)]
pub struct ProgressState {
    open: bool,
}

impl ProgressState {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Ports `_ProgressOn`. Always transitions to "open"; the return value
    /// tells the caller whether a previous dialog was silently orphaned.
    pub fn on(&mut self) -> ProgressOnOutcome {
        let outcome = if self.open {
            ProgressOnOutcome::ReplacedWithoutClosingPrevious
        } else {
            ProgressOnOutcome::Opened
        };
        self.open = true;
        outcome
    }

    /// Ports `_ProgressOff` (UniExtract.au3:5663): `GUIDelete($hProgress)`.
    pub fn off(&mut self) {
        self.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_restart_without_admin_command, should_attempt_console_shutdown,
        should_force_kill_process, should_kill_helper, should_repair_program_files,
        should_send_ctrl_c, should_show_update_check_failed_dialog, ProgressOnOutcome,
        ProgressState,
    };

    #[test]
    fn kill_helper_skipped_without_a_run_handle() {
        assert!(!should_kill_helper(false));
        assert!(should_kill_helper(true));
    }

    #[test]
    fn console_shutdown_requires_clean_close_and_a_recorded_title() {
        assert!(should_attempt_console_shutdown(true, false));
        assert!(!should_attempt_console_shutdown(false, false));
        assert!(!should_attempt_console_shutdown(true, true));
    }

    #[test]
    fn ctrl_c_only_sent_to_an_active_window() {
        assert!(should_send_ctrl_c(true));
        assert!(!should_send_ctrl_c(false));
    }

    #[test]
    fn force_kill_only_when_process_survived_graceful_shutdown() {
        assert!(should_force_kill_process(true));
        assert!(!should_force_kill_process(false));
    }

    #[test]
    fn restart_command_matches_source_concatenation_exactly() {
        let cmd = "cmd.exe /d /c ";
        let path = "C:\\App\\UniExtract.exe";
        assert_eq!(
            build_restart_without_admin_command(cmd, path, ""),
            "cmd.exe /d /c runas /trustlevel:0x20000 \"C:\\App\\UniExtract.exe\""
        );
    }

    /// The verified bug: an unescaped quote in parameters breaks out of
    /// the quoted path segment, and there's no automatic separating space.
    #[test]
    fn restart_command_does_not_escape_or_separate_parameters() {
        let cmd = "cmd.exe /d /c ";
        let path = "C:\\App\\UniExtract.exe";
        let built = build_restart_without_admin_command(cmd, path, "\" & calc.exe & \"");
        assert_eq!(
            built,
            "cmd.exe /d /c runas /trustlevel:0x20000 \"C:\\App\\UniExtract.exe\" & calc.exe & \"\""
        );
        // A well-behaved caller must supply its own leading space.
        let with_flag = build_restart_without_admin_command(cmd, path, " /main");
        assert!(with_flag.contains("UniExtract.exe /main\""));
    }

    #[test]
    fn update_check_failed_dialog_only_shown_when_not_silent() {
        assert!(should_show_update_check_failed_dialog(false));
        assert!(!should_show_update_check_failed_dialog(true));
    }

    #[test]
    fn repair_program_files_requires_explicit_yes() {
        assert!(should_repair_program_files(true));
        assert!(!should_repair_program_files(false));
    }

    #[test]
    fn progress_state_tracks_open_closed() {
        let mut state = ProgressState::new();
        assert!(!state.is_open());
        assert_eq!(state.on(), ProgressOnOutcome::Opened);
        assert!(state.is_open());
        state.off();
        assert!(!state.is_open());
    }

    /// The verified architectural constraint: opening again while already
    /// open silently orphans the previous dialog rather than erroring.
    #[test]
    fn progress_state_flags_reopen_without_closing_previous() {
        let mut state = ProgressState::new();
        state.on();
        assert_eq!(
            state.on(),
            ProgressOnOutcome::ReplacedWithoutClosingPrevious
        );
        assert!(state.is_open());
    }
}
