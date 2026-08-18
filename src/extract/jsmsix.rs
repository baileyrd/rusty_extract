//! jsMSIx (`jsMSIx.exe`) — MSI fallback method 1.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `$TYPE_MSI` fallback candidate
/// list makes for its "jsMSI Unpacker" choice (UniExtract.au3:2858):
/// `<program> "<file>|<outdir>"`, run in `filedir` with the window
/// hidden.
///
/// The source's literal command-line string is `' "' & $file & '"|"'
/// & $outdir & '"'` — two quoted segments joined by a bare `|` with no
/// whitespace between any of the three pieces. Standard Windows
/// command-line tokenization only splits on whitespace, so after
/// dequoting this collapses into a *single* argument:
/// `<file>|<outdir>` — jsMSIx's own file/output-path delimiter
/// convention, not a shell pipe. This function's one-element `args`
/// reflects that effective single token.
///
/// **Scope — invocation only, one candidate of several.** This is only
/// reached through `$TYPE_MSI`'s GUI candidate-list fallback (see
/// `extract::lessmsi`'s doc comment for the full chain) — composite,
/// conditional dispatch, not registered in
/// `extract::dispatch::HARDCODED_CASES`. Also out of scope: reading
/// `<outdir>\MSI Unpack.log` and the follow-up `Cleanup("*.cab")` call
/// (UniExtract.au3:2859-2860), both real filesystem I/O.
pub fn invocation(program: &str, file: &str, outdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![format!("{file}|{outdir}")],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C085: the constructed invocation
    /// matches UniExtract.au3:2858's effective `jsMSIx.exe
    /// "<file>|<outdir>"` call — program, the single collapsed
    /// pipe-joined argument, the `filedir` working directory, and the
    /// hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\jsMSIx.exe",
            r"C:\downloads\installer.msi",
            r"C:\downloads\installer",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\jsMSIx.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\installer.msi|C:\downloads\installer".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
