//! Persisted preference values from `UniExtract.ini`'s `[UniExtract
//! Preferences]` section, read at startup by `ReadPrefs()`
//! (UniExtract.au3:697-791) via the generic `LoadPref()` helper
//! (UniExtract.au3:825-841). Each preference is its own capability; this
//! module ports the ones whose resolved value follows a rule non-trivial
//! enough to need a dedicated function rather than a plain default.

/// Ports `LoadPref($STATUS_TIMEOUT, $Timeout)` followed by the two lines
/// immediately after it (UniExtract.au3:744-746), together the whole of
/// capability C026.
///
/// `stored_seconds` is `LoadPref`'s normal-path result: the `timeout` ini
/// value already coerced through `_Max(Int($return), -1)` — `Some(n)`.
/// `None` represents `LoadPref`'s *error* path (the `timeout` key is
/// missing or unreadable): that branch never assigns its `ByRef $value`
/// parameter, so `$Timeout` keeps whatever it already held — the
/// `Global $Timeout = 60000` declaration at UniExtract.au3:151, in
/// *milliseconds* — and this function still multiplies it by 1000 as if
/// it were seconds, same as the source does. That's a genuine unit-mismatch
/// quirk in UniExtract2 (not something this port invented or fixed): a
/// truly first-run process with no `timeout` key ends up with a
/// 60,000,000ms (~16.7 hour) extraction timeout, not the 60 seconds the
/// `Global` declaration's own comment implies. Every other path — any
/// stored value, however small or negative — clamps to the intended
/// 60-second default once converted to ms and found under 10 seconds.
pub fn resolve_timeout_ms(stored_seconds: Option<i64>) -> i64 {
    let value_before_multiply = stored_seconds.unwrap_or(60_000);
    let timeout_ms = value_before_multiply * 1000;
    if timeout_ms < 10_000 {
        60_000
    } else {
        timeout_ms
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_timeout_ms;

    /// Parity test for capability C026: a normally-stored preference value
    /// (in seconds, already through LoadPref's `_Max(Int(x), -1)` clamp)
    /// converts to milliseconds, matching `$Timeout *= 1000`
    /// (UniExtract.au3:745).
    #[test]
    fn stored_seconds_convert_to_milliseconds() {
        assert_eq!(resolve_timeout_ms(Some(60)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(30)), 30_000);
        assert_eq!(resolve_timeout_ms(Some(120)), 120_000);
    }

    /// Any stored value under 10 seconds (once converted to ms) — the
    /// "minimum enforced 10s" half of C026 — resets to the 60-second
    /// default rather than being clamped up to 10s (UniExtract.au3:746:
    /// `If $Timeout < 10000 Then $Timeout = 60000`).
    #[test]
    fn values_under_ten_seconds_reset_to_the_sixty_second_default() {
        assert_eq!(resolve_timeout_ms(Some(9)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(1)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(0)), 60_000);
        // LoadPref's own `_Max(Int($return), -1)` clamp means the lowest
        // value that can ever reach this function is -1.
        assert_eq!(resolve_timeout_ms(Some(-1)), 60_000);
    }

    /// Exactly 10 seconds is the boundary: `< 10000`, not `<= 10000`, so
    /// 10000ms survives unchanged.
    #[test]
    fn exactly_ten_seconds_is_not_reset() {
        assert_eq!(resolve_timeout_ms(Some(10)), 10_000);
    }

    /// A missing/unreadable `timeout` preference key hits LoadPref's error
    /// branch, which never assigns `$Timeout` — so the pre-call value (the
    /// 60000-millisecond `Global` default) survives, gets misinterpreted
    /// as seconds by the unconditional `*= 1000`, and comes out far larger
    /// than the intended 60-second default. A real, preserved quirk.
    #[test]
    fn missing_preference_key_reproduces_the_sixty_million_millisecond_quirk() {
        assert_eq!(resolve_timeout_ms(None), 60_000_000);
    }
}
