//! Per-install GUID generation: ports the persistent pseudonymous
//! identifier created on first run (UniExtract.au3:157,335-341,776),
//! attached to both telemetry channels (`telemetry`/C214, `gui::feedback`/
//! C212) and shown in the About dialog (`gui::about`/C197).
//!
//! This capability covers the pure shaping logic — string trimming,
//! truncation, and the version-tag prefix. The two real entropy/API
//! sources (`_WinAPI_CreateGUID()`, and the fallback's
//! `Random()`-seeded `_Crypt_EncryptData(...)` call) are the caller's
//! job; this module only decides what to do with their outputs.

/// Ports `$sOptGuid = "" Or StringIsSpace($sOptGuid)`
/// (UniExtract.au3:335): a GUID is regenerated if the loaded value is
/// empty or entirely whitespace.
pub fn should_generate_new_guid(loaded_guid: &str) -> bool {
    loaded_guid.trim().is_empty()
}

/// Ports `StringTrimLeft(StringTrimRight(_WinAPI_CreateGUID(), 1), 1)`
/// (UniExtract.au3:336): unconditionally removes exactly one character
/// from each end, regardless of what that character actually is. This
/// assumes the real `_WinAPI_CreateGUID()` output is always braced
/// (`{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`), so in practice it strips
/// exactly the braces — but, faithfully, a caller passing anything else
/// still gets its first and last character removed regardless. Fewer
/// than two characters yields an empty string, matching AutoIt's
/// `StringTrimLeft`/`StringTrimRight` on an over-length trim count rather
/// than erroring.
pub fn strip_guid_braces(raw_guid: &str) -> String {
    let char_count = raw_guid.chars().count();
    if char_count < 2 {
        return String::new();
    }
    raw_guid.chars().skip(1).take(char_count - 2).collect()
}

/// Ports the fallback ID's truncation (UniExtract.au3:337):
/// `StringRight(String(_Crypt_EncryptData(...)), 25)` — the last 25
/// characters of the encrypted output's string representation.
///
/// **Verified quirk, flagged for an explicit fix-vs-preserve decision
/// rather than silently changed**: the two `Random(10000, 1000000)`
/// calls that seed this fallback (the AES key and the plaintext) are
/// AutoIt's non-cryptographic PRNG, each contributing well under 20 bits
/// of real entropy. Encrypting one weak-random value with another
/// weak-random key adds no entropy beyond what those two `Random()`
/// calls already have — the result is a meaningfully smaller and more
/// guessable ID space than a true GUID (~122 bits), not a cosmetic
/// difference. This function only ports the shape (truncate to the last
/// 25 characters); whether to keep this exact fallback or intentionally
/// strengthen it needs sign-off either way, per this row's manifest note.
pub fn fallback_guid_from_encrypted_output(encrypted_output: &str) -> String {
    let char_count = encrypted_output.chars().count();
    let skip = char_count.saturating_sub(25);
    encrypted_output.chars().skip(skip).collect()
}

/// Ports the primary-then-fallback dispatch (UniExtract.au3:336-337):
/// try `_WinAPI_CreateGUID()` first ([`strip_guid_braces`]'d), and only
/// fall back to the weaker generator if that result is empty.
/// `fallback` is lazy (`FnOnce`) so the caller doesn't need to compute
/// the fallback's `Random()`/`_Crypt_EncryptData` inputs unless the
/// primary path actually failed, matching the source's own short-circuit
/// `If` — it never calls `_Crypt_EncryptData` when the WinAPI GUID
/// succeeded.
pub fn resolve_guid_body(winapi_raw: Option<&str>, fallback: impl FnOnce() -> String) -> String {
    let stripped = winapi_raw.map(strip_guid_braces).unwrap_or_default();
    if stripped.is_empty() {
        fallback()
    } else {
        stripped
    }
}

/// Ports `$sVersionId` (UniExtract.au3:70): a hardcoded version-family
/// tag. **Verified quirk, preserved rather than "fixed"**: this prefix is
/// permanent — it's baked into the GUID once, on first generation, and
/// never updates on a later upgrade to a new version family. Preserve
/// this exact prefix format if the same telemetry endpoint/backend is
/// ever kept, since it's presumably used server-side to bucket IDs by
/// the version family that originally created them.
pub const VERSION_ID: &str = "2R4";

/// Ports `$sOptGuid = $sVersionId & "-" & $sOptGuid`
/// (UniExtract.au3:338): the final stored/sent value is always the
/// version tag, a literal hyphen, then the generated body.
pub fn build_guid_with_version_prefix(version_id: &str, guid_body: &str) -> String {
    format!("{version_id}-{guid_body}")
}

#[cfg(test)]
mod tests {
    use super::{
        build_guid_with_version_prefix, fallback_guid_from_encrypted_output, resolve_guid_body,
        should_generate_new_guid, strip_guid_braces, VERSION_ID,
    };

    #[test]
    fn regeneration_triggered_by_empty_or_whitespace_only_guid() {
        assert!(should_generate_new_guid(""));
        assert!(should_generate_new_guid("   "));
        assert!(should_generate_new_guid("\t\n"));
        assert!(!should_generate_new_guid("2R4-abc123"));
    }

    #[test]
    fn brace_strip_removes_exactly_one_character_from_each_end() {
        assert_eq!(
            strip_guid_braces("{12345678-1234-1234-1234-123456789012}"),
            "12345678-1234-1234-1234-123456789012"
        );
    }

    /// Faithful to the source: any single leading/trailing character is
    /// removed, not specifically braces.
    #[test]
    fn brace_strip_is_unconditional_not_brace_specific() {
        assert_eq!(strip_guid_braces("Xabc123Y"), "abc123");
        assert_eq!(strip_guid_braces("ab"), "");
        assert_eq!(strip_guid_braces("a"), "");
        assert_eq!(strip_guid_braces(""), "");
    }

    #[test]
    fn fallback_truncates_to_last_twenty_five_characters() {
        let encrypted = "0".repeat(30) + "ABCDEFGHIJKLMNOPQRSTUVWXY";
        let result = fallback_guid_from_encrypted_output(&encrypted);
        assert_eq!(result.chars().count(), 25);
        assert_eq!(result, "ABCDEFGHIJKLMNOPQRSTUVWXY");
    }

    #[test]
    fn fallback_returns_whole_string_if_shorter_than_twenty_five() {
        assert_eq!(fallback_guid_from_encrypted_output("short"), "short");
    }

    #[test]
    fn resolve_prefers_winapi_result_and_skips_the_fallback_entirely() {
        let mut fallback_called = false;
        let body = resolve_guid_body(Some("{deadbeef-0000-0000-0000-000000000000}"), || {
            fallback_called = true;
            "should-not-be-used".to_string()
        });
        assert_eq!(body, "deadbeef-0000-0000-0000-000000000000");
        assert!(!fallback_called);
    }

    #[test]
    fn resolve_falls_back_when_winapi_result_is_empty() {
        let body = resolve_guid_body(Some(""), || "fallback-value".to_string());
        assert_eq!(body, "fallback-value");

        let body_none = resolve_guid_body(None, || "fallback-value".to_string());
        assert_eq!(body_none, "fallback-value");
    }

    /// A brace-only string strips down to empty, which also triggers the
    /// fallback -- matching the source's `$sOptGuid = ""` re-check.
    #[test]
    fn resolve_falls_back_when_winapi_result_strips_to_empty() {
        let body = resolve_guid_body(Some("{}"), || "fallback-value".to_string());
        assert_eq!(body, "fallback-value");
    }

    #[test]
    fn version_prefix_is_hyphen_joined() {
        assert_eq!(
            build_guid_with_version_prefix(VERSION_ID, "abc123"),
            "2R4-abc123"
        );
    }
}
