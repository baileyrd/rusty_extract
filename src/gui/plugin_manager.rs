//! Plugin manager GUI — capability C192. Ports the pure decisions inside
//! `GUI_Plugins` (UniExtract.au3:7778-7859), `GUI_Plugins_Install`
//! (UniExtract.au3:7861-7916), and `GUI_Plugins_Update`
//! (UniExtract.au3:7919-7941). Also covers D008's `/plugins` CLI verb,
//! which just dispatches into this same GUI pre-selecting a plugin — out
//! of scope for the same reason every other CLI flag (C007-C013) is:
//! this port's composition root only takes positional arguments.
//!
//! **The plugin manager window itself stays unwired** — same treatment
//! as every other large dialog this migration phase has ported so far
//! (C188's queue-edit dialog, C189's confirm dialog, C190's Preferences
//! dialog, C191's wizard): it's a list-plus-description window built
//! around a hardcoded 12-entry plugin table with real network downloads
//! (`OpenURL`) and filesystem installs, none of which this port's GUI
//! drives yet.
//!
//! **Correction to this row's own earlier inventory note.** The manifest
//! previously flagged `StringRight($sPath, 3) == ".7z"`
//! (UniExtract.au3:7879-7880) as a bug on the claim that `".7z"` is 4
//! characters and so could never match. Recounted precisely: `".7z"` is
//! **3** characters (`.`, `7`, `z`), exactly what `StringRight(..., 3)`
//! extracts — the comparison is correct, and `.7z` plugin archives *do*
//! route to the 7-Zip-extraction branch like `rar`/`zip` do. There is no
//! bug here; [`resolve_install_mechanism`] below ports the check as
//! written, matching all three extensions correctly.

/// `GUI_Plugins_Install`'s unpack-vs-copy dispatch
/// (UniExtract.au3:7879-7880). Case-insensitive, like every other
/// `StringInStr`/`=`-comparison this port has encountered (C007-C013,
/// C144, C145, C147) — AutoIt's `=` operator is case-insensitive by
/// default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMechanism {
    /// `.7z`/`rar`/`zip`: run 7-Zip against the downloaded archive.
    UnpackArchive,
    /// Anything else: copy the selected file(s) directly into `\bin\`.
    CopyFiles,
}

/// Ports `StringRight($sPath, 3)` then the three-way `Or` comparison
/// (UniExtract.au3:7879-7880) — see this module's own doc comment for
/// why this is correct as written, not a bug to work around.
pub fn resolve_install_mechanism(path: &str) -> InstallMechanism {
    let chars: Vec<char> = path.chars().collect();
    let start = chars.len().saturating_sub(3);
    let last_three: String = chars[start..].iter().collect::<String>().to_lowercase();
    match last_three.as_str() {
        ".7z" | "rar" | "zip" => InstallMechanism::UnpackArchive,
        _ => InstallMechanism::CopyFiles,
    }
}

/// What clicking the overloaded Select/Finish button does
/// (UniExtract.au3:7831): closing needs either no selection at all, or a
/// selection that's already installed — anything else means the user
/// still needs to pick a file for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectCloseAction {
    CloseDialog,
    PromptForFile,
}

/// Ports the `$GUI_Plugins_SelectClose` handler's guard
/// (UniExtract.au3:7831): `$current == -1 Or HasPlugin(...)`.
pub fn decide_select_close_action(
    has_selection: bool,
    plugin_already_installed: bool,
) -> SelectCloseAction {
    if !has_selection || plugin_already_installed {
        SelectCloseAction::CloseDialog
    } else {
        SelectCloseAction::PromptForFile
    }
}

/// The Download and Select/Finish buttons' per-selection label/enabled
/// state (`GUI_Plugins_Update`, UniExtract.au3:7931-7938).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginSelectionDisplay {
    pub download_button_enabled: bool,
    pub download_button_shows_installed: bool,
    pub select_close_button_shows_finish: bool,
}

/// Ports `GUI_Plugins_Update`'s installed-vs-not branch
/// (UniExtract.au3:7931-7938). `is_compiled` stands in for `@Compiled` —
/// the source only ever runs this installed-check when compiled, so in
/// dev/script-mode a plugin always displays as not-installed regardless
/// of `already_installed`'s real value, exactly the quirk this row's
/// manifest note already flags.
pub fn resolve_plugin_selection_display(
    is_compiled: bool,
    already_installed: bool,
) -> PluginSelectionDisplay {
    if is_compiled && already_installed {
        PluginSelectionDisplay {
            download_button_enabled: false,
            download_button_shows_installed: true,
            select_close_button_shows_finish: true,
        }
    } else {
        PluginSelectionDisplay {
            download_button_enabled: true,
            download_button_shows_installed: false,
            select_close_button_shows_finish: false,
        }
    }
}

/// Ports the required-file presence check inside `GUI_Plugins_Install`'s
/// copy-files branch (UniExtract.au3:7894-7900): returns every required
/// file that's genuinely missing from `raw_selected_parts` (the file
/// dialog's own raw `StringSplit($sPath, "|", ...)` parts — bare
/// filenames for a multi-selection, or the one full path for a single
/// selection). A required entry containing a wildcard (`*`) is always
/// treated as present — `_ArraySearch` can't match wildcards, and rather
/// than fix that, the source just skips checking those entries entirely
/// (UniExtract.au3:7897). An empty result means installation may
/// proceed; a non-empty one is exactly the missing-files list
/// `PLUGIN_IMPORT_MISSINGFILES` reports.
pub fn missing_required_files(
    required_files: &[String],
    raw_selected_parts: &[String],
) -> Vec<String> {
    required_files
        .iter()
        .filter(|required| !required.contains('*'))
        .filter(|required| {
            !raw_selected_parts
                .iter()
                .any(|selected| selected.eq_ignore_ascii_case(required))
        })
        .cloned()
        .collect()
}

/// Ports the single-vs-multiple-file copy destination resolution
/// (UniExtract.au3:7903-7914). `files` is the *reconstructed* full-path
/// list (`gui::file_input::parse_file_dialog_result`'s own output — the
/// same pipe-delimited multi-select format `GUI_Plugins_Install`
/// reconstructs by hand at UniExtract.au3:7910, so that existing C186
/// parser is reused here rather than re-derived). A single file is
/// renamed to the plugin's own `newfilename` field on the way in; two or
/// more keep their original names and land directly in `outdir`.
pub fn resolve_copy_plan(
    files: &[String],
    outdir: &str,
    new_filename: &str,
) -> Vec<(String, String)> {
    match files {
        [] => Vec::new(),
        [only] => vec![(only.clone(), format!("{outdir}{new_filename}"))],
        multiple => multiple
            .iter()
            .map(|f| (f.clone(), outdir.to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_mechanism_unpacks_7z_rar_and_zip() {
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\arc_conv_r123.7z"),
            InstallMechanism::UnpackArchive
        );
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\i5comp21.rar"),
            InstallMechanism::UnpackArchive
        );
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\dgca_v2.zip"),
            InstallMechanism::UnpackArchive
        );
    }

    #[test]
    fn install_mechanism_copies_anything_else() {
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\iscab.exe"),
            InstallMechanism::CopyFiles
        );
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\gentee.dll"),
            InstallMechanism::CopyFiles
        );
    }

    #[test]
    fn install_mechanism_is_case_insensitive() {
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\ARCHIVE.ZIP"),
            InstallMechanism::UnpackArchive
        );
        assert_eq!(
            resolve_install_mechanism(r"C:\dl\PLUGIN.7Z"),
            InstallMechanism::UnpackArchive
        );
    }

    #[test]
    fn select_close_closes_with_no_selection() {
        assert_eq!(
            decide_select_close_action(false, false),
            SelectCloseAction::CloseDialog
        );
    }

    #[test]
    fn select_close_closes_when_already_installed() {
        assert_eq!(
            decide_select_close_action(true, true),
            SelectCloseAction::CloseDialog
        );
    }

    #[test]
    fn select_close_prompts_for_uninstalled_selection() {
        assert_eq!(
            decide_select_close_action(true, false),
            SelectCloseAction::PromptForFile
        );
    }

    #[test]
    fn selection_display_shows_installed_when_compiled_and_installed() {
        let display = resolve_plugin_selection_display(true, true);
        assert!(!display.download_button_enabled);
        assert!(display.download_button_shows_installed);
        assert!(display.select_close_button_shows_finish);
    }

    /// Parity test: the `@Compiled` gate means dev/script-mode always
    /// shows "not installed", even if `already_installed` is true.
    #[test]
    fn selection_display_ignores_installed_state_when_not_compiled() {
        let display = resolve_plugin_selection_display(false, true);
        assert!(display.download_button_enabled);
        assert!(!display.download_button_shows_installed);
        assert!(!display.select_close_button_shows_finish);
    }

    #[test]
    fn selection_display_shows_not_installed_when_never_installed() {
        let display = resolve_plugin_selection_display(true, false);
        assert!(display.download_button_enabled);
        assert!(!display.download_button_shows_installed);
    }

    #[test]
    fn missing_required_files_none_when_all_present() {
        let required = vec!["ci-extractor.exe".to_string(), "gea.dll".to_string()];
        let selected = vec!["ci-extractor.exe".to_string(), "gea.dll".to_string()];
        assert!(missing_required_files(&required, &selected).is_empty());
    }

    #[test]
    fn missing_required_files_reports_absent_entries() {
        let required = vec!["ci-extractor.exe".to_string(), "gea.dll".to_string()];
        let selected = vec!["ci-extractor.exe".to_string()];
        assert_eq!(
            missing_required_files(&required, &selected),
            vec!["gea.dll".to_string()]
        );
    }

    /// Parity test: a wildcard entry is always treated as present.
    #[test]
    fn missing_required_files_skips_wildcard_entries() {
        let required = vec!["dgca_v*.zip".to_string()];
        assert!(missing_required_files(&required, &[]).is_empty());
    }

    #[test]
    fn missing_required_files_is_case_insensitive() {
        let required = vec!["Gea.DLL".to_string()];
        let selected = vec!["gea.dll".to_string()];
        assert!(missing_required_files(&required, &selected).is_empty());
    }

    /// Parity test: a single selected file is renamed to the plugin's
    /// own `newfilename` on the way into `outdir`.
    #[test]
    fn copy_plan_renames_single_file() {
        let files = vec![r"C:\dl\bitrock-unpacker123.exe".to_string()];
        let plan = resolve_copy_plan(&files, r"C:\UniExtract\bin\", "bitrock.exe");
        assert_eq!(
            plan,
            vec![(
                r"C:\dl\bitrock-unpacker123.exe".to_string(),
                r"C:\UniExtract\bin\bitrock.exe".to_string()
            )]
        );
    }

    /// Parity test: multiple selected files keep their own names and all
    /// land directly in `outdir`.
    #[test]
    fn copy_plan_keeps_names_for_multiple_files() {
        let files = vec![
            r"C:\dl\ci-extractor.exe".to_string(),
            r"C:\dl\gea.dll".to_string(),
            r"C:\dl\gentee.dll".to_string(),
        ];
        let plan = resolve_copy_plan(&files, r"C:\UniExtract\bin\", "");
        assert_eq!(
            plan,
            vec![
                (
                    r"C:\dl\ci-extractor.exe".to_string(),
                    r"C:\UniExtract\bin\".to_string()
                ),
                (
                    r"C:\dl\gea.dll".to_string(),
                    r"C:\UniExtract\bin\".to_string()
                ),
                (
                    r"C:\dl\gentee.dll".to_string(),
                    r"C:\UniExtract\bin\".to_string()
                ),
            ]
        );
    }

    #[test]
    fn copy_plan_empty_files_is_empty_plan() {
        assert!(resolve_copy_plan(&[], "outdir", "new.exe").is_empty());
    }
}
