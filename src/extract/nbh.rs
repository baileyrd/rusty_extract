//! NBHextract (`NBHextract.exe`) — HTC NBH ROM images.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_NBH`
/// (UniExtract.au3:2952-2953) makes: `<program> "<file>"`, run in `outdir`
/// with the window shown normally.
///
/// The source calls `RunWait(_MakeCommand($nbh, True) & ' "' & $file &
/// '"', $outdir)` — AutoIt's native `RunWait`, not this script's own
/// `_Run` wrapper, and with no explicit `show_flag` argument, so it takes
/// `RunWait`'s own default, `@SW_SHOWNORMAL` (`WindowMode::Show`, the same
/// mapping this crate already uses for an explicit `True` show-flag
/// literal — see the `extract` module doc comment).
///
/// **Scope note — shell wrapping not modeled:** `_MakeCommand($nbh, True)`
/// routes through the same generic `cmd.exe /d /c` shell-wrapping as
/// `extract::sqlite`'s call site, and has no effect on the arguments
/// `NBHextract.exe` itself receives here (no redirection/piping at this
/// call site) — this port's `Invocation` targets `NBHextract.exe`
/// directly, the same as every other module in this crate.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C088: the constructed invocation matches
    /// UniExtract.au3:2952-2953's effective `NBHextract.exe "<file>"` call
    /// — program, args, the `$outdir` working directory, and a normally
    /// shown window (`RunWait`'s own default show flag).
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\NBHextract.exe",
            r"C:\downloads\ROM.nbh",
            r"C:\downloads\ROM_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\NBHextract.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\ROM.nbh".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\ROM_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
