//! Feedback dialog and submission: ports `GUI_Feedback`,
//! `GUI_Feedback_Outdated`, `GUI_Feedback_Send`, `GUI_Feedback_Error`
//! (UniExtract.au3:6756-6935) — the system-info/log/message capture
//! dialog, the remote-version staleness check shown inline, and the
//! hand-rolled multipart HTTP POST that submits it all to a fixed
//! endpoint.
//!
//! **This is the heaviest PII surface in this migration phase** — none of
//! it is transmitted, or even assembled with real data, by this module.
//! Every function here is pure string/byte formatting over caller-supplied
//! values (sample file name/size/hash, the captured log text, the
//! free-text message, OS/locale info, the per-install GUID); the real
//! network request (`winhttp.winhttprequest.5.1`), the actual zlib
//! compression, and the privacy-policy-checkbox gate on the real dialog
//! are the caller's job, driven by the outcomes these functions return.
//! Nothing here decides *whether* to submit PII — only what the submission
//! looks like once the caller has already decided to send it.

use crate::update_orchestration::{main_executable_needs_update, UpdateMode};

/// Ports `GUI_Feedback`'s opening block (UniExtract.au3:6760-6766):
/// attaching a sample file triggers a hex dump of its first 1024 bytes,
/// but only for non-executable files. **Verified quirk, preserved rather
/// than "fixed"**: this happens unconditionally the instant the dialog
/// opens with a file attached — before the user has typed a message,
/// reviewed the pre-filled log text, or decided whether to actually send
/// anything. The dump is appended to the persistent session log
/// (`$sFullLog`), not scoped to this one feedback attempt, so it can
/// outlive a cancelled submission and resurface in a *later* one.
pub fn should_hex_dump_sample_file(has_file: bool, is_exe: bool) -> bool {
    has_file && !is_exe
}

/// Ports the unconditional metadata dump alongside the hex dump
/// (UniExtract.au3:6763-6764) — logged for any attached file, executable
/// or not.
pub fn should_log_file_metadata(has_file: bool) -> bool {
    has_file
}

/// Ports `Global $bOptAskForFeedback = 0` (UniExtract.au3:6765): opening
/// the feedback dialog with a file attached disables the "ask for
/// feedback" prompt for future runs, regardless of whether this feedback
/// attempt is ever actually sent.
pub fn should_disable_future_feedback_prompt(has_file: bool) -> bool {
    has_file
}

/// Ports the outdated-version check shown inline in the feedback dialog
/// (UniExtract.au3:6801-6802): `IsArray($aReturn) And (($aReturn[0])[1]
/// <> FileGetSize($sUniExtract) Or FileGetMD5($sUniExtract) <>
/// ($aReturn[0])[2])`. This is the exact same main-executable comparison
/// `CheckUpdate` (C207) uses — reused here directly via
/// [`main_executable_needs_update`] with [`UpdateMode::All`] (this call
/// site has no helper-only mode to gate against) rather than re-deriving
/// the size/hash comparison a second time.
pub fn should_show_outdated_warning(
    index_fetch_succeeded: bool,
    local_size: i64,
    index_size: i64,
    local_md5: &str,
    index_md5: &str,
) -> bool {
    index_fetch_succeeded
        && main_executable_needs_update(
            UpdateMode::All,
            local_size,
            index_size,
            local_md5,
            index_md5,
        )
}

/// Ports `GUI_Feedback_Send`'s empty-submission guard
/// (UniExtract.au3:6861): reject a submission with nothing in any of the
/// three fields.
pub fn should_reject_empty_feedback(file: &str, output: &str, message: &str) -> bool {
    file.is_empty() && output.is_empty() && message.is_empty()
}

/// The fields `GUI_Feedback_Send` interpolates into the human-readable
/// report body (UniExtract.au3:6867-6875).
pub struct FeedbackReport<'a> {
    pub app_name: &'a str,
    pub app_version: &'a str,
    pub exe_timestamp: &'a str,
    pub window_title: &'a str,
    pub sys_info: &'a str,
    pub sample_file: &'a str,
    pub file_size: &'a str,
    pub file_hash: &'a str,
    pub file_type: &'a str,
    pub message: &'a str,
    pub output_log: &'a str,
    pub guid: &'a str,
}

/// Ports the `$FB_Text` concatenation verbatim (UniExtract.au3:6867-6875),
/// including its exact 100-dash separator lines and `\r\n` line endings.
pub fn build_feedback_text(report: &FeedbackReport<'_>) -> String {
    const SEPARATOR: &str = "----------------------------------------------------------------------------------------------------";
    let FeedbackReport {
        app_name,
        app_version,
        exe_timestamp,
        window_title,
        sys_info,
        sample_file,
        file_size,
        file_hash,
        file_type,
        message,
        output_log,
        guid,
    } = *report;

    format!(
        "{app_name} Feedback v{app_version} ({exe_timestamp})\r\n\
{SEPARATOR}\r\n\r\n\
System Information: {window_title}, {sys_info}\r\n\r\n\
Sample file: {sample_file}\r\n\
File size: {file_size}\r\n\
File hash: {file_hash}\r\n\r\n\
File type: {file_type}\r\n\r\n\
Message: {message}\r\n\r\n\
{SEPARATOR}\r\n\r\n\
Output:\r\n\
{output_log}\r\n\r\n\
{SEPARATOR}\r\n\
Sent by: \r\n\
{guid}"
    )
}

/// Ports `$bUseGzip` (UniExtract.au3:6879): `StringInStr($language,
/// "Chinese") Or $language = "Japanese" Or $iSize > 1024 * 1024`. AutoIt's
/// default `StringInStr`/`=` comparisons here are case-insensitive,
/// matched with `eq_ignore_ascii_case`/a lowercased `contains` rather than
/// an exact-case match.
pub fn should_use_gzip(language: &str, text_byte_len: usize) -> bool {
    language.to_ascii_lowercase().contains("chinese")
        || language.eq_ignore_ascii_case("japanese")
        || text_byte_len > 1024 * 1024
}

/// The multipart boundary token, including its `--` prefix as used in the
/// body's delimiter lines (UniExtract.au3:6877).
pub const MULTIPART_BOUNDARY: &str = "--UniExtractLog";

/// The boundary value as it appears in the `Content-Type` header
/// (UniExtract.au3:6900): `StringTrimLeft($boundary, 2)` — the same token
/// with its leading `--` stripped, per the multipart/form-data convention.
pub fn multipart_boundary_header_value() -> &'static str {
    "UniExtractLog"
}

/// Ports the `Content-Type` request header value
/// (UniExtract.au3:6900).
pub fn build_multipart_content_type_header() -> String {
    format!(
        "multipart/form-data; boundary={}",
        multipart_boundary_header_value()
    )
}

/// The feedback body part, either as plain text or as bytes already
/// compressed by the caller (this module doesn't reimplement zlib).
pub enum FeedbackBodyEncoding<'a> {
    Plain(&'a str),
    /// Pre-compressed bytes, matching `StringTrimLeft(String(_Zlib_Compress(...)),
    /// 2)` (UniExtract.au3:6882) — the caller performs the compression and
    /// strips whatever leading marker bytes their zlib binding adds.
    Gzip(&'a [u8]),
}

/// Ports the `Content-Type: gzip`/`Content-Type: text/plain` wrapping
/// (UniExtract.au3:6881-6885).
pub fn wrap_feedback_part(encoding: &FeedbackBodyEncoding<'_>) -> Vec<u8> {
    match encoding {
        FeedbackBodyEncoding::Plain(text) => {
            format!("Content-Type: text/plain\r\n\r\n{text}").into_bytes()
        }
        FeedbackBodyEncoding::Gzip(bytes) => {
            let mut out = b"Content-Type: gzip\r\n\r\n".to_vec();
            out.extend_from_slice(bytes);
            out
        }
    }
}

/// Ports the full multipart body assembly (UniExtract.au3:6887-6888)
/// verbatim: a `file` part carrying the wrapped feedback text/bytes, an
/// `id` part carrying the plain GUID, and the closing boundary.
pub fn build_multipart_body(wrapped_feedback_part: &[u8], guid: &str) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(MULTIPART_BOUNDARY.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"UE_Feedback\"\r\n",
    );
    body.extend_from_slice(wrapped_feedback_part);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(MULTIPART_BOUNDARY.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"id\"\r\n\r\n");
    body.extend_from_slice(guid.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(MULTIPART_BOUNDARY.as_bytes());
    body.extend_from_slice(b"--");
    body
}

/// Ports `If $sResponse = "1" Then ...` (UniExtract.au3:6914). **Verified
/// bug, preserved rather than "fixed"**: this is a bare string-equality
/// check against the literal `"1"` in the HTTP response body — there is no
/// HTTP status-code check at all. A `200 OK` response with any other body
/// (including a server error page) is indistinguishable here from a
/// genuine transport failure; both fall through to
/// [`resolve_feedback_error_message`].
pub fn feedback_submission_succeeded(response_text: &str) -> bool {
    response_text == "1"
}

/// Ports `GUI_Feedback_Error`'s message selection
/// (UniExtract.au3:6919): `$sComError == 0 ? "Invalid response from
/// server" : $sComError`. `com_error` is `None` when `$sComError` is still
/// its initial `0` (no COM-level exception occurred — the failure is just
/// an unexpected response body), `Some(text)` when the COM error handler
/// actually recorded one.
pub fn resolve_feedback_error_message(com_error: Option<&str>) -> String {
    match com_error {
        None => "Invalid response from server".to_string(),
        Some(text) => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_feedback_text, build_multipart_body, build_multipart_content_type_header,
        feedback_submission_succeeded, multipart_boundary_header_value,
        resolve_feedback_error_message, should_disable_future_feedback_prompt,
        should_hex_dump_sample_file, should_log_file_metadata, should_reject_empty_feedback,
        should_show_outdated_warning, should_use_gzip, wrap_feedback_part, FeedbackBodyEncoding,
        FeedbackReport,
    };

    #[test]
    fn hex_dump_skipped_for_executables_but_not_other_files() {
        assert!(should_hex_dump_sample_file(true, false));
        assert!(!should_hex_dump_sample_file(true, true));
        assert!(!should_hex_dump_sample_file(false, false));
    }

    #[test]
    fn metadata_logged_for_any_attached_file_including_executables() {
        assert!(should_log_file_metadata(true));
        assert!(!should_log_file_metadata(false));
    }

    #[test]
    fn future_feedback_prompt_disabled_whenever_a_file_is_attached() {
        assert!(should_disable_future_feedback_prompt(true));
        assert!(!should_disable_future_feedback_prompt(false));
    }

    #[test]
    fn outdated_warning_requires_successful_fetch_and_a_mismatch() {
        assert!(should_show_outdated_warning(true, 100, 200, "a", "a"));
        assert!(should_show_outdated_warning(true, 100, 100, "a", "b"));
        assert!(!should_show_outdated_warning(true, 100, 100, "a", "a"));
        assert!(!should_show_outdated_warning(false, 100, 200, "a", "a"));
    }

    #[test]
    fn empty_feedback_rejected_only_when_all_three_fields_are_empty() {
        assert!(should_reject_empty_feedback("", "", ""));
        assert!(!should_reject_empty_feedback("file.zip", "", ""));
        assert!(!should_reject_empty_feedback("", "log output", ""));
        assert!(!should_reject_empty_feedback("", "", "a message"));
    }

    #[test]
    fn feedback_text_matches_source_layout_exactly() {
        let report = FeedbackReport {
            app_name: "UniExtract",
            app_version: "2.0.0",
            exe_timestamp: "2026-01-01",
            window_title: "UniExtract2",
            sys_info: "WIN11 X64, Lang: en, UE: English",
            sample_file: "archive.zip",
            file_size: "1234",
            file_hash: "deadbeef",
            file_type: "Zip",
            message: "it broke",
            output_log: "extraction log here",
            guid: "guid-1234",
        };
        let text = build_feedback_text(&report);
        let sep = "-".repeat(100);
        let expected = format!(
            "UniExtract Feedback v2.0.0 (2026-01-01)\r\n{sep}\r\n\r\nSystem Information: UniExtract2, WIN11 X64, Lang: en, UE: English\r\n\r\nSample file: archive.zip\r\nFile size: 1234\r\nFile hash: deadbeef\r\n\r\nFile type: Zip\r\n\r\nMessage: it broke\r\n\r\n{sep}\r\n\r\nOutput:\r\nextraction log here\r\n\r\n{sep}\r\nSent by: \r\nguid-1234"
        );
        assert_eq!(text, expected);
    }

    #[test]
    fn gzip_chosen_for_chinese_or_japanese_language_case_insensitively() {
        assert!(should_use_gzip("Chinese (Simplified)", 10));
        assert!(should_use_gzip("chinese", 10));
        assert!(should_use_gzip("Japanese", 10));
        assert!(should_use_gzip("JAPANESE", 10));
        assert!(!should_use_gzip("English", 10));
    }

    #[test]
    fn gzip_chosen_when_text_exceeds_one_megabyte() {
        assert!(should_use_gzip("English", 1024 * 1024 + 1));
        assert!(!should_use_gzip("English", 1024 * 1024));
    }

    #[test]
    fn content_type_header_uses_boundary_without_leading_dashes() {
        assert_eq!(multipart_boundary_header_value(), "UniExtractLog");
        assert_eq!(
            build_multipart_content_type_header(),
            "multipart/form-data; boundary=UniExtractLog"
        );
    }

    #[test]
    fn plain_part_wraps_with_plain_text_content_type() {
        let wrapped = wrap_feedback_part(&FeedbackBodyEncoding::Plain("hello"));
        assert_eq!(wrapped, b"Content-Type: text/plain\r\n\r\nhello");
    }

    #[test]
    fn gzip_part_wraps_with_gzip_content_type() {
        let wrapped = wrap_feedback_part(&FeedbackBodyEncoding::Gzip(&[1, 2, 3]));
        let mut expected = b"Content-Type: gzip\r\n\r\n".to_vec();
        expected.extend_from_slice(&[1, 2, 3]);
        assert_eq!(wrapped, expected);
    }

    #[test]
    fn multipart_body_matches_source_structure_exactly() {
        let wrapped = wrap_feedback_part(&FeedbackBodyEncoding::Plain("report text"));
        let body = build_multipart_body(&wrapped, "guid-1234");
        let expected = b"--UniExtractLog\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"UE_Feedback\"\r\n\
Content-Type: text/plain\r\n\r\n\
report text\r\n\
--UniExtractLog\r\n\
Content-Disposition: form-data; name=\"id\"\r\n\r\n\
guid-1234\r\n\
--UniExtractLog--"
            .to_vec();
        assert_eq!(body, expected);
    }

    /// The verified bug: only the literal "1" counts as success; any other
    /// body (including a real HTTP error page on a 200) reads as failure.
    #[test]
    fn submission_success_requires_exact_literal_one() {
        assert!(feedback_submission_succeeded("1"));
        assert!(!feedback_submission_succeeded("1 "));
        assert!(!feedback_submission_succeeded("0"));
        assert!(!feedback_submission_succeeded(""));
        assert!(!feedback_submission_succeeded("<html>error</html>"));
    }

    #[test]
    fn error_message_falls_back_when_no_com_error_was_recorded() {
        assert_eq!(
            resolve_feedback_error_message(None),
            "Invalid response from server"
        );
        assert_eq!(
            resolve_feedback_error_message(Some("0x80072EE7")),
            "0x80072EE7"
        );
    }
}
