//! helpdeco (`helpdeco.exe`) — Windows `.hlp` help files, with an RTF
//! reconstruction pass.

use super::{Invocation, WindowMode};

/// Builds the invocation `Case $TYPE_HLP`'s primary extraction pass
/// makes (UniExtract.au3:2606): `<program> "<file>"`, run in `outdir`.
/// No `$show_flag` argument is passed at this call site, so `_Run`'s own
/// default (`@SW_MINIMIZE`) applies — the same convention already
/// documented across this crate's other bare `_Run($cmd, $dir)` call
/// sites (e.g. `extract::daa`, `extract::wolf`).
pub fn extract_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Reproduces the size-growth gate between `Case $TYPE_HLP`'s two passes
/// (UniExtract.au3:2607: `If _DirGetSize($outdir, $initdirsize + 1) >
/// $initdirsize Then`): the RTF reconstruction pass only runs if the
/// primary pass actually produced output — `outdir_size_after_first_pass`
/// grew past `initdirsize`, the directory's size before extraction
/// started.
pub fn should_reconstruct_rtf(initdirsize: i64, outdir_size_after_first_pass: i64) -> bool {
    outdir_size_after_first_pass > initdirsize
}

/// Builds the invocation the RTF reconstruction pass makes
/// (UniExtract.au3:2609): `<program> /r /n "<file>"`, run in
/// `tempoutdir` — a separate staging directory from `outdir`, cleaned up
/// by the caller afterward (`DirRemove($tempoutdir, 1)`, real filesystem
/// I/O, out of scope here). Same bare-`_Run` minimized-window default as
/// [`extract_invocation`].
pub fn reconstruct_invocation(program: &str, file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["/r".to_string(), "/n".to_string(), file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds the reconstructed RTF's final destination filename
/// (UniExtract.au3:2610: `$filename & '_' & t('TERM_RECONSTRUCTED') &
/// '.rtf'`), moved there from `<tempoutdir><filename>.rtf` by the
/// caller. `reconstructed_term` stands in for `t('TERM_RECONSTRUCTED')`
/// — the localized term, injected rather than hardcoded, the same
/// dependency-injection convention `outdir::default_output_subfolder`
/// (C138) already uses for its own translated suffix.
pub fn reconstructed_rtf_filename(filename: &str, reconstructed_term: &str) -> String {
    format!("{filename}_{reconstructed_term}.rtf")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C073: the primary-pass invocation
    /// matches UniExtract.au3:2606's `helpdeco.exe "<file>"` call, with
    /// the default minimized window `_Run` applies when no `$show_flag`
    /// is passed.
    #[test]
    fn extract_invocation_matches_source() {
        let inv = extract_invocation(
            r"C:\UniExtract\bin\helpdeco.exe",
            r"C:\downloads\manual.hlp",
            r"C:\downloads\manual_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\helpdeco.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\manual.hlp".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\manual_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn should_reconstruct_rtf_when_outdir_grew() {
        assert!(should_reconstruct_rtf(100, 150));
    }

    #[test]
    fn should_not_reconstruct_rtf_when_outdir_unchanged() {
        assert!(!should_reconstruct_rtf(100, 100));
    }

    /// Parity test for capability C073: the reconstruction-pass
    /// invocation matches UniExtract.au3:2609's `helpdeco.exe /r /n
    /// "<file>"` call, run in the temp staging directory.
    #[test]
    fn reconstruct_invocation_matches_source() {
        let inv = reconstruct_invocation(
            r"C:\UniExtract\bin\helpdeco.exe",
            r"C:\downloads\manual.hlp",
            r"C:\downloads\temp",
        );
        assert_eq!(
            inv.args,
            vec![
                "/r".to_string(),
                "/n".to_string(),
                r"C:\downloads\manual.hlp".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\temp");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn reconstructed_rtf_filename_matches_source_shape() {
        assert_eq!(
            reconstructed_rtf_filename("manual", "reconstructed"),
            "manual_reconstructed.rtf"
        );
    }
}
