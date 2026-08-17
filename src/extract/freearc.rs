//! FreeArc (`unarc.exe`) — FreeArc `.arc` archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_FREEARC`
/// (UniExtract.au3:2556-2557) makes: `<program> x -dp"<outdir>" "<file>"`,
/// run in `filedir` (the input file's own directory), window hidden.
///
/// The source concatenates `-dp` directly onto the quoted outdir with no
/// space (`-dp"' & $outdir & '"'`), so the resulting command-line token is
/// a single argument `-dp"<outdir>"` — the embedded quote characters are
/// literally part of the argument value. This is deliberate, not a typo to
/// "fix" into `-dp` and `"<outdir>"` as two separate args, or into
/// `-dp<outdir>` without the quotes.
pub fn invocation(program: &str, file_dir: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "x".to_string(),
            format!("-dp\"{outdir}\""),
            file.to_string(),
        ],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C071: matches UniExtract.au3:2556-2557's
    /// `_Run($freearc & ' x -dp"' & $outdir & '" "' & $file & '"',
    /// $filedir, @SW_HIDE, True, True, False, False)` — program, argument
    /// order (including the single-token `-dp"<outdir>"` argument with its
    /// embedded quotes), the `$filedir` working directory, and the hidden
    /// window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unarc.exe",
            r"C:\downloads",
            r"C:\downloads\archive.arc",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unarc.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r#"-dp"C:\downloads\archive_unpacked""#.to_string(),
                r"C:\downloads\archive.arc".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
