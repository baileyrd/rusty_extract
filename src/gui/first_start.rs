//! First-start wizard — capability C191. Ports the page-navigation state
//! machine inside `GUI_FirstStart_Prev`/`_Next`/`_ShowPage`
//! (UniExtract.au3:7373-7416) as a pure function of the *current* page
//! number, and the missing-translation hard-failure branch inside
//! `GUI_FirstStart` itself (UniExtract.au3:7358-7365).
//!
//! **The wizard window itself stays unwired.** Its own two dynamic pages
//! (2 and 3) link out to `GUI_Prefs` and `GUI_ContextMenu`, neither of
//! which is a real window in this port yet (C190/C192) — and the whole
//! wizard's page titles/text come from a translated, pipe-delimited
//! string this port has no translation-catalog infrastructure for (D006,
//! still genuinely deferred). Building this window now would have
//! nothing real to link out to and no real text to show.
//!
//! **`GUI_FirstStart`'s missing-translation branch needs explicit
//! sign-off before it's ever wired for real, not just ported.** On a
//! missing `FIRSTSTART_PAGES` translation key, the source unconditionally
//! clears the per-install ID (`SavePref("ID", "")`, capability C215) and
//! *always* exits the process (`Exit 0`) right after the prompt — whether
//! or not the user agreed to the offered update-and-restart. That's a
//! genuinely hard-to-reverse, unusually severe response to a missing
//! string (most of this port's other missing-translation-key handling is
//! cosmetic degradation, not data clearing plus a forced exit), which is
//! exactly why the capability manifest itself calls this out as a
//! preserve-vs-simplify decision that needs sign-off at implementation
//! time — [`decide_missing_translation_outcome`] below only documents and
//! tests the source's own decision shape; it is not wired to any real
//! `SavePref`/exit call.

/// Ports the Prev-button visibility rule that's implicit across
/// `GUI_FirstStart_Prev` (UniExtract.au3:7374-7375) and
/// `GUI_FirstStart_Next` (UniExtract.au3:7386-7387): both functions only
/// *toggle* visibility at the specific page boundary where it changes
/// (hide when leaving page 2 going back, show when leaving page 1 going
/// forward) rather than recomputing it on every page. This reformulates
/// that pair of side-effecting deltas as one pure function of the
/// resulting page number — verified equivalent by tracing both
/// directions: `Prev` only hides when the *new* page is `1`, `Next` only
/// shows when the *old* page was `1` (so the *new* page is `2`, still
/// `> 1`) — every other page transition leaves visibility unchanged
/// because it was already correct for this rule.
pub fn prev_button_visible(page: usize) -> bool {
    page > 1
}

/// What the Next button currently is: `GUI_FirstStart_Next`'s own default
/// role, or `GUI_FirstStart_Prev`/`_Next`'s relabeled-to-`Finish` role
/// once the wizard is on its last page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextButtonMode {
    /// `t('NEXT_BUT')`, bound to `GUI_FirstStart_Next` — advances a page.
    Next,
    /// `t('FINISH_BUT')`, bound to `GUI_FirstStart_Exit` — closes the
    /// wizard.
    Finish,
}

/// Ports the Next-button relabel rule implicit across
/// `GUI_FirstStart_Next` (UniExtract.au3:7388-7391) and
/// `GUI_FirstStart_Prev` (UniExtract.au3:7376-7379) — the same
/// resulting-page reformulation as [`prev_button_visible`]: `Next`
/// switches to `Finish` only when the *new* page is the last one;
/// `Prev` switches back to `Next` only when the *new* page is one before
/// the last (i.e. no longer the last).
pub fn next_button_mode(page: usize, total_pages: usize) -> NextButtonMode {
    if total_pages > 0 && page >= total_pages {
        NextButtonMode::Finish
    } else {
        NextButtonMode::Next
    }
}

/// The wizard's per-page center action button
/// (`$FS_Button`, UniExtract.au3:7401-7415).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionButton {
    /// Page 1, or any page beyond the wizard's own hardcoded 3 pages
    /// (`Case Else`, UniExtract.au3:7413-7414) — no action button.
    Hidden,
    /// Page 2: opens the Preferences dialog (`GUI_Prefs`, C190).
    Preferences,
    /// Page 3: opens the context-menu entries dialog (`GUI_ContextMenu`,
    /// C192).
    ContextMenuEntries,
}

/// Ports `GUI_FirstStart_ShowPage`'s `Switch $page`
/// (UniExtract.au3:7403-7415). The source's own page 3 case doesn't
/// re-issue a `$GUI_SHOW` call the way page 2's does, relying on the
/// button already being visible from the page-2-to-3 transition — this
/// function instead declares page 3's state directly, which is
/// equivalent along the only page sequence the wizard's fixed 3-page
/// structure (`$FS_Texts`'s hardcoded 3-entry literal, UniExtract.au3:
/// 7366) ever actually produces (1 -> 2 -> 3 and back).
pub fn resolve_action_button(page: usize) -> ActionButton {
    match page {
        2 => ActionButton::Preferences,
        3 => ActionButton::ContextMenuEntries,
        _ => ActionButton::Hidden,
    }
}

/// What `GUI_FirstStart`'s missing-`FIRSTSTART_PAGES`-translation branch
/// decides once it has the user's answer to its download prompt
/// (UniExtract.au3:7360-7364). See this module's own doc comment for why
/// this is documented and tested, but deliberately not wired to any real
/// `SavePref`/exit call yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingTranslationOutcome {
    /// Whether to call `CheckUpdate($UPDATEMSG_SILENT, False,
    /// $UPDATE_HELPER)` then `Restart()` before the process exits.
    /// **The process exits either way** (`Exit 0` runs unconditionally
    /// right after, UniExtract.au3:7364) — this field only controls
    /// whether an update-and-restart is attempted first, not whether the
    /// app quits.
    pub trigger_update_and_restart: bool,
}

/// Ports UniExtract.au3:7360-7364's decision once the download-prompt
/// `MsgBox` has an answer. The per-install ID clear
/// (`SavePref("ID", "")`, UniExtract.au3:7359, capability C215) happens
/// unconditionally *before* this prompt is even shown, so it isn't
/// modeled as part of this function's output — it always happens on this
/// path, regardless of the answer.
pub fn decide_missing_translation_outcome(
    user_confirmed_download: bool,
) -> MissingTranslationOutcome {
    MissingTranslationOutcome {
        trigger_update_and_restart: user_confirmed_download,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prev_button_hidden_only_on_first_page() {
        assert!(!prev_button_visible(1));
        assert!(prev_button_visible(2));
        assert!(prev_button_visible(3));
    }

    #[test]
    fn next_button_is_finish_only_on_last_page() {
        assert_eq!(next_button_mode(1, 3), NextButtonMode::Next);
        assert_eq!(next_button_mode(2, 3), NextButtonMode::Next);
        assert_eq!(next_button_mode(3, 3), NextButtonMode::Finish);
    }

    /// Parity test: the transition sequence a real run of the wizard
    /// actually produces (forward through all 3 pages, then back).
    #[test]
    fn full_forward_and_backward_traversal_matches_source_toggle_points() {
        let total = 3;
        // Forward: 1 -> 2 -> 3.
        assert!(!prev_button_visible(1));
        assert_eq!(next_button_mode(1, total), NextButtonMode::Next);
        assert!(prev_button_visible(2));
        assert_eq!(next_button_mode(2, total), NextButtonMode::Next);
        assert!(prev_button_visible(3));
        assert_eq!(next_button_mode(3, total), NextButtonMode::Finish);
        // Backward: 3 -> 2 -> 1.
        assert_eq!(next_button_mode(2, total), NextButtonMode::Next);
        assert!(!prev_button_visible(1));
    }

    #[test]
    fn action_button_matches_each_page() {
        assert_eq!(resolve_action_button(1), ActionButton::Hidden);
        assert_eq!(resolve_action_button(2), ActionButton::Preferences);
        assert_eq!(resolve_action_button(3), ActionButton::ContextMenuEntries);
    }

    /// Parity test: a page beyond the wizard's own fixed 3 pages hides
    /// the action button, matching `Case Else`.
    #[test]
    fn action_button_hidden_beyond_third_page() {
        assert_eq!(resolve_action_button(4), ActionButton::Hidden);
    }

    #[test]
    fn missing_translation_outcome_follows_user_answer() {
        assert!(decide_missing_translation_outcome(true).trigger_update_and_restart);
        assert!(!decide_missing_translation_outcome(false).trigger_update_and_restart);
    }
}
