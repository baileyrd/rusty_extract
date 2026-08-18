//! Excelsior Installer self-extraction (native switches).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_EI`
/// (UniExtract.au3:2514-2516) makes: the archive is itself a
/// self-extracting Excelsior Installer executable, run directly as
/// `<file> /batch /no-reg /no-postinstall /dest "<outdir>"`, run in
/// `outdir` with a normally shown window.
///
/// The source runs this via `ShellExecuteWait($file, '/batch /no-reg
/// /no-postinstall /dest "' & $outdir & '"', $outdir)` — AutoIt's
/// `ShellExecuteWait`, not `_Run`/`Run`/`RunWait` — needed here (per the
/// sibling `$TYPE_AI` case's own comment) so the OS can raise a UAC
/// elevation prompt, which plain `Run` can't trigger. Its `$iShowFlag`
/// parameter defaults to `@SW_SHOWNORMAL` when omitted, exactly as it is
/// here, so this maps to `WindowMode::Show` the same as every other
/// unspecified-show-flag call in this crate — `ShellExecuteWait`'s
/// `($sFilePath, $sParameters, $sWorkingDir)` shape otherwise matches
/// this crate's `Invocation` (program, args, working dir) directly.
///
/// **Not modeled here:** the preceding `Warn_Execute(...)` call — the
/// "you're about to run an executable, continue?" confirmation gate
/// (`warnexecute` preference, C023) — a deferred-GUI-subsystem concern
/// (manifest row D001) that either passes the command through unchanged
/// or aborts the run entirely; this function reproduces only the command
/// it passes through, the same as `extract::expand::cab_self_extract_invocation`.
pub fn invocation(file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec![
            "/batch".to_string(),
            "/no-reg".to_string(),
            "/no-postinstall".to_string(),
            "/dest".to_string(),
            outdir.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C116: the constructed invocation matches
    /// UniExtract.au3:2514-2516's effective `<file> /batch /no-reg
    /// /no-postinstall /dest "<outdir>"` call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\installer.exe",
            r"C:\downloads\installer_unpacked",
        );
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![
                "/batch".to_string(),
                "/no-reg".to_string(),
                "/no-postinstall".to_string(),
                "/dest".to_string(),
                r"C:\downloads\installer_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
