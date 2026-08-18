//! ci-extractor (`ci-extractor.exe`) — CreateInstall installers,
//! GUI-automated.

use super::{Invocation, WindowMode};

/// Builds the control-file content UniExtract2's `Case $TYPE_CI`
/// (UniExtract.au3:2461-2463) writes before invoking `ci-extractor.exe`:
/// `1\n<file>\n<outdir>\n3\n1` — matching `"1" & @LF & $file & @LF &
/// $outdir & @LF & "3" & @LF & "1"` exactly (AutoIt's `@LF` is a bare line
/// feed, not `\r\n`).
///
/// `ci-extractor.exe` reads this as a scripted-answer file for its
/// interactive extraction wizard; this port doesn't need to know the
/// meaning of each line beyond reproducing the exact bytes the source
/// writes.
pub fn control_file_content(file: &str, outdir: &str) -> String {
    format!("1\n{file}\n{outdir}\n3\n1")
}

/// Builds the invocation UniExtract2's `Case $TYPE_CI`
/// (UniExtract.au3:2465) makes: `<program> <tempfile>`, run in `outdir`
/// with the window shown normally.
///
/// `tempfile` is the caller-resolved path to the control file
/// [`control_file_content`] must already have been written to — the
/// source computes it as `@TempDir & "\ci.txt"`, a real OS temp-directory
/// lookup this pure function doesn't perform itself.
///
/// **Not modeled here:** the preceding `HasPlugin($ci)` precondition
/// check; the `WinWait`/`ControlClick` GUI automation that clicks
/// "Finish" on `ci-extractor.exe`'s wizard, `ProcessClose`, the temp-file
/// cleanup (`FileDelete`), and the `terminate($STATUS_SILENT)` call that
/// follows — GUI automation is out of scope (deferred GUI subsystem,
/// manifest row D001), and the rest is separate runtime behavior.
pub fn invocation(program: &str, tempfile: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![tempfile.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C066: the control file's content matches
    /// UniExtract.au3:2461-2463's exact `@LF`-joined lines.
    #[test]
    fn control_file_content_matches_source() {
        assert_eq!(
            control_file_content(r"C:\downloads\installer.exe", r"C:\downloads\unpacked"),
            "1\nC:\\downloads\\installer.exe\nC:\\downloads\\unpacked\n3\n1"
        );
    }

    /// Parity test for capability C066: the constructed invocation matches
    /// UniExtract.au3:2465's effective `ci-extractor.exe <tempfile>` call.
    #[test]
    fn invocation_matches_source() {
        let inv = invocation(
            r"C:\UniExtract\bin\ci-extractor.exe",
            r"C:\Temp\ci.txt",
            r"C:\downloads\unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\ci-extractor.exe");
        assert_eq!(inv.args, vec![r"C:\Temp\ci.txt".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
