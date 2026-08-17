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

/// Mirrors UniExtract2's `$OPTION_*` enum (UniExtract.au3:97:
/// `Enum $OPTION_KEEP, $OPTION_DELETE, $OPTION_ASK, $OPTION_MOVE`).
/// Shared by two preferences — `deletesourcefile` (C024) and `cleanup`
/// (C033) — though only Keep/Ask/Delete are ever offered for
/// `deletesourcefile` through the GUI (UniExtract.au3:6393-6395); `Move`
/// is representable here because `LoadPref` stores whatever integer was
/// in the ini without validating it against the option's own UI, and
/// [`should_delete_source_file`] (C158) treats it exactly like `Keep`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteSourceFileOption {
    #[default]
    Keep,
    Delete,
    Ask,
    Move,
}

/// C024: parses the `deletesourcefile` preference's raw ini integer the
/// way `LoadPref`'s `_Max(Int($return), -1)` clamp plus AutoIt's `Enum`
/// numbering does — 1=Delete, 2=Ask, 3=Move, anything else (including a
/// missing/unreadable key, `LoadPref`'s error path) falls back to the
/// `Global $eOptDeleteSourceFile = $OPTION_KEEP` default
/// (UniExtract.au3:150).
pub fn parse_delete_source_file_option(raw: Option<i64>) -> DeleteSourceFileOption {
    match raw {
        Some(1) => DeleteSourceFileOption::Delete,
        Some(2) => DeleteSourceFileOption::Ask,
        Some(3) => DeleteSourceFileOption::Move,
        _ => DeleteSourceFileOption::Keep,
    }
}

/// C158: ports the deletion condition inside `terminate()`'s
/// `$STATUS_SUCCESS` case (UniExtract.au3:4204):
/// `$eOptDeleteSourceFile = $OPTION_DELETE Or ($eOptDeleteSourceFile =
/// $OPTION_ASK And Not $silentmode And Prompt(32 + 4, 'FILE_DELETE',
/// $file))`. `user_confirmed_delete` stands in for the `Prompt(...)` call
/// (the GUI confirmation dialog itself is out of scope, deferred GUI
/// subsystem, manifest row D001) — AutoIt's `And` short-circuits, so the
/// source never actually shows that prompt in silent mode, matching this
/// function's `!silent_mode &&` guard before it's consulted.
pub fn should_delete_source_file(
    option: DeleteSourceFileOption,
    silent_mode: bool,
    user_confirmed_delete: bool,
) -> bool {
    match option {
        DeleteSourceFileOption::Delete => true,
        DeleteSourceFileOption::Ask => !silent_mode && user_confirmed_delete,
        DeleteSourceFileOption::Keep | DeleteSourceFileOption::Move => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_delete_source_file_option, resolve_timeout_ms, should_delete_source_file,
        DeleteSourceFileOption,
    };

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

    /// Parity test for capability C024: the `$OPTION_*` enum's AutoIt
    /// numbering (`Keep`=0, `Delete`=1, `Ask`=2, `Move`=3), and a
    /// missing/unreadable/out-of-range value falling back to the
    /// `$OPTION_KEEP` default, exactly as `LoadPref`'s error path (never
    /// assigning its `ByRef` output) leaves `$eOptDeleteSourceFile` at its
    /// `Global` declaration's value.
    #[test]
    fn delete_source_file_option_parses_autoit_enum_numbering() {
        assert_eq!(
            parse_delete_source_file_option(Some(1)),
            DeleteSourceFileOption::Delete
        );
        assert_eq!(
            parse_delete_source_file_option(Some(2)),
            DeleteSourceFileOption::Ask
        );
        assert_eq!(
            parse_delete_source_file_option(Some(3)),
            DeleteSourceFileOption::Move
        );
        assert_eq!(
            parse_delete_source_file_option(Some(0)),
            DeleteSourceFileOption::Keep
        );
        assert_eq!(
            parse_delete_source_file_option(Some(99)),
            DeleteSourceFileOption::Keep
        );
        assert_eq!(
            parse_delete_source_file_option(None),
            DeleteSourceFileOption::Keep
        );
    }

    /// Parity test for capability C158: `$OPTION_DELETE` always deletes;
    /// `$OPTION_ASK` deletes only outside silent mode and only if the
    /// (out-of-scope) confirmation prompt returned true; `$OPTION_KEEP`
    /// and `$OPTION_MOVE` never delete (UniExtract.au3:4204).
    #[test]
    fn should_delete_source_file_matches_source_condition() {
        assert!(should_delete_source_file(
            DeleteSourceFileOption::Delete,
            false,
            false
        ));
        assert!(should_delete_source_file(
            DeleteSourceFileOption::Delete,
            true,
            false
        ));

        assert!(should_delete_source_file(
            DeleteSourceFileOption::Ask,
            false,
            true
        ));
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Ask,
            false,
            false
        ));
        // Silent mode short-circuits before the (out-of-scope) prompt is
        // ever consulted, matching AutoIt's `And` short-circuit.
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Ask,
            true,
            true
        ));

        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Keep,
            false,
            true
        ));
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Move,
            false,
            true
        ));
    }
}
