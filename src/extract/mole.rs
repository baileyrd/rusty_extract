//! demoleition / MoleBox (`demoleition.exe`) — MoleBox-packaged executables.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_MOLE`
/// (UniExtract.au3:2792-2811) makes: `<program> /nogui "<file>"`, run in
/// `outdir` with the window hidden.
///
/// The source calls `_RunInTempOutdir($tempoutdir, $mole & ' /nogui "' &
/// $file & '"', $outdir, @SW_HIDE, True, False, False)` — unlike
/// `extract::lzip`/`extract::isz`, which also call `_RunInTempOutdir` but
/// pass `$tempoutdir` as both the staging argument *and* the working
/// directory, this case passes `$outdir` as the explicit third positional
/// argument (`$sWorkingDir`). So the working directory for this invocation
/// is `outdir`, not `tempoutdir` — `tempoutdir` is only the staging
/// directory for `_RunInTempOutdir`'s own temp-then-move orchestration,
/// which is separate runtime behavior, not part of this invocation. Same
/// quirk, same reasoning as `extract::wolf`'s precedent (see its doc
/// comment).
///
/// `' /nogui "' & $file & '"'` decomposes into two already-split argument
/// tokens in this repo's `Invocation` model (a command-line string, not a
/// shell string): the bare flag `/nogui` and a single argument equal to the
/// plain `file` value (the surrounding quotes are just command-line
/// quoting, same as `extract::kgb`'s `"' & $file & '"'` pattern).
///
/// Out of scope for this row (separate runtime-behavior capabilities, not
/// part of building this invocation): the trailing file-move logic (moving
/// `<filename>_unpacked.exe` and the `_extracted` directory into place);
/// reading and deleting the `!unpacker.log` file; and evaluating that log's
/// contents against `'[x] Not a Molebox or unknown version'` /
/// `'[i] Finished! Have a nice day!'` to determine `$success`.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["/nogui".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C083: matches UniExtract.au3:2792-2811's
    /// `_RunInTempOutdir($tempoutdir, $mole & ' /nogui "' & $file & '"',
    /// $outdir, @SW_HIDE, True, False, False)` — same program, same
    /// arguments, the `$outdir` working directory (not `$tempoutdir`), and
    /// the hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\demoleition.exe",
            r"C:\downloads\archive.exe",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\demoleition.exe");
        assert_eq!(
            inv.args,
            vec![
                "/nogui".to_string(),
                r"C:\downloads\archive.exe".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
