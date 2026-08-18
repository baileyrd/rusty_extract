//! Unicode/UNC-path input relocation reversion — C159. The full
//! relocation state machine `MoveInputFileIfNecessary()`
//! (UniExtract.au3:2218-2266, C175/C176) sets up isn't ported yet, but
//! its end-of-run counterpart — `terminate()`'s unconditional `$iUnicodeMode`
//! reversal (UniExtract.au3:4101-4114) — stands on its own: given
//! whichever mode a relocation left behind, decide how to revert it.

/// Mirrors `$UNICODE_NONE`/`$UNICODE_MOVE`/`$UNICODE_COPY`
/// (UniExtract.au3:100): `Move` when the input file and its ASCII
/// replacement shared a drive letter and could be renamed in place,
/// `Copy` when they didn't and a copy was made instead — set by
/// `MoveInputFileIfNecessary()` (not yet ported), consumed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeMode {
    None,
    Move,
    Copy,
}

/// What to do with the ASCII working copy of the input file on
/// reversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRevertAction {
    /// No relocation happened; nothing to revert.
    None,
    /// `_FileMove($file, $oldpath, 1)` — move the working copy back to
    /// the original unicode path.
    MoveBack,
    /// `FileRecycle($file)` — the working copy was a *copy*, not a
    /// rename, so reverting just discards it via the recycle bin.
    Recycle,
}

/// Reproduces `terminate()`'s unconditional `$iUnicodeMode` reversal
/// (UniExtract.au3:4101-4114): `If $iUnicodeMode Then ... EndIf`, run at
/// the top of every `terminate()` call — success, failure, or anything
/// else — never gated on the run's outcome, only on whether a relocation
/// happened at all. Returns the action to take on the input file, and
/// whether the output directory should be moved back to its original
/// location too (`_DirMove($outdir, $oldoutdir)`, done for both `Move`
/// and `Copy`, never for `None`).
pub fn decide_unicode_reversion(mode: UnicodeMode) -> (FileRevertAction, bool) {
    match mode {
        UnicodeMode::None => (FileRevertAction::None, false),
        UnicodeMode::Move => (FileRevertAction::MoveBack, true),
        UnicodeMode::Copy => (FileRevertAction::Recycle, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_relocation_means_no_reversion() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::None),
            (FileRevertAction::None, false)
        );
    }

    #[test]
    fn move_mode_reverts_by_moving_back_and_moves_outdir() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::Move),
            (FileRevertAction::MoveBack, true)
        );
    }

    #[test]
    fn copy_mode_reverts_by_recycling_and_still_moves_outdir() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::Copy),
            (FileRevertAction::Recycle, true)
        );
    }
}
