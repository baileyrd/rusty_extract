//! Missing-plugin and missing-FFmpeg first-run dialogs — capability
//! C200. Ports the pure decisions inside `HasPlugin`
//! (UniExtract.au3:3735-3771) and `HasFFMPEG` (UniExtract.au3:3811-3884).
//!
//! **Neither dialog is wired to a real window** — same treatment as
//! every other dialog this migration phase has ported so far. The
//! network side effects both functions can trigger (`SendStats`,
//! `GetFFmpeg`'s actual download) are separate, deferred capabilities
//! (C214/D012, D003) — this module only covers the decisions around
//! them, not the network calls themselves.

/// Ports `HasPlugin`'s three-location existence check
/// (UniExtract.au3:3738): the plugin path itself, or — only if it's a
/// relative path — resolved against either `$bindir` or `$archdir`.
pub fn plugin_exists(
    is_relative_path: bool,
    absolute_exists: bool,
    bindir_relative_exists: bool,
    archdir_relative_exists: bool,
) -> bool {
    absolute_exists || (is_relative_path && (bindir_relative_exists || archdir_relative_exists))
}

/// What `HasPlugin` does once it knows whether the plugin exists
/// (UniExtract.au3:3738-3744).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasPluginOutcome {
    /// The plugin exists — nothing else to do.
    Found,
    /// Missing, but `$returnFail` -- `Return False` with no dialog and
    /// no termination, used by `HasFFMPEG`'s own probe-only call.
    NotFoundReturnFalse,
    /// Missing, silent mode -- `terminate($STATUS_MISSINGEXE, ...)`
    /// immediately, no dialog shown.
    NotFoundTerminateSilently,
    /// Missing, interactive -- show the "select/download plugin"
    /// dialog.
    NotFoundShowDialog,
}

/// Ports `HasPlugin`'s dispatch (UniExtract.au3:3738-3744).
pub fn decide_has_plugin_outcome(
    exists: bool,
    return_fail: bool,
    silent_mode: bool,
) -> HasPluginOutcome {
    if exists {
        HasPluginOutcome::Found
    } else if return_fail {
        HasPluginOutcome::NotFoundReturnFalse
    } else if silent_mode {
        HasPluginOutcome::NotFoundTerminateSilently
    } else {
        HasPluginOutcome::NotFoundShowDialog
    }
}

/// What clicking Download does on the FFmpeg-needed dialog
/// (UniExtract.au3:3840-3846): the license checkbox gates the actual
/// download attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadClickAction {
    Download,
    ShowLicenseNotAcceptedWarning,
}

/// Ports the Download button's checkbox gate (UniExtract.au3:3840-3846).
pub fn decide_download_click_action(license_accepted: bool) -> DownloadClickAction {
    if license_accepted {
        DownloadClickAction::Download
    } else {
        DownloadClickAction::ShowLicenseNotAcceptedWarning
    }
}

/// Ports the "Select file" dialog's early-out gate
/// (UniExtract.au3:3852): a cancelled dialog or a path that doesn't
/// exist skips validation entirely (`ContinueLoop`).
pub fn should_validate_selected_ffmpeg(dialog_cancelled: bool, path_exists: bool) -> bool {
    !dialog_cancelled && path_exists
}

/// Ports the FFmpeg-binary validation heuristic (UniExtract.au3:3863):
/// **fragile by construction, preserved exactly as this row's own
/// manifest note calls for, not "fixed."** A file under 1MB, or whose
/// captured stdout doesn't contain the literal substring `"ffmpeg
/// version"`, is rejected — a legitimate but unusually small or
/// differently-labeled FFmpeg build would be rejected by this same
/// heuristic in the real source, not just this port. `StringInStr` has
/// no case-sensitivity argument here either, so — like every other
/// bare `StringInStr` this port has encountered — the substring check
/// is case-insensitive.
pub fn is_valid_ffmpeg_binary(file_size_bytes: u64, captured_stdout: &str) -> bool {
    file_size_bytes >= 1024 * 1024 && captured_stdout.to_lowercase().contains("ffmpeg version")
}

/// What installing a validated FFmpeg binary resolves to
/// (UniExtract.au3:3877-3883). **The hardlink-over-copy preference is a
/// documented, deliberate architectural choice, not an oversight**: a
/// hardlink survives deleting the user's originally-selected file
/// (unlike a plain reference) and works on Windows XP (unlike a
/// symlink) — preserved here as an explicit choice, not just replicated
/// control flow. `copy_succeeded` is only meaningful when
/// `hardlink_succeeded` is `false` — the source only even attempts the
/// copy fallback once the hardlink attempt has already failed, the same
/// short-circuit-parameter contract as `warn_execute`'s own
/// `warn_execute_enabled` (C189).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfmpegInstallOutcome {
    HardlinkSucceeded,
    CopySucceeded,
    BothFailed,
}

/// Ports the hardlink-then-copy-fallback dispatch
/// (UniExtract.au3:3877-3883).
pub fn resolve_ffmpeg_install_outcome(
    hardlink_succeeded: bool,
    copy_succeeded: bool,
) -> FfmpegInstallOutcome {
    if hardlink_succeeded {
        FfmpegInstallOutcome::HardlinkSucceeded
    } else if copy_succeeded {
        FfmpegInstallOutcome::CopySucceeded
    } else {
        FfmpegInstallOutcome::BothFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_exists_absolute_path_found_directly() {
        assert!(plugin_exists(false, true, false, false));
    }

    #[test]
    fn plugin_exists_relative_path_checks_both_dirs() {
        assert!(plugin_exists(true, false, true, false));
        assert!(plugin_exists(true, false, false, true));
    }

    /// Parity test: an absolute path that doesn't exist is never
    /// resolved against bindir/archdir at all.
    #[test]
    fn plugin_exists_absolute_path_missing_ignores_relative_dirs() {
        assert!(!plugin_exists(false, false, true, true));
    }

    #[test]
    fn plugin_exists_relative_path_missing_from_both_dirs_is_false() {
        assert!(!plugin_exists(true, false, false, false));
    }

    #[test]
    fn has_plugin_outcome_found_wins_regardless_of_other_flags() {
        assert_eq!(
            decide_has_plugin_outcome(true, true, true),
            HasPluginOutcome::Found
        );
    }

    #[test]
    fn has_plugin_outcome_return_fail_short_circuits_before_silent_check() {
        assert_eq!(
            decide_has_plugin_outcome(false, true, false),
            HasPluginOutcome::NotFoundReturnFalse
        );
    }

    #[test]
    fn has_plugin_outcome_silent_mode_terminates() {
        assert_eq!(
            decide_has_plugin_outcome(false, false, true),
            HasPluginOutcome::NotFoundTerminateSilently
        );
    }

    #[test]
    fn has_plugin_outcome_interactive_shows_dialog() {
        assert_eq!(
            decide_has_plugin_outcome(false, false, false),
            HasPluginOutcome::NotFoundShowDialog
        );
    }

    #[test]
    fn download_click_requires_license_acceptance() {
        assert_eq!(
            decide_download_click_action(true),
            DownloadClickAction::Download
        );
        assert_eq!(
            decide_download_click_action(false),
            DownloadClickAction::ShowLicenseNotAcceptedWarning
        );
    }

    #[test]
    fn validation_skipped_on_cancel_or_missing_path() {
        assert!(!should_validate_selected_ffmpeg(true, true));
        assert!(!should_validate_selected_ffmpeg(false, false));
        assert!(should_validate_selected_ffmpeg(false, true));
    }

    #[test]
    fn ffmpeg_binary_valid_when_large_enough_and_labeled() {
        assert!(is_valid_ffmpeg_binary(
            1024 * 1024,
            "ffmpeg version 6.0 Copyright..."
        ));
    }

    #[test]
    fn ffmpeg_binary_invalid_when_too_small() {
        assert!(!is_valid_ffmpeg_binary(
            1024 * 1024 - 1,
            "ffmpeg version 6.0"
        ));
    }

    #[test]
    fn ffmpeg_binary_invalid_when_missing_version_string() {
        assert!(!is_valid_ffmpeg_binary(
            2 * 1024 * 1024,
            "not ffmpeg at all"
        ));
    }

    #[test]
    fn ffmpeg_binary_version_check_is_case_insensitive() {
        assert!(is_valid_ffmpeg_binary(
            2 * 1024 * 1024,
            "FFmpeg Version 6.0"
        ));
    }

    #[test]
    fn ffmpeg_install_prefers_hardlink() {
        assert_eq!(
            resolve_ffmpeg_install_outcome(true, true),
            FfmpegInstallOutcome::HardlinkSucceeded
        );
    }

    #[test]
    fn ffmpeg_install_falls_back_to_copy() {
        assert_eq!(
            resolve_ffmpeg_install_outcome(false, true),
            FfmpegInstallOutcome::CopySucceeded
        );
    }

    #[test]
    fn ffmpeg_install_both_failed() {
        assert_eq!(
            resolve_ffmpeg_install_outcome(false, false),
            FfmpegInstallOutcome::BothFailed
        );
    }
}
