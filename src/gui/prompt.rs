//! Generic prompt/confirm dialogs — capability C193. Ports `Prompt`
//! (UniExtract.au3:5980-5993) and `CustomPrompt` (UniExtract.au3:5996-
//! 6033), the two generic confirmation primitives many already-DONE
//! capabilities reference as a deferred GUI dependency (`cleanup`'s
//! `user_confirmed_delete` for C158, `batch`'s `user_confirmed_duplicate`
//! for C147) — this is where those stand-ins finally get their own real
//! decision logic ported.
//!
//! `_IsChecked`/`_IsAnyChecked`/`_SetState` (UniExtract.au3:6049-6071)
//! aren't ported as functions here: they're Win32 control-state plumbing
//! (a bitmask test against `GUICtrlRead`, a loop over a control-ID array
//! calling `GUICtrlSetState`) that only exists because AutoIt represents
//! a checkbox's checked state as a bit in an opaque control handle's
//! state integer. This port's checkboxes are plain `bool` fields bound
//! directly by `egui::Checkbox` — there is no control-state integer to
//! bit-test or array of IDs to loop over, so these three helpers are moot
//! under the new toolkit, the same class of supersession as C183's
//! DPI-scaling note, C185's tooltip-workaround note, C187's
//! `WM_DROPFILES_UNICODE_FUNC` note, C189's `GUIOnEventMode` note, and
//! C190's window-recreate note.
//!
//! **Neither dialog is wired to a real window here** — same treatment as
//! every other dialog this migration phase has ported so far. What's
//! real and useful today is `Prompt`'s silent-mode auto-affirm (already
//! load-bearing: several DONE capabilities' `user_confirmed_*`
//! parameters are this behavior standing in for a real dialog) and
//! `CustomPrompt`'s persisted Always/Never short-circuit.

/// Ports `Prompt`'s `$return == 1 Or $return == 6` check
/// (UniExtract.au3:5986): `MsgBox`'s `IDOK` (1) and `IDYES` (6) are the
/// only two return values this port's `Prompt` treats as affirmative —
/// everything else (`IDCANCEL`, `IDNO`, `IDRETRY`, ...) is a decline.
pub fn is_affirmative_msgbox_response(raw_return: i32) -> bool {
    raw_return == 1 || raw_return == 6
}

/// What `Prompt` resolves to once a raw `MsgBox` response is in hand
/// (UniExtract.au3:5981-5992) — or is bypassed entirely by silent mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptOutcome {
    /// Silent mode auto-affirmed, or the user answered OK/Yes.
    Affirmed,
    /// The user declined and `$bTerminate` is `false` — just `Return 0`.
    Declined,
    /// The user declined and `$bTerminate` is `true`
    /// (UniExtract.au3:5989-5991): clean up only a `$createdir`-created
    /// output directory (never a pre-existing one, the same gate as
    /// C179/C189), then terminate silently.
    DeclinedAndTerminate { remove_created_outdir: bool },
}

/// Ports `Prompt`'s full dispatch (UniExtract.au3:5981-5992).
/// `silent_mode` short-circuits before a real `MsgBox` would ever be
/// shown (UniExtract.au3:5981-5983) — `user_affirmed` is meaningless in
/// that case, the same short-circuit contract as `warn_execute`'s own
/// `warn_execute_enabled` parameter (C189).
pub fn decide_prompt_outcome(
    silent_mode: bool,
    user_affirmed: bool,
    terminate_on_decline: bool,
    created_outdir_this_run: bool,
) -> PromptOutcome {
    if silent_mode || user_affirmed {
        PromptOutcome::Affirmed
    } else if terminate_on_decline {
        PromptOutcome::DeclinedAndTerminate {
            remove_created_outdir: created_outdir_this_run,
        }
    } else {
        PromptOutcome::Declined
    }
}

/// `$eCustomPromptSetting`'s three states (UniExtract.au3's own
/// `$PROMPT_ASK`/`$PROMPT_ALWAYS`/`$PROMPT_NEVER` enum) — a single
/// shared, persisted setting consulted by every `CustomPrompt` call site
/// across the whole app, not scoped to one dialog instance. `Ask` (the
/// default) means "show the dialog"; the other two are sticky answers
/// from a previous Always/Never click.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomPromptSetting {
    #[default]
    Ask,
    Always,
    Never,
}

/// Whether `CustomPrompt` can answer immediately without ever showing
/// its dialog (UniExtract.au3:5997-5999) — `None` means the dialog
/// genuinely has to be shown to get an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomPromptShortCircuit {
    Affirmed,
    Declined,
}

/// Ports `CustomPrompt`'s three-way short-circuit
/// (UniExtract.au3:5997-5999): a sticky `Always`/`Never` answer from a
/// previous call always wins, checked *before* silent mode; only a fresh
/// `Ask` setting even looks at `silent_mode` (which then always affirms,
/// same as `Prompt`'s own silent-mode behavior).
pub fn decide_custom_prompt_short_circuit(
    setting: CustomPromptSetting,
    silent_mode: bool,
) -> Option<CustomPromptShortCircuit> {
    match setting {
        CustomPromptSetting::Always => Some(CustomPromptShortCircuit::Affirmed),
        CustomPromptSetting::Never => Some(CustomPromptShortCircuit::Declined),
        CustomPromptSetting::Ask if silent_mode => Some(CustomPromptShortCircuit::Affirmed),
        CustomPromptSetting::Ask => None,
    }
}

/// Which button closed the `CustomPrompt` dialog
/// (UniExtract.au3:6015-6026).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomPromptButton {
    Yes,
    /// Includes the window being closed (`$GUI_EVENT_CLOSE`) — the
    /// source's own `Switch` groups both under the same `No`-equivalent
    /// case (UniExtract.au3:6015-6016).
    NoOrClosed,
    Always,
    Never,
}

/// What answering with a given button resolves to, including whether it
/// mutates the shared [`CustomPromptSetting`] for every future call
/// (UniExtract.au3:6017-6026). Only `Always`/`Never` do; `Yes` and
/// `NoOrClosed` leave the setting exactly as `Ask` for next time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomPromptResult {
    pub affirmed: bool,
    pub new_setting: Option<CustomPromptSetting>,
}

/// Ports `CustomPrompt`'s dialog-loop button dispatch
/// (UniExtract.au3:6015-6026).
pub fn resolve_custom_prompt_button(button: CustomPromptButton) -> CustomPromptResult {
    match button {
        CustomPromptButton::Yes => CustomPromptResult {
            affirmed: true,
            new_setting: None,
        },
        CustomPromptButton::NoOrClosed => CustomPromptResult {
            affirmed: false,
            new_setting: None,
        },
        CustomPromptButton::Always => CustomPromptResult {
            affirmed: true,
            new_setting: Some(CustomPromptSetting::Always),
        },
        CustomPromptButton::Never => CustomPromptResult {
            affirmed: false,
            new_setting: Some(CustomPromptSetting::Never),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affirmative_response_matches_idok_and_idyes_only() {
        assert!(is_affirmative_msgbox_response(1));
        assert!(is_affirmative_msgbox_response(6));
        assert!(!is_affirmative_msgbox_response(2));
        assert!(!is_affirmative_msgbox_response(7));
        assert!(!is_affirmative_msgbox_response(4));
    }

    #[test]
    fn silent_mode_always_affirms_regardless_of_response() {
        assert_eq!(
            decide_prompt_outcome(true, false, true, true),
            PromptOutcome::Affirmed
        );
    }

    #[test]
    fn affirmed_response_returns_affirmed() {
        assert_eq!(
            decide_prompt_outcome(false, true, true, true),
            PromptOutcome::Affirmed
        );
    }

    #[test]
    fn decline_without_terminate_flag_just_declines() {
        assert_eq!(
            decide_prompt_outcome(false, false, false, true),
            PromptOutcome::Declined
        );
    }

    /// Parity test: a decline with `$bTerminate` set removes the output
    /// directory only when this run actually created it.
    #[test]
    fn decline_with_terminate_flag_removes_only_if_created() {
        assert_eq!(
            decide_prompt_outcome(false, false, true, true),
            PromptOutcome::DeclinedAndTerminate {
                remove_created_outdir: true
            }
        );
        assert_eq!(
            decide_prompt_outcome(false, false, true, false),
            PromptOutcome::DeclinedAndTerminate {
                remove_created_outdir: false
            }
        );
    }

    #[test]
    fn always_and_never_short_circuit_before_silent_mode_check() {
        assert_eq!(
            decide_custom_prompt_short_circuit(CustomPromptSetting::Always, false),
            Some(CustomPromptShortCircuit::Affirmed)
        );
        assert_eq!(
            decide_custom_prompt_short_circuit(CustomPromptSetting::Never, true),
            Some(CustomPromptShortCircuit::Declined)
        );
    }

    #[test]
    fn ask_setting_defers_to_silent_mode() {
        assert_eq!(
            decide_custom_prompt_short_circuit(CustomPromptSetting::Ask, true),
            Some(CustomPromptShortCircuit::Affirmed)
        );
    }

    #[test]
    fn ask_setting_with_no_silent_mode_needs_the_real_dialog() {
        assert_eq!(
            decide_custom_prompt_short_circuit(CustomPromptSetting::Ask, false),
            None
        );
    }

    #[test]
    fn custom_prompt_setting_defaults_to_ask() {
        assert_eq!(CustomPromptSetting::default(), CustomPromptSetting::Ask);
    }

    #[test]
    fn yes_and_no_do_not_mutate_the_shared_setting() {
        assert_eq!(
            resolve_custom_prompt_button(CustomPromptButton::Yes),
            CustomPromptResult {
                affirmed: true,
                new_setting: None
            }
        );
        assert_eq!(
            resolve_custom_prompt_button(CustomPromptButton::NoOrClosed),
            CustomPromptResult {
                affirmed: false,
                new_setting: None
            }
        );
    }

    /// Parity test: Always affirms *and* stickily sets the shared
    /// setting; Never declines *and* stickily sets it, in the opposite
    /// direction.
    #[test]
    fn always_and_never_mutate_the_shared_setting() {
        assert_eq!(
            resolve_custom_prompt_button(CustomPromptButton::Always),
            CustomPromptResult {
                affirmed: true,
                new_setting: Some(CustomPromptSetting::Always)
            }
        );
        assert_eq!(
            resolve_custom_prompt_button(CustomPromptButton::Never),
            CustomPromptResult {
                affirmed: false,
                new_setting: Some(CustomPromptSetting::Never)
            }
        );
    }
}
