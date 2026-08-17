//! FSB extractor (`fsbext.exe`) — FMOD Sample Bank (`.fsb`).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_FSB`
/// (UniExtract.au3:2559-2562) makes: `<program> -o -1 -A -d "<outdir>"
/// "<file>"`, run in `file_dir` (the input file's own directory, `$filedir`
/// in the source), window minimized.
///
/// `-o -1` and `-A` are passed through as separate argument tokens exactly
/// as the source writes them (no flag is fused to an adjacent value the
/// way `extract::freearc`'s `-dp"<outdir>"` or `extract::lzop`'s
/// `-p"<outdir>"` are); only `-d`'s value is quoted in the source, matching
/// how every other extractor here quotes a path argument.
///
/// **Scope note:** the source's `Case $TYPE_FSB` also calls
/// `Cleanup("*.ogg")` after the `_Run`, deleting the raw `.ogg` dumps FSB
/// extraction produces (they "cannot be played" per the source's own
/// comment). That post-extraction cleanup is separate runtime behavior —
/// not part of building this invocation — and is not represented here; it
/// is tracked as its own capability.
pub fn invocation(program: &str, outdir: &str, file: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-o".to_string(),
            "-1".to_string(),
            "-A".to_string(),
            "-d".to_string(),
            outdir.to_string(),
            file.to_string(),
        ],
        working_dir: file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C072: matches UniExtract.au3:2559-2562's
    /// `_Run($fsb & ' -o -1 -A -d "' & $outdir & '" "' & $file & '"',
    /// $filedir, @SW_MINIMIZE, True, True, False)` — program, argument
    /// order (`-o -1 -A -d <outdir> <file>`), the `$filedir` working
    /// directory, and the minimized window. The `Cleanup("*.ogg")` call
    /// that follows in the source is out of scope (see the module doc
    /// comment).
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\fsbext.exe",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.fsb",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\fsbext.exe");
        assert_eq!(
            inv.args,
            vec![
                "-o".to_string(),
                "-1".to_string(),
                "-A".to_string(),
                "-d".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.fsb".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
