//! Main update orchestration: ports `CheckUpdate`'s end-to-end decision
//! tree (UniExtract.au3:5374-5445) — interval debounce, main-executable
//! comparison, GUI-prompt-gated relaunch, mode-driven helper/FFmpeg
//! delegation, and the final "up to date" outcome.
//!
//! This capability covers only the pure decision logic threaded through
//! `CheckUpdate`'s control flow. The real seams it drives — the index
//! fetch and local file comparison (`update_index`, C206), the GUI prompts
//! (`GUI_UpdatePrompt`/C199, `Prompt`/C193), `ShellExecute`, `Exit`, and
//! `SendStats` — are each either already-ported pure logic elsewhere or
//! deferred real I/O the caller performs at exactly the seams these
//! functions expose as plain booleans/enums.

/// Ports `$silent`'s three states (`$UPDATEMSG_PROMPT`/`$UPDATEMSG_SILENT`/
/// `$UPDATEMSG_FOUND_ONLY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMsgMode {
    Prompt,
    Silent,
    FoundOnly,
}

/// Ports `$iMode`'s three states (`$UPDATE_ALL`/`$UPDATE_HELPER`/`$UPDATE_MAIN`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    All,
    Helper,
    Main,
}

/// Ports UniExtract.au3:5375: `$bCheckInterval And _DateDiff("D", $lastupdate,
/// _NowCalc()) < $iOptUpdateInterval` — skip this run entirely if an interval
/// check was requested and fewer than the configured number of days have
/// passed since the last update check.
pub fn should_skip_due_to_interval(
    check_interval: bool,
    days_since_last_update: i64,
    update_interval_days: i64,
) -> bool {
    check_interval && days_since_last_update < update_interval_days
}

/// Ports UniExtract.au3:5386: the index fetch is silent (suppressing
/// network-error messages) when the caller asked for either fully silent or
/// "found only" reporting — only the interactive `Prompt` mode surfaces
/// fetch failures.
pub fn index_fetch_is_silent(mode: UpdateMsgMode) -> bool {
    matches!(mode, UpdateMsgMode::Silent | UpdateMsgMode::FoundOnly)
}

/// Ports UniExtract.au3:5392: `If StringLen($prefs) > 0 Then
/// SavePref('lastupdate', $lastupdate)`. **Verified quirk, preserved rather
/// than "fixed"**: if preferences haven't been loaded yet at call time (the
/// comment above the line notes `CheckUpdate` can run before that happens,
/// e.g. to recover from missing files), the freshly-computed `$lastupdate`
/// is silently never persisted for this run — not deferred, just dropped.
pub fn should_persist_last_update(prefs_loaded: bool) -> bool {
    prefs_loaded
}

/// Ports UniExtract.au3:5395-5396's main-executable comparison, gated by
/// mode. **Verified quirk, preserved rather than "fixed"**: unlike the
/// helper-file comparison (`_UpdateFileCompare`/`decide_file_needs_update`,
/// C206), which only computes a hash once sizes already match, this check
/// always evaluates *both* the size and hash comparison (`Or`), with no
/// early-exit optimization skipping the hash when sizes already differ.
pub fn main_executable_needs_update(
    mode: UpdateMode,
    local_size: i64,
    index_size: i64,
    local_md5: &str,
    index_md5: &str,
) -> bool {
    mode != UpdateMode::Helper && (local_size != index_size || local_md5 != index_md5)
}

/// The relaunch parameters `CheckUpdate` builds when the user accepts the
/// main-executable update prompt (UniExtract.au3:5400-5406): which updater
/// binary to launch (the no-admin-required one, when the script directory
/// is writable) and whether to pass `/nightly`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainUpdateLaunch {
    pub use_no_admin_updater: bool,
    pub parameters: String,
}

pub fn resolve_main_update_launch(
    can_access_script_dir: bool,
    nightly_updates: bool,
) -> MainUpdateLaunch {
    let mut parameters = "/main".to_string();
    if nightly_updates {
        parameters.push_str(" /nightly");
    }
    MainUpdateLaunch {
        use_no_admin_updater: can_access_script_dir,
        parameters,
    }
}

/// Ports UniExtract.au3:5410: declining the main-executable update prompt
/// overrides `$iMode` to `$UPDATE_MAIN`. **Verified quirk, preserved rather
/// than "fixed"**: because the later helper-files/FFmpeg section is gated
/// on `$iMode <> $UPDATE_MAIN` ([`should_check_helpers_and_ffmpeg`],
/// UniExtract.au3:5417), this override doesn't just suppress further
/// main-update prompts for the run — it also silently skips the
/// helper-files and FFmpeg update checks entirely. Declining the main
/// update is not independent of the other two checks, despite reading like
/// it should only affect the main-executable path.
pub fn mode_after_main_update_declined() -> UpdateMode {
    UpdateMode::Main
}

/// Ports UniExtract.au3:5417's gate for the helper-files/FFmpeg section.
pub fn should_check_helpers_and_ffmpeg(mode: UpdateMode) -> bool {
    mode != UpdateMode::Main
}

/// Ports UniExtract.au3:5425: whether to proceed with applying a found
/// helper-file update, once `CheckUpdateHelpers` (C206) has reported one
/// exists — either the run is fully silent, or the user accepted the
/// confirmation prompt.
pub fn should_apply_helper_update(mode: UpdateMsgMode, prompt_accepted: bool) -> bool {
    mode == UpdateMsgMode::Silent || prompt_accepted
}

/// Ports UniExtract.au3:5426-5430's branch once a helper update is being
/// applied: without write access to the bin directory, the app must
/// relaunch itself elevated (`/helper`) and exit, rather than writing files
/// in place from the current, unprivileged process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperUpdatePath {
    ApplyDirectly,
    RelaunchElevatedAndExit,
}

pub fn resolve_helper_update_path(can_access_bindir: bool) -> HelperUpdatePath {
    if can_access_bindir {
        HelperUpdatePath::ApplyDirectly
    } else {
        HelperUpdatePath::RelaunchElevatedAndExit
    }
}

/// Ports UniExtract.au3:5438-5440: the "up to date" message is shown only
/// when nothing was found across all three checks (main executable, helper
/// files, FFmpeg) *and* the caller asked for interactive prompting — silent
/// and found-only runs never show it either way.
pub fn should_show_up_to_date_message(found: bool, mode: UpdateMsgMode) -> bool {
    !found && mode == UpdateMsgMode::Prompt
}

#[cfg(test)]
mod tests {
    use super::{
        index_fetch_is_silent, main_executable_needs_update, mode_after_main_update_declined,
        resolve_helper_update_path, resolve_main_update_launch, should_apply_helper_update,
        should_check_helpers_and_ffmpeg, should_persist_last_update,
        should_show_up_to_date_message, should_skip_due_to_interval, HelperUpdatePath,
        MainUpdateLaunch, UpdateMode, UpdateMsgMode,
    };

    #[test]
    fn interval_skip_requires_both_flag_and_recent_check() {
        assert!(should_skip_due_to_interval(true, 0, 1));
        assert!(!should_skip_due_to_interval(true, 1, 1));
        assert!(!should_skip_due_to_interval(false, 0, 1));
    }

    #[test]
    fn index_fetch_is_silent_for_silent_and_found_only_modes() {
        assert!(index_fetch_is_silent(UpdateMsgMode::Silent));
        assert!(index_fetch_is_silent(UpdateMsgMode::FoundOnly));
        assert!(!index_fetch_is_silent(UpdateMsgMode::Prompt));
    }

    /// The verified quirk: an unloaded prefs store silently drops the
    /// persistence entirely, rather than deferring it.
    #[test]
    fn last_update_only_persists_once_prefs_are_loaded() {
        assert!(should_persist_last_update(true));
        assert!(!should_persist_last_update(false));
    }

    #[test]
    fn main_executable_update_skipped_entirely_in_helper_only_mode() {
        assert!(!main_executable_needs_update(
            UpdateMode::Helper,
            100,
            200,
            "aaa",
            "bbb"
        ));
    }

    #[test]
    fn main_executable_update_needed_on_size_or_hash_mismatch() {
        assert!(main_executable_needs_update(
            UpdateMode::All,
            100,
            200,
            "aaa",
            "aaa"
        ));
        assert!(main_executable_needs_update(
            UpdateMode::All,
            100,
            100,
            "aaa",
            "bbb"
        ));
        assert!(!main_executable_needs_update(
            UpdateMode::All,
            100,
            100,
            "aaa",
            "aaa"
        ));
    }

    #[test]
    fn main_update_launch_picks_updater_and_appends_nightly_flag() {
        assert_eq!(
            resolve_main_update_launch(true, false),
            MainUpdateLaunch {
                use_no_admin_updater: true,
                parameters: "/main".to_string(),
            }
        );
        assert_eq!(
            resolve_main_update_launch(false, true),
            MainUpdateLaunch {
                use_no_admin_updater: false,
                parameters: "/main /nightly".to_string(),
            }
        );
    }

    /// The verified quirk: declining the main update doesn't just suppress
    /// further main-update prompts — it silently skips the helper/FFmpeg
    /// checks too, once the two functions are composed the way `CheckUpdate`
    /// composes them.
    #[test]
    fn declining_main_update_also_suppresses_helper_and_ffmpeg_checks() {
        let mode = mode_after_main_update_declined();
        assert_eq!(mode, UpdateMode::Main);
        assert!(!should_check_helpers_and_ffmpeg(mode));
    }

    #[test]
    fn helper_and_ffmpeg_checks_run_in_all_and_helper_modes() {
        assert!(should_check_helpers_and_ffmpeg(UpdateMode::All));
        assert!(should_check_helpers_and_ffmpeg(UpdateMode::Helper));
        assert!(!should_check_helpers_and_ffmpeg(UpdateMode::Main));
    }

    #[test]
    fn helper_update_applies_silently_or_on_accepted_prompt() {
        assert!(should_apply_helper_update(UpdateMsgMode::Silent, false));
        assert!(should_apply_helper_update(UpdateMsgMode::Prompt, true));
        assert!(!should_apply_helper_update(UpdateMsgMode::Prompt, false));
    }

    #[test]
    fn helper_update_path_relaunches_elevated_without_bindir_access() {
        assert_eq!(
            resolve_helper_update_path(true),
            HelperUpdatePath::ApplyDirectly
        );
        assert_eq!(
            resolve_helper_update_path(false),
            HelperUpdatePath::RelaunchElevatedAndExit
        );
    }

    #[test]
    fn up_to_date_message_only_shown_when_nothing_found_and_prompting() {
        assert!(should_show_up_to_date_message(false, UpdateMsgMode::Prompt));
        assert!(!should_show_up_to_date_message(true, UpdateMsgMode::Prompt));
        assert!(!should_show_up_to_date_message(
            false,
            UpdateMsgMode::Silent
        ));
        assert!(!should_show_up_to_date_message(
            false,
            UpdateMsgMode::FoundOnly
        ));
    }
}
