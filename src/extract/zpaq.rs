//! zpaq (`zpaq.exe`) — `.zpaq` ZPAQ archives.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_ZPAQ`
/// (UniExtract.au3:3396-3399) makes: `<program> x "<file>" -to "<outdir>"`,
/// run in `outdir` with the window shown.
///
/// The source's comment on this case — "ZPaq uses a different executable
/// for Windows XP, so a definition file cannot be used" — explains why
/// this capability is a hardcoded Rust module rather than a `def/*.ini`
/// plugin row like most other simple extractors: it's context for the
/// shape of the port, not behavior this port needs to replicate (Windows
/// XP executable selection is out of scope).
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "x".to_string(),
            file.to_string(),
            "-to".to_string(),
            outdir.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C111: the constructed invocation matches
    /// UniExtract.au3:3396-3399's `_Run($zpaq & ' x "' & $file & '" -to "'
    /// & $outdir & '"', $outdir, @SW_SHOW, True, True, False)` — same
    /// program, same argument order, same `outdir` working directory, same
    /// shown window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\zpaq.exe",
            r"C:\downloads\archive.zpaq",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\zpaq.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\downloads\archive.zpaq".to_string(),
                "-to".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
