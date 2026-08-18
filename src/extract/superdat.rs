//! SuperDAT Updater self-extraction (native switches).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_SUPERDAT`
/// (UniExtract.au3:3038-3043) makes: the archive is itself a
/// self-extracting SuperDAT Updater executable, run directly as `<file>
/// /LOGFILE "<outdir>\SuperDAT.log" /e "<outdir>"`, run in `outdir` with a
/// normally shown window.
///
/// The source runs this via `ShellExecuteWait($file, $sParameters,
/// $outdir)` — AutoIt's `ShellExecuteWait`, not `_Run`/`Run`/`RunWait` —
/// needed (per the sibling `$TYPE_AI` case's own comment) so the OS can
/// raise a UAC elevation prompt. Its `$iShowFlag` parameter defaults to
/// `@SW_SHOWNORMAL` when omitted here, mapped to `WindowMode::Show`, the
/// same as every other unspecified-show-flag call in this crate.
///
/// **Not modeled here:** the preceding `Warn_Execute(...)` confirmation
/// gate (`warnexecute` preference, C023, deferred GUI subsystem D001);
/// the trailing `_FileRead($sPath, True)` call that reads the log file
/// back in — separate runtime behavior, not part of building this
/// invocation.
pub fn invocation(file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec![
            "/LOGFILE".to_string(),
            format!("{outdir}\\SuperDAT.log"),
            "/e".to_string(),
            outdir.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C118: the constructed invocation matches
    /// UniExtract.au3:3038-3043's effective `<file> /LOGFILE
    /// "<outdir>\SuperDAT.log" /e "<outdir>"` call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\updater.exe",
            r"C:\downloads\updater_unpacked",
        );
        assert_eq!(inv.program, r"C:\downloads\updater.exe");
        assert_eq!(
            inv.args,
            vec![
                "/LOGFILE".to_string(),
                r"C:\downloads\updater_unpacked\SuperDAT.log".to_string(),
                "/e".to_string(),
                r"C:\downloads\updater_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\updater_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
