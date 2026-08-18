//! Output-directory resolution: ports `ValidateOutputDirectory()`
//! (UniExtract.au3:526-544) and `GetLastOutdir()`
//! (UniExtract.au3:872-878).

/// C005: ports `GetLastOutdir()` (UniExtract.au3:872-878) — the most
/// recently used output directory is the `"Directory History"` section's
/// ini key `"0"` (the newest slot, in `prefs::push_history`'s convention,
/// C021). `None` here represents the source's failure path: no history
/// yet, which shows a `MsgBox` (out of scope, deferred GUI subsystem)
/// and calls `terminate($STATUS_SILENT)` (exit 0, C016) — the source
/// never returns a directory at all in that case, so a caller reaching
/// [`resolve_output_directory`]'s `/last` branch must already have
/// handled this `None` case the same way (i.e. it can't happen in a
/// faithful driver of this function).
pub fn get_last_outdir(history_dir_slot0: Option<&str>) -> Option<String> {
    history_dir_slot0.map(str::to_string)
}

fn is_drive_absolute(path: &str) -> bool {
    path.as_bytes().get(1) == Some(&b':')
}

/// C004, C139, C140: ports `ValidateOutputDirectory()`
/// (UniExtract.au3:526-544).
///
/// - **`/sub` (C004):** resolves to `initoutdir` (a subdirectory named
///   after the archive, computed by the caller — UniExtract.au3:645).
/// - **`/last` (C005):** resolves to `last_outdir`, the already-resolved
///   result of [`get_last_outdir`] — see that function's doc comment for
///   why this parameter is a plain `Option<&str>` rather than something
///   this function computes itself.
/// - **Drive-absolute (`X:...`) or UNC (`\\...`) paths:** returned
///   unchanged.
/// - **A single leading backslash, not a UNC double backslash (C139):**
///   inherits the input file's drive letter (`filedir`'s first two
///   characters) rather than being treated as relative.
/// - **Anything else (C139):** resolved against `filedir` by
///   concatenation, mirroring `$filedir & '\' & $outdir`.
/// - **Trailing slash (C140):** a trailing `/` is stripped first, then a
///   trailing `\` is unconditionally appended if not already present —
///   applied regardless of which branch above produced the path.
///
/// **Not modeled — `_PathFull`'s own segment normalization.** The
/// relative-path branch mirrors the source's exact string concatenation
/// but doesn't reproduce whatever `.`/`..`-collapsing AutoIt's
/// `_PathFull(path)` (the single-argument form the source calls here,
/// UniExtract.au3:535) performs internally — that UDF isn't defined
/// anywhere in this port's source checkout, the same gap already noted
/// for the two-argument `_PathFull` behind `prefs::resolve_batchqueue_path`
/// (C018) and `prefs::resolve_filescanlogfile_path` (C019).
pub fn resolve_output_directory(
    outdir: &str,
    initoutdir: &str,
    filedir: &str,
    last_outdir: Option<&str>,
) -> String {
    let mut resolved = if outdir.eq_ignore_ascii_case("/sub") {
        initoutdir.to_string()
    } else if outdir.eq_ignore_ascii_case("/last") {
        last_outdir.unwrap_or_default().to_string()
    } else if is_drive_absolute(outdir) {
        outdir.to_string()
    } else if outdir.starts_with('\\') && !outdir.starts_with(r"\\") {
        let drive_letter = &filedir[..filedir.len().min(2)];
        format!("{drive_letter}{outdir}")
    } else if !outdir.starts_with(r"\\") {
        format!("{filedir}\\{outdir}")
    } else {
        outdir.to_string()
    };

    if resolved.ends_with('/') {
        resolved.pop();
    }
    if !resolved.ends_with('\\') {
        resolved.push('\\');
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::{get_last_outdir, resolve_output_directory};

    /// Parity test for capability C005: a present history slot 0 is
    /// returned as the resolved directory; a missing one maps to `None`,
    /// standing in for the source's `terminate($STATUS_SILENT)` path.
    #[test]
    fn get_last_outdir_matches_source() {
        assert_eq!(
            get_last_outdir(Some(r"C:\downloads\unpacked")),
            Some(r"C:\downloads\unpacked".to_string())
        );
        assert_eq!(get_last_outdir(None), None);
    }

    /// Parity test for capability C004: `/sub` resolves to `initoutdir`.
    #[test]
    fn sub_token_resolves_to_initoutdir() {
        assert_eq!(
            resolve_output_directory("/sub", r"C:\downloads\archive", r"C:\downloads", None),
            r"C:\downloads\archive\"
        );
        // Case-insensitive, matching AutoIt's default `=` comparison.
        assert_eq!(
            resolve_output_directory("/SUB", r"C:\downloads\archive", r"C:\downloads", None),
            r"C:\downloads\archive\"
        );
    }

    /// Parity test for capability C005 (as consumed by
    /// `resolve_output_directory`): `/last` resolves to the pre-resolved
    /// `last_outdir`.
    #[test]
    fn last_token_resolves_to_last_outdir() {
        assert_eq!(
            resolve_output_directory(
                "/last",
                r"C:\downloads\archive",
                r"C:\downloads",
                Some(r"D:\history\unpacked")
            ),
            r"D:\history\unpacked\"
        );
    }

    /// Parity test for capability C139: a drive-absolute or UNC outdir
    /// passes through unchanged (aside from trailing-slash handling).
    #[test]
    fn drive_absolute_and_unc_paths_pass_through() {
        assert_eq!(
            resolve_output_directory(r"D:\custom", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"\\server\share\out", "", r"C:\downloads", None),
            r"\\server\share\out\"
        );
    }

    /// Parity test for capability C139: a single leading backslash
    /// inherits the input file's drive letter rather than being resolved
    /// relative to `filedir`.
    #[test]
    fn single_leading_backslash_inherits_drive_letter() {
        assert_eq!(
            resolve_output_directory(r"\output", "", r"D:\downloads\sub", None),
            r"D:\output\"
        );
    }

    /// Parity test for capability C139: a plain relative outdir resolves
    /// against `filedir` by concatenation.
    #[test]
    fn relative_path_resolves_against_filedir() {
        assert_eq!(
            resolve_output_directory("unpacked", "", r"C:\downloads", None),
            r"C:\downloads\unpacked\"
        );
    }

    /// Parity test for capability C140: a trailing `/` is stripped, then
    /// a trailing `\` is always appended.
    #[test]
    fn trailing_slash_normalized_to_backslash() {
        assert_eq!(
            resolve_output_directory(r"D:\custom/", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"D:\custom\", "", r"C:\downloads", None),
            r"D:\custom\"
        );
        assert_eq!(
            resolve_output_directory(r"D:\custom", "", r"C:\downloads", None),
            r"D:\custom\"
        );
    }
}
