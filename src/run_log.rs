//! Per-run application log policy: whether a batch-mode error-log line
//! gets appended (C169), whether `SaveLog()` actually writes a log file
//! at all for a given terminal status (C170), the log file's own name
//! (C165), and each debug line's own format (C164). Ports pieces of
//! `terminate()` (UniExtract.au3:4216-4233), `SaveLog()` itself
//! (UniExtract.au3:4764-4775), and `Cout()` (UniExtract.au3:5352-5357)
//! — not to be confused with `log_eval`, which classifies a helper
//! binary's captured *subprocess* output, a different log entirely.

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

/// C164: ports `Cout()`'s debug-line format (UniExtract.au3:5352-5357):
/// `<datetime>:<msec>\t<msg>\r\n`. `datetime` (`GetDateTime()`'s result,
/// itself `@YEAR-@MON-@MDAY @HOUR:@MIN:@SEC`) and `msec` (`@MSEC`) are
/// both real-clock reads and stay the caller's job, matching this
/// module's existing `datetime` convention (C169's
/// `build_error_log_line`).
///
/// **Scope:** the source appends every formatted line onto a
/// growing `$sFullLog` string for the whole run's duration ("buffered
/// in memory for the full run") — that accumulation is the caller's own
/// trivial responsibility (one `push_str` per call), not modeled as its
/// own function here. `ConsoleWrite`ing the line when not running as a
/// compiled executable (`If Not @Compiled Then ConsoleWrite(...)`) is
/// also real I/O, out of scope.
pub fn build_debug_line(datetime: &str, msec: &str, msg: &str) -> String {
    format!("{datetime}:{msec}\t{msg}\r\n")
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

/// C165: ports `SaveLog()`'s log file name construction
/// (UniExtract.au3:4765-4768):
///
/// ```text
/// $sName = $logdir & @YEAR & "-" & @MON & "-" & @MDAY & "_" & @HOUR & "-" & @MIN & "-" & @SEC & "_"
/// If $status <> $STATUS_SUCCESS Then $sName &= StringUpper($status)
/// If $file <> "" Then $sName &= "_" & GetFileName() & "." & $fileext
/// $sName &= ".log"
/// ```
///
/// `timestamp` is caller-supplied — reading `@YEAR`/`@MON`/.../`@SEC`
/// is real I/O, matching this crate's existing convention (see
/// [`build_error_log_line`]'s `datetime` parameter). `file_name` is
/// `GetFileName()`'s result — itself a runtime choice between the
/// unicode-relocation working name and the real one (capabilities
/// C159/C175, not yet ported); the caller resolves it, this function
/// only formats.
///
/// **Quirk, preserved as-is:** the trailing `"_"` after `timestamp` is
/// unconditional, and the `"_"` before the file segment is *also*
/// unconditional — neither one only appears when needed to separate two
/// non-empty pieces. A successful run (`is_success = true`) with a
/// non-empty `file` therefore gets a doubled `"__"` between the
/// timestamp and the file name (no status marker was appended to
/// consume the first `"_"`), and a successful run with an empty `file`
/// ends up with that first `"_"` immediately followed by `".log"` — e.g.
/// `...12-00-00_.log`. Neither is a typo in this port; the source
/// produces exactly this.
pub fn build_log_file_name(
    logdir: &str,
    timestamp: &str,
    is_success: bool,
    status_name: &str,
    file: &str,
    file_name: &str,
    file_ext: &str,
) -> String {
    let mut name = format!("{logdir}{timestamp}_");
    if !is_success {
        name.push_str(&status_name.to_uppercase());
    }
    if !file.is_empty() {
        name.push('_');
        name.push_str(file_name);
        name.push('.');
        name.push_str(file_ext);
    }
    name.push_str(".log");
    name
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

    /// Parity test for capability C165: a failed run with a file
    /// includes both the uppercased status marker and the
    /// `_<name>.<ext>` segment.
    #[test]
    fn build_log_file_name_failed_run_includes_status_and_file() {
        assert_eq!(
            build_log_file_name(
                r"C:\settings\log\",
                "2026-08-18_12-00-00",
                false,
                "failed",
                r"C:\downloads\archive.zip",
                "archive",
                "zip"
            ),
            r"C:\settings\log\2026-08-18_12-00-00_FAILED_archive.zip.log"
        );
    }

    /// Parity test for capability C165: a successful run with a file
    /// omits the status marker, but still includes the file segment —
    /// note the double underscore this leaves behind, since the source
    /// unconditionally prefixes the file segment with its own `"_"`
    /// regardless of whether a status marker was appended just before
    /// it.
    #[test]
    fn build_log_file_name_success_run_omits_status_marker() {
        assert_eq!(
            build_log_file_name(
                r"C:\settings\log\",
                "2026-08-18_12-00-00",
                true,
                "success",
                r"C:\downloads\archive.zip",
                "archive",
                "zip"
            ),
            r"C:\settings\log\2026-08-18_12-00-00__archive.zip.log"
        );
    }

    /// Parity test for capability C165: a successful run with no file
    /// (e.g. a syntax error before a file was resolved) reproduces the
    /// source's trailing-underscore-before-extension quirk exactly.
    #[test]
    fn build_log_file_name_success_no_file_reproduces_trailing_underscore_quirk() {
        assert_eq!(
            build_log_file_name(
                r"C:\settings\log\",
                "2026-08-18_12-00-00",
                true,
                "success",
                "",
                "",
                ""
            ),
            r"C:\settings\log\2026-08-18_12-00-00_.log"
        );
    }

    /// Parity test for capability C165: a failed run with no file
    /// includes the status marker but not the file segment.
    #[test]
    fn build_log_file_name_failed_no_file_includes_status_only() {
        assert_eq!(
            build_log_file_name(
                r"C:\settings\log\",
                "2026-08-18_12-00-00",
                false,
                "syntax",
                "",
                "",
                ""
            ),
            r"C:\settings\log\2026-08-18_12-00-00_SYNTAX.log"
        );
    }

    /// Parity test for capability C164: the debug-line format matches
    /// the source's exact concatenation — datetime, colon, millisecond,
    /// tab, message, CRLF.
    #[test]
    fn build_debug_line_matches_source_format() {
        assert_eq!(
            build_debug_line("2026-08-18 12:00:00", "123", "Starting extraction"),
            "2026-08-18 12:00:00:123\tStarting extraction\r\n"
        );
    }

    /// Parity test for capability C164: an empty message still produces
    /// a well-formed line (timestamp/tab/CRLF present, message empty).
    #[test]
    fn build_debug_line_with_empty_message() {
        assert_eq!(
            build_debug_line("2026-08-18 12:00:00", "007", ""),
            "2026-08-18 12:00:00:007\t\r\n"
        );
    }
}
