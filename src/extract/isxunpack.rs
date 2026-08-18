//! IsXunpack (`IsXunpack.exe`) — legacy InstallShield installers.

use super::{Invocation, WindowMode};

/// Builds the invocation `Case $TYPE_ISEXE`'s isxunpack candidate makes
/// (UniExtract.au3:2711): `<program> "<outdir>\<filenamefull>"`, run in
/// `outdir` with the window shown. This call site uses the raw AutoIt
/// `Run()` built-in directly (via `_MakeCommand`), not the crate's usual
/// `_Run` wrapper — `Run()`'s own default `$show_flag` is
/// `@SW_SHOWNORMAL`, mapped to [`WindowMode::Show`] (see
/// `extract::WindowMode`'s doc comment), distinct from `_Run`'s
/// minimized default used everywhere else this omits `$show_flag`.
///
/// **Scope — invocation only.** Reached only through `$TYPE_ISEXE`'s GUI
/// candidate list (`GUI_MethodSelect`, C053, deferred GUI subsystem,
/// D001) — composite, conditional dispatch, not registered in
/// `extract::dispatch::HARDCODED_CASES`. Also out of scope: the
/// `_FileMove($file, $outdir)` that relocates the input file into
/// `outdir` *before* this runs (this invocation's own `filenamefull`
/// argument assumes that move already happened), the
/// `WinWait`/`WinActivate`/`Send("{ENTER}")` keypress automation that
/// dismisses IsXunpack's own confirmation prompt afterward, and the
/// final `_FileMove` back to `filedir` once it's done — all real
/// filesystem/window-automation concerns.
pub fn invocation(program: &str, outdir: &str, filenamefull: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![format!("{outdir}\\{filenamefull}")],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C076: the constructed invocation
    /// matches UniExtract.au3:2711's `IsXunpack.exe
    /// "<outdir>\<filenamefull>"` call — program, args, `outdir` as the
    /// working directory, and the shown window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\IsXunpack.exe",
            r"C:\downloads\installer_unpacked",
            "installer.exe",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\IsXunpack.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\installer_unpacked\installer.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
