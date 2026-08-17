//! lzip (`lzip.exe`) — `.lz` LZIP compressed files.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_LZ`
/// (UniExtract.au3:2783-2784) makes: `<program> -d -k -v -v "<file>"`, run
/// in the temp output directory, window shown normally.
///
/// The source calls `_RunInTempOutdir`, not plain `_Run` — a variant that
/// stages output in a temp directory before moving it into place — but for
/// the invocation itself (program, args, working directory, window) the
/// shape is identical to every other `_Run`-based extractor here. The
/// temp-dir-then-move orchestration `_RunInTempOutdir` layers on top is a
/// separate, already-tracked runtime-behavior capability, not part of this
/// one.
pub fn invocation(program: &str, file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-d".to_string(),
            "-k".to_string(),
            "-v".to_string(),
            "-v".to_string(),
            file.to_string(),
        ],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C080: matches UniExtract.au3:2783-2784's
    /// `_RunInTempOutdir($tempoutdir, $lz & ' -d -k -v -v "' & $file & '"',
    /// $tempoutdir, @SW_SHOW, True, True, False)` — same program, same
    /// argument order (including `-v` appearing twice as separate tokens),
    /// the `$tempoutdir` working directory, and the shown window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\lzip.exe",
            r"C:\downloads\archive.tar.lz",
            r"C:\downloads\archive_unpacked\tmp123456",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\lzip.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                "-k".to_string(),
                "-v".to_string(),
                "-v".to_string(),
                r"C:\downloads\archive.tar.lz".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
