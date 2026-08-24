//! FFmpeg update check: ports `_UpdateFFmpeg` (UniExtract.au3:5553-5577)
//! and `GetFFmpeg` (UniExtract.au3:5772-5779) — comparing the
//! locally-installed FFmpeg's reported version against a remote version
//! string, and the elevation-aware relaunch that applies a found update.
//!
//! This capability covers only the pure string/decision logic. The real
//! seams — `HasPlugin`'s existence check (reuse
//! [`crate::gui::missing_helper::plugin_exists`] directly; with
//! `HasPlugin($ffmpeg, True)`'s `$returnFail = True`, its dispatch always
//! degenerates to either "found" or "not found, return false", so no
//! separate wrapper is needed here), `FetchStdout`, `_INetGetSource`, the
//! `Prompt` confirmation (C193), and `ShellExecuteWait` — are real I/O the
//! caller performs, driven by the outcomes these functions return.

/// Ports `_StringBetween($return, "ffmpeg version ", " Copyright")`
/// (UniExtract.au3:5560-5561). **Verified defensive behavior, not a
/// bug**: when the marker pair isn't found in the captured stdout (e.g.
/// FFmpeg exists as a file but crashes instead of printing a version),
/// AutoIt's `@error` path defaults `$sVersion` to `0` — deliberately
/// forcing [`ffmpeg_update_available`] to treat *any* remote version as
/// newer, so a broken local FFmpeg always gets redownloaded rather than
/// silently staying broken.
pub fn extract_local_ffmpeg_version(stdout: &str) -> String {
    const START: &str = "ffmpeg version ";
    const END: &str = " Copyright";
    match stdout.find(START) {
        Some(start) => {
            let after = &stdout[start + START.len()..];
            match after.find(END) {
                Some(end) => after[..end].to_string(),
                None => "0".to_string(),
            }
        }
        None => "0".to_string(),
    }
}

/// Ports the remote-index suffix selection (UniExtract.au3:5563):
/// `_IsWinXP() ? "-xp" : $iOsArch == 32 ? "-32" : ""`. **Verified
/// assumption, preserved rather than "fixed"**: only these two special
/// variants are accounted for — any other OS/architecture combination
/// (e.g. ARM64) silently falls through to the default suffix.
pub fn resolve_ffmpeg_url_suffix(is_win_xp: bool, os_arch: u32) -> &'static str {
    if is_win_xp {
        "-xp"
    } else if os_arch == 32 {
        "-32"
    } else {
        ""
    }
}

/// Ports `$return > $sVersion` (UniExtract.au3:5571). **Verified bug,
/// preserved rather than "fixed"** (per this migration's default
/// convention: decide fix-vs-preserve explicitly at the call site, not
/// silently): this is a plain lexicographic string comparison, not a
/// numeric/semver one. `"9.0" > "10.0"` is `true` here (`'9' > '1'`
/// lexicographically), so a same-or-older numeric version can still
/// register as an "update", and multi-digit segments don't compare the
/// way a human reading version numbers would expect.
pub fn ffmpeg_update_available(remote_version: &str, local_version: &str) -> bool {
    remote_version > local_version
}

/// Which updater binary `GetFFmpeg` launches (UniExtract.au3:5774):
/// `CanAccess($bindir) ? $sUpdaterNoAdmin : $sUpdater`, run via
/// `ShellExecuteWait` (blocking until the elevated/non-elevated updater
/// process exits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfmpegUpdaterBinary {
    NoAdminRequired,
    RequiresAdmin,
}

pub fn resolve_ffmpeg_updater_binary(can_access_bindir: bool) -> FfmpegUpdaterBinary {
    if can_access_bindir {
        FfmpegUpdaterBinary::NoAdminRequired
    } else {
        FfmpegUpdaterBinary::RequiresAdmin
    }
}

/// Ports `GetFFmpeg`'s post-launch success check (UniExtract.au3:5775):
/// `If @error Or Not HasPlugin($ffmpeg, True) Then Return SetError(1, 0,
/// False)`. Success requires both that `ShellExecuteWait` itself didn't
/// error *and* that FFmpeg is now actually present — a launch that
/// "succeeds" without ever actually installing the plugin (e.g. the user
/// cancels the updater dialog) is still reported as a failure.
pub fn ffmpeg_download_succeeded(shell_execute_had_error: bool, plugin_now_exists: bool) -> bool {
    !shell_execute_had_error && plugin_now_exists
}

#[cfg(test)]
mod tests {
    use super::{
        extract_local_ffmpeg_version, ffmpeg_download_succeeded, ffmpeg_update_available,
        resolve_ffmpeg_updater_binary, resolve_ffmpeg_url_suffix, FfmpegUpdaterBinary,
    };

    #[test]
    fn extracts_version_between_markers() {
        let stdout = "ffmpeg version 6.1.1 Copyright (c) 2000-2023 the FFmpeg developers";
        assert_eq!(extract_local_ffmpeg_version(stdout), "6.1.1");
    }

    /// The verified defensive-default behavior: a missing marker pair
    /// forces version "0", guaranteeing any remote version looks newer.
    #[test]
    fn missing_markers_default_to_zero() {
        assert_eq!(extract_local_ffmpeg_version("garbage crash output"), "0");
        assert_eq!(
            extract_local_ffmpeg_version("ffmpeg version 6.1.1 but no copyright marker"),
            "0"
        );
    }

    #[test]
    fn url_suffix_prefers_winxp_over_arch() {
        assert_eq!(resolve_ffmpeg_url_suffix(true, 32), "-xp");
        assert_eq!(resolve_ffmpeg_url_suffix(false, 32), "-32");
        assert_eq!(resolve_ffmpeg_url_suffix(false, 64), "");
    }

    #[test]
    fn update_available_uses_plain_string_comparison() {
        assert!(ffmpeg_update_available("6.1.2", "6.1.1"));
        assert!(!ffmpeg_update_available("6.1.1", "6.1.1"));
    }

    /// The verified bug: lexicographic comparison misorders multi-digit
    /// version segments, unlike a numeric/semver comparison.
    #[test]
    fn lexicographic_comparison_misorders_multi_digit_segments() {
        assert!(ffmpeg_update_available("9.0", "10.0"));
    }

    #[test]
    fn updater_binary_avoids_admin_when_bindir_is_writable() {
        assert_eq!(
            resolve_ffmpeg_updater_binary(true),
            FfmpegUpdaterBinary::NoAdminRequired
        );
        assert_eq!(
            resolve_ffmpeg_updater_binary(false),
            FfmpegUpdaterBinary::RequiresAdmin
        );
    }

    #[test]
    fn download_succeeds_only_without_error_and_with_plugin_present() {
        assert!(ffmpeg_download_succeeded(false, true));
        assert!(!ffmpeg_download_succeeded(true, true));
        assert!(!ffmpeg_download_succeeded(false, false));
        assert!(!ffmpeg_download_succeeded(true, false));
    }
}
