//! Drag-and-drop file/directory handling, ported from `GUI_Drop`
//! (UniExtract.au3:6710-6732) — capability C187.
//!
//! **`WM_DROPFILES_UNICODE_FUNC` (UniExtract.au3:6202-6216) isn't ported
//! as its own piece** — it's a raw `DragQueryFileW` enumeration loop
//! whose entire job is turning an OS drop event into a list of file
//! paths, a job `egui`'s own native drag-drop input (`ctx.input(|i|
//! i.raw.dropped_files)`) already does, superseding the Win32 workaround
//! entirely rather than needing a parallel implementation of it (same
//! class of "old workaround made moot by the new toolkit" as C183's
//! DPI-scaling note and C185's tooltip-workaround note).

/// What to do with one dropped path (`GUI_Drop`'s per-item branch,
/// UniExtract.au3:6715-6726).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAction {
    /// The path no longer exists (`Not FileExists($sPath)`); skip it
    /// silently — no error is ever shown for this.
    Skip,
    /// A directory: recursively expand into the batch queue
    /// (`GUI_Batch_AddDirectory`, capability C188).
    AddDirectory,
    /// A file, and the *only* path in this drop: populate the input
    /// fields (`GUI_Drop_Parse`) and stop — nothing gets auto-queued.
    PopulateOnly,
    /// A file, and one of *multiple* paths in this drop: populate the
    /// fields, then queue it (`GUI_Batch`, capability C188).
    PopulateAndQueue,
}

/// Ports `GUI_Drop`'s per-item dispatch (UniExtract.au3:6715-6726).
/// `is_only_dropped_path` is `UBound($gaDropFiles) == 1` — the whole
/// drop batch being a single path, not just this being the first one
/// checked.
pub fn decide_drop_action(
    path_exists: bool,
    is_directory: bool,
    is_only_dropped_path: bool,
) -> DropAction {
    if !path_exists {
        return DropAction::Skip;
    }
    if is_directory {
        return DropAction::AddDirectory;
    }
    if is_only_dropped_path {
        DropAction::PopulateOnly
    } else {
        DropAction::PopulateAndQueue
    }
}

/// One dropped path, with the filesystem facts [`decide_drop_action`]
/// needs already resolved by the caller (real I/O, not this module's
/// concern).
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedPath {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
}

/// Ports `GUI_Drop`'s full loop (UniExtract.au3:6713-6727), including
/// the early exit for a single-file drop (UniExtract.au3:6723): once a
/// [`DropAction::PopulateOnly`] fires, no further paths are processed —
/// though since that only ever happens when the whole batch is exactly
/// one path, there is nothing left to process regardless. Does not
/// itself perform the actions (real I/O/UI); it returns the decision
/// per path for the caller to act on.
pub fn decide_drop_actions(paths: &[DroppedPath]) -> Vec<(String, DropAction)> {
    let is_only_dropped_path = paths.len() == 1;
    let mut actions = Vec::with_capacity(paths.len());
    for item in paths {
        let action = decide_drop_action(item.exists, item.is_directory, is_only_dropped_path);
        let should_stop = action == DropAction::PopulateOnly;
        actions.push((item.path.clone(), action));
        if should_stop {
            break;
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_path_is_skipped_regardless_of_type() {
        assert_eq!(decide_drop_action(false, false, true), DropAction::Skip);
        assert_eq!(decide_drop_action(false, true, true), DropAction::Skip);
    }

    #[test]
    fn directory_always_expands_regardless_of_batch_size() {
        assert_eq!(
            decide_drop_action(true, true, true),
            DropAction::AddDirectory
        );
        assert_eq!(
            decide_drop_action(true, true, false),
            DropAction::AddDirectory
        );
    }

    /// Parity test for capability C187: a single dropped file only
    /// populates the fields; it is never auto-queued.
    #[test]
    fn single_dropped_file_populates_only() {
        assert_eq!(
            decide_drop_action(true, false, true),
            DropAction::PopulateOnly
        );
    }

    /// Parity test for capability C187: a file among multiple dropped
    /// paths is populated *and* queued.
    #[test]
    fn file_among_multiple_populates_and_queues() {
        assert_eq!(
            decide_drop_action(true, false, false),
            DropAction::PopulateAndQueue
        );
    }

    /// Parity test for capability C187: mixed drops (files and
    /// directories together) are dispatched independently per item.
    #[test]
    fn mixed_drop_dispatches_each_item_independently() {
        let paths = vec![
            DroppedPath {
                path: r"C:\downloads\a.zip".to_string(),
                exists: true,
                is_directory: false,
            },
            DroppedPath {
                path: r"C:\downloads\folder".to_string(),
                exists: true,
                is_directory: true,
            },
            DroppedPath {
                path: r"C:\downloads\gone.zip".to_string(),
                exists: false,
                is_directory: false,
            },
        ];
        assert_eq!(
            decide_drop_actions(&paths),
            vec![
                (
                    r"C:\downloads\a.zip".to_string(),
                    DropAction::PopulateAndQueue
                ),
                (r"C:\downloads\folder".to_string(), DropAction::AddDirectory),
                (r"C:\downloads\gone.zip".to_string(), DropAction::Skip),
            ]
        );
    }

    /// Parity test for capability C187: a single-file drop stops after
    /// that one item -- there is nothing left in the batch anyway, but
    /// this confirms the early-exit is honored rather than iterating
    /// past it.
    #[test]
    fn single_file_batch_produces_exactly_one_action() {
        let paths = vec![DroppedPath {
            path: r"C:\downloads\only.zip".to_string(),
            exists: true,
            is_directory: false,
        }];
        assert_eq!(
            decide_drop_actions(&paths),
            vec![(
                r"C:\downloads\only.zip".to_string(),
                DropAction::PopulateOnly
            )]
        );
    }
}
