//! Usage-stats telemetry beacon: ports `SendStats` (UniExtract.au3:5366-5370)
//! — a lightweight, fire-and-forget HTTP GET ping sent from many call
//! sites across the update/uninstall/feedback/extraction-exit flows.
//!
//! This capability covers the pure gate and URL-construction logic. The
//! actual `InetRead` network call (deliberately fire-and-forget: the
//! source discards its result and has no error handling at all — a
//! network failure here is silently swallowed) is the caller's job.
//!
//! **The second parameter's meaning is overloaded per call site** —
//! there is no one consistent semantic to name it after. Concrete
//! examples from the source: `SendStats("UpdateMain", 0)` and
//! `SendStats("UpdateHelpers", 1)` use it as a numeric result code;
//! `SendStats($STATUS_MISSINGEXE, $sPlugin)` (UniExtract.au3:3759) uses it
//! as a plugin name; `SendStats($status, $arctype)` (UniExtract.au3:4245)
//! uses it as an archive-type string; `SendStats("uninstall")` and
//! `SendStats("DisableStats")`/`SendStats("EnableStats")`
//! (UniExtract.au3:6529-6531) omit it entirely, relying on the default of
//! `1`. [`build_stats_url`] therefore takes it as a plain already-formatted
//! string rather than a typed result code.

/// Ports `SendStats`'s own gate (UniExtract.au3:5367): `If Not
/// $bOptSendStats Then Return`.
pub fn should_send_stats(enabled: bool) -> bool {
    enabled
}

/// Ports the outgoing URL construction (UniExtract.au3:5369):
/// `$sUrlStats & $a & "&r=" & $sResult & "&id=" & $sOptGuid & "&v=" &
/// $sVersion`. `result` is deliberately `&str` — see the module docs on
/// why the second parameter has no single consistent type across call
/// sites. **Verified quirk, preserved rather than "fixed"**: this URL,
/// including the per-install GUID (C215), is logged to the local debug
/// log verbatim (`Cout(...)` wraps the URL before it's read by
/// `InetRead`) — a PII-adjacent value ends up in the on-disk log
/// alongside everything else `Cout` captures.
pub fn build_stats_url(
    base_url: &str,
    event: &str,
    result: &str,
    guid: &str,
    version: &str,
) -> String {
    format!("{base_url}{event}&r={result}&id={guid}&v={version}")
}

/// The default `result` value (UniExtract.au3:5366's `$sResult = 1`) as a
/// string, for call sites that omit the second argument entirely (e.g.
/// `SendStats("uninstall")`, `SendStats("DisableStats")`).
pub const DEFAULT_RESULT: &str = "1";

#[cfg(test)]
mod tests {
    use super::{build_stats_url, should_send_stats, DEFAULT_RESULT};
    use crate::gui::prefs_dialog::{decide_send_stats_command, SendStatsCommand};

    #[test]
    fn stats_gated_by_the_enabled_flag() {
        assert!(should_send_stats(true));
        assert!(!should_send_stats(false));
    }

    #[test]
    fn stats_url_matches_source_concatenation() {
        let url = build_stats_url(
            "https://stats.example.com/?a=",
            "UpdateMain",
            "0",
            "guid-1234",
            "2.0.0",
        );
        assert_eq!(
            url,
            "https://stats.example.com/?a=UpdateMain&r=0&id=guid-1234&v=2.0.0"
        );
    }

    #[test]
    fn default_result_matches_source_default_of_one() {
        let url = build_stats_url(
            "https://stats.example.com/?a=",
            "uninstall",
            DEFAULT_RESULT,
            "g",
            "v",
        );
        assert_eq!(url, "https://stats.example.com/?a=uninstall&r=1&id=g&v=v");
    }

    /// The verified ordering quirk (UniExtract.au3:6528-6531): disabling
    /// the preference must send its "DisableStats" ping using the *old*,
    /// still-true value -- `should_send_stats` gates on whatever value is
    /// live at call time, so calling it with the value already flipped to
    /// `false` would silently swallow the opt-out ping. This test proves
    /// the correct sequence works and the "fixed" (flip-then-send) order
    /// doesn't.
    #[test]
    fn disable_ping_only_fires_when_sent_before_the_flag_flips() {
        let previous_enabled = true;
        let new_enabled = false;
        let command = decide_send_stats_command(previous_enabled, new_enabled);
        assert_eq!(command, Some(SendStatsCommand::Disable));

        // Correct order: send while $bOptSendStats is still the old value.
        assert!(should_send_stats(previous_enabled));

        // The bug this ordering avoids: if the flag were flipped first,
        // SendStats's own gate would silently swallow the opt-out ping.
        assert!(!should_send_stats(new_enabled));
    }

    #[test]
    fn enable_ping_fires_using_the_already_updated_value() {
        let previous_enabled = false;
        let new_enabled = true;
        let command = decide_send_stats_command(previous_enabled, new_enabled);
        assert_eq!(command, Some(SendStatsCommand::Enable));
        assert!(should_send_stats(new_enabled));
    }
}
