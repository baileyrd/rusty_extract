//! Windows `msiexec.exe` — MSI administrative-install fallback.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `$TYPE_MSI` fallback candidate
/// list makes for its "Administrative install" choice
/// (UniExtract.au3:2882-2883): `msiexec.exe /a "<file>" /qb
/// TARGETDIR="<outdir>"`, run in `filedir` with the window shown.
///
/// The source wraps the command string in `Warn_Execute(...)` before
/// passing it to `RunWait` — a gate on the `warnexecute` preference
/// (already ported as its own capability) that either returns the
/// command string unchanged or shows a confirmation dialog (deferred
/// GUI subsystem, manifest row D001) and terminates silently if the
/// user doesn't continue. Either way, the command string itself is
/// unaffected; that gate is a separate concern from building this
/// invocation.
///
/// The `TARGETDIR="<outdir>"` segment has its quotes only around the
/// `outdir` value, matching this crate's `Invocation` model: after
/// standard Windows command-line quote parsing, `msiexec.exe` receives
/// `TARGETDIR=<outdir>` as one argument.
///
/// **Scope — invocation only, one candidate of several.** This is only
/// reached through `$TYPE_MSI`'s GUI candidate-list fallback (see
/// `extract::lessmsi`'s doc comment for the full chain) — composite,
/// conditional dispatch, not registered in
/// `extract::dispatch::HARDCODED_CASES`.
pub fn invocation(file: &str, outdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: "msiexec.exe".to_string(),
        args: vec![
            "/a".to_string(),
            file.to_string(),
            "/qb".to_string(),
            format!("TARGETDIR={outdir}"),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C087: the constructed invocation
    /// matches UniExtract.au3:2882-2883's effective `msiexec.exe /a
    /// "<file>" /qb TARGETDIR="<outdir>"` call — program, args, the
    /// `filedir` working directory, and the shown window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\installer.msi",
            r"C:\downloads\installer",
            r"C:\downloads",
        );
        assert_eq!(inv.program, "msiexec.exe");
        assert_eq!(
            inv.args,
            vec![
                "/a".to_string(),
                r"C:\downloads\installer.msi".to_string(),
                "/qb".to_string(),
                r"TARGETDIR=C:\downloads\installer".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
