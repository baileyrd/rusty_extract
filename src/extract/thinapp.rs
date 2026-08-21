//! ThinApp/Thinstall (`Extractor.exe`) — virtualized executables, unwrapped
//! by relocating the input into a temp directory first.
//!
//! ```autoit
//! Case $TYPE_THINSTALL
//!     DirCreate($tempoutdir)
//!     _FileMove($file, $tempoutdir & $filenamefull)
//!     _Run($thinstall & ' "' & $tempoutdir & $filenamefull & '"', $tempoutdir, @SW_HIDE, True, True, False)
//!     MoveFiles($tempoutdir, $outdir, False, "", True, True)
//!     DirRemove($tempoutdir, 1)
//! ```
//!
//! **Scope — invocation only.** `DirCreate`, `_FileMove` (relocating the
//! input into `tempoutdir` before the tool runs — the tool operates on a
//! copy in a scratch directory, not the original input in place),
//! `MoveFiles` (collecting the tool's output back into `outdir`), and
//! `DirRemove` (scratch-directory cleanup) are all real filesystem I/O,
//! left to the caller — the same split `extract::raiu` already uses for
//! its own temp-directory unwrap step.

use super::{Invocation, WindowMode};

/// Builds the relocated input path `Case $TYPE_THINSTALL` computes
/// (UniExtract.au3:3107-3128): `<tempoutdir><filename_full>` — where the
/// tool is pointed once `_FileMove` has placed the input there.
pub fn relocated_file_path(tempoutdir: &str, filename_full: &str) -> String {
    format!("{tempoutdir}{filename_full}")
}

/// Builds the invocation `Case $TYPE_THINSTALL` makes on the relocated
/// input: `<program> "<tempoutdir><filename_full>"`, run in `tempoutdir`
/// with the window hidden.
pub fn invocation(program: &str, tempoutdir: &str, filename_full: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![relocated_file_path(tempoutdir, filename_full)],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocated_file_path_matches_source_shape() {
        assert_eq!(
            relocated_file_path(r"C:\temp\", "game_installer.exe"),
            r"C:\temp\game_installer.exe"
        );
    }

    /// Parity test for capability C099: the constructed invocation matches
    /// UniExtract.au3:3107-3128's effective `Extractor.exe
    /// "<tempoutdir><filename_full>"` call — program, the single relocated
    /// path as the argument, `tempoutdir` as the working directory, and
    /// the hidden window `@SW_HIDE` selects.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\Extractor.exe",
            r"C:\temp\uextmp\",
            "game_installer.exe",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\Extractor.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\temp\uextmp\game_installer.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\temp\uextmp\");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
