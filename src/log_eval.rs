//! Extraction output-log evaluation: ports pieces of `EvaluateLog()`
//! (UniExtract.au3:4778-4825), the substring-classifier `extract()` runs
//! over a helper binary's captured stdout/stderr to decide success vs.
//! failure. `EvaluateLog()` is a long `ElseIf` chain — invalid password,
//! user cancellation, low disk space, missing archive part, several
//! generic success/failure phrasings, and finally the overwrite case
//! this module ports — each branch is its own capability (or not yet
//! ported); this module doesn't attempt the whole chain.

/// C144: ports the "already exists"/"Overwrite" branch of `EvaluateLog()`
/// (UniExtract.au3:4819-4823): a log mentioning either substring is
/// treated as `$RESULT_SUCCESS`, not a failure — the source's own
/// reasoning, right there in the comment, is that an overwritten file
/// leaves the output folder's total size roughly unchanged, so the
/// separate "did the folder size change" check that would otherwise
/// flag this as a failure gets skipped for exactly that reason.
///
/// **Scope — one branch, not the whole chain.** This predicate only
/// reproduces this one `ElseIf` arm. In the source, it's reached only
/// after several higher-priority classifications (invalid password, user
/// cancellation, low disk space, missing archive part, and multiple
/// generic success/failure phrasings, UniExtract.au3:4778-4818) have all
/// failed to match — each of those is its own capability, not ported
/// here. A caller applying this predicate must replicate that ordering:
/// check this only once every higher-priority classification has been
/// ruled out, matching the source's `ElseIf` chain.
pub fn is_overwrite_success_message(log: &str) -> bool {
    log.contains("already exists.") || log.contains("Overwrite")
}

/// Reproduces `_StringGetLine($sLog, -1)` (UniExtract.au3:4577-4583) for
/// the one purpose [`is_password_failure`] uses it: a substring search
/// over its result. For negative `$iLine`, the source's own
/// implementation searches for the `(1 - $iLine)`-th `@CRLF` occurrence
/// counting from the end — for `$iLine = -1` that's the *second*-to-last
/// `@CRLF` — and returns everything from there to the end of the
/// string. **If fewer than two `@CRLF`s exist, `StringInStr` returns 0
/// and the source falls back to `StringTrimLeft($sString, 0)`, i.e. the
/// *entire, unmodified string* — not just its (only) line.** That
/// means a log with zero or exactly one line break has its whole text
/// searched, while a log with two or more line breaks only has its true
/// last line searched. Preserved exactly, not "fixed" into a plain
/// last-line helper.
fn tail_for_password_prompt_search(log: &str) -> &str {
    let mut from_end = log.rmatch_indices("\r\n");
    from_end.next(); // the last occurrence itself, not the one we want
    match from_end.next() {
        Some((pos, _)) => &log[pos..],
        None => log,
    }
}

/// C162: ports the invalid-password branch of `EvaluateLog()`
/// (UniExtract.au3:4782-4787) — the first, highest-priority arm of its
/// `ElseIf` chain. A log is classified as a password failure when it
/// contains any of five substrings anywhere, or when
/// [`tail_for_password_prompt_search`]'s result contains a sixth
/// ("Enter password").
///
/// Matches case-insensitively: AutoIt's `StringInStr` defaults its case
/// parameter to not-case-sensitive when omitted, as it is for all six
/// calls here — the same convention already documented for
/// `needs_manual_input` (C145).
///
/// **Scope — one branch, not the whole chain** (see
/// [`is_overwrite_success_message`]'s doc comment for the general note
/// on `EvaluateLog`'s `ElseIf` ordering). This is the chain's *first*
/// arm, so a caller applies it before every other classification, not
/// after.
pub fn is_password_failure(log: &str) -> bool {
    let lower = log.to_lowercase();
    let whole_log_match = [
        "wrong password?",
        "the specified password is incorrect.",
        "archive encrypted.",
        "corrupt file or wrong password",
        "error: wrong password",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    whole_log_match
        || tail_for_password_prompt_search(log)
            .to_lowercase()
            .contains("enter password")
}

/// C145: ports the live user-input-needed detection inside the
/// subprocess-output-streaming loop (UniExtract.au3:4930-4933): as each
/// new chunk of a helper binary's live output arrives, it's scanned for
/// any of eight substrings signaling the process is blocked on a modal
/// prompt (overwrite confirmation, password request, low disk space,
/// a request for a new filename, a request to insert removable media, or
/// a bare `[R]etry` option). A match doesn't answer the prompt — the
/// source has no auto-answer logic at all — it only force-shows the
/// extractor's window (`WinSetState(..., @SW_SHOW)`) so a human can
/// respond manually. That windowing (and the tray-status/GUI side
/// effects around it) is out of scope, deferred GUI subsystem; this
/// function reproduces only the substring predicate driving it.
///
/// Matches case-insensitively: AutoIt's `StringInStr` defaults its case
/// parameter to `0` (not case sensitive) when omitted, as it is for
/// every one of these eight calls, the same AutoIt default already
/// documented for `cli`'s flag-detection functions (C007-C013) and
/// `EvaluateLog`'s other `StringInStr` calls.
pub fn needs_manual_input(chunk: &str) -> bool {
    let chunk_lower = chunk.to_lowercase();
    [
        "already exist",
        "overwrite",
        " replace",
        "password",
        "not enough free space available",
        "you must choose a new filename",
        "insert disk with",
        "[r]etry",
    ]
    .iter()
    .any(|needle| chunk_lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{is_overwrite_success_message, is_password_failure, needs_manual_input};

    /// Parity test for capability C144: both substrings the source
    /// checks for are recognized.
    #[test]
    fn recognizes_both_overwrite_substrings() {
        assert!(is_overwrite_success_message("output.txt already exists."));
        assert!(is_overwrite_success_message(
            "Overwrite output.txt (Yes/No/All)?"
        ));
    }

    /// Parity test for capability C144: unrelated log text doesn't match.
    #[test]
    fn does_not_match_unrelated_log_text() {
        assert!(!is_overwrite_success_message("Everything is Ok"));
        assert!(!is_overwrite_success_message(""));
    }

    /// Parity test for capability C145: all eight substrings the source
    /// checks for are each individually recognized.
    #[test]
    fn recognizes_all_eight_prompt_substrings() {
        assert!(needs_manual_input("output.txt already exists"));
        assert!(needs_manual_input("overwrite output.txt?"));
        assert!(needs_manual_input("would you like to replace it"));
        assert!(needs_manual_input("Enter password:"));
        assert!(needs_manual_input("Not enough free space available"));
        assert!(needs_manual_input("you must choose a new filename"));
        assert!(needs_manual_input("Insert disk with volume 2"));
        assert!(needs_manual_input("[R]etry, [A]bort?"));
    }

    /// Parity test for capability C145: matching is case-insensitive,
    /// matching AutoIt's `StringInStr` default.
    #[test]
    fn matches_case_insensitively() {
        assert!(needs_manual_input("OVERWRITE output.txt?"));
        assert!(needs_manual_input("PASSWORD required"));
    }

    /// Parity test for capability C145: ordinary progress output doesn't
    /// match.
    #[test]
    fn does_not_match_ordinary_progress_output() {
        assert!(!needs_manual_input("Extracting: file.txt  50%"));
        assert!(!needs_manual_input(""));
    }

    /// Parity test for capability C162: each of the five whole-log
    /// substrings is recognized anywhere in the text.
    #[test]
    fn recognizes_all_five_whole_log_password_substrings() {
        assert!(is_password_failure("Wrong password?\r\nsome other line"));
        assert!(is_password_failure(
            "first line\r\nThe specified password is incorrect."
        ));
        assert!(is_password_failure("Archive encrypted.\r\nmore text"));
        assert!(is_password_failure("Corrupt file or wrong password\r\n"));
        assert!(is_password_failure("ERROR: Wrong password\r\ntail"));
    }

    /// Parity test for capability C162: "Enter password" on the true
    /// last line of a 3+-line log (2+ line breaks) is recognized.
    #[test]
    fn recognizes_enter_password_on_true_last_line() {
        assert!(is_password_failure(
            "starting extraction\r\nsome progress\r\nEnter password:"
        ));
    }

    /// Parity test for capability C162: "Enter password" on a
    /// *non*-last line of a 3+-line log is NOT recognized, since only
    /// the true last line is searched once there are 2+ line breaks.
    #[test]
    fn does_not_recognize_enter_password_on_earlier_line_of_long_log() {
        assert!(!is_password_failure(
            "Enter password:\r\nsome progress\r\ndone extracting"
        ));
    }

    /// Parity test for capability C162: the source quirk — a log with
    /// exactly one line break (two lines) has its *entire* text
    /// searched, not just the true last line, because
    /// `_StringGetLine($sLog, -1)` can't find a second-to-last `@CRLF`
    /// and falls back to returning the whole string unmodified.
    #[test]
    fn two_line_log_searches_whole_text_not_just_last_line() {
        assert!(is_password_failure("Enter password:\r\nsome progress"));
    }

    /// Parity test for capability C162: a single-line log (no line
    /// breaks at all) also has its entire text searched.
    #[test]
    fn single_line_log_searches_whole_text() {
        assert!(is_password_failure("Enter password:"));
    }

    /// Parity test for capability C162: matching is case-insensitive.
    #[test]
    fn password_failure_matches_case_insensitively() {
        assert!(is_password_failure("ARCHIVE ENCRYPTED."));
        assert!(is_password_failure("enter PASSWORD:"));
    }

    /// Parity test for capability C162: ordinary log text with none of
    /// the six substrings doesn't match.
    #[test]
    fn does_not_match_unrelated_password_log_text() {
        assert!(!is_password_failure(
            "Extracting: file.txt\r\nEverything is Ok"
        ));
        assert!(!is_password_failure(""));
    }
}
