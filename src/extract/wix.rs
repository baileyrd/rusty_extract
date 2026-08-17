//! dark / WiX Toolset (`dark.exe`) — WiX MSI-based installers.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_WIX`
/// (UniExtract.au3:3373-3375) makes: `<program> -x "<outdir>" "<file>"`,
/// run in `outdir` with the window minimized.
///
/// The source's preceding `HasNetFramework(4)` call is a precondition
/// check (the .NET Framework version `dark.exe` requires), not part of
/// building this invocation — it's separate runtime behavior, tracked as
/// its own capability, not this row.
pub fn invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-x".to_string(), outdir.to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C107: the constructed invocation matches
    /// UniExtract.au3:3373-3375's `_Run($wix & ' -x "' & $outdir & '" "' &
    /// $file & '"', $outdir, @SW_MINIMIZE, True, True, False)` — same
    /// program, same arguments in order, same `$outdir` working directory,
    /// same minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\dark.exe",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.msi",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\dark.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.msi".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
