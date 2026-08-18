//! Post-extraction cleanup: ports pieces of `Cleanup()`
//! (UniExtract.au3:3645-3703) — capability C155, **partial**. Each
//! extractor case that leaves behind installer cruft (readme files,
//! uninstallers, `.ini` config it wrote) calls this per output item to
//! either delete it or move it into an "additional files" subfolder.
//!
//! **Scope — partial, manifest row stays REQUIRED.** Ported: the
//! mode-gating (`$iCleanup`/`$iMode` disables the whole call when
//! `$OPTION_KEEP`), the per-item delete-vs-move/folder-vs-file action
//! selection, the `$outdir`-prefixing path resolution, and wildcard
//! *classification*. **Not ported:** actually expanding a wildcard
//! target into the files it matches (`_FileListToArray`, real
//! filesystem I/O) and the actual `DirRemove`/`FileDelete`/`_DirMove`/
//! `_FileMove` calls themselves, real I/O the caller performs.

use crate::prefs::DeleteSourceFileOption;

/// Ports the `$outdir`-prefixing step (UniExtract.au3:3661): `If Not
/// StringInStr($sFile, $outdir) Then $sFile = $outdir & "\" & $sFile` —
/// a file path not already containing `outdir` gets it prefixed.
/// Matches case-insensitively, AutoIt's `StringInStr` default (bare
/// call, no case-sensitivity argument).
pub fn resolve_target_path(file: &str, outdir: &str) -> String {
    if file.to_lowercase().contains(&outdir.to_lowercase()) {
        file.to_string()
    } else {
        format!("{outdir}\\{file}")
    }
}

/// A cleanup target's shape, per `$bIsFolderWildcard`/`$bIsWildcard`
/// (UniExtract.au3:3664-3665).
///
/// **Behavioral finding — `FolderWildcard` is silently a no-op.**
/// `$bIsFolderWildcard` (a path ending `\*`, meant to mean "everything
/// inside this folder") is computed but never read again anywhere in
/// the function after excluding it from `$bIsWildcard` — it drives no
/// branch of its own. Such a path also isn't a real, existing directory
/// (`_IsDirectory` on a literal `...\*` string returns false), so a
/// `FolderWildcard` target falls straight through to the *file*
/// delete/move calls (`FileDelete`/`_FileMove`) operating on a path
/// that can't exist, which silently do nothing — the source logs
/// "Cleanup: Deleting ..."/"Cleanup: Moving ..." for it regardless.
/// Reproduced as-is: this crate's [`decide_cleanup_action`] takes
/// `is_folder` as a plain caller-supplied fact, so a `FolderWildcard`
/// target's caller-measured `is_folder = false` reproduces the same
/// no-op outcome without this module needing its own special case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// Ends with `\*`.
    FolderWildcard,
    /// Contains `*`, not a `FolderWildcard`.
    Wildcard,
    Plain,
}

/// Ports `$bIsFolderWildcard = StringRight($sFile, 2) == "\*"` and
/// `$bIsWildcard = $bIsFolderWildcard == False And StringInStr($sFile,
/// "*") > 0` (UniExtract.au3:3664-3665).
pub fn classify_target(file: &str) -> TargetKind {
    if file.ends_with("\\*") {
        TargetKind::FolderWildcard
    } else if file.contains('*') {
        TargetKind::Wildcard
    } else {
        TargetKind::Plain
    }
}

/// Ports the `If $bIsWildcard Then` gate (UniExtract.au3:3668): only a
/// [`TargetKind::Wildcard`] target triggers expanding into the files it
/// matches (real filesystem I/O, not ported — see module scope note)
/// and skipping the direct delete/move handling below it in the
/// source's loop.
pub fn should_expand_wildcard(kind: TargetKind) -> bool {
    kind == TargetKind::Wildcard
}

/// What one cleanup target's delete-or-move call should be, given the
/// resolved mode and whether the target is a folder — ports the nested
/// `If $iMode = $OPTION_DELETE Then ... Else ...` /
/// `If $bIsFolder Then ... Else ...` structure
/// (UniExtract.au3:3682-3701). Only reached for a non-Keep mode (see
/// [`decide_cleanup_action`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupItemAction {
    DeleteFolder,
    DeleteFile,
    MoveFolder,
    MoveFile,
}

/// Ports `Cleanup()`'s mode gating and per-item action selection
/// (UniExtract.au3:3646,3682-3701) together. `None` reproduces `If Not
/// $iMode Then Return` — `$OPTION_KEEP` disables cleanup entirely, no
/// action taken for any target. Any other mode reaches `If $iMode =
/// $OPTION_DELETE Then (delete) Else (move)` — note `$OPTION_ASK` is
/// treated exactly like `$OPTION_MOVE` here (the source's `Else` covers
/// everything that isn't `$OPTION_DELETE`), matching
/// [`DeleteSourceFileOption`]'s existing documented behavior for this
/// same enum shared with the `deletesourcefile` preference.
pub fn decide_cleanup_action(
    mode: DeleteSourceFileOption,
    is_folder: bool,
) -> Option<CleanupItemAction> {
    match mode {
        DeleteSourceFileOption::Keep => None,
        DeleteSourceFileOption::Delete => Some(if is_folder {
            CleanupItemAction::DeleteFolder
        } else {
            CleanupItemAction::DeleteFile
        }),
        DeleteSourceFileOption::Ask | DeleteSourceFileOption::Move => Some(if is_folder {
            CleanupItemAction::MoveFolder
        } else {
            CleanupItemAction::MoveFile
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_target, decide_cleanup_action, resolve_target_path, should_expand_wildcard,
        CleanupItemAction, TargetKind,
    };
    use crate::prefs::DeleteSourceFileOption;

    /// Parity test for capability C155: a file already containing
    /// `outdir` is left unchanged.
    #[test]
    fn resolve_target_path_leaves_outdir_prefixed_path_unchanged() {
        assert_eq!(
            resolve_target_path(r"C:\out\readme.txt", r"C:\out"),
            r"C:\out\readme.txt"
        );
    }

    /// Parity test for capability C155: a bare relative name gets
    /// `outdir` prefixed.
    #[test]
    fn resolve_target_path_prefixes_relative_name() {
        assert_eq!(
            resolve_target_path("readme.txt", r"C:\out"),
            r"C:\out\readme.txt"
        );
    }

    /// Parity test for capability C155: the `outdir` containment check
    /// is case-insensitive.
    #[test]
    fn resolve_target_path_containment_check_is_case_insensitive() {
        assert_eq!(
            resolve_target_path(r"c:\OUT\readme.txt", r"C:\out"),
            r"c:\OUT\readme.txt"
        );
    }

    /// Parity test for capability C155: target classification
    /// distinguishes all three shapes.
    #[test]
    fn classify_target_distinguishes_all_three_shapes() {
        assert_eq!(
            classify_target(r"C:\out\extras\*"),
            TargetKind::FolderWildcard
        );
        assert_eq!(classify_target(r"C:\out\*.log"), TargetKind::Wildcard);
        assert_eq!(classify_target(r"C:\out\readme.txt"), TargetKind::Plain);
    }

    /// Parity test for capability C155: only `Wildcard` triggers
    /// expansion — `FolderWildcard` does not, matching the source's
    /// `$bIsFolderWildcard == False` guard on `$bIsWildcard`.
    #[test]
    fn should_expand_wildcard_only_for_wildcard_kind() {
        assert!(should_expand_wildcard(TargetKind::Wildcard));
        assert!(!should_expand_wildcard(TargetKind::FolderWildcard));
        assert!(!should_expand_wildcard(TargetKind::Plain));
    }

    /// Parity test for capability C155: `Keep` disables cleanup
    /// entirely, regardless of `is_folder`.
    #[test]
    fn keep_mode_disables_cleanup() {
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Keep, true),
            None
        );
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Keep, false),
            None
        );
    }

    /// Parity test for capability C155: `Delete` mode selects the
    /// folder or file delete action.
    #[test]
    fn delete_mode_selects_folder_or_file_delete() {
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Delete, true),
            Some(CleanupItemAction::DeleteFolder)
        );
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Delete, false),
            Some(CleanupItemAction::DeleteFile)
        );
    }

    /// Parity test for capability C155: both `Move` and `Ask` select
    /// the move action — the source's `Else` branch covers everything
    /// that isn't `$OPTION_DELETE`.
    #[test]
    fn move_and_ask_modes_both_select_move_action() {
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Move, true),
            Some(CleanupItemAction::MoveFolder)
        );
        assert_eq!(
            decide_cleanup_action(DeleteSourceFileOption::Ask, false),
            Some(CleanupItemAction::MoveFile)
        );
    }
}
