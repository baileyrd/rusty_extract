//! System tray icon lifecycle decisions, ported from `Tray_Create`,
//! `Tray_ShowHide`, and `Tray_Exit` (UniExtract.au3:8149-8197) —
//! capability C184.

/// Ports the `Tray_Statusbox`-checkbox source-of-truth `Tray_Create`
/// reads when building its menu (UniExtract.au3:8151): the "Hide
/// status" tray-menu item's initial checked state mirrors
/// `$bOptNoStatusBox` directly, not its negation — the item reads "Hide
/// status" and is checked precisely when status *is* hidden.
pub fn hide_status_item_checked(no_status_box_preference: bool) -> bool {
    no_status_box_preference
}

/// Ports `Tray_Create`'s icon-hide gate (UniExtract.au3:8160,
/// `Opt("TrayIconHide", 1)`): hiding the icon is purely cosmetic — the
/// menu items and their click handlers are still created and wired
/// regardless, matching the source (this function only decides whether
/// the icon itself should be visible, not whether the menu exists).
pub fn should_hide_icon(no_tray_icon_preference: bool) -> bool {
    no_tray_icon_preference
}

/// What `Tray_ShowHide` (UniExtract.au3:8164-8172) does to the spawned
/// helper's own console window — **not** the tray icon or the main
/// application window, an easy conflation the source's own naming
/// invites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleVisibilityAction {
    /// Hide the console (it was visible).
    Hide,
    /// Show and activate the console (it was hidden).
    ShowAndActivate,
    /// The helper process no longer exists; do nothing.
    NoOp,
}

/// Ports `Tray_ShowHide`'s decision: a no-op if the helper process isn't
/// running, otherwise a toggle based on the console window's current
/// visibility bit (`WinGetState`'s bit 2, `$WIN_STATE_VISIBLE`).
pub fn decide_console_visibility_action(
    helper_process_exists: bool,
    console_currently_visible: bool,
) -> ConsoleVisibilityAction {
    if !helper_process_exists {
        ConsoleVisibilityAction::NoOp
    } else if console_currently_visible {
        ConsoleVisibilityAction::Hide
    } else {
        ConsoleVisibilityAction::ShowAndActivate
    }
}

/// Ports `Tray_Exit`'s conditional status log (UniExtract.au3:8193-
/// 8195): `SaveLog($STATUS_TRAYEXIT)` only fires when the main window
/// was never created — a pure-tray/silent session exiting via the tray.
/// Exiting via the tray while the main window *is* open logs nothing
/// extra here; an easy-to-miss conditional, preserved exactly.
pub fn should_log_tray_exit(main_window_was_created: bool) -> bool {
    !main_window_was_created
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_status_item_mirrors_the_preference_directly() {
        assert!(hide_status_item_checked(true));
        assert!(!hide_status_item_checked(false));
    }

    #[test]
    fn hiding_the_icon_is_purely_a_visibility_toggle() {
        assert!(should_hide_icon(true));
        assert!(!should_hide_icon(false));
    }

    #[test]
    fn console_visibility_is_a_noop_when_helper_is_gone() {
        assert_eq!(
            decide_console_visibility_action(false, true),
            ConsoleVisibilityAction::NoOp
        );
        assert_eq!(
            decide_console_visibility_action(false, false),
            ConsoleVisibilityAction::NoOp
        );
    }

    /// Parity test for capability C184: with the helper still running,
    /// this is a plain visibility toggle.
    #[test]
    fn console_visibility_toggles_when_helper_is_running() {
        assert_eq!(
            decide_console_visibility_action(true, true),
            ConsoleVisibilityAction::Hide
        );
        assert_eq!(
            decide_console_visibility_action(true, false),
            ConsoleVisibilityAction::ShowAndActivate
        );
    }

    /// Parity test for capability C184: the tray-exit status log only
    /// fires for a pure-tray session, not when exiting the tray while
    /// the main window is still open.
    #[test]
    fn tray_exit_logs_only_when_main_window_was_never_created() {
        assert!(should_log_tray_exit(false));
        assert!(!should_log_tray_exit(true));
    }
}
