//! Netopsystems FEAD self-extraction (native switches).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_FEAD`
/// (UniExtract.au3:2530-2536) makes: the archive is itself a
/// self-extracting FEAD executable, run directly as `<file> /s -nos_ne
/// -nos_o<tempoutdir>\`, run in `filedir` with a normally shown window.
///
/// `-nos_o<tempoutdir>\` is a single concatenated-flag argument token
/// (flag directly joined to the destination, including a trailing literal
/// backslash the source's string concatenation adds), the same pattern
/// already established in `extract::bcm`/`extract::lzop`/`extract::unreal`.
///
/// The source runs this via `ShellExecuteWait($file, $sParameters,
/// $filedir)` — AutoIt's `ShellExecuteWait`, not `_Run`/`Run`/`RunWait` —
/// needed (per the sibling `$TYPE_AI` case's own comment) so the OS can
/// raise a UAC elevation prompt. Its `$iShowFlag` parameter defaults to
/// `@SW_SHOWNORMAL` when omitted, the same as every other
/// unspecified-show-flag call in this crate, mapped to `WindowMode::Show`.
///
/// **Not modeled here:** the preceding `Warn_Execute(...)` confirmation
/// gate (`warnexecute` preference, C023, deferred GUI subsystem D001);
/// the trailing `FileSetAttrib`/`MoveFiles`/`DirRemove` calls that move
/// `tempoutdir`'s contents into `outdir` and clean up afterward — all
/// separate runtime behavior, not part of building this invocation.
pub fn invocation(file: &str, tempoutdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec![
            "/s".to_string(),
            "-nos_ne".to_string(),
            format!("-nos_o{tempoutdir}\\"),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C117: the constructed invocation matches
    /// UniExtract.au3:2530-2536's effective `<file> /s -nos_ne
    /// -nos_o<tempoutdir>\` call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\installer.exe",
            r"C:\downloads\installer_temp",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![
                "/s".to_string(),
                "-nos_ne".to_string(),
                r"-nos_oC:\downloads\installer_temp\".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
