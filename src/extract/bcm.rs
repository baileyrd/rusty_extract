//! BCM (`bcm.exe`) — BCM-compressed files.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_BCM`
/// (UniExtract.au3:2418-2419) makes: `<program> -d "<file>" "<outdir>\<filename_stem>"`,
/// run in the input file's own directory (`$filedir` in the source — not
/// `outdir`) with the window hidden.
///
/// Like `extract::sfark`, the source names the output path explicitly
/// rather than letting the tool pick — `filename_stem` is the input file's
/// name without extension, standing in for the source's
/// `GetFileName()` (UniExtract.au3:896-898) call in the common case; that
/// function's separate Unicode-handling branch is a different, already
/// tracked capability and not represented here.
pub fn invocation(
    program: &str,
    file_dir: &str,
    file: &str,
    outdir: &str,
    filename_stem: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-d".to_string(),
            file.to_string(),
            format!("{outdir}\\{filename_stem}"),
        ],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C062: matches UniExtract.au3:2418-2419's
    /// `_Run($bcm & ' -d "' & $file & '" "' & $outdir & '\' & GetFileName()
    /// & '"', $filedir, @SW_HIDE, True, True, False)` — program, argument
    /// order (including the explicit output path built from the filename
    /// stem), the `$filedir` working directory, and the hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\bcm.exe",
            r"C:\downloads",
            r"C:\downloads\archive.bcm",
            r"C:\downloads\archive_unpacked",
            "archive",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\bcm.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                r"C:\downloads\archive.bcm".to_string(),
                r"C:\downloads\archive_unpacked\archive".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
