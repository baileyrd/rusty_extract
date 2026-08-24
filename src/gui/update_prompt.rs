//! Update-available/changelog prompt dialog — capability C199. Ports
//! the one real content decision inside `GUI_UpdatePrompt`
//! (UniExtract.au3:7741-7775): the changelog-fetch failure fallback.
//!
//! **Open design question, not resolved here.** The source fetches the
//! remote changelog (`_INetGetSource($sUpdateURL & "news")`) as an
//! integral, *blocking* part of constructing the dialog — the window is
//! created, then the fetch runs synchronously before the message loop
//! even starts, with the edit box showing a "Loading..." placeholder in
//! the meantime that in practice never gets a chance to render. Since no
//! real window exists yet for this dialog (same gap as every other
//! dialog this migration phase has ported), *whether* to reproduce that
//! blocking-fetch-during-construction shape or restructure it as a real
//! async fetch-then-populate (more idiomatic for `egui`'s immediate-mode
//! render loop, which would otherwise freeze entirely for the fetch's
//! duration) is left as an open design decision for whoever wires the
//! real dialog — flagged here explicitly rather than silently picked,
//! per this row's own manifest note. Either way, the *content* this
//! module ports — the Yes/No semantics and the failure-text fallback —
//! must still match.
//!
//! The Yes/No button result itself (UniExtract.au3:7764-7768) is the
//! same trivial boolean dialog-outcome shape already established by
//! `CustomPrompt` (C193) — not re-derived as its own function here.

/// What the changelog edit box ends up showing
/// (UniExtract.au3:7758-7760): `_INetGetSource`'s real text on success,
/// or the translated `DOWNLOAD_FAILED` fallback message on failure
/// (`@error` after the fetch call).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangelogFetchOutcome {
    Loaded(String),
    Failed,
}

/// Ports the `If @error Then $return = t('DOWNLOAD_FAILED', ...)`
/// fallback (UniExtract.au3:7759). `fetch_result` is `None` for the
/// source's own `@error` branch.
pub fn resolve_changelog_text(fetch_result: Option<String>) -> ChangelogFetchOutcome {
    match fetch_result {
        Some(text) => ChangelogFetchOutcome::Loaded(text),
        None => ChangelogFetchOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_fetch_shows_the_fetched_text() {
        assert_eq!(
            resolve_changelog_text(Some("- Fixed a bug\n- Added a feature".to_string())),
            ChangelogFetchOutcome::Loaded("- Fixed a bug\n- Added a feature".to_string())
        );
    }

    #[test]
    fn failed_fetch_falls_back_to_the_failure_message() {
        assert_eq!(resolve_changelog_text(None), ChangelogFetchOutcome::Failed);
    }
}
