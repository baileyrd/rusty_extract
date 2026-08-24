//! Pre-execution warning gate — capability C189. Ports `Warn_Execute`'s
//! decision (UniExtract.au3:6219-6224), the wrapper called before ~13
//! self-extracting-installer `Case`s actually run their target
//! executable, and `GUI_Warn_Execute`'s "don't ask again" persistence
//! (UniExtract.au3:6227-6261). Mirrors `free_space::decide_prompt_action`'s
//! own `AbortAndTerminateSilently { remove_created_outdir }` shape
//! (capability C179) — the same `$createdir`-gated cleanup pattern, on a
//! decline instead of an insufficient-space abort.
//!
//! **The real confirm/cancel dialog itself stays unwired.** Every one of
//! `Warn_Execute`'s call sites sits deep in the extraction dispatch table
//! (self-extracting installer cases), which this port's GUI doesn't drive
//! at all yet — the OK button (C183/C186) and the Batch button's Run
//! branch (C188) don't invoke real extraction either, for the same
//! reason: no detection cascade (C037-046) is wired into the GUI. Adding
//! a real popup with no caller that could ever trigger it would be dead
//! code, not a meaningful port; this capability stays pure-decision-logic
//! only until the GUI actually drives an extraction.
//!
//! **Verified, not fixed: a real source bug in `GUI_Warn_Execute`.** It
//! sets `Opt("GUIOnEventMode", 0)` on entry (UniExtract.au3:6231) but
//! sets it to `0` again on exit (UniExtract.au3:6258) instead of restoring
//! `1` — unlike `GUI_Batch_Show`'s equivalent dialog (UniExtract.au3:6650,
//! 6700), which does restore it correctly. This looks like a copy-paste
//! typo left uncorrected in the source. It's moot for this port, though:
//! `egui`'s immediate-mode `update()` loop has no equivalent of AutoIt's
//! `GUIOnEventMode` global event-dispatch toggle to get wrong in the
//! first place — the same class of "old workaround made moot by the new
//! toolkit" as C183's DPI-scaling note, C185's tooltip-workaround note,
//! and C187's `WM_DROPFILES_UNICODE_FUNC` note.

/// What `Warn_Execute` does once it knows whether the dialog was even
/// shown and, if so, how the user responded (UniExtract.au3:6219-6224).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarnExecuteOutcome {
    /// `Return $sCommand` — the command line is returned unchanged and
    /// the caller runs it.
    Proceed,
    /// `If $createdir Then DirRemove($outdir, 0)` then
    /// `terminate($STATUS_SILENT)` — remove the output directory only if
    /// this run created it, then terminate silently either way.
    AbortAndTerminateSilently { remove_created_outdir: bool },
}

/// Ports `Warn_Execute`'s short-circuit dispatch
/// (UniExtract.au3:6220): warning disabled skips the dialog outright and
/// always proceeds; only a shown-and-declined dialog aborts.
/// `user_chose_continue` is meaningless when `warn_execute_enabled` is
/// `false` (the dialog is never shown in that case) — same short-circuit
/// contract as the source's own `Or`.
pub fn decide_warn_execute_outcome(
    warn_execute_enabled: bool,
    user_chose_continue: bool,
    created_outdir_this_run: bool,
) -> WarnExecuteOutcome {
    if !warn_execute_enabled || user_chose_continue {
        WarnExecuteOutcome::Proceed
    } else {
        WarnExecuteOutcome::AbortAndTerminateSilently {
            remove_created_outdir: created_outdir_this_run,
        }
    }
}

/// Ports `GUI_Warn_Execute`'s "don't ask again" persistence
/// (UniExtract.au3:6252-6255): checking the box permanently disables the
/// warning (`$bOptWarnExecute = 0`, `SavePref("warnexecute", 0)`) for
/// every future run, not just this one.
pub fn should_disable_warn_execute_permanently(remember_checked: bool) -> bool {
    remember_checked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_warning_always_proceeds_without_asking() {
        assert_eq!(
            decide_warn_execute_outcome(false, false, true),
            WarnExecuteOutcome::Proceed
        );
        assert_eq!(
            decide_warn_execute_outcome(false, false, false),
            WarnExecuteOutcome::Proceed
        );
    }

    #[test]
    fn enabled_warning_with_continue_proceeds() {
        assert_eq!(
            decide_warn_execute_outcome(true, true, true),
            WarnExecuteOutcome::Proceed
        );
    }

    /// Parity test: a decline only removes the output directory when
    /// this run actually created it.
    #[test]
    fn enabled_warning_declined_aborts_and_removes_only_if_created() {
        assert_eq!(
            decide_warn_execute_outcome(true, false, true),
            WarnExecuteOutcome::AbortAndTerminateSilently {
                remove_created_outdir: true
            }
        );
        assert_eq!(
            decide_warn_execute_outcome(true, false, false),
            WarnExecuteOutcome::AbortAndTerminateSilently {
                remove_created_outdir: false
            }
        );
    }

    #[test]
    fn dont_ask_again_mirrors_the_checkbox_state() {
        assert!(should_disable_warn_execute_permanently(true));
        assert!(!should_disable_warn_execute_permanently(false));
    }
}
