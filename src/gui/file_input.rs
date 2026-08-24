//! File/directory input, validation, and output-directory auto-fill,
//! ported from `GUI_File`/`GUI_Directory`/`GUI_OnFileInputChanged`/
//! `GUI_OK`/`GUI_OK_Set`/`GUI_Drop_Parse` (UniExtract.au3:6264-6303,
//! 6555-6585,6735-6756) — capability C186.

use crate::outdir::{default_output_subfolder, split_file_path};

/// Ports `FilenameParse`'s whitespace gate (UniExtract.au3:501,
/// `StringIsSpace($f)`) — AutoIt's `StringIsSpace` is true for both an
/// empty string and a whitespace-only one, matching `str::trim`.
pub fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

/// The globals `FilenameParse` sets as a side effect (UniExtract.au3:
/// 503-518), modeled as a return value instead. `_PathFull`'s own
/// resolution isn't reproduced here — the caller supplies the already-
/// resolved full path, the same documented gap as `file_arg`'s own
/// `resolve_file_argument_path`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFilename {
    pub filedir: String,
    pub filename: String,
    pub fileext: String,
    pub filenamefull: String,
    pub initoutdir: String,
}

/// Ports `FilenameParse`'s parsing half (UniExtract.au3:503-518) — not
/// its existence check, which the source runs *after* this parsing
/// regardless of outcome (see [`decide_ok_set`]). Reuses
/// `outdir::split_file_path`/`outdir::default_output_subfolder` (C138)
/// for the `$initoutdir` computation, including its own multi-extension
/// collision-avoidance quirk, rather than re-deriving it here.
pub fn parse_filename(resolved_full_path: &str, initoutdir_collision: bool) -> ParsedFilename {
    let (filedir, stem, has_extension) = split_file_path(resolved_full_path);
    let filename_part = resolved_full_path
        .rsplit('\\')
        .next()
        .unwrap_or(resolved_full_path);
    let fileext = if has_extension {
        filename_part
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
    } else {
        String::new()
    };
    let initoutdir = default_output_subfolder(
        &filedir,
        &stem,
        has_extension,
        initoutdir_collision,
        "unpacked",
    );
    ParsedFilename {
        filedir,
        filename: stem,
        fileext,
        filenamefull: filename_part.to_string(),
        initoutdir,
    }
}

/// What clicking OK should do (`GUI_OK`/`GUI_OK_Set`, UniExtract.au3:
/// 6563-6582): invalid input never closes the window.
#[derive(Debug, Clone, PartialEq)]
pub enum OkOutcome {
    /// `FilenameParse` failed (blank field or the file doesn't exist) —
    /// the source shows an "invalid file" `MsgBox` here when
    /// `$bShowError` is set; that dialog itself is out of this
    /// function's scope, only the fail/succeed decision is.
    Invalid,
    /// Ready to commit and close, with `$outdir` resolved (the `/sub`
    /// sentinel when the directory field was left blank).
    Valid { outdir: String },
}

/// Ports `GUI_OK_Set` (UniExtract.au3:6571-6582): fails if the file field
/// is blank or the resolved file doesn't exist (`FilenameParse`'s own two
/// failure modes, not distinguished by the caller); otherwise resolves
/// `$outdir` from the directory field, defaulting to `/sub` when blank.
pub fn decide_ok_set(
    file_field_blank: bool,
    file_exists: bool,
    directory_field: &str,
) -> OkOutcome {
    if file_field_blank || !file_exists {
        return OkOutcome::Invalid;
    }
    let outdir = if directory_field.is_empty() {
        "/sub".to_string()
    } else {
        directory_field.to_string()
    };
    OkOutcome::Valid { outdir }
}

/// Ports `GUI_OnFileInputChanged`'s own auto-fill gate (UniExtract.au3:
/// 6555-6560): simpler than [`should_auto_fill_output_dir`]'s fuller OR
/// condition, since this handler fires on every keystroke in the file
/// field rather than only on an explicit file-selection event — it only
/// auto-fills when the directory field is currently blank, with no
/// lock-option check at all.
pub fn should_auto_fill_on_file_input_changed(directory_field_blank: bool) -> bool {
    directory_field_blank
}

/// Ports `GUI_Drop_Parse`'s own auto-fill gate (UniExtract.au3:6743):
/// **an OR, not an AND** — the directory field is overwritten with the
/// freshly-derived `$initoutdir` whenever *either* it's currently blank
/// *or* the "lock output directory" option is off. An empty *locked*
/// field still gets auto-filled; only a non-empty field is actually
/// protected by the lock.
pub fn should_auto_fill_output_dir(
    directory_field_blank: bool,
    lock_output_directory: bool,
) -> bool {
    directory_field_blank || !lock_output_directory
}

/// Ports `GUI_File`'s multi-select-string parsing (UniExtract.au3:6264-
/// 6277): `FileOpenDialog`'s pipe-delimited return is either a single
/// full path (no `|`) or `dir|file1|file2|...` for a multi-select.
/// Reconstructs full paths by joining `dir` onto each filename with a
/// literal backslash (this port's paths are always Windows paths). An
/// empty `raw` (dialog cancelled) yields no files.
pub fn parse_file_dialog_result(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = raw.split('|').collect();
    if parts.len() == 1 {
        vec![parts[0].to_string()]
    } else {
        let dir = parts[0];
        parts[1..].iter().map(|f| format!("{dir}\\{f}")).collect()
    }
}

/// Ports `GUI_Directory`'s fallback seed-directory derivation
/// (UniExtract.au3:6286-6290): if the current output-directory field
/// isn't a real existing path, falls back to deriving one from the file
/// field's own parent directory (only if *that* exists); otherwise an
/// empty seed (browse from the OS default/last location).
pub fn resolve_folder_picker_seed(
    current_dir_field: &str,
    current_dir_exists: bool,
    file_field: &str,
    file_field_exists: bool,
) -> String {
    if current_dir_exists {
        return current_dir_field.to_string();
    }
    if file_field_exists {
        if let Some(idx) = file_field.rfind('\\') {
            return file_field[..idx].to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_matches_empty_and_whitespace_only() {
        assert!(is_blank(""));
        assert!(is_blank("   "));
        assert!(!is_blank(" x "));
    }

    /// Parity test for capability C186: the extension is lowercased and
    /// `initoutdir` reuses C138's own collision-avoidance quirk.
    #[test]
    fn parse_filename_extracts_all_fields() {
        let parsed = parse_filename(r"C:\downloads\Archive.TAR.GZ", false);
        assert_eq!(parsed.filedir, r"C:\downloads");
        assert_eq!(parsed.filename, "Archive.TAR");
        assert_eq!(parsed.fileext, "gz");
        assert_eq!(parsed.filenamefull, "Archive.TAR.GZ");
        assert_eq!(parsed.initoutdir, r"C:\downloads\Archive.TAR");
    }

    #[test]
    fn parse_filename_applies_collision_avoidance_for_multi_extension_names() {
        let parsed = parse_filename(r"C:\downloads\archive.tar.gz", true);
        assert_eq!(parsed.initoutdir, r"C:\downloads\archive_tar");
    }

    #[test]
    fn parse_filename_handles_no_extension() {
        let parsed = parse_filename(r"C:\downloads\noext", false);
        assert_eq!(parsed.fileext, "");
        assert_eq!(parsed.filenamefull, "noext");
        assert_eq!(parsed.initoutdir, r"C:\downloads\noext_unpacked");
    }

    #[test]
    fn ok_set_invalid_on_blank_or_missing_file() {
        assert_eq!(decide_ok_set(true, true, ""), OkOutcome::Invalid);
        assert_eq!(decide_ok_set(false, false, ""), OkOutcome::Invalid);
    }

    /// Parity test for capability C186: a blank directory field resolves
    /// to the `/sub` sentinel, not an empty string.
    #[test]
    fn ok_set_defaults_blank_directory_to_sub_sentinel() {
        assert_eq!(
            decide_ok_set(false, true, ""),
            OkOutcome::Valid {
                outdir: "/sub".to_string()
            }
        );
        assert_eq!(
            decide_ok_set(false, true, r"C:\out"),
            OkOutcome::Valid {
                outdir: r"C:\out".to_string()
            }
        );
    }

    #[test]
    fn file_input_changed_only_checks_blank_directory() {
        assert!(should_auto_fill_on_file_input_changed(true));
        assert!(!should_auto_fill_on_file_input_changed(false));
    }

    /// Parity test for capability C186: an empty *locked* directory field
    /// still gets auto-filled -- the lock only protects a non-empty field.
    #[test]
    fn drop_parse_auto_fills_on_blank_field_even_when_locked() {
        assert!(should_auto_fill_output_dir(true, true));
        assert!(should_auto_fill_output_dir(true, false));
        assert!(!should_auto_fill_output_dir(false, true));
        assert!(should_auto_fill_output_dir(false, false));
    }

    #[test]
    fn dialog_result_single_selection_has_no_pipe() {
        assert_eq!(
            parse_file_dialog_result(r"C:\downloads\archive.zip"),
            vec![r"C:\downloads\archive.zip".to_string()]
        );
    }

    #[test]
    fn dialog_result_multi_selection_reconstructs_full_paths() {
        assert_eq!(
            parse_file_dialog_result(r"C:\downloads|a.zip|b.zip"),
            vec![
                r"C:\downloads\a.zip".to_string(),
                r"C:\downloads\b.zip".to_string()
            ]
        );
    }

    #[test]
    fn dialog_result_empty_means_cancelled() {
        assert_eq!(parse_file_dialog_result(""), Vec::<String>::new());
    }

    #[test]
    fn folder_picker_seed_prefers_existing_current_directory() {
        assert_eq!(
            resolve_folder_picker_seed(r"C:\out", true, r"C:\downloads\a.zip", true),
            r"C:\out"
        );
    }

    #[test]
    fn folder_picker_seed_falls_back_to_file_parent() {
        assert_eq!(
            resolve_folder_picker_seed("", false, r"C:\downloads\a.zip", true),
            r"C:\downloads"
        );
    }

    #[test]
    fn folder_picker_seed_falls_back_to_empty() {
        assert_eq!(resolve_folder_picker_seed("", false, "", false), "");
    }
}
