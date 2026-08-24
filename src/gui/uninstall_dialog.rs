//! Uninstall confirmation dialog: ports `GUI_Uninstall`
//! (UniExtract.au3:7427-7458) — the interactive "remove logs"/"remove
//! user data" checkbox dialog gating the actual uninstall sequence
//! (`crate::uninstall::resolve_uninstall_steps`, C216).
//!
//! This dialog has almost no computed logic: two checkboxes with fixed
//! default states, and a message loop with exactly one exit condition.
//! Once it exits, the caller reads the two checkbox states directly and
//! passes them straight to [`crate::uninstall::resolve_uninstall_steps`] —
//! there's no separate "resolve the outcome" function to write here
//! beyond that composition point, since the source performs no
//! transformation on the checkbox values at all (UniExtract.au3:7451-7457).

/// Ports `GUICtrlSetState(-1, $GUI_CHECKED)` applied to the "remove logs"
/// checkbox only (UniExtract.au3:7435-7436) — it starts checked; "remove
/// user data" (UniExtract.au3:7437) has no such call and starts
/// unchecked.
pub const REMOVE_LOGS_DEFAULT_CHECKED: bool = true;
pub const REMOVE_USER_DATA_DEFAULT_CHECKED: bool = false;

/// Ports the message loop's sole exit condition (UniExtract.au3:7447-7449):
/// `While 1 ... If GUIGetMsg() == $idOk Then ExitLoop`. There is no other
/// exit condition anywhere in the source — no Cancel button, and (see
/// [`DialogEscapeHatch`]) the system Close button is explicitly disabled.
pub fn should_exit_dialog_loop(clicked_uninstall_button: bool) -> bool {
    clicked_uninstall_button
}

/// **A UX decision for the port to make deliberately — not to silently
/// replicate or silently "fix".** The source disables the dialog's system
/// Close (X) button and provides no Cancel button at all
/// (UniExtract.au3:7441-7442, `_GUICtrlMenu_EnableMenuItem($hMenu,
/// $SC_CLOSE, $MF_GRAYED, False)`): clicking "Uninstall" is the *only* way
/// to leave this dialog once it's open.
///
/// Preserving this exactly means the real window (egui or otherwise) must
/// also disable its close affordance and not treat Esc as cancel — easy
/// for a toolkit's defaults to quietly undo without anyone deciding to.
/// Diverging from it (adding a real way to back out) is also a reasonable
/// call, but it's a new control-flow path the original never had. This
/// type exists so wiring the real dialog requires picking one of these
/// explicitly, rather than the choice being made by omission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogEscapeHatch {
    /// Preserve the source exactly: no close button, no Cancel, no Esc —
    /// "Uninstall" is the only exit.
    NoneMatchingSource,
    /// Deliberately diverge: allow the user to back out via Cancel/Esc/
    /// the window's own close button.
    AllowCancel,
}

#[cfg(test)]
mod tests {
    use super::{should_exit_dialog_loop, DialogEscapeHatch};

    #[test]
    fn dialog_loop_only_exits_on_the_uninstall_button() {
        assert!(should_exit_dialog_loop(true));
        assert!(!should_exit_dialog_loop(false));
    }

    #[test]
    fn escape_hatch_choice_is_an_explicit_two_way_decision() {
        assert_ne!(
            DialogEscapeHatch::NoneMatchingSource,
            DialogEscapeHatch::AllowCancel
        );
    }
}
