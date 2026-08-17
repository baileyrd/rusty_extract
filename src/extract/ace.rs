//! acefile (`acefile.exe`) — ACE archives, ACE SFX.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_ACE`
/// (UniExtract.au3:2346-2349) makes: `<program> -x -v -d "<outdir>"
/// "<file>"`, run in `outdir` with the window hidden.
///
/// Argument order mirrors the source's literal string concatenation:
/// `-x` (extract), `-v` (verbose — required for `acefile.exe` to report
/// success/failure in a form the caller can parse), `-d "<outdir>"` (the
/// destination directory), then the archive path itself. `outdir` doubles
/// as both the `-d` argument and the working directory, matching the
/// source's `_Run(..., $outdir, @SW_HIDE, True, True, True, True)` call.
///
/// Scope note: the source's `If $success == $RESULT_FAILED Then
/// check7z($arcdisp)` — falling back to 7-Zip when `acefile.exe` fails —
/// is separate runtime behavior (result handling and a different
/// capability's invocation), not part of this row; this function only
/// builds the `acefile.exe` invocation itself.
pub fn invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-x".to_string(),
            "-v".to_string(),
            "-d".to_string(),
            outdir.to_string(),
            file.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C057: matches UniExtract.au3:2346-2349's
    /// `_Run($ace & ' -x -v -d "' & $outdir & '" "' & $file & '"',
    /// $outdir, @SW_HIDE, True, True, True, True)` — program, argument
    /// order, the `$outdir` working directory, and the hidden window. The
    /// following `check7z($arcdisp)` fallback-on-failure call is out of
    /// scope for this capability (see the module doc comment).
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\acefile.exe",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.ace",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\acefile.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                "-v".to_string(),
                "-d".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.ace".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
