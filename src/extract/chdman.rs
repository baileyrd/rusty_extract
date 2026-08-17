//! chdman (`chdman.exe`) — MAME CHD compressed hard disk images.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_CHD`
/// (UniExtract.au3:2441-2442) makes: `<program> extracthd -i "<file>" -o
/// "<outdir>\<filename_stem>.img"`.
///
/// Like `extract::sfark`, the source names the output file explicitly
/// (`filename_stem` is the input file's name without extension, `$filename`
/// in the source) rather than letting the tool pick one.
///
/// Unlike most other extractor cases (including `sfark`), this one runs in
/// `outdir`, not the input file's own directory (`$filedir`) — that's a
/// faithful match to the source's `_Run(..., $outdir)` call, not an
/// inconsistency introduced by this port.
///
/// The source's `_Run` call passes no explicit show-flag argument, so
/// `_Run`'s own default applies (`Func _Run($f, $sWorkingDir = $outdir,
/// $show_flag = @SW_MINIMIZE, ...)`) — the window is minimized, not because
/// this port guessed a default, but because that omission *is* the source
/// selecting `@SW_MINIMIZE`.
pub fn invocation(program: &str, outdir: &str, file: &str, filename_stem: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "extracthd".to_string(),
            "-i".to_string(),
            file.to_string(),
            "-o".to_string(),
            format!("{outdir}\\{filename_stem}.img"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C065: matches UniExtract.au3:2441-2442's
    /// `_Run($chd & ' extracthd -i "' & $file & '" -o "' & $outdir & '\' &
    /// $filename & '.img"', $outdir)` — program, argument order (including
    /// the explicit `.img` output path), the `$outdir` working directory
    /// (not `$filedir`, unlike most other cases), and the window mode. The
    /// source's `_Run` call omits the third positional (show-flag)
    /// argument entirely, so `_Run`'s own default of `@SW_MINIMIZE` applies
    /// — `WindowMode::Minimized` here reflects that default, not a guess.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\chdman.exe",
            r"C:\images\disk_unpacked",
            r"C:\images\disk.chd",
            "disk",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\chdman.exe");
        assert_eq!(
            inv.args,
            vec![
                "extracthd".to_string(),
                "-i".to_string(),
                r"C:\images\disk.chd".to_string(),
                "-o".to_string(),
                r"C:\images\disk_unpacked\disk.img".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\images\disk_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
