//! unzoo (`unzoo.exe`) — `.zoo` Zoo archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_ZOO`
/// (UniExtract.au3:3390-3394) makes: `<program> -x <filename_full>`, run in
/// `tempoutdir` with the window hidden (`@SW_HIDE`).
///
/// Unlike most other extractor invocations in this repo, the source does
/// **not** wrap `$filenamefull` in quotes here — `' -x ' & $filenamefull` is
/// a bare, unquoted concatenation (compare `extract::kgb`'s `' "' & $file &
/// '"'`, which does quote). That's a deliberate observation about the
/// source's quoting behavior, not a bug to normalize away: `filename_full`
/// is passed through as the plain value, with no quote characters added.
/// Since `Invocation::args` entries are already-split tokens rather than a
/// raw shell string, the built `Invocation` ends up the same either way —
/// the note is preserved here purely for parity-review accuracy.
///
/// Scope note: the surrounding `_FileMove($file, $tempoutdir, 8)` (staging
/// the file into `tempoutdir` before running), `_FileMove($tempoutdir &
/// $filenamefull, $file)` (moving it back), and `MoveFiles($tempoutdir,
/// $outdir, False, "", True)` (relocating results to `outdir` afterward)
/// are separate runtime behavior — temp-staging and output-relocation,
/// already tracked as their own capabilities — and are not represented by
/// this function.
pub fn invocation(program: &str, filename_full: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-x".to_string(), filename_full.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C110: matches UniExtract.au3:3390-3394's
    /// `_Run($zoo & ' -x ' & $filenamefull, $tempoutdir, @SW_HIDE)` — same
    /// program, same `-x`/`filenamefull` arguments, same `$tempoutdir`
    /// working directory, same hidden window. The surrounding
    /// `_FileMove`/`MoveFiles` staging calls are out of scope (see the
    /// function's doc comment).
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unzoo.exe",
            "archive.zoo",
            r"C:\downloads\archive_temp",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unzoo.exe");
        assert_eq!(inv.args, vec!["-x".to_string(), "archive.zoo".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_temp");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
