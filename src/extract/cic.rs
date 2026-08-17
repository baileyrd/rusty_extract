//! cicdec (`cicdec.exe`) — Clickteam Install Creator installers.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_CIC`
/// (UniExtract.au3:2472-2475) makes: `<program> -db "<file>" "<outdir>"`,
/// run in the input file's own directory (`$filedir` in the source — not
/// `outdir`) with the window hidden.
///
/// **Scope note:** the source surrounds this `_Run` call with
/// `HasNetFramework(4.5)` (a precondition check run before the extractor is
/// invoked at all) and `Cleanup("Block 0x*.bin")` (a post-extraction glob
/// delete). Both are separate, already-tracked runtime-behavior
/// capabilities, not part of this row — this function builds only the
/// invocation itself: program, args, working directory, and window.
pub fn invocation(program: &str, file: &str, outdir: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-db".to_string(), file.to_string(), outdir.to_string()],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C067: matches UniExtract.au3:2472-2475's
    /// `_Run($cic & ' -db "' & $file & '" "' & $outdir & '"', $filedir,
    /// @SW_HIDE)` — program, argument order (`-db`, the quoted file, the
    /// quoted outdir), the `$filedir` working directory, and the hidden
    /// window. `HasNetFramework` and `Cleanup` are excluded per the scope
    /// note above.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\cicdec.exe",
            r"C:\downloads\installer.exe",
            r"C:\downloads\installer_unpacked",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\cicdec.exe");
        assert_eq!(
            inv.args,
            vec![
                "-db".to_string(),
                r"C:\downloads\installer.exe".to_string(),
                r"C:\downloads\installer_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
