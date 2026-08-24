//! Error dialogs with feedback/scan integration — capability C194. Ports
//! the pure decisions inside `GUI_Error_WithFeedbackButton`
//! (UniExtract.au3:7648-7681), `GUI_Error_UnknownExt`
//! (UniExtract.au3:7684-7738), and `_GUI_FileScan`
//! (UniExtract.au3:7611-7645).
//!
//! **Neither dialog is wired to a real window** — same treatment as
//! every other dialog this migration phase has ported so far (C188-C193).
//! The unknown-file-type dialog's logo image doubling as a button that
//! launches Exeinfo PE (`Run($exeinfope & ' "' & $file & '"', $filedir)`,
//! UniExtract.au3:7732) is real process-spawning I/O with no decision
//! logic of its own to port — it stays a real-wiring concern for whoever
//! eventually builds this window, same as `_GUI_FileScan`'s clipboard
//! write itself (`ClipPut`).

/// Ports the `If $silentmode Then Return` no-op gate both
/// `GUI_Error_WithFeedbackButton` (UniExtract.au3:7649) and
/// `GUI_Error_UnknownExt` (UniExtract.au3:7685) open with — neither error
/// dialog ever shows in silent/unattended runs.
pub fn should_show_error_dialog(silent_mode: bool) -> bool {
    !silent_mode
}

/// What the Copy button copies (`_GUI_FileScan`, UniExtract.au3:7636-
/// 7639, and `GUI_Error_UnknownExt`'s own inline duplicate of the same
/// logic, UniExtract.au3:7723-7726): the whole file-scan text when
/// nothing is selected, or just the selection. The source implements this
/// identically in two places rather than sharing one function — this
/// port unifies it into the one function here instead of reproducing the
/// duplication, per this capability's own manifest note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyTarget {
    All,
    Selection,
}

/// Ports the `$iLen < 1 ? $sFileType : StringMid(...)` branch
/// (UniExtract.au3:7639,7726). `selection_len` is `$aReturn[1] -
/// $aReturn[0]` — the selection's character length, already computed by
/// the caller from `_GUICtrlEdit_GetSel`'s raw start/end pair.
pub fn resolve_copy_target(selection_len: i32) -> CopyTarget {
    if selection_len < 1 {
        CopyTarget::All
    } else {
        CopyTarget::Selection
    }
}

/// Ports the file-scan edit box's vertical-scrollbar gate — present at
/// two call sites with two *different* thresholds because the two edit
/// boxes have different heights: `_GUI_FileScan`'s standalone dialog uses
/// `$iCount > 13` (UniExtract.au3:7622), while `GUI_Error_UnknownExt`'s
/// embedded, shorter edit box uses `$iCount > 7` (UniExtract.au3:7707).
/// Both callers pass their own real threshold rather than this function
/// hardcoding either one.
pub fn needs_vertical_scrollbar(line_count: usize, threshold: usize) -> bool {
    line_count > threshold
}

/// `GUI_Error_UnknownExt`'s dynamic sizing based on whether a file-scan
/// result was found (UniExtract.au3:7691-7693,7703): a shorter dialog
/// with no scan section at all when there's nothing to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownExtLayout {
    pub height: u32,
    pub show_scan_section: bool,
}

/// Ports `Local Const $bHasResult = StringLen($sFileType) > 0` through to
/// `Local Const $iHeight = $bHasResult? 290: 190` and the `If $bHasResult
/// Then` block that conditionally creates the scan-result edit box and
/// its Copy button (UniExtract.au3:7691-7711).
pub fn resolve_unknown_ext_layout(has_scan_result: bool) -> UnknownExtLayout {
    UnknownExtLayout {
        height: if has_scan_result { 290 } else { 190 },
        show_scan_section: has_scan_result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_dialog_hidden_in_silent_mode() {
        assert!(!should_show_error_dialog(true));
        assert!(should_show_error_dialog(false));
    }

    #[test]
    fn copy_target_is_all_when_nothing_selected() {
        assert_eq!(resolve_copy_target(0), CopyTarget::All);
    }

    /// Parity test: the source's own comparison is `< 1`, so a negative
    /// selection length (never produced by a real edit control, but not
    /// guarded against either) also copies everything.
    #[test]
    fn copy_target_is_all_for_negative_length_too() {
        assert_eq!(resolve_copy_target(-1), CopyTarget::All);
    }

    #[test]
    fn copy_target_is_selection_when_something_is_selected() {
        assert_eq!(resolve_copy_target(1), CopyTarget::Selection);
        assert_eq!(resolve_copy_target(42), CopyTarget::Selection);
    }

    /// Parity test: the standalone `_GUI_FileScan` dialog's own threshold
    /// (13), distinct from the embedded version's.
    #[test]
    fn scrollbar_threshold_matches_standalone_filescan_dialog() {
        assert!(!needs_vertical_scrollbar(13, 13));
        assert!(needs_vertical_scrollbar(14, 13));
    }

    /// Parity test: `GUI_Error_UnknownExt`'s own, smaller threshold (7).
    #[test]
    fn scrollbar_threshold_matches_embedded_unknown_ext_dialog() {
        assert!(!needs_vertical_scrollbar(7, 7));
        assert!(needs_vertical_scrollbar(8, 7));
    }

    #[test]
    fn unknown_ext_layout_with_scan_result() {
        assert_eq!(
            resolve_unknown_ext_layout(true),
            UnknownExtLayout {
                height: 290,
                show_scan_section: true
            }
        );
    }

    #[test]
    fn unknown_ext_layout_without_scan_result() {
        assert_eq!(
            resolve_unknown_ext_layout(false),
            UnknownExtLayout {
                height: 190,
                show_scan_section: false
            }
        );
    }
}
