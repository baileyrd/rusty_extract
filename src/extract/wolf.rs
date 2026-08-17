//! WolfDec (`WolfDec.exe`) — Wolf RPG Editor game archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_WOLF`
/// (UniExtract.au3:3377-3382) makes: `<program> "<file>"`, run in `outdir`
/// with the window minimized.
///
/// The source calls `_RunInTempOutdir($tempoutdir, $wolf & ' ' &
/// Quote($file), $outdir, @SW_MINIMIZE, True, True, False)` — unlike
/// `extract::lzip`/`extract::isz`, which also call `_RunInTempOutdir` but
/// pass `$tempoutdir` as both the staging argument *and* the working
/// directory, this case passes `$outdir` as the explicit third positional
/// argument (`$sWorkingDir`). So the working directory for this invocation
/// is `outdir`, not `tempoutdir` — `tempoutdir` is only the staging
/// directory for `_RunInTempOutdir`'s own temp-then-move orchestration,
/// which is separate runtime behavior, not part of this invocation.
///
/// `Quote($file)` produces a quoted command-line token equivalent to `'"' &
/// $file & '"'` — in this repo's `Invocation` model (already-split argument
/// tokens, not a shell string) that's just a single argument equal to the
/// plain `file` value, same as `extract::kgb`'s `"' & $file & '"'` pattern.
///
/// Out of scope for this row (separate runtime-behavior capabilities, not
/// part of building this invocation): `HasPlugin($wolf)`; the preceding
/// `_CreateTrayMessageBox(...)` UI progress notification, part of the
/// deferred GUI subsystem (manifest row D001); the `_Sleep(1000,
/// "CLEANING_UP")` pause; and the trailing `MoveFiles(...)` call that
/// relocates extracted output.
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

    /// Parity test for capability C108: matches UniExtract.au3:3377-3382's
    /// `_RunInTempOutdir($tempoutdir, $wolf & ' ' & Quote($file), $outdir,
    /// @SW_MINIMIZE, True, True, False)` — same program, same argument, the
    /// `$outdir` working directory (not `$tempoutdir`), and the minimized
    /// window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\WolfDec.exe",
            r"C:\downloads\archive.wolf",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\WolfDec.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.wolf".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
