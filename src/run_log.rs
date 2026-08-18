//! Per-run application log policy: whether a batch-mode error-log line
//! gets appended (C169) and whether `SaveLog()` actually writes a log
//! file at all for a given terminal status (C170). Ports pieces of
//! `terminate()` (UniExtract.au3:4216-4233) — not to be confused with
//! `log_eval`, which classifies a helper binary's captured *subprocess*
//! output, a different log entirely.

use crate::status::Status;

/// C169: ports the guard on the batch-mode error-log append
/// (UniExtract.au3:4216): `If $exitcode <> 0 And $silentmode And
/// $extract Then` — a line is appended only when the process is exiting
/// non-zero, running silently, and this was an extraction run (not a
/// scan-only run, `$extract` here being the "actually extract" flag, not
/// the `/scan` mode).
pub fn should_append_error_log(exit_code: i32, silent_mode: bool, is_extract: bool) -> bool {
    exit_code != 0 && silent_mode && is_extract
}

/// C169: ports the error-log line's exact format
/// (UniExtract.au3:4218): `<datetime> <name> (<STATUS-UPPERCASE>) -
/// <arctype>\r\n`. `filenamefull` is preferred over `fname` when
/// non-empty (source: `$filenamefull = ""? $fname: $filenamefull`).
/// `datetime` is caller-supplied — `GetDateTime()`'s current-time read is
/// real I/O, not part of this pure formatting step.
pub fn build_error_log_line(
    datetime: &str,
    filenamefull: &str,
    fname: &str,
    status_name: &str,
    arctype: &str,
) -> String {
    let name = if filenamefull.is_empty() {
        fname
    } else {
        filenamefull
    };
    format!(
        "{datetime} {name} ({}) - {arctype}\r\n",
        status_name.to_uppercase()
    )
}

/// C170: ports the guard deciding whether `SaveLog()` actually writes a
/// log file for this run (UniExtract.au3:4231-4233): logging must be
/// enabled and not already saved this run, and the status isn't one of
/// the five terminal statuses log-writing is suppressed for (`Silent`,
/// `Syntax`, `FileInfo`, `NotPacked`, `Batch`) — **unless** it's
/// `FileInfo` in silent mode, which writes a log unconditionally,
/// bypassing both the enabled and already-saved gates.
///
/// Operator precedence matters here and is easy to misread: AutoIt's
/// `And` binds tighter than `Or` (the same as most languages, despite
/// appearing at first glance to read left-to-right), so the source's
/// `A And B And C Or D` parses as `(A And B And C) Or D`, not `A And B
/// And (C Or D)` — exactly the shape this function reproduces.
pub fn should_save_log(
    create_log_enabled: bool,
    already_saved: bool,
    status: Status,
    silent_mode: bool,
) -> bool {
    let suppressed = matches!(
        status,
        Status::Silent
            | Status::Syntax
            | Status::FileInfo { .. }
            | Status::NotPacked
            | Status::Batch
    );
    (create_log_enabled && !already_saved && !suppressed)
        || (matches!(status, Status::FileInfo { .. }) && silent_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C169: the error-log line is appended
    /// only when all three conditions hold.
    #[test]
    fn should_append_error_log_requires_all_three_conditions() {
        assert!(should_append_error_log(1, true, true));
        assert!(!should_append_error_log(0, true, true));
        assert!(!should_append_error_log(1, false, true));
        assert!(!should_append_error_log(1, true, false));
    }

    /// Parity test for capability C169: the line format matches the
    /// source's exact concatenation, and `filenamefull` wins when
    /// non-empty.
    #[test]
    fn build_error_log_line_prefers_filenamefull_when_present() {
        assert_eq!(
            build_error_log_line(
                "2026-08-18 12:00:00",
                r"C:\downloads\archive.zip",
                "archive.zip",
                "failed",
                "zip"
            ),
            "2026-08-18 12:00:00 C:\\downloads\\archive.zip (FAILED) - zip\r\n"
        );
    }

    /// Parity test for capability C169: `fname` is used when
    /// `filenamefull` is empty.
    #[test]
    fn build_error_log_line_falls_back_to_fname_when_filenamefull_empty() {
        assert_eq!(
            build_error_log_line("2026-08-18 12:00:00", "", "archive.zip", "failed", "zip"),
            "2026-08-18 12:00:00 archive.zip (FAILED) - zip\r\n"
        );
    }

    /// Parity test for capability C170: logging enabled, not yet saved,
    /// and a non-suppressed status writes a log.
    #[test]
    fn should_save_log_writes_for_ordinary_enabled_case() {
        assert!(should_save_log(true, false, Status::Success, false));
        assert!(should_save_log(true, false, Status::Failed, false));
    }

    /// Parity test for capability C170: each of the five suppressed
    /// statuses is skipped when not the silent-FileInfo special case.
    #[test]
    fn should_save_log_suppresses_five_terminal_statuses() {
        for status in [
            Status::Silent,
            Status::Syntax,
            Status::FileInfo {
                silent_mode: false,
                filetype_identified: true,
            },
            Status::NotPacked,
            Status::Batch,
        ] {
            assert!(!should_save_log(true, false, status, false));
        }
    }

    /// Parity test for capability C170: `FileInfo` in silent mode writes
    /// a log unconditionally, even with logging disabled and already
    /// saved — the source's `Or ($status = $STATUS_FILEINFO And
    /// $silentmode)` branch bypasses both gates.
    #[test]
    fn should_save_log_fileinfo_in_silent_mode_bypasses_other_gates() {
        let fileinfo = Status::FileInfo {
            silent_mode: true,
            filetype_identified: true,
        };
        assert!(should_save_log(false, true, fileinfo, true));
    }

    /// Parity test for capability C170: logging disabled or already
    /// saved suppresses an otherwise-ordinary status.
    #[test]
    fn should_save_log_respects_enabled_and_already_saved_gates() {
        assert!(!should_save_log(false, false, Status::Success, false));
        assert!(!should_save_log(true, true, Status::Success, false));
    }
}
