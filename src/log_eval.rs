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

#[cfg(test)]
mod tests {
    use super::is_overwrite_success_message;

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
}
