//! Advanced Installer self-extraction (native `/extract:` switch, no
//! external binary).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_AI`
/// (UniExtract.au3:2385-2390) makes: the archive is itself a
/// self-extracting Advanced Installer executable, run directly as
/// `<file> /extract:<outdir>`, run in `outdir` with a normally shown
/// window.
///
/// `/extract:<outdir>` is a single concatenated-flag argument token
/// (flag directly joined to the destination, no space), the same pattern
/// already established in `extract::bcm`/`extract::lzop`/`extract::unreal`.
///
/// The source runs this via `ShellExecute($file, ' /extract:"' & $outdir
/// & '"', $outdir)` — AutoIt's `ShellExecute`, not `_Run`/`Run`/
/// `RunWait` — needed (per the source's own comment on this exact case)
/// so the OS can raise a UAC elevation prompt, which plain `Run` can't
/// trigger. Its `$iShowFlag` parameter defaults to `@SW_SHOWNORMAL` when
/// omitted, the same as every other unspecified-show-flag call in this
/// crate, mapped to `WindowMode::Show`.
///
/// **Not modeled here:** the preceding `Warn_Execute(...)` confirmation
/// gate (`warnexecute` preference, C023, deferred GUI subsystem D001);
/// the trailing `ProcessWait`/`ProcessWaitClose` calls that wait for the
/// self-extractor to finish — separate runtime behavior, not part of
/// building this invocation.
pub fn invocation(file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec![format!("/extract:{outdir}")],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C115: the constructed invocation
    /// matches UniExtract.au3:2385-2390's effective `<file>
    /// /extract:<outdir>` call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\installer.exe",
            r"C:\downloads\installer_unpacked",
        );
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![r"/extract:C:\downloads\installer_unpacked".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
