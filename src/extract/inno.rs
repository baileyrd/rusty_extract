//! innounp + innoextract (`innounp.exe`, `innoextract.exe`) — Inno Setup
//! installers, GOG installers; primary/fallback pair.

use super::{Invocation, WindowMode};

/// Builds the primary invocation `Case $TYPE_INNO` makes
/// (UniExtract.au3:2616): `<program> -x -m -a "<file>"`, run in
/// `outdir`. No `$show_flag` argument is passed at this call site, so
/// `_Run`'s own default (`@SW_MINIMIZE`) applies — the same convention
/// documented across this crate's other bare `_Run($cmd, $dir)` call
/// sites (e.g. `extract::daa`, `extract::raiu`).
pub fn unnp_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-x".to_string(),
            "-m".to_string(),
            "-a".to_string(),
            file.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds the fallback invocation (UniExtract.au3:2649): `<program> -e
/// --progress=1 --collisions rename -d "<outdir>" "<file>"`, run in
/// `filedir`. Same bare-`_Run` minimized-window default as
/// [`unnp_invocation`].
pub fn innoextract_invocation(
    program: &str,
    file: &str,
    outdir: &str,
    filedir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-e".to_string(),
            "--progress=1".to_string(),
            "--collisions".to_string(),
            "rename".to_string(),
            "-d".to_string(),
            outdir.to_string(),
            file.to_string(),
        ],
        working_dir: filedir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Reproduces the fallback gate between the two passes
/// (UniExtract.au3:2648: `If $additionalParameters Or $success ==
/// $RESULT_FAILED Then`): innoextract runs either when the caller passed
/// `$additionalParameters` (a GOG-installer marker used elsewhere in the
/// dispatch chain to skip straight to it) or when the primary innounp
/// pass already failed.
pub fn should_use_innoextract_fallback(
    additional_parameters_present: bool,
    primary_failed: bool,
) -> bool {
    additional_parameters_present || primary_failed
}

/// Reproduces the multi-version file rename target
/// (UniExtract.au3:2625: `StringReplace($aFiles[$i], ",1", "", -1)`):
/// Inno Setup can extract multiple versions of the same file, suffixed
/// `,1`/`,2`/`,3`... — the first version is renamed back to its plain
/// name so extracted programs don't fail with "not found" exceptions.
/// `-1` means every occurrence of `,1` is replaced, not just the first —
/// preserved exactly, even though a real filename containing `,1`
/// mid-string (not as this suffix) would also be stripped.
pub fn rename_first_version_target(file: &str) -> String {
    file.replace(",1", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C074: the primary-pass invocation
    /// matches UniExtract.au3:2616's `innounp.exe -x -m -a "<file>"`
    /// call.
    #[test]
    fn unnp_invocation_matches_source() {
        let inv = unnp_invocation(
            r"C:\UniExtract\bin\innounp.exe",
            r"C:\downloads\setup.exe",
            r"C:\downloads\setup_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\innounp.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                "-m".to_string(),
                "-a".to_string(),
                r"C:\downloads\setup.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\setup_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C074: the fallback invocation matches
    /// UniExtract.au3:2649's `innoextract.exe -e --progress=1
    /// --collisions rename -d "<outdir>" "<file>"` call, run in
    /// `filedir` rather than `outdir`.
    #[test]
    fn innoextract_invocation_matches_source() {
        let inv = innoextract_invocation(
            r"C:\UniExtract\bin\innoextract.exe",
            r"C:\downloads\setup.exe",
            r"C:\downloads\setup_unpacked",
            r"C:\downloads",
        );
        assert_eq!(
            inv.args,
            vec![
                "-e".to_string(),
                "--progress=1".to_string(),
                "--collisions".to_string(),
                "rename".to_string(),
                "-d".to_string(),
                r"C:\downloads\setup_unpacked".to_string(),
                r"C:\downloads\setup.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn fallback_runs_when_additional_parameters_present() {
        assert!(should_use_innoextract_fallback(true, false));
    }

    #[test]
    fn fallback_runs_when_primary_failed() {
        assert!(should_use_innoextract_fallback(false, true));
    }

    #[test]
    fn fallback_skipped_when_neither_condition_holds() {
        assert!(!should_use_innoextract_fallback(false, false));
    }

    #[test]
    fn rename_first_version_target_strips_all_occurrences() {
        assert_eq!(rename_first_version_target("readme,1.txt"), "readme.txt");
        assert_eq!(
            rename_first_version_target("odd,1name,1.txt"),
            "oddname.txt"
        );
    }
}
