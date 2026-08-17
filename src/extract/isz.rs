//! unisz (`unisz.exe`) — ISZ compressed ISO (stage 1 of disk-image conversion).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_ISZ`
/// (UniExtract.au3:2775-2778) makes: `<program> "<file>"`, run in the temp
/// output directory, window shown normally.
///
/// The source calls `_RunInTempOutdir`, not plain `_Run` — a variant that
/// stages output in a temp directory before moving it into place — but for
/// the invocation itself (program, args, working directory, window) the
/// shape is identical to every other `_Run`-based extractor here. The
/// temp-dir-then-move orchestration `_RunInTempOutdir` layers on top is a
/// separate, already-tracked runtime-behavior capability, not part of this
/// one.
///
/// The source passes `True` as the explicit 4th positional argument
/// (`$show_flag`), which is AutoIt's `@SW_SHOWNORMAL` — mapped to
/// [`WindowMode::Show`] here (see the doc comment on `WindowMode`).
///
/// The preceding `_CreateTrayMessageBox(...)` call (UniExtract.au3:2776) is
/// a UI notification — out of scope, deferred GUI subsystem work tracked
/// under manifest row D001, not part of this row.
pub fn invocation(program: &str, file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C078: matches UniExtract.au3:2775-2778's
    /// `_RunInTempOutdir($tempoutdir, $isz & ' "' & $file & '"',
    /// $tempoutdir, True, True)` — same program, same argument, the
    /// `$tempoutdir` working directory, and the `True` (`@SW_SHOWNORMAL`)
    /// window flag mapped to `Show`.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unisz.exe",
            r"C:\downloads\archive.isz",
            r"C:\downloads\archive_unpacked\tmp123456",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unisz.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.isz".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
