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
    use super::{is_overwrite_success_message, needs_manual_input};

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
}
