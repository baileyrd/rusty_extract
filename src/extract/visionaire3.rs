//! VIS3Ext (`Visionaire3.exe`) — Visionaire Engine v3 archives, two-pass
//! invocation.

use super::{Invocation, WindowMode};

/// Builds the first-pass invocation `Case $TYPE_VISIONAIRE3` makes
/// (UniExtract.au3:3310) to generate `<outdir>\names.txt` from the
/// archive's main `.vis` data file: `<program> "<main_vis_file>" /force
/// /generateNames="<outdir>\names.txt"`, run in `outdir` with the window
/// hidden.
///
/// **Scope — invocation only.** Locating `main_vis_file` itself — up to
/// three parent directories are searched for `*.vis` files, with a GUI
/// candidate list (C053, deferred GUI subsystem, D001) shown when more
/// than one is found — and the `names.txt`-already-exists check that
/// skips this pass entirely (UniExtract.au3:3293) are both real
/// filesystem concerns left to the caller.
pub fn generate_names_invocation(program: &str, main_vis_file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            main_vis_file.to_string(),
            "/force".to_string(),
            format!("/generateNames={outdir}\\names.txt"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the second-pass extraction invocation
/// (UniExtract.au3:3315-3322): if the first pass's `<outdir>\names.txt`
/// came out non-empty, extraction runs with
/// `/names="<outdir>\names.txt"`; otherwise — the file is empty, or the
/// first pass never ran (`names_file_size` is `0` in both cases, the
/// same value AutoIt's `FileGetSize` returns for a missing file as for
/// an empty one) — it falls back to a bare `/force` with no `/names`
/// argument at all, matching `FileGetSize($tmp) > 0`'s single-condition
/// branch exactly.
pub fn extract_invocation(
    program: &str,
    file: &str,
    outdir: &str,
    names_file_size: u64,
) -> Invocation {
    let mut args = vec![file.to_string(), "/force".to_string()];
    if names_file_size > 0 {
        args.push(format!("/names={outdir}\\names.txt"));
    }
    Invocation {
        program: program.to_string(),
        args,
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C105: the first-pass invocation
    /// matches UniExtract.au3:3310's `Visionaire3.exe "<main_vis_file>"
    /// /force /generateNames="<outdir>\names.txt"` call.
    #[test]
    fn generate_names_matches_source_invocation() {
        let inv = generate_names_invocation(
            r"C:\UniExtract\bin\Visionaire3.exe",
            r"C:\downloads\data.vis",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\Visionaire3.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\data.vis".to_string(),
                "/force".to_string(),
                r"/generateNames=C:\downloads\archive_unpacked\names.txt".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C105: a non-empty `names.txt` from the
    /// first pass makes the second pass include the `/names=` argument.
    #[test]
    fn extract_with_names_file_includes_names_argument() {
        let inv = extract_invocation(
            r"C:\UniExtract\bin\Visionaire3.exe",
            r"C:\downloads\archive.vis",
            r"C:\downloads\archive_unpacked",
            42,
        );
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive.vis".to_string(),
                "/force".to_string(),
                r"/names=C:\downloads\archive_unpacked\names.txt".to_string(),
            ]
        );
    }

    /// Parity test for capability C105: an empty (or missing) `names.txt`
    /// falls back to a bare `/force` with no `/names` argument.
    #[test]
    fn extract_without_names_file_omits_names_argument() {
        let inv = extract_invocation(
            r"C:\UniExtract\bin\Visionaire3.exe",
            r"C:\downloads\archive.vis",
            r"C:\downloads\archive_unpacked",
            0,
        );
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive.vis".to_string(),
                "/force".to_string(),
            ]
        );
    }
}
