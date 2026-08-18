//! RAIU (`RAIU.exe`) — Reflexive Arcade Installer wrapper, chains into
//! Inno Setup extraction (C074).
//!
//! **Scope — invocation only.** After this invocation unwraps the RAIU
//! shell, `Case $TYPE_RAI` recursively re-invokes `extract($TYPE_INNO,
//! ...)` on the unwrapped file (UniExtract.au3:2999) — the recursive
//! dispatch mechanism itself is C181, out of scope here — and then
//! cleans up the temp file and directory (`Cleanup`, `DirRemove`), real
//! filesystem I/O left to the caller.

use super::{Invocation, WindowMode};

/// Builds the intermediate unpacked-executable path `Case $TYPE_RAI`
/// computes (UniExtract.au3:2996): `<tempoutdir><filename>_<term>.exe`,
/// where `unpacked_term` stands in for `t('TERM_UNPACKED')` — the
/// localized term, injected rather than hardcoded, the same
/// dependency-injection convention `outdir::default_output_subfolder`
/// (C138) and `extract::helpdeco::reconstructed_rtf_filename` (C073)
/// already use for their own translated suffixes.
pub fn intermediate_file_path(tempoutdir: &str, filename: &str, unpacked_term: &str) -> String {
    format!("{tempoutdir}{filename}_{unpacked_term}.exe")
}

/// Builds the invocation `Case $TYPE_RAI` makes to unwrap the Reflexive
/// Arcade Installer shell around its embedded Inno Setup installer
/// (UniExtract.au3:2997): `<program> "<file>" "<tmp>"`, run in
/// `filedir`. No `$show_flag` argument is passed at this call site, so
/// `_Run`'s own default (`@SW_MINIMIZE`) applies — the same convention
/// documented across this crate's other bare `_Run($cmd, $dir)` call
/// sites (e.g. `extract::daa`, `extract::helpdeco`).
pub fn invocation(program: &str, file: &str, tmp: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), tmp.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intermediate_file_path_matches_source_shape() {
        assert_eq!(
            intermediate_file_path(r"C:\temp\", "game_installer", "unpacked"),
            r"C:\temp\game_installer_unpacked.exe"
        );
    }

    /// Parity test for capability C091: the constructed invocation
    /// matches UniExtract.au3:2997's `RAIU.exe "<file>" "<tmp>"` call —
    /// program, args, `filedir` as the working directory, and the
    /// minimized window `_Run`'s default applies.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\RAIU.exe",
            r"C:\downloads\game_installer.exe",
            r"C:\temp\game_installer_unpacked.exe",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\RAIU.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\game_installer.exe".to_string(),
                r"C:\temp\game_installer_unpacked.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
