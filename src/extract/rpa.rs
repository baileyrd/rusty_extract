//! unrpa (`unrpa.exe`) — Ren'Py `.rpa` archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_RPA`
/// (UniExtract.au3:3016-3017) makes: `<program> -m -v --continue-on-error
/// -p "<outdir>" "<file>"`, run in the program's own install directory
/// (`@ScriptDir` in the source — not `outdir`, unlike most extractor
/// cases), with the window shown normally rather than hidden/minimized.
pub fn invocation(program: &str, script_dir: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-m".to_string(),
            "-v".to_string(),
            "--continue-on-error".to_string(),
            "-p".to_string(),
            outdir.to_string(),
            file.to_string(),
        ],
        working_dir: script_dir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C094: matches UniExtract.au3:3016-3017's
    /// `_Run($rpa & ' -m -v --continue-on-error -p "' & $outdir & '" "' &
    /// $file & '"', @ScriptDir, True, True, True)` — program, argument
    /// order, the `@ScriptDir` working directory (not `outdir`), and the
    /// non-hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unrpa.exe",
            r"C:\UniExtract",
            r"C:\games\game.rpa",
            r"C:\games\game_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unrpa.exe");
        assert_eq!(
            inv.args,
            vec![
                "-m".to_string(),
                "-v".to_string(),
                "--continue-on-error".to_string(),
                "-p".to_string(),
                r"C:\games\game_unpacked".to_string(),
                r"C:\games\game.rpa".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\UniExtract");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
