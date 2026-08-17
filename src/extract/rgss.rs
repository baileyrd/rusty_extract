//! RGSS Decryptor (`RgssDecrypter.exe`) — RPG Maker RGSS(2/3)A archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_RGSS`
/// (UniExtract.au3:3009-3011) makes: `<program> -p -o="<outdir>" "<file>"`,
/// run in `outdir` with the window hidden.
///
/// The source also calls `HasNetFramework(2)` before running this (RGSS
/// Decryptor requires .NET Framework 2.0) — that's a separate environment
/// precondition, not part of the invocation itself, so it isn't represented
/// here.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-p".to_string(), format!("-o={outdir}"), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C093: the constructed invocation matches
    /// UniExtract.au3:3009-3011's `_Run($rgss & ' -p -o="' & $outdir & '"
    /// "' & $file & '"', $outdir, @SW_HIDE)` — same program, same argument
    /// order, same working directory, same hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\RgssDecrypter.exe",
            r"C:\games\Game.rgss3a",
            r"C:\games\Game_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\RgssDecrypter.exe");
        assert_eq!(
            inv.args,
            vec![
                "-p".to_string(),
                r"-o=C:\games\Game_unpacked".to_string(),
                r"C:\games\Game.rgss3a".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\games\Game_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
