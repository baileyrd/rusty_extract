//! Extraction output-log evaluation: ports `EvaluateLog()`
//! (UniExtract.au3:4778-4825) and `ParseWarnings()`
//! (UniExtract.au3:4832-4845), the substring-classifier and
//! warning-block extractor `extract()` runs over a helper binary's
//! captured stdout/stderr. `EvaluateLog()` is a long `ElseIf` chain —
//! invalid password (C162), user cancellation, low disk space, missing
//! archive part, several generic success/failure phrasings, and finally
//! the overwrite case (C144) — each branch has its own predicate
//! function, individually documented and independently testable, plus
//! [`evaluate_log`] (C167) which applies them all in the source's exact
//! priority order.

/// C144: ports the "already exists"/"Overwrite" branch of `EvaluateLog()`
/// (UniExtract.au3:4819-4823): a log mentioning either substring is
/// treated as `$RESULT_SUCCESS`, not a failure — the source's own
/// reasoning, right there in the comment, is that an overwritten file
/// leaves the output folder's total size roughly unchanged, so the
/// separate "did the folder size change" check that would otherwise
/// flag this as a failure gets skipped for exactly that reason.
///
/// **Scope — one branch, standalone use.** This predicate only
/// reproduces this one `ElseIf` arm; it's the *last* one in the chain,
/// reached only once every higher-priority classification (invalid
/// password, user cancellation, low disk space, missing archive part,
/// and the generic success/failure phrasings, UniExtract.au3:4778-4818)
/// has failed to match. A caller applying this predicate on its own must
/// replicate that ordering; [`evaluate_log`] already does, for every
/// branch, if the whole chain is what's needed.
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

/// Ports the user-cancellation branch of `EvaluateLog()`
/// (UniExtract.au3:4788): `$RESULT_CANCELED`. Matches case-insensitively
/// (all three calls are bare `StringInStr`).
pub fn is_canceled_message(log: &str) -> bool {
    let lower = log.to_lowercase();
    ["break signaled", "program aborted", "user break"]
        .iter()
        .any(|needle| lower.contains(needle))
}

/// Ports the low-disk-space branch of `EvaluateLog()`
/// (UniExtract.au3:4791-4792): `$RESULT_NOFREESPACE`. Matches
/// case-insensitively (both calls are bare `StringInStr`).
pub fn is_no_free_space_message(log: &str) -> bool {
    let lower = log.to_lowercase();
    lower.contains("there is not enough space on the disk")
        || lower.contains(
            "[x] there is not enough space in working directory. unpacking would most likely fail!",
        )
}

/// Ports the missing-archive-part branch of `EvaluateLog()`
/// (UniExtract.au3:4795-4796): `$RESULT_FAILED`. Matches
/// case-insensitively (all three calls are bare `StringInStr`).
pub fn is_missing_part_message(log: &str) -> bool {
    let lower = log.to_lowercase();
    [
        "you need to start extraction from a previous volume",
        "unavailable start of archive",
        "missing volume",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// Ports the generic-success branch of `EvaluateLog()`
/// (UniExtract.au3:4800-4806): `$RESULT_SUCCESS`. Matches
/// case-insensitively (all thirteen calls are bare `StringInStr`). The
/// literal tab character inside `"Result:\tSuccessful, errorcode 0"`
/// (shown as a gap in the source) is preserved exactly.
pub fn is_generic_success_message(log: &str) -> bool {
    let lower = log.to_lowercase();
    [
        "everything is ok",
        "0 failed",
        "all files ok",
        "all ok",
        "done.",
        "done ...",
        ": done",
        "result:\tsuccessful, errorcode 0",
        "... successful",
        "extract files [ ",
        "done; file is ok",
        "successfully extracted to",
        "[+] finished!",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// Ports the generic-failure branch of `EvaluateLog()`
/// (UniExtract.au3:4809-4816): `$RESULT_FAILED`.
///
/// **Behavioral finding — mixed case sensitivity within one branch,**
/// unlike every other arm of this `ElseIf` chain (all uniformly
/// case-insensitive): five of these `StringInStr` calls pass an
/// explicit case-sensitive mode (`1`) — `"err code("`, `"stacktrace"`,
/// `"Write error: "`, `"ERROR: Wrong tag in package"`, and
/// `"unzip:  cannot find"` (note the double space) — while the
/// remaining nine are bare (case-insensitive), matching every other
/// branch's convention. **One nested `And`:** `"Cannot create"` and
/// `"No files to extract"` (both case-sensitive) must *both* appear for
/// that pair to count, unlike every other substring here which is
/// independently `Or`'d.
pub fn is_generic_failure_message(log: &str) -> bool {
    let case_sensitive_hit = [
        "err code(",
        "stacktrace",
        "Write error: ",
        "ERROR: Wrong tag in package",
        "unzip:  cannot find",
    ]
    .iter()
    .any(|needle| log.contains(needle));

    let cannot_create_and_no_files =
        log.contains("Cannot create") && log.contains("No files to extract");

    let lower = log.to_lowercase();
    let case_insensitive_hit = [
        "archives with errors: 1",
        "open error: can not open the file as",
        "error: system.exception:",
        "unknown wise-version -> contact author",
        "critical error:",
        "[error] ",
        "mainheadernotfounderror",
        "*** error:",
        "expected section name \".enigma2\"",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    case_sensitive_hit || cannot_create_and_no_files || case_insensitive_hit
}

/// C167: what a log gets classified as by `EvaluateLog()`'s `ElseIf`
/// chain (UniExtract.au3:4778-4825), in the source's own priority
/// order. `Unclassified` means no branch matched, leaving `$success`
/// at whatever it already was (typically `$RESULT_UNKNOWN`, feeding
/// into this crate's `result_heuristic` module).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogEvalOutcome {
    /// `$RESULT_FAILED`, `SetError(1, 1)` (UniExtract.au3:4785-4787).
    PasswordFailure,
    /// `$RESULT_CANCELED` (UniExtract.au3:4789-4790).
    Canceled,
    /// `$RESULT_NOFREESPACE`, `SetError(2)` (UniExtract.au3:4793-4794).
    NoFreeSpace,
    /// `$RESULT_FAILED`, `SetError(3)` (UniExtract.au3:4797-4799).
    MissingPart,
    /// `$RESULT_SUCCESS` (UniExtract.au3:4807-4808).
    Success,
    /// `$RESULT_FAILED`, `SetError(1)` (UniExtract.au3:4817-4818).
    Failed,
    /// `$RESULT_SUCCESS` (UniExtract.au3:4820-4823) — see
    /// [`is_overwrite_success_message`]'s doc comment for why an
    /// overwrite prompt counts as success here.
    OverwriteSuccess,
    /// No arm matched.
    Unclassified,
}

/// C167: ports the whole of `EvaluateLog()`'s classification chain
/// (UniExtract.au3:4782-4825) as a single ordered decision, reusing
/// every already-shipped branch predicate ([`is_password_failure`]
/// C162, [`is_overwrite_success_message`] C144) alongside the five new
/// ones this capability adds. Branch order matters — this function
/// checks them in exactly the source's `ElseIf` sequence, so a caller
/// no longer needs to replicate that ordering itself (unlike the
/// individual predicates' own doc comments, which still describe it for
/// standalone use).
pub fn evaluate_log(log: &str) -> LogEvalOutcome {
    if is_password_failure(log) {
        LogEvalOutcome::PasswordFailure
    } else if is_canceled_message(log) {
        LogEvalOutcome::Canceled
    } else if is_no_free_space_message(log) {
        LogEvalOutcome::NoFreeSpace
    } else if is_missing_part_message(log) {
        LogEvalOutcome::MissingPart
    } else if is_generic_success_message(log) {
        LogEvalOutcome::Success
    } else if is_generic_failure_message(log) {
        LogEvalOutcome::Failed
    } else if is_overwrite_success_message(log) {
        LogEvalOutcome::OverwriteSuccess
    } else {
        LogEvalOutcome::Unclassified
    }
}

/// Reproduces `_StringExtractAfter($sString, $sSubstring, $sEnd =
/// @CRLF)` (UniExtract.au3:4586-4594): finds `marker` (case-insensitive,
/// matching its bare `StringInStr` call), then returns everything from
/// right after that match up to the next occurrence of `end`
/// (case-insensitive). Returns `None` if either isn't found, matching
/// the source's two `SetError` cases.
///
/// Case-insensitive matching is done by lowercasing the whole string
/// once; that only preserves byte offsets between the lowercased and
/// original text for text whose case-folding doesn't change its byte
/// length (true for plain ASCII, which is what these helper-binary
/// output logs are in practice) — this function bails out (`None`)
/// rather than risk a misaligned slice when that's not the case.
fn extract_after<'a>(log: &'a str, marker: &str, end: &str) -> Option<&'a str> {
    let lower = log.to_lowercase();
    if lower.len() != log.len() {
        return None;
    }
    let match_start = lower.find(&marker.to_lowercase())?;
    let start = match_start + marker.len();
    let end_pos = start + lower.get(start..)?.find(&end.to_lowercase())?;
    Some(&log[start..end_pos])
}

/// Reproduces `_StringInStrGetLine($sString, $sSubstring, $sLineEnd =
/// @CRLF)` (UniExtract.au3:4597-4609): finds `needle` (case-insensitive)
/// and returns the whole line it appears on — scanning backward from
/// the match for the previous `@CRLF` (or the start of the string if
/// there isn't one) and forward from the end of the match for the next
/// `@CRLF` (or the end of the string). `@CRLF` itself is ASCII, so its
/// scan doesn't need the lowercased copy; only the `needle` search
/// does, under the same byte-length-preservation guard as
/// [`extract_after`].
fn in_str_get_line<'a>(log: &'a str, needle: &str) -> Option<&'a str> {
    let lower = log.to_lowercase();
    if lower.len() != log.len() {
        return None;
    }
    let pos = lower.find(&needle.to_lowercase())?;

    let prefix_end = (pos + 1).min(log.len());
    let start = match log[..prefix_end].rfind("\r\n") {
        Some(p) => p + 2,
        None => 0,
    };

    let search_from = pos + needle.len();
    let end = log[search_from..]
        .find("\r\n")
        .map(|p| search_from + p)
        .unwrap_or(log.len());

    Some(&log[start..end])
}

/// C167: ports `ParseWarnings()` (UniExtract.au3:4832-4845) — three
/// tool-specific warning-block extractions, each appended to the result
/// (source: `AddWarning()`, a bare push onto a global array) only when
/// found. Order matches the source: 7-Zip's `WARNINGS:` block first,
/// then UnRAR's checksum-error line, then a generic `Open WARNING: `
/// line.
pub fn parse_warnings(log: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    if let Some(w) = extract_after(log, "WARNINGS:\r\n", "\r\n") {
        warnings.push(w.to_string());
    }
    if let Some(w) = in_str_get_line(log, " - checksum error") {
        warnings.push(w.to_string());
    }
    if let Some(w) = extract_after(log, "Open WARNING: ", "\r\n") {
        warnings.push(w.to_string());
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_log, is_canceled_message, is_generic_failure_message, is_generic_success_message,
        is_missing_part_message, is_no_free_space_message, is_overwrite_success_message,
        is_password_failure, needs_manual_input, parse_warnings, LogEvalOutcome,
    };

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

    /// Parity test for capability C167: each of the three cancellation
    /// substrings is recognized.
    #[test]
    fn recognizes_all_three_cancellation_substrings() {
        assert!(is_canceled_message("Break signaled"));
        assert!(is_canceled_message("Program aborted"));
        assert!(is_canceled_message("User break"));
        assert!(is_canceled_message("USER BREAK")); // case-insensitive
    }

    /// Parity test for capability C167: each of the two no-free-space
    /// substrings is recognized.
    #[test]
    fn recognizes_both_no_free_space_substrings() {
        assert!(is_no_free_space_message(
            "There is not enough space on the disk"
        ));
        assert!(is_no_free_space_message(
            "[x] There is not enough space in working directory. Unpacking would most likely fail!"
        ));
    }

    /// Parity test for capability C167: each of the three missing-part
    /// substrings is recognized.
    #[test]
    fn recognizes_all_three_missing_part_substrings() {
        assert!(is_missing_part_message(
            "You need to start extraction from a previous volume"
        ));
        assert!(is_missing_part_message("Unavailable start of archive"));
        assert!(is_missing_part_message("Missing volume"));
    }

    /// Parity test for capability C167: each of the thirteen
    /// generic-success substrings is recognized, including the tab
    /// character embedded in one of them.
    #[test]
    fn recognizes_all_thirteen_generic_success_substrings() {
        for text in [
            "Everything is Ok",
            "0 failed",
            "All files OK",
            "All OK",
            "done.",
            "Done ...",
            ": done",
            "Result:\tSuccessful, errorcode 0",
            "... Successful",
            "Extract files [ ",
            "Done; file is OK",
            "Successfully extracted to",
            "[+] Finished!",
        ] {
            assert!(is_generic_success_message(text), "expected match: {text}");
        }
    }

    /// Parity test for capability C167: the five case-sensitive
    /// substrings only match with exact casing.
    #[test]
    fn generic_failure_case_sensitive_substrings_require_exact_case() {
        assert!(is_generic_failure_message("err code(1)"));
        assert!(!is_generic_failure_message("ERR CODE(1)"));
        assert!(is_generic_failure_message("a stacktrace follows"));
        assert!(!is_generic_failure_message("a STACKTRACE follows"));
    }

    /// Parity test for capability C167: the nested `And` pair only
    /// counts when *both* substrings are present.
    #[test]
    fn generic_failure_and_combo_requires_both_substrings() {
        assert!(is_generic_failure_message(
            "Cannot create output\r\nNo files to extract"
        ));
        assert!(!is_generic_failure_message("Cannot create output"));
        assert!(!is_generic_failure_message("No files to extract"));
    }

    /// Parity test for capability C167: the nine case-insensitive
    /// substrings match regardless of case.
    #[test]
    fn generic_failure_case_insensitive_substrings_match_any_case() {
        assert!(is_generic_failure_message("ARCHIVES WITH ERRORS: 1"));
        assert!(is_generic_failure_message("critical error: disk full"));
    }

    /// Parity test for capability C167: `evaluate_log` applies the
    /// branches in the source's exact priority order — a password
    /// failure wins even when generic-success text also appears.
    #[test]
    fn evaluate_log_password_failure_takes_priority_over_success_text() {
        assert_eq!(
            evaluate_log("Everything is Ok\r\nWrong password?"),
            LogEvalOutcome::PasswordFailure
        );
    }

    /// Parity test for capability C167: `evaluate_log` covers every
    /// classification in order.
    #[test]
    fn evaluate_log_classifies_each_outcome() {
        assert_eq!(evaluate_log("User break"), LogEvalOutcome::Canceled);
        assert_eq!(
            evaluate_log("There is not enough space on the disk"),
            LogEvalOutcome::NoFreeSpace
        );
        assert_eq!(evaluate_log("Missing volume"), LogEvalOutcome::MissingPart);
        assert_eq!(evaluate_log("Everything is Ok"), LogEvalOutcome::Success);
        assert_eq!(evaluate_log("stacktrace"), LogEvalOutcome::Failed);
        assert_eq!(
            evaluate_log("output.txt already exists."),
            LogEvalOutcome::OverwriteSuccess
        );
        assert_eq!(
            evaluate_log("nothing recognizable here"),
            LogEvalOutcome::Unclassified
        );
    }

    /// Parity test for capability C167: `parse_warnings` extracts the
    /// 7-Zip `WARNINGS:` block.
    #[test]
    fn parse_warnings_extracts_7zip_warnings_block() {
        let log = "some output\r\nWARNINGS:\r\n1 file was skipped\r\nmore output";
        assert_eq!(parse_warnings(log), vec!["1 file was skipped"]);
    }

    /// Parity test for capability C167: `parse_warnings` extracts the
    /// UnRAR checksum-error line in full.
    #[test]
    fn parse_warnings_extracts_unrar_checksum_error_line() {
        let log = "extracting file.txt\r\nfile.dat - checksum error\r\ndone";
        assert_eq!(parse_warnings(log), vec!["file.dat - checksum error"]);
    }

    /// Parity test for capability C167: `parse_warnings` extracts a
    /// generic `Open WARNING: ` line.
    #[test]
    fn parse_warnings_extracts_open_warning_line() {
        let log = "start\r\nOpen WARNING: archive header truncated\r\nend";
        assert_eq!(parse_warnings(log), vec!["archive header truncated"]);
    }

    /// Parity test for capability C167: `parse_warnings` can return
    /// multiple warnings from a single log, and none when nothing
    /// matches.
    #[test]
    fn parse_warnings_collects_multiple_and_none() {
        let log =
            "junk\r\nWARNINGS:\r\nheader warning\r\nfile.dat - checksum error\r\nOpen WARNING: trailer bad\r\n";
        assert_eq!(
            parse_warnings(log),
            vec!["header warning", "file.dat - checksum error", "trailer bad"]
        );
        assert!(parse_warnings("nothing interesting here").is_empty());
    }
}
