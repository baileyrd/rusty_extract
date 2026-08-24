//! Feedback-prompt gate: ports `GUI_Feedback_Prompt`
//! (UniExtract.au3:6938-6977) and the caller-side trigger condition that
//! decides whether to invoke it at all (UniExtract.au3:4226) — whether to
//! offer the feedback dialog after specific extraction outcomes, with a
//! persisted tri-state (never/ask/always) preference (`feedbackprompt`,
//! D012).
//!
//! **The trigger condition lives in the caller, not in
//! `GUI_Feedback_Prompt` itself** — [`should_prompt_for_feedback_after_extraction`]
//! ports it separately so it isn't lost by porting only the function body.
//! This capability covers only the decision logic; the real dialog
//! (window creation, checkbox state, button clicks) and the actual
//! [`crate::gui::feedback::GUI_Feedback`]-equivalent submission flow are
//! the caller's job. **"Always send" still requires the privacy checkbox
//! to be checked on the actual feedback form** (C212) — remembering "yes"
//! here only skips *this* prompt, it never bypasses that consent gate.

/// Ports `$bOptAskForFeedback`'s three meaningful values (0/1/2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackPromptPref {
    Never,
    Ask,
    Always,
}

/// Ports the caller-side trigger condition (UniExtract.au3:4226):
/// `($exitcode == 1 Or $exitcode == 3 Or $exitcode == 4 Or $exitcode ==
/// 12) And $fileext <> "dll"`. Only these four exit codes, and never for a
/// `.dll` input file, trigger a feedback prompt at all — a real,
/// easy-to-lose condition since it's checked by the caller, not inside
/// `GUI_Feedback_Prompt` itself.
pub fn should_prompt_for_feedback_after_extraction(exit_code: i32, file_extension: &str) -> bool {
    matches!(exit_code, 1 | 3 | 4 | 12) && !file_extension.eq_ignore_ascii_case("dll")
}

/// What `GUI_Feedback_Prompt` does once its caller has already decided to
/// invoke it (UniExtract.au3:6939-6940).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackPromptAction {
    /// The preference is `Never`, this wasn't an extraction, or the run is
    /// silent — the whole function is a no-op.
    DoNothing,
    /// The preference is `Always` — skip the yes/no dialog and go
    /// straight to the feedback form.
    OpenFeedbackDirectly,
    /// The preference is `Ask` — show the yes/no/remember dialog.
    ShowYesNoPrompt,
}

/// Ports `GUI_Feedback_Prompt`'s own gate and dispatch
/// (UniExtract.au3:6939-6940): `If Not ($bOptAskForFeedback And $extract)
/// Or $silentmode Then Return` followed by `If $bOptAskForFeedback == 2
/// Then Return GUI_Feedback()`.
pub fn resolve_feedback_prompt_action(
    pref: FeedbackPromptPref,
    was_extraction: bool,
    silent_mode: bool,
) -> FeedbackPromptAction {
    if pref == FeedbackPromptPref::Never || !was_extraction || silent_mode {
        FeedbackPromptAction::DoNothing
    } else if pref == FeedbackPromptPref::Always {
        FeedbackPromptAction::OpenFeedbackDirectly
    } else {
        FeedbackPromptAction::ShowYesNoPrompt
    }
}

/// Ports the "Yes" branch's remember-checkbox gate (UniExtract.au3:6959-6962):
/// the preference is only persisted (to `Always`) if "remember" is
/// checked. `None` means leave the stored preference exactly as it is —
/// clicking "Yes" without checking "remember" still opens the feedback
/// form for *this* run, it just doesn't change what happens next time.
pub fn resolve_yes_choice_pref_update(remember_checked: bool) -> Option<FeedbackPromptPref> {
    remember_checked.then_some(FeedbackPromptPref::Always)
}

/// Ports the "No" branch's remember-checkbox gate (UniExtract.au3:6967-6970):
/// symmetric to [`resolve_yes_choice_pref_update`], persisting `Never`
/// only if "remember" is checked.
pub fn resolve_no_choice_pref_update(remember_checked: bool) -> Option<FeedbackPromptPref> {
    remember_checked.then_some(FeedbackPromptPref::Never)
}

/// Ports the final action taken once the yes/no dialog closes
/// (UniExtract.au3:6958-6971): "Yes" always opens the feedback form for
/// this run; "No" and closing the window both just exit without opening
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDialogOutcome {
    OpenFeedback,
    DoNothing,
}

pub fn resolve_prompt_dialog_outcome(user_clicked_yes: bool) -> PromptDialogOutcome {
    if user_clicked_yes {
        PromptDialogOutcome::OpenFeedback
    } else {
        PromptDialogOutcome::DoNothing
    }
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_feedback_prompt_action, resolve_no_choice_pref_update,
        resolve_prompt_dialog_outcome, resolve_yes_choice_pref_update,
        should_prompt_for_feedback_after_extraction, FeedbackPromptAction, FeedbackPromptPref,
        PromptDialogOutcome,
    };

    #[test]
    fn trigger_fires_only_for_specific_exit_codes() {
        for code in [1, 3, 4, 12] {
            assert!(should_prompt_for_feedback_after_extraction(code, "zip"));
        }
        for code in [0, 2, 5, 11, 13] {
            assert!(!should_prompt_for_feedback_after_extraction(code, "zip"));
        }
    }

    /// The verified quirk: a .dll input file is excluded regardless of
    /// exit code, and the extension check is case-insensitive.
    #[test]
    fn trigger_excludes_dll_files_case_insensitively() {
        assert!(!should_prompt_for_feedback_after_extraction(1, "dll"));
        assert!(!should_prompt_for_feedback_after_extraction(1, "DLL"));
        assert!(should_prompt_for_feedback_after_extraction(1, "exe"));
    }

    #[test]
    fn prompt_action_skipped_when_never_not_extraction_or_silent() {
        assert_eq!(
            resolve_feedback_prompt_action(FeedbackPromptPref::Never, true, false),
            FeedbackPromptAction::DoNothing
        );
        assert_eq!(
            resolve_feedback_prompt_action(FeedbackPromptPref::Always, false, false),
            FeedbackPromptAction::DoNothing
        );
        assert_eq!(
            resolve_feedback_prompt_action(FeedbackPromptPref::Always, true, true),
            FeedbackPromptAction::DoNothing
        );
    }

    #[test]
    fn always_preference_opens_feedback_directly() {
        assert_eq!(
            resolve_feedback_prompt_action(FeedbackPromptPref::Always, true, false),
            FeedbackPromptAction::OpenFeedbackDirectly
        );
    }

    #[test]
    fn ask_preference_shows_yes_no_prompt() {
        assert_eq!(
            resolve_feedback_prompt_action(FeedbackPromptPref::Ask, true, false),
            FeedbackPromptAction::ShowYesNoPrompt
        );
    }

    #[test]
    fn yes_choice_only_persists_when_remember_is_checked() {
        assert_eq!(
            resolve_yes_choice_pref_update(true),
            Some(FeedbackPromptPref::Always)
        );
        assert_eq!(resolve_yes_choice_pref_update(false), None);
    }

    #[test]
    fn no_choice_only_persists_when_remember_is_checked() {
        assert_eq!(
            resolve_no_choice_pref_update(true),
            Some(FeedbackPromptPref::Never)
        );
        assert_eq!(resolve_no_choice_pref_update(false), None);
    }

    #[test]
    fn dialog_outcome_opens_feedback_only_on_yes() {
        assert_eq!(
            resolve_prompt_dialog_outcome(true),
            PromptDialogOutcome::OpenFeedback
        );
        assert_eq!(
            resolve_prompt_dialog_outcome(false),
            PromptDialogOutcome::DoNothing
        );
    }
}
