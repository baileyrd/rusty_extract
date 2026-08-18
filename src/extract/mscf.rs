//! MSCF Cab installer (wraps `7z.exe` + Exeinfo-PE GUI rip).

use super::{Invocation, WindowMode};

/// Builds the per-file 7-Zip extraction invocation UniExtract2's `Case
/// $TYPE_MSCF` (UniExtract.au3:2827) makes for each `.cab` file
/// `RipExeInfo`'s GUI automation extracted from the MSCF installer:
/// `<program> x "<cab_file>"`, run in `tempoutdir` with the window hidden.
///
/// **Not modeled here:** the recursive `extract($TYPE_7Z, -1, "", False,
/// True)` dispatch that runs first (composite/recursive dispatch,
/// capability C054, not yet ported); `RipExeInfo`'s Exeinfo-PE GUI
/// automation (a scripted keystroke sequence, `"{DOWN}{DOWN}..."`) that
/// rips the `.cab` files out of the installer in the first place —
/// deferred GUI subsystem, manifest row D001, matching this row's own
/// "Exeinfo-PE GUI rip" description; the surrounding `MoveFiles`/
/// `DirRemove`/`Cleanup` staging; and the recursive `.cab`-file listing
/// (`_FileListToArrayRec`) that decides which files to run this
/// invocation against. All separate runtime behavior, not part of
/// building this one invocation.
pub fn cab_extract_invocation(program: &str, cab_file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), cab_file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C120: the constructed invocation
    /// matches UniExtract.au3:2827's effective `7z.exe x "<cab_file>"`
    /// call.
    #[test]
    fn cab_extract_invocation_matches_source() {
        let inv = cab_extract_invocation(
            r"C:\UniExtract\bin\7z.exe",
            r"C:\Temp\mscf_tmp\data1.cab",
            r"C:\Temp\mscf_tmp",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\Temp\mscf_tmp\data1.cab".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\Temp\mscf_tmp");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
