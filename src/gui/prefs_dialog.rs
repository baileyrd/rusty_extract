//! Preferences dialog and instant-apply tray toggles — capability C190.
//! Ports the pure decisions inside `GUI_Prefs_OK` (UniExtract.au3:6482-
//! 6552) and the four standalone toggles `GUI_ScanOnly`
//! (UniExtract.au3:6309-6327), `GUI_Silent` (UniExtract.au3:6330-6340),
//! `GUI_KeepOpen` (UniExtract.au3:6343-6353), and `GUI_Topmost`
//! (UniExtract.au3:6356-6369). `GUI_KeepOutdir` (UniExtract.au3:6303-6306)
//! needs no function of its own here — it's a direct
//! read-checkbox-then-persist with no decision logic, and its persisted
//! value (`lock_output_directory`) already exists and is already
//! consumed by `gui::file_input` (C186).
//!
//! **The Preferences dialog window itself stays unwired** — same
//! treatment as C188's queue-edit dialog and C189's confirm dialog: it's
//! a ~20-control settings window with no real prefs-file read/write
//! pathway to back it yet (the same category of gap C183/C185's own
//! position/preference-persistence notes already document), so building
//! the window now would just be inert controls with nothing to load from
//! or save to. What's real and testable today is the *decision logic*
//! each control's handler runs once a value is in hand.
//!
//! `GUI_Prefs_OK`'s many direct `$bOptX = Number(_IsChecked($idOptX))`
//! assignments (free-space check, unicode check, append-extension,
//! create-log, extract-video, remember-window-position, open-output-dir)
//! are plain 1:1 checkbox mirrors with no decision logic of their own —
//! not ported as functions here for the same reason `GUI_KeepOutdir`
//! isn't; only the assignments below have a real *rule* worth a name.

use crate::prefs::DeleteSourceFileOption;

/// `WS_EX_TOPMOST` (0x00000008), the same ex-style bit `GUI_Topmost`
/// applies (UniExtract.au3:6362).
pub const WS_EX_TOPMOST: i32 = 0x0000_0008;

/// What toggling the "remember history" checkbox does
/// (UniExtract.au3:6485-6497): the notable behavior is `Disable`, which
/// doesn't just clear a flag but deletes both persisted history ini keys
/// outright (`IniDelete($prefs, $HISTORY_FILE)`,
/// `IniDelete($prefs, $HISTORY_DIR)`) — real data loss on uncheck, not a
/// simple flag flip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryToggleOutcome {
    /// Checkbox state already matches `current_history` — nothing to do.
    NoChange,
    /// Was off, now checked: `$history = 1`, redraw the main window.
    Enable,
    /// Was on, now unchecked: `$history = 0`, delete both ini keys,
    /// redraw the main window.
    Disable,
}

/// Ports the `$historyopt` branch of `GUI_Prefs_OK`
/// (UniExtract.au3:6485-6497).
pub fn decide_history_toggle_outcome(checked: bool, current_history: bool) -> HistoryToggleOutcome {
    match (checked, current_history) {
        (true, false) => HistoryToggleOutcome::Enable,
        (false, true) => HistoryToggleOutcome::Disable,
        _ => HistoryToggleOutcome::NoChange,
    }
}

/// Ports the update-interval combo's preset-to-display-index `Switch`
/// (UniExtract.au3:6450-6461): the five preset day counts map to their
/// own index; anything else (including a genuinely custom value the GUI
/// itself never offers a way to enter) falls back to index `5`, the
/// combo's own "Custom" entry.
pub fn resolve_update_interval_display_index(stored_days: i64) -> usize {
    match stored_days {
        1 => 0,
        7 => 1,
        30 => 2,
        365 => 3,
        999_999 => 4,
        _ => 5,
    }
}

/// Ports the reverse mapping in `GUI_Prefs_OK`
/// (UniExtract.au3:6505-6507): `Local $aReturn = [1, 7, 30, 365, 999999,
/// $iOptUpdateInterval]` — selecting the "Custom" entry (index `5`)
/// doesn't let the user type a value, it just keeps whatever was already
/// stored. An out-of-range index (never produced by the real 6-item
/// combo) also falls back to the previous value rather than panicking.
pub fn resolve_update_interval_value(selected_index: usize, previous_value: i64) -> i64 {
    const PRESETS: [i64; 5] = [1, 7, 30, 365, 999_999];
    PRESETS
        .get(selected_index)
        .copied()
        .unwrap_or(previous_value)
}

/// What changing the "send anonymous statistics" checkbox does
/// (UniExtract.au3:6527-6532): the network call only fires on an actual
/// change, and only one direction fires per change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendStatsCommand {
    Enable,
    Disable,
}

/// Ports the `$bOptSendStats <> $tmp` change-gate
/// (UniExtract.au3:6527-6532): `None` when the checkbox didn't actually
/// change, so no `SendStats(...)` call is made at all.
pub fn decide_send_stats_command(
    previous_enabled: bool,
    new_enabled: bool,
) -> Option<SendStatsCommand> {
    if previous_enabled == new_enabled {
        None
    } else if new_enabled {
        Some(SendStatsCommand::Enable)
    } else {
        Some(SendStatsCommand::Disable)
    }
}

/// Ports the nightly-updates change-gate (UniExtract.au3:6534-6537):
/// `CheckUpdate()` only fires when the beta-updates checkbox's value
/// actually changed, in either direction.
pub fn should_check_update_after_nightly_toggle(previous_nightly: bool, new_nightly: bool) -> bool {
    previous_nightly != new_nightly
}

/// Ports the delete-source-file radio group's read loop
/// (UniExtract.au3:6539-6541): iterates `$OPTION_KEEP`(0)/
/// `$OPTION_DELETE`(1)/`$OPTION_ASK`(2) in that numeric order, and
/// whichever is checked wins — the *last* one, if more than one somehow
/// is (real radio buttons enforce mutual exclusion, so in practice
/// exactly one is checked). `None` if none are checked at all, a state
/// the loop itself doesn't have a case for (it just leaves the previous
/// value untouched) — modeled explicitly here rather than silently
/// keeping stale state hidden inside the function.
pub fn resolve_checked_delete_source_file_radio(
    checked: [bool; 3],
) -> Option<DeleteSourceFileOption> {
    let mut result = None;
    for (i, &is_checked) in checked.iter().enumerate() {
        if is_checked {
            result = crate::prefs::parse_delete_source_file_option(Some(i as i64)).into();
        }
    }
    result
}

/// Ports `GUI_ScanOnly`'s save-skip parameter
/// (UniExtract.au3:6309,6326): `If @NumParams < 1 Or $bSave Then
/// SavePref(...)`. `CreateGUI()` calls `GUI_ScanOnly(False)`
/// (UniExtract.au3:5963) once at startup purely to apply the persisted
/// `extract` preference to the output-dir controls' enabled state,
/// without re-saving the same value it just loaded; every real user
/// click uses the default (`explicit_save_arg: None`), which always
/// saves.
pub fn should_persist_scan_only_pref(explicit_save_arg: Option<bool>) -> bool {
    explicit_save_arg.unwrap_or(true)
}

/// Ports `GUI_Topmost`'s ex-style resolution (UniExtract.au3:6357-6363).
/// The source applies this by destroying and recreating the entire main
/// window (`GUIDelete($guimain)` + `CreateGUI()`, UniExtract.au3:6365-
/// 6366) because `WS_EX_TOPMOST` can't be changed on a live Win32 window
/// handle. `egui` has no such limitation — `ViewportCommand::WindowLevel`
/// changes it on the live viewport directly, superseding the
/// recreate-the-whole-window workaround entirely, the same class of "old
/// workaround made moot by the new toolkit" as C183's DPI-scaling note,
/// C185's tooltip-workaround note, C187's `WM_DROPFILES_UNICODE_FUNC`
/// note, and C189's `GUIOnEventMode` note.
pub fn resolve_topmost_ex_style(checked: bool) -> i32 {
    if checked {
        WS_EX_TOPMOST
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_toggle_no_change_when_already_matching() {
        assert_eq!(
            decide_history_toggle_outcome(true, true),
            HistoryToggleOutcome::NoChange
        );
        assert_eq!(
            decide_history_toggle_outcome(false, false),
            HistoryToggleOutcome::NoChange
        );
    }

    #[test]
    fn history_toggle_enable_and_disable() {
        assert_eq!(
            decide_history_toggle_outcome(true, false),
            HistoryToggleOutcome::Enable
        );
        assert_eq!(
            decide_history_toggle_outcome(false, true),
            HistoryToggleOutcome::Disable
        );
    }

    #[test]
    fn update_interval_display_index_matches_presets() {
        assert_eq!(resolve_update_interval_display_index(1), 0);
        assert_eq!(resolve_update_interval_display_index(7), 1);
        assert_eq!(resolve_update_interval_display_index(30), 2);
        assert_eq!(resolve_update_interval_display_index(365), 3);
        assert_eq!(resolve_update_interval_display_index(999_999), 4);
    }

    /// Parity test: any non-preset stored value falls back to the
    /// "Custom" display slot rather than matching nothing.
    #[test]
    fn update_interval_display_index_falls_back_to_custom() {
        assert_eq!(resolve_update_interval_display_index(3), 5);
        assert_eq!(resolve_update_interval_display_index(0), 5);
        assert_eq!(resolve_update_interval_display_index(-1), 5);
    }

    #[test]
    fn update_interval_value_reads_presets_by_index() {
        assert_eq!(resolve_update_interval_value(0, 999), 1);
        assert_eq!(resolve_update_interval_value(4, 999), 999_999);
    }

    /// Parity test: selecting "Custom" (index 5) keeps the previous
    /// value rather than resolving to a preset.
    #[test]
    fn update_interval_value_custom_index_keeps_previous_value() {
        assert_eq!(resolve_update_interval_value(5, 42), 42);
    }

    #[test]
    fn send_stats_command_none_when_unchanged() {
        assert_eq!(decide_send_stats_command(true, true), None);
        assert_eq!(decide_send_stats_command(false, false), None);
    }

    #[test]
    fn send_stats_command_fires_on_change() {
        assert_eq!(
            decide_send_stats_command(false, true),
            Some(SendStatsCommand::Enable)
        );
        assert_eq!(
            decide_send_stats_command(true, false),
            Some(SendStatsCommand::Disable)
        );
    }

    #[test]
    fn nightly_toggle_check_fires_only_on_change() {
        assert!(!should_check_update_after_nightly_toggle(true, true));
        assert!(should_check_update_after_nightly_toggle(true, false));
        assert!(should_check_update_after_nightly_toggle(false, true));
    }

    #[test]
    fn delete_source_file_radio_resolves_checked_entry() {
        assert_eq!(
            resolve_checked_delete_source_file_radio([true, false, false]),
            Some(DeleteSourceFileOption::Keep)
        );
        assert_eq!(
            resolve_checked_delete_source_file_radio([false, true, false]),
            Some(DeleteSourceFileOption::Delete)
        );
        assert_eq!(
            resolve_checked_delete_source_file_radio([false, false, true]),
            Some(DeleteSourceFileOption::Ask)
        );
    }

    #[test]
    fn delete_source_file_radio_none_when_none_checked() {
        assert_eq!(
            resolve_checked_delete_source_file_radio([false, false, false]),
            None
        );
    }

    /// Parity test: if more than one is somehow checked, the last one in
    /// iteration order (Keep, Delete, Ask) wins.
    #[test]
    fn delete_source_file_radio_last_checked_wins() {
        assert_eq!(
            resolve_checked_delete_source_file_radio([true, false, true]),
            Some(DeleteSourceFileOption::Ask)
        );
    }

    #[test]
    fn scan_only_persists_by_default() {
        assert!(should_persist_scan_only_pref(None));
    }

    #[test]
    fn scan_only_persistence_follows_explicit_argument() {
        assert!(!should_persist_scan_only_pref(Some(false)));
        assert!(should_persist_scan_only_pref(Some(true)));
    }

    #[test]
    fn topmost_ex_style_matches_checkbox_state() {
        assert_eq!(resolve_topmost_ex_style(true), WS_EX_TOPMOST);
        assert_eq!(resolve_topmost_ex_style(false), 0);
    }
}
