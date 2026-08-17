//! GARbro (`GARbro.Console.exe`) — 500+ visual-novel/game-engine archive formats.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_GARBRO`
/// (UniExtract.au3:2565-2566) makes: `<program> x -ocu -if png -o
/// "<outdir>" "<file>"`, run in `outdir`, window minimized.
///
/// `-ocu` selects overwrite-and-continue behavior; `-if png` forces PNG as
/// the output format for any image-format conversion GARbro performs during
/// extraction — preserved verbatim from the source rather than simplified,
/// since GARbro can otherwise emit an archive-format-specific native image
/// encoding. `-o "<outdir>"` and the trailing `<file>` are separate
/// arguments (unlike e.g. `extract::freearc`'s single concatenated
/// `-dp"<outdir>"` token), matching the source's `' -o "' & $outdir & '" "'
/// & $file & '"'` spacing. The working directory is `$outdir` (not
/// `$filedir`), and the window is minimized (`@SW_MINIMIZE`).
pub fn invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "x".to_string(),
            "-ocu".to_string(),
            "-if".to_string(),
            "png".to_string(),
            "-o".to_string(),
            outdir.to_string(),
            file.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C068: matches UniExtract.au3:2565-2566's
    /// `_Run($garbro & ' x -ocu -if png -o "' & $outdir & '" "' & $file &
    /// '"', $outdir, @SW_MINIMIZE)` — program, argument order (including
    /// the `-if png` forced-output-format flag), the `$outdir` working
    /// directory, and the minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\GARbro.Console.exe",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.arc",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\GARbro.Console.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                "-ocu".to_string(),
                "-if".to_string(),
                "png".to_string(),
                "-o".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.arc".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
