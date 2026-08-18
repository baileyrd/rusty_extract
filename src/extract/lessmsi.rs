//! lessmsi (`lessmsi.exe`) — Windows Installer `.msi`, primary extractor.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_MSI` tries first
/// (UniExtract.au3:2843-2845): `<program> x "<file>" "<outdir>\"`, run
/// in `outdir` with the window hidden.
///
/// **Scope — invocation only, one method of several.** `$TYPE_MSI`'s
/// full source behavior is a fallback chain: lessmsi is tried first,
/// and only if it fails (or .NET isn't available) does the source
/// present a GUI candidate list among jsMSI Unpacker (C085), MsiX
/// (C086), an MSI Total Commander plugin path, and an administrative
/// `msiexec.exe` install (C087) — a composite/conditional dispatch case
/// like C075's InstallShield chain, not a single unconditional
/// invocation, so this type isn't registered in
/// `extract::dispatch::HARDCODED_CASES`. Also out of scope: the
/// post-extraction `MoveFiles($outdir & "\SourceDir", $outdir, ...)`
/// step that flattens lessmsi's `SourceDir` output subfolder up into
/// `outdir` directly, and the `DirGetSize($outdir) == $initdirsize`
/// success/failure check right after it — both real filesystem I/O,
/// the caller's job.
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), file.to_string(), format!("{outdir}\\")],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C084: the constructed invocation
    /// matches UniExtract.au3:2843-2845's `lessmsi.exe x "<file>"
    /// "<outdir>\"` call — program, args, `outdir` as both the working
    /// directory and the trailing destination argument, and the hidden
    /// window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\lessmsi.exe",
            r"C:\downloads\installer.msi",
            r"C:\downloads\installer",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\lessmsi.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\downloads\installer.msi".to_string(),
                r"C:\downloads\installer\".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
