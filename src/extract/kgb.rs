//! KGB Archiver (`kgb2_console.exe`) — `.kgb`/`.kge` archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_KGB`
/// (UniExtract.au3:2780-2781) makes: `<program> "<file>"`, run in `outdir`
/// with the window minimized.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C079: the constructed invocation matches
    /// UniExtract.au3:2780-2781's `_Run($kgb & ' "' & $file & '"', $outdir,
    /// @SW_MINIMIZE, True, False, False)` — same program, same argument,
    /// same working directory, same minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\kgb2_console.exe",
            r"C:\downloads\archive.kgb",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\kgb2_console.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.kgb".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
