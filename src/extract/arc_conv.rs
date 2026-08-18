//! arc_conv (`arc_conv.exe`, UniExtract-authored) — KiriKiri/ERISA/YU-RIS
//! engine archive conversion, GUI-automated.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_ARC_CONV`
/// (UniExtract.au3:2394) makes: `<program> "<file>"`, run in `outdir`
/// with the window hidden.
///
/// **Scope note — shell wrapping not modeled as a literal string:** the
/// source builds this via `Run(Cout(_MakeCommand($arc_conv & ' "' &
/// $file & '"', True)), $outdir, @SW_HIDE)` — `_MakeCommand`'s generic
/// `cmd.exe /d /c` shell-wrapping (and `Cout`, a debug-log passthrough
/// that returns its argument unchanged) — with no effect on the arguments
/// `arc_conv.exe` itself receives, so this port's `Invocation` targets
/// the exe directly, the same as every other module in this crate.
///
/// **Not modeled here:** the preceding `HasPlugin($arc_conv, ...)`
/// precondition check; the `WinWait`/window-text-polling loop that
/// follows launch (arc_conv reports per-file progress through its own
/// window title/text, which this loop reads to drive a tray-status
/// display) — deferred GUI subsystem, manifest row D001, matching this
/// row's own "GUI-automated" description; and the trailing
/// `MoveFiles($file & "~", $outdir, ...)` relocation. All separate
/// runtime behavior, not part of building this one invocation.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C113: the constructed invocation
    /// matches UniExtract.au3:2394's effective `arc_conv.exe "<file>"`
    /// call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\arc_conv.exe",
            r"C:\downloads\archive.arc",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\arc_conv.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.arc".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
