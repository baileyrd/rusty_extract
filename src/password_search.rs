//! Archive password-protection probing and password-list trial
//! (`_FindArchivePassword`, UniExtract.au3:4847-4877): probes whether an
//! archive is encrypted using a caller-run "is it protected" check, then
//! tries each password from a list against a caller-run "does this
//! password work" check until one succeeds or the list is exhausted.
//! Used by 7z, DGCA, and RAR extraction (UniExtract.au3:2290,2501,3004) —
//! C160.
//!
//! **Scope — decision policy only, no process spawning.** The source
//! builds and runs two shell commands per archive type (an "is-protected"
//! probe and a `%PASSWORD%`-templated test command) via `FetchStdout`;
//! this module doesn't run anything (see `extract::runner` for that
//! port), it only decides, from already-captured command output, what
//! the search does next — [`probe_shows_protected`] and [`find_password`]
//! both take the probe/test output as plain `&str`, the same
//! dependency-injection split already used by
//! `extract::plugin::resolve_plugin_ini`/`resolve_plugin_ini_with`.
//! Reading the password-list file (with its `@ScriptDir\passwords.txt`
//! fallback on read failure) is likewise left to the caller.

/// Reproduces `_StringGetLine($sString, $iLine)` for `$iLine < 0`
/// (UniExtract.au3:4577-4583), generalizing the `$iLine = -1` case
/// already ported as `log_eval`'s private
/// `tail_for_password_prompt_search`: for `$iLine = -1` the source
/// searches for the *second*-to-last `@CRLF` and returns everything from
/// there on; more negative values search further back — `$iLine = -3`
/// (this module's default probe line, per `_FindArchivePassword`'s own
/// default parameter) searches for the *fourth*-to-last `@CRLF`. If the
/// string doesn't have that many `@CRLF`s, `StringInStr` returns 0 and
/// the source falls back to `StringTrimLeft($sString, 0)`, i.e. the
/// entire, unmodified string — preserved here exactly, not "fixed" into
/// a plain last-N-lines helper.
fn nth_line_from_end(s: &str, line: i64) -> &str {
    debug_assert!(line < 0, "only the negative-$iLine branch is ported");
    let k = (1 - line) as usize;
    match s.rmatch_indices("\r\n").nth(k - 1) {
        Some((pos, _)) => &s[pos..],
        None => s,
    }
}

/// Reproduces the "is archive encrypted" check at the top of
/// `_FindArchivePassword` (UniExtract.au3:4850-4851). `probe_output` is
/// the probe command's full captured output; `line` is the source's
/// `$iLine` parameter (default `-3`) selecting which part of it to
/// search — `0` searches the whole output unchanged (as both the C056
/// 7-Zip and C092 RAR call sites do, via `FetchStdout`'s own `$iLine <>
/// 0` gate), any negative value narrows it via [`nth_line_from_end`].
/// Only these two branches are ported: no `_FindArchivePassword` call
/// site in the source ever passes a positive `$iLine`.
///
/// A match on `protected_text` (default `"encrypted"`) alone is enough;
/// `protected_text2` is an optional second marker checked only when
/// present (the source's `$sIsProtectedText2 == 0` "not provided"
/// sentinel becomes `None` here). Matches case-insensitively —
/// `StringInStr`'s default, the same convention documented for
/// `log_eval::is_password_failure` (C162).
pub fn probe_shows_protected(
    probe_output: &str,
    line: i64,
    protected_text: &str,
    protected_text2: Option<&str>,
) -> bool {
    let searched = if line == 0 {
        probe_output
    } else {
        nth_line_from_end(probe_output, line)
    };
    let lower = searched.to_lowercase();
    lower.contains(&protected_text.to_lowercase())
        || protected_text2.is_some_and(|t| lower.contains(&t.to_lowercase()))
}

/// Reproduces the per-password success check inside
/// `_FindArchivePassword`'s trial loop (UniExtract.au3:4868):
/// `test_output` is one password's test-command output, checked for
/// `success_text` (default `"All OK"`). Matches case-insensitively,
/// `StringInStr`'s default.
pub fn test_output_shows_success(test_output: &str, success_text: &str) -> bool {
    test_output
        .to_lowercase()
        .contains(&success_text.to_lowercase())
}

/// Reproduces the password-list trial loop of `_FindArchivePassword`
/// (UniExtract.au3:4862-4873): tries each password in order, calling
/// `test_password` (standing in for the source's `%PASSWORD%`-templated
/// `FetchStdout` call) and stopping at the first one whose returned
/// output satisfies [`test_output_shows_success`]. Returns `None` —
/// matching the source's `$sPassword = 0` sentinel — if the list is
/// empty or every password fails, the same outcome the source produces
/// whether or not the archive actually turns out to be protected (that
/// check is [`probe_shows_protected`]'s job, applied by the caller
/// before reaching for this loop, exactly as `_FindArchivePassword`
/// itself gates the loop on it).
pub fn find_password<'a>(
    passwords: &'a [String],
    success_text: &str,
    mut test_password: impl FnMut(&str) -> String,
) -> Option<&'a str> {
    passwords
        .iter()
        .find(|p| test_output_shows_success(&test_password(p), success_text))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nth_line_from_end_minus_one_returns_from_second_to_last_crlf_onward() {
        assert_eq!(nth_line_from_end("a\r\nb\r\nc", -1), "\r\nb\r\nc");
    }

    #[test]
    fn nth_line_from_end_falls_back_to_whole_string_when_too_few_crlfs() {
        assert_eq!(
            nth_line_from_end("only one line, no CRLF", -1),
            "only one line, no CRLF"
        );
        assert_eq!(nth_line_from_end("a\r\nb", -3), "a\r\nb");
    }

    #[test]
    fn probe_shows_protected_with_line_zero_searches_whole_output() {
        let output = "Listing archive: x.7z\r\nEncrypted = +\r\nPath = x.7z";
        assert!(probe_shows_protected(
            output,
            0,
            "Encrypted = +",
            Some("Wrong password?")
        ));
    }

    #[test]
    fn probe_shows_protected_matches_second_marker() {
        let output = "some header\r\nWrong password? y/n";
        assert!(probe_shows_protected(
            output,
            0,
            "Encrypted = +",
            Some("Wrong password?")
        ));
    }

    #[test]
    fn probe_shows_protected_is_case_insensitive() {
        assert!(probe_shows_protected(
            "ARCHIVE IS ENCRYPTED",
            0,
            "encrypted",
            None
        ));
    }

    #[test]
    fn probe_shows_protected_false_when_neither_marker_present() {
        assert!(!probe_shows_protected(
            "plain listing, nothing special",
            0,
            "encrypted",
            Some("password?")
        ));
    }

    #[test]
    fn probe_shows_protected_narrows_to_nth_line_from_end_when_line_negative() {
        // The marker is on an earlier line than the one `line = -1` selects
        // (the text starting at the second-to-last CRLF), so it must NOT be
        // found — proving the narrowing actually applies.
        let output = "Archive encrypted.\r\nheader2\r\nlast line has nothing";
        assert!(!probe_shows_protected(output, -1, "encrypted", None));
    }

    #[test]
    fn test_output_shows_success_is_case_insensitive() {
        assert!(test_output_shows_success(
            "Everything went all ok here",
            "All OK"
        ));
    }

    #[test]
    fn find_password_returns_first_match() {
        let passwords = vec![
            "wrong1".to_string(),
            "right".to_string(),
            "wrong2".to_string(),
        ];
        let tried = std::cell::RefCell::new(Vec::new());
        let found = find_password(&passwords, "All OK", |p| {
            tried.borrow_mut().push(p.to_string());
            if p == "right" {
                "Everything is All OK".to_string()
            } else {
                "Wrong password".to_string()
            }
        });
        assert_eq!(found, Some("right"));
        assert_eq!(
            *tried.borrow(),
            vec!["wrong1".to_string(), "right".to_string()]
        );
    }

    #[test]
    fn find_password_returns_none_when_list_exhausted() {
        let passwords = vec!["a".to_string(), "b".to_string()];
        let found = find_password(&passwords, "All OK", |_| "Wrong password".to_string());
        assert_eq!(found, None);
    }

    #[test]
    fn find_password_returns_none_for_empty_list() {
        let passwords: Vec<String> = vec![];
        let found = find_password(&passwords, "All OK", |_| "Everything is Ok".to_string());
        assert_eq!(found, None);
    }
}
