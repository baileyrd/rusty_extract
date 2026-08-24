//! Batch queue GUI management — capability C188. Ports the GUI-layer
//! decisions of `GUI_Batch` (UniExtract.au3:6585-6594), `GUI_Batch_AddDirectory`
//! (UniExtract.au3:6610-6633), and `GUI_Batch_Show` (UniExtract.au3:6636-6701).
//! `GUI_Batch_Clear` (UniExtract.au3:6704-6707) is a one-line unconditional
//! `EnableBatchMode(False)` call with no decision logic of its own, so it
//! isn't given a dedicated function here.
//!
//! The add-vs-skip duplicate/multipart decision inside `AddToBatch`, the
//! re-invocable command-line format, and the FIFO queue-pop mechanics are
//! already ported (capability C147/C148, `crate::batch`) — this module
//! reuses those rather than re-deriving them. What's new here is the
//! GUI-specific layer on top: the overloaded Batch button's three-way
//! dispatch, the directory-recursion preference clamp, and the queue-edit
//! dialog's list/delete/save-or-cancel decisions.
//!
//! **Real batch *execution* stays out of scope** — `GUI_Batch_OK`'s
//! `terminate($STATUS_BATCH)` call and the subsequent
//! `BatchQueuePop`/self-relaunch chain (`crate::batch_runner`, C148) need
//! the GUI to be able to build a real extractor invocation for a queued
//! entry, which requires the detection cascade (C037-046) this port's GUI
//! doesn't wire up yet. The "Run" branch of the Batch button is decided
//! correctly by [`decide_batch_button_action`] but not yet acted on for
//! real, the same category of gap C187 left for [`crate::gui::drag_drop`]'s
//! `AddDirectory`/`PopulateAndQueue` variants — which this capability's
//! real wiring (in `gui::app`) finally acts on, since the in-memory queue
//! this module manages is exactly what those variants had nowhere to go
//! before.

/// What the overloaded Batch button does, ported from `GUI_Batch`'s own
/// top-level dispatch (UniExtract.au3:6586-6593): the current fields
/// validating (`GUI_OK_Set()`) always wins and adds to the queue; only
/// when they don't, and the queue already has items, does the button run
/// the queue instead; with neither, it's an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchButtonAction {
    /// `AddToBatch()` plus clearing the input fields.
    AddToQueue,
    /// `GUI_Batch_OK()` — finalize and execute the queue.
    RunQueue,
    /// `MsgBox($iTopmost + 48, $title, t('INVALID_FILE', $file))`.
    ShowInvalidFileError,
}

/// Ports `GUI_Batch`'s dispatch (UniExtract.au3:6586-6593).
pub fn decide_batch_button_action(fields_valid: bool, queue_has_items: bool) -> BatchButtonAction {
    if fields_valid {
        BatchButtonAction::AddToQueue
    } else if queue_has_items {
        BatchButtonAction::RunQueue
    } else {
        BatchButtonAction::ShowInvalidFileError
    }
}

/// Ports the field-clearing half of `GUI_Batch`'s add branch
/// (UniExtract.au3:6588-6589): the file field is always cleared, but the
/// output-directory field only when it isn't locked — this function is
/// just that second, conditional half.
pub fn should_clear_output_dir_on_batch_add(lock_output_directory: bool) -> bool {
    !lock_output_directory
}

/// Ports `GUI_Batch_AddDirectory`'s recursion-preference resolution
/// (UniExtract.au3:6611-6612): `IniRead`'s value is clamped so anything
/// above `1` collapses to `1`, then used as a boolean recurse flag. This
/// doesn't floor negative values (the source doesn't either) — in
/// practice the preference is only ever `0` or `1`.
pub fn resolve_batch_recurse(pref_value: i64) -> bool {
    pref_value.min(1) != 0
}

/// Ports `GUI_Batch_AddDirectory`'s file enumeration
/// (`_FileListToArrayRec($sDir, "*", $FLTAR_FILES, $bRecurse,
/// $FLTAR_NOSORT, $FLTAR_FULLPATH)`, UniExtract.au3:6614): every regular
/// file under `dir`, in filesystem enumeration order (matching the
/// source's own `$FLTAR_NOSORT` — no explicit sort is applied), recursing
/// into subdirectories only when `recurse` is true. An unreadable `dir`
/// returns an empty list, matching the source's own `@error` branch
/// (UniExtract.au3:6615-6618) rather than panicking.
pub fn list_directory_files(dir: &std::path::Path, recurse: bool) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recurse {
                files.extend(list_directory_files(&path, recurse));
            }
        } else {
            files.push(path);
        }
    }
    files
}

/// Ports `GUI_Batch_Show`'s OK-button gate (UniExtract.au3:6659-6662):
/// accepting the edited queue disables batch mode outright when it's been
/// emptied out entirely, rather than saving an empty queue.
pub fn should_disable_batch_mode_on_ok(queue_len: usize) -> bool {
    queue_len < 1
}

/// Ports `GUI_Batch_Show`'s Delete-button handler
/// (UniExtract.au3:6668-6672): `_ArrayDelete($queueArray, $iPos)` only
/// when a row is actually selected (`$iPos >= 0`) and in range — this
/// mirrors the source's own `> -1` success check by returning whether the
/// removal happened, so the caller only refreshes the displayed list on
/// an actual change. Cancel/close (UniExtract.au3:6654-6656) never calls
/// this at all — it just re-reads the last-persisted queue, discarding
/// any in-memory deletes; that asymmetry lives in the caller, not here.
pub fn delete_queue_item(queue: &mut Vec<String>, index: i64) -> bool {
    if index < 0 {
        return false;
    }
    let index = index as usize;
    if index >= queue.len() {
        return false;
    }
    queue.remove(index);
    true
}

/// Ports `GUI_Batch_Show`'s hover-tooltip gate (UniExtract.au3:6682):
/// a queue entry's full text is only worth a tooltip once it's long
/// enough that the listbox itself would be truncating it.
pub fn should_show_full_text_tooltip(text_len: usize) -> bool {
    text_len > 72
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test: valid fields always add, regardless of queue state.
    #[test]
    fn valid_fields_always_add_to_queue() {
        assert_eq!(
            decide_batch_button_action(true, true),
            BatchButtonAction::AddToQueue
        );
        assert_eq!(
            decide_batch_button_action(true, false),
            BatchButtonAction::AddToQueue
        );
    }

    /// Parity test: invalid fields with a non-empty queue run it instead.
    #[test]
    fn invalid_fields_with_queue_run_it() {
        assert_eq!(
            decide_batch_button_action(false, true),
            BatchButtonAction::RunQueue
        );
    }

    /// Parity test: invalid fields with an empty queue is an error.
    #[test]
    fn invalid_fields_with_empty_queue_is_error() {
        assert_eq!(
            decide_batch_button_action(false, false),
            BatchButtonAction::ShowInvalidFileError
        );
    }

    #[test]
    fn output_dir_cleared_only_when_unlocked() {
        assert!(should_clear_output_dir_on_batch_add(false));
        assert!(!should_clear_output_dir_on_batch_add(true));
    }

    /// Parity test: `0`/`1` pass through unchanged; anything above `1`
    /// clamps down to `1` (still truthy).
    #[test]
    fn resolve_batch_recurse_clamps_above_one() {
        assert!(!resolve_batch_recurse(0));
        assert!(resolve_batch_recurse(1));
        assert!(resolve_batch_recurse(2));
        assert!(resolve_batch_recurse(999));
    }

    #[test]
    fn list_directory_files_respects_recursion_flag() {
        let dir = std::env::temp_dir().join(format!(
            "rusty_extract_batch_queue_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.zip"), b"").unwrap();
        std::fs::write(dir.join("sub").join("b.rar"), b"").unwrap();

        let non_recursive = list_directory_files(&dir, false);
        assert_eq!(non_recursive.len(), 1);
        assert!(non_recursive[0].ends_with("a.zip"));

        let recursive = list_directory_files(&dir, true);
        assert_eq!(recursive.len(), 2);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_directory_files_on_unreadable_dir_returns_empty() {
        let missing = std::path::Path::new("/definitely/does/not/exist/anywhere");
        assert!(list_directory_files(missing, true).is_empty());
    }

    #[test]
    fn disable_batch_mode_only_when_queue_emptied() {
        assert!(should_disable_batch_mode_on_ok(0));
        assert!(!should_disable_batch_mode_on_ok(1));
    }

    #[test]
    fn delete_queue_item_removes_in_range_index() {
        let mut queue = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(delete_queue_item(&mut queue, 1));
        assert_eq!(queue, vec!["a".to_string(), "c".to_string()]);
    }

    #[test]
    fn delete_queue_item_negative_or_out_of_range_is_a_no_op() {
        let mut queue = vec!["a".to_string()];
        assert!(!delete_queue_item(&mut queue, -1));
        assert!(!delete_queue_item(&mut queue, 5));
        assert_eq!(queue, vec!["a".to_string()]);
    }

    #[test]
    fn tooltip_only_for_long_entries() {
        assert!(!should_show_full_text_tooltip(72));
        assert!(should_show_full_text_tooltip(73));
    }
}
