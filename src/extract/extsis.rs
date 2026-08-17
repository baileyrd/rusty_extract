//! extsis (`extsis.exe`) — Symbian OS `.sis`/`.sisx` installers.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_SIS`
/// (UniExtract.au3:3026) makes: `<program> -x -xcsd "<file>" -d
/// "<tempoutdir>"`, run in the temp output directory, window minimized.
///
/// The source precedes this with a QuickBMS test-extract
/// (`PDunSIS.wcx`, C077) and follows it with a move-from-tempoutdir step
/// and bindir/MyDocuments cleanup (UniExtract.au3:3023,3027-3030) — those
/// are separate capabilities (the QuickBMS probe, and the generic
/// post-extraction cleanup utility, C155), not part of this one.
pub fn invocation(program: &str, file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-x".to_string(),
            "-xcsd".to_string(),
            file.to_string(),
            "-d".to_string(),
            tempoutdir.to_string(),
        ],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C096: matches UniExtract.au3:3026's
    /// `_Run($extsis & ' -x -xcsd "' & $file & '" -d "' & $tempoutdir &
    /// '"', $tempoutdir, @SW_MINIMIZE)` — program, argument order, the
    /// `$tempoutdir` working directory, and the minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\extsis.exe",
            r"C:\downloads\app.sis",
            r"C:\downloads\app_unpacked\tmp123456",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\extsis.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                "-xcsd".to_string(),
                r"C:\downloads\app.sis".to_string(),
                "-d".to_string(),
                r"C:\downloads\app_unpacked\tmp123456".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\app_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
