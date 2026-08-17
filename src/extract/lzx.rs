//! unlzx (`unlzx.exe`) — `.lzx` LZX (Amiga) archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_LZX`
/// (UniExtract.au3:2789-2790) makes: `<program> -x "<file>"`, run in
/// `outdir` with the window minimized.
///
/// The source's `_Run($lzx & ' -x "' & $file & '"', $outdir)` call omits
/// the `$show_flag` argument, so it takes `_Run`'s own default
/// (UniExtract.au3:4880: `Func _Run($f, $sWorkingDir = $outdir, $show_flag
/// = @SW_MINIMIZE, ...)`) — `@SW_MINIMIZE`, mapped to `WindowMode::Minimized`
/// here rather than guessed.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C082: the constructed invocation matches
    /// UniExtract.au3:2789-2790's `_Run($lzx & ' -x "' & $file & '"',
    /// $outdir)` — same program, same argument, same working directory,
    /// and the window minimized (`_Run`'s own default for the omitted
    /// `$show_flag` argument).
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unlzx.exe",
            r"C:\downloads\archive.lzx",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unlzx.exe");
        assert_eq!(
            inv.args,
            vec!["-x".to_string(), r"C:\downloads\archive.lzx".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
