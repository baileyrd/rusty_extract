//! uif2iso (`uif2iso.exe`) — MagicISO `.uif` images (stage-1 conversion to ISO).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_UIF`
/// (UniExtract.au3:3161-3163) makes: `<program> "<file>" "<outdir>\<filename>"`,
/// run in the input file's own directory (`$filedir` in the source — not
/// `outdir`) with the window shown normally.
///
/// Like `extract::rpa`, the source's `_Run` call passes an explicit `True`
/// as its third positional argument (`$show_flag`) rather than omitting it
/// — per the `WindowMode` mapping documented on that enum, the literal
/// `True`/`1` is AutoIt's `@SW_SHOWNORMAL`, so this maps to
/// `WindowMode::Show`, not `Hidden`/`Minimized`.
///
/// The source line also calls `_CreateTrayMessageBox(...)` immediately
/// before `_Run` to post a "stage 1" progress notification. That's UI
/// behavior belonging to the deferred GUI subsystem (manifest row D001),
/// not part of the invocation this capability builds — out of scope here.
pub fn invocation(
    program: &str,
    file: &str,
    outdir: &str,
    filename: &str,
    file_dir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), format!("{outdir}\\{filename}")],
        working_dir: file_dir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C102: matches UniExtract.au3:3161-3163's
    /// `_Run($uif & ' "' & $file & '" "' & $outdir & "\" & $filename & '"',
    /// $filedir, True, True, True)` — program, argument order (the input
    /// file followed by the explicit output path built from `outdir` and
    /// `filename`), the `$filedir` working directory, and the
    /// `True` → `WindowMode::Show` window mapping (same precedent as
    /// `extract::rpa`). `_CreateTrayMessageBox` is excluded — out of scope,
    /// manifest row D001.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\uif2iso.exe",
            r"C:\downloads\image.uif",
            r"C:\downloads\image_unpacked",
            "image.iso",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\uif2iso.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\image.uif".to_string(),
                r"C:\downloads\image_unpacked\image.iso".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
