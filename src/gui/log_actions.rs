//! Misc file/log utility actions — capability C195. Ports the pure
//! decisions inside `GUI_OpenLastLog` (UniExtract.au3:7944-7950),
//! `GUI_DeleteLogs` (UniExtract.au3:7958-7972), `GUI_UpdateLogItem`
//! (UniExtract.au3:7975-7980), and `GUI_Password`
//! (UniExtract.au3:8064-8071). `GUI_OpenLogDir`, `GUI_ProgDir`, and
//! `GUI_ConfigFile` (UniExtract.au3:7953-7955,8074-8081) are plain
//! `ShellExecute` one-liners with no decision logic of their own — real
//! I/O, not ported as functions here.
//!
//! **None of this is wired to real I/O in this pass.** `logdir` itself
//! (`$settingsdir & "\log\"`, UniExtract.au3:722) depends on settings-
//! directory resolution this port's GUI doesn't have plumbed into its
//! own state yet — the same category of gap as every preference this
//! migration phase has flagged as "no real prefs-file pathway exists" (C183,
//! C185, C188-C190). Wiring "Open log"/"Open log folder" for real needs
//! that path first; until then, only the decision logic below is real
//! and tested.

/// Ports `GUI_OpenLastLog`'s early-return gate
/// (UniExtract.au3:7946): `If @error Or $aFiles[0] < 1 Then Return`.
pub fn should_open_a_log(directory_read_succeeded: bool, file_count: usize) -> bool {
    directory_read_succeeded && file_count >= 1
}

/// Ports `GUI_OpenLastLog`'s "most recent" selection
/// (UniExtract.au3:7948-7949: `Local $iIndex = UBound($aFiles) - 1;
/// ShellExecute($aFiles[$iIndex])`). **Not an mtime sort** — this relies
/// entirely on `sorted_log_files` already being in whatever order
/// `_FileListToArray` itself returns them in (alphabetical filename
/// order), which only actually corresponds to chronological order
/// because this app's own log filenames happen to be date-prefixed
/// (`run_log`'s own `build_log_filename`, e.g. `2026-08-24_...`). A
/// literally most-recently-modified file with an out-of-band name would
/// not necessarily be picked. Preserved as-is per this row's own
/// manifest note, not silently upgraded to a real mtime sort.
pub fn resolve_most_recent_log(sorted_log_files: &[String]) -> Option<&String> {
    sorted_log_files.last()
}

/// Ports `GUI_UpdateLogItem`'s existence gate (UniExtract.au3:7976):
/// `If Not $guimain Then Return` — the log-directory-size menu label
/// only gets refreshed while the main window actually exists.
pub fn should_update_log_menu_item(main_window_exists: bool) -> bool {
    main_window_exists
}

/// Ports `GUI_UpdateLogItem`'s size formatting
/// (UniExtract.au3:7978: `Round(DirGetSize($logdir) / 1024 / 1024, 2)`)
/// — the log directory's total size in mebibytes, rounded to 2 decimal
/// places (half-away-from-zero, matching AutoIt's `Round`).
pub fn format_log_dir_size_mb(size_bytes: u64) -> f64 {
    let mb = size_bytes as f64 / 1024.0 / 1024.0;
    (mb * 100.0).round() / 100.0
}

/// Ports `GUI_Password`'s touch-if-missing gate
/// (UniExtract.au3:8065-8068): the password list file is created empty
/// before being opened, only if it doesn't already exist — an existing
/// file's contents are never touched.
pub fn should_create_password_file(file_exists: bool) -> bool {
    !file_exists
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_open_a_log_requires_successful_read_and_at_least_one_file() {
        assert!(should_open_a_log(true, 1));
        assert!(!should_open_a_log(false, 5));
        assert!(!should_open_a_log(true, 0));
    }

    #[test]
    fn resolve_most_recent_log_returns_last_entry() {
        let files = vec![
            "2026-08-01_120000_run.log".to_string(),
            "2026-08-24_090000_run.log".to_string(),
        ];
        assert_eq!(
            resolve_most_recent_log(&files),
            Some(&"2026-08-24_090000_run.log".to_string())
        );
    }

    #[test]
    fn resolve_most_recent_log_none_when_empty() {
        assert_eq!(resolve_most_recent_log(&[]), None);
    }

    #[test]
    fn log_menu_item_only_updates_with_main_window() {
        assert!(should_update_log_menu_item(true));
        assert!(!should_update_log_menu_item(false));
    }

    #[test]
    fn log_dir_size_rounds_to_two_decimals() {
        assert_eq!(format_log_dir_size_mb(0), 0.0);
        assert_eq!(format_log_dir_size_mb(1024 * 1024), 1.0);
        // 1 MiB + 100000 bytes = ~1.0954 MiB, rounds to 1.10, not
        // truncated to 1.09.
        assert_eq!(format_log_dir_size_mb(1024 * 1024 + 100_000), 1.10);
    }

    #[test]
    fn password_file_created_only_when_missing() {
        assert!(should_create_password_file(false));
        assert!(!should_create_password_file(true));
    }
}
