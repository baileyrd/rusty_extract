//! sfarkxtc (`sfarkxtc.exe`) — sfArk compressed SoundFont.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_SFARK`
/// (UniExtract.au3:3019-3020) makes: `<program> "<file>" "<outdir>\<filename_stem>.sf2"`,
/// run in the input file's own directory (`$filedir` in the source — not
/// `outdir`, unlike most extractor cases) with the window shown normally.
///
/// Unlike every other extractor ported so far, the source names the output
/// file explicitly rather than letting the tool pick — `filename_stem` is
/// the input file's name without extension (`$filename` in the source),
/// used to build the `.sf2` output path.
pub fn invocation(
    program: &str,
    file_dir: &str,
    file: &str,
    outdir: &str,
    filename_stem: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), format!("{outdir}\\{filename_stem}.sf2")],
        working_dir: file_dir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C095: matches UniExtract.au3:3019-3020's
    /// `_Run($sfark & ' "' & $file & '" "' & $outdir & '\' & $filename &
    /// '.sf2"', $filedir, @SW_SHOW)` — program, argument order (including
    /// the explicit `.sf2` output path), the `$filedir` working directory,
    /// and the non-hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\sfarkxtc.exe",
            r"C:\downloads",
            r"C:\downloads\MySoundFont.sfArk",
            r"C:\downloads\MySoundFont_unpacked",
            "MySoundFont",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\sfarkxtc.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\MySoundFont.sfArk".to_string(),
                r"C:\downloads\MySoundFont_unpacked\MySoundFont.sf2".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
