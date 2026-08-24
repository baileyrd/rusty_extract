//! Uninstall orchestration and CLI dispatch: ports the `/uninstall`
//! CLI-argument dispatch (UniExtract.au3:623-628, part of the broader
//! `ParseCommandLine` chain C205 also ports) and `Uninstall`'s actual
//! delete sequence (UniExtract.au3:5791-5801) — context-menu/
//! file-association cleanup, logs, and an optional user-data wipe.
//!
//! This capability covers the pure dispatch/ordering decisions. The real
//! actions — `GUI_ContextMenu_remove`/`GUI_ContextMenu_fileassoc` (already
//! ported as pure logic by C202-C204; their actual registry writes are
//! real I/O), `GUI_DeleteLogs`, `DirRemove`, `SendStats`, and `terminate`
//! — are the caller's job, performed in the order [`resolve_uninstall_steps`]
//! returns.

/// What the `/uninstall` CLI verb does once dispatched
/// (UniExtract.au3:623-628): a silent run performs the uninstall directly;
/// an interactive run defers to the GUI uninstall flow (C217) instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallDispatch {
    RunSilentUninstall { remove_user_data: bool },
    ShowGuiUninstall,
}

/// Ports UniExtract.au3:624-628's branch. `remove_user_data_arg_index` is
/// the caller's already-computed `_ArraySearch($cmdline,
/// "/removeuserdata")` result, passed through
/// [`array_search_result_is_truthy`] unchanged from the source — see that
/// function's docs for the real bug this preserves.
pub fn resolve_uninstall_dispatch(
    silent_mode: bool,
    remove_user_data_arg_index: i32,
) -> UninstallDispatch {
    if silent_mode {
        UninstallDispatch::RunSilentUninstall {
            remove_user_data: array_search_result_is_truthy(remove_user_data_arg_index),
        }
    } else {
        UninstallDispatch::ShowGuiUninstall
    }
}

/// Ports the raw expression `Uninstall(True, _ArraySearch($cmdline,
/// "/removeuserdata"))` (UniExtract.au3:625): `_ArraySearch`'s return
/// value — an index `>= 1` if found (index `0` of `$cmdline` is always
/// the argument count, never a string that could match, so a match is
/// never at index `0`), or `-1` if not found — is passed *directly* as
/// the `$bRemoveUserData` boolean parameter with no comparison against
/// `-1` first.
///
/// **Real bug, preserved rather than "fixed"**: AutoIt treats any nonzero
/// number as truthy, and both possible outcomes here (a found index, or
/// `-1` for "not found") are nonzero. So this is true in effectively
/// every real case — a silent `/uninstall` invocation always wipes user
/// data, regardless of whether `/removeuserdata` was actually supplied,
/// diverging from the GUI path (C217), which correctly derives this
/// boolean from a real checkbox. Decide whether to preserve this exact
/// defect or fix it (e.g. `index > -1`) at implementation/sign-off time,
/// per this row's own instruction — not silently changed here.
pub fn array_search_result_is_truthy(index: i32) -> bool {
    index != 0
}

/// One step of `Uninstall`'s delete sequence (UniExtract.au3:5792-5798).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallStep {
    /// `SendStats("uninstall")` (C214) — fire-and-forget, no result checked.
    SendUninstallTelemetry,
    /// `GUI_ContextMenu_remove()` (C202/C203) — always runs, unconditional.
    RemoveContextMenuRegistrations,
    /// `GUI_ContextMenu_fileassoc(0)` (C204) — always runs, unconditional.
    RemoveFileAssociations,
    /// `GUI_DeleteLogs()` — only if `$bRemoveLogs`.
    DeleteLogs,
    /// `DirRemove($settingsdir, 1)` — only if `$bRemoveUserData`.
    RemoveUserData,
}

/// Ports `Uninstall`'s full step sequence (UniExtract.au3:5791-5798)
/// verbatim, in order. **Verified quirk, preserved rather than
/// "fixed"**: the registry teardown ([`UninstallStep::RemoveContextMenuRegistrations`]/
/// [`UninstallStep::RemoveFileAssociations`]) always runs, synchronously,
/// before either file-system step — there is no way to remove file
/// associations/context-menu entries without also being subject to
/// whatever order-dependent side effects that teardown has, and no
/// per-step error handling at all: a failure partway through (e.g. the
/// registry teardown erroring) doesn't stop the log deletion or user-data
/// wipe from being attempted, and none of it is reported back to the
/// caller. There is also no confirmation step anywhere in this sequence
/// — by the time it runs, the decision to uninstall is final.
pub fn resolve_uninstall_steps(remove_logs: bool, remove_user_data: bool) -> Vec<UninstallStep> {
    let mut steps = vec![
        UninstallStep::SendUninstallTelemetry,
        UninstallStep::RemoveContextMenuRegistrations,
        UninstallStep::RemoveFileAssociations,
    ];
    if remove_logs {
        steps.push(UninstallStep::DeleteLogs);
    }
    if remove_user_data {
        steps.push(UninstallStep::RemoveUserData);
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::{
        array_search_result_is_truthy, resolve_uninstall_dispatch, resolve_uninstall_steps,
        UninstallDispatch, UninstallStep,
    };

    #[test]
    fn interactive_run_always_defers_to_the_gui_flow() {
        assert_eq!(
            resolve_uninstall_dispatch(false, -1),
            UninstallDispatch::ShowGuiUninstall
        );
        assert_eq!(
            resolve_uninstall_dispatch(false, 3),
            UninstallDispatch::ShowGuiUninstall
        );
    }

    /// The verified bug, demonstrated concretely: both "found" and "not
    /// found" produce `remove_user_data: true` for a silent run.
    #[test]
    fn silent_run_always_removes_user_data_regardless_of_the_flag() {
        assert_eq!(
            resolve_uninstall_dispatch(true, 3),
            UninstallDispatch::RunSilentUninstall {
                remove_user_data: true
            }
        );
        assert_eq!(
            resolve_uninstall_dispatch(true, -1),
            UninstallDispatch::RunSilentUninstall {
                remove_user_data: true
            }
        );
    }

    #[test]
    fn array_search_truthiness_is_nonzero_not_found_check() {
        assert!(array_search_result_is_truthy(-1));
        assert!(array_search_result_is_truthy(1));
        assert!(array_search_result_is_truthy(5));
        assert!(!array_search_result_is_truthy(0));
    }

    #[test]
    fn minimal_steps_always_include_telemetry_and_registry_teardown() {
        let steps = resolve_uninstall_steps(false, false);
        assert_eq!(
            steps,
            vec![
                UninstallStep::SendUninstallTelemetry,
                UninstallStep::RemoveContextMenuRegistrations,
                UninstallStep::RemoveFileAssociations,
            ]
        );
    }

    #[test]
    fn logs_and_user_data_steps_appended_in_order_when_requested() {
        let steps = resolve_uninstall_steps(true, true);
        assert_eq!(
            steps,
            vec![
                UninstallStep::SendUninstallTelemetry,
                UninstallStep::RemoveContextMenuRegistrations,
                UninstallStep::RemoveFileAssociations,
                UninstallStep::DeleteLogs,
                UninstallStep::RemoveUserData,
            ]
        );
    }

    #[test]
    fn registry_teardown_always_precedes_file_system_steps() {
        let steps = resolve_uninstall_steps(true, true);
        let registry_pos = steps
            .iter()
            .position(|s| *s == UninstallStep::RemoveFileAssociations)
            .unwrap();
        let logs_pos = steps
            .iter()
            .position(|s| *s == UninstallStep::DeleteLogs)
            .unwrap();
        let data_pos = steps
            .iter()
            .position(|s| *s == UninstallStep::RemoveUserData)
            .unwrap();
        assert!(registry_pos < logs_pos);
        assert!(registry_pos < data_pos);
    }
}
