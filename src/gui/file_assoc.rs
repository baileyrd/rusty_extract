//! File-association registration — capability C204. Ports the pure
//! decisions inside `GUI_ContextMenu_fileassoc` (UniExtract.au3:7237-
//! 7273), `_ShellFile_Install` (UniExtract.au3:7278-7292), and
//! `_ShellFile_Uninstall` (UniExtract.au3:7297-7306). Folds in D011's
//! `addassocenabled`/`addassoc`/`addassocallusers` preferences.
//!
//! **Required extending `automation` first**, alongside C202/C203's own
//! `HKCR`/`reg_write_string` additions: `GuiAutomation::reg_read_string`
//! (`_ShellFile_Uninstall`'s `RegRead($sRegistryKey & "." &
//! $sFileType, "")` lookup) and `GuiAutomation::reg_write_expand_string`
//! (`_ShellFile_Install`'s two `Icon` value writes, which use
//! `REG_EXPAND_SZ` — the one real `REG_EXPAND_SZ` call site C202's own
//! "verified correction" noted was *absent* from `GUI_ContextMenu_OK`
//! specifically; it exists here instead, in a different function).
//!
//! **Not wired to a real window or real registry I/O.** This module
//! produces write/removal *plans* for a caller to execute against a
//! real [`crate::automation::GuiAutomation`], the same shape as C202's
//! `gui::context_menu_registry`.

/// Ports the leading-dot-strip both `_ShellFile_Install`
/// (UniExtract.au3:7280) and `_ShellFile_Uninstall`
/// (UniExtract.au3:7299) apply identically to their `$sFileType`
/// argument before using it.
pub fn strip_leading_dot(file_type: &str) -> &str {
    file_type.strip_prefix('.').unwrap_or(file_type)
}

/// Ports the comma-separated extension list parsing shared by both the
/// uninstall loop (UniExtract.au3:7241-7244, over the *old* `$addassoc`)
/// and the install loop (UniExtract.au3:7263-7267, over the *new* input
/// field's text): `StringSplit($addassoc, ",")` then
/// `StringStripWS($files[$i], 1)` (trim leading/trailing whitespace) on
/// each entry. An empty string splits to a single empty entry in AutoIt
/// (`StringSplit` never returns zero elements for a non-delimiter
/// input) — filtered out here since an empty extension is never a real
/// association to act on.
pub fn parse_extension_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Ports the registry-scope root selection used twice in
/// `GUI_ContextMenu_fileassoc` — once from the *stored* `addassocallusers`
/// preference for the uninstall pass (UniExtract.au3:7238), once from
/// the *current* checkbox state for the install pass
/// (UniExtract.au3:7254-7260). Returns just the root name; the caller
/// appends the rest (`$reg64 & "\SOFTWARE\Classes\"`), a real-wiring
/// concern this pure function doesn't need to know about.
pub fn resolve_registry_scope(all_users: bool) -> &'static str {
    if all_users {
        "HKLM"
    } else {
        "HKCU"
    }
}

/// Ports `_ShellFile_Uninstall`'s existence gate (UniExtract.au3:7301-
/// 7302): `RegRead(...)`'s `@error` means the extension was never
/// actually associated, so `Return SetError(...)` skips both deletes
/// entirely rather than attempting to remove something that was never
/// there.
pub fn should_remove_association(existing_progid: Option<&str>) -> bool {
    existing_progid.is_some()
}

/// One registry write [`build_install_writes`] plans -- `REG_SZ` and
/// `REG_EXPAND_SZ` are tracked as distinct variants since they need
/// different [`crate::automation::GuiAutomation`] methods
/// (`reg_write_string` vs. `reg_write_expand_string`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryValueKind {
    Str(String),
    ExpandStr(String),
}

/// One planned write: `RegWrite($key, $value_name, "REG_SZ"|"REG_EXPAND_SZ", $value)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAssocWrite {
    pub key: String,
    pub value_name: String,
    pub value: RegistryValueKind,
}

/// Ports `_ShellFile_Install`'s full eight-write sequence
/// (UniExtract.au3:7282-7289) exactly, in order: the extension-to-ProgID
/// mapping, the ProgID's icon and `shell\open` display/command entries,
/// then a second, top-level copy of the display text/icon/command
/// directly under the ProgID key itself -- the "looks like two
/// overlapping association styles" this row's own manifest note flags,
/// preserved exactly rather than deduplicated, since both are genuinely
/// written by the real source.
pub fn build_install_writes(
    registry_key: &str,
    file_type: &str,
    prog_id: &str,
    display_text: &str,
    script_path: &str,
) -> Vec<FileAssocWrite> {
    let file_type = strip_leading_dot(file_type);
    let command = format!("\"{script_path}\" \"%1\"");
    let icon = format!("{script_path},0");

    vec![
        FileAssocWrite {
            key: format!("{registry_key}.{file_type}"),
            value_name: String::new(),
            value: RegistryValueKind::Str(prog_id.to_string()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}\\DefaultIcon\\"),
            value_name: String::new(),
            value: RegistryValueKind::Str(icon.clone()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}\\shell\\open"),
            value_name: String::new(),
            value: RegistryValueKind::Str(display_text.to_string()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}\\shell\\open"),
            value_name: "Icon".to_string(),
            value: RegistryValueKind::ExpandStr(icon.clone()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}\\shell\\open\\command\\"),
            value_name: String::new(),
            value: RegistryValueKind::Str(command.clone()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}"),
            value_name: String::new(),
            value: RegistryValueKind::Str(display_text.to_string()),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}"),
            value_name: "Icon".to_string(),
            value: RegistryValueKind::ExpandStr(icon),
        },
        FileAssocWrite {
            key: format!("{registry_key}{prog_id}\\command"),
            value_name: String::new(),
            value: RegistryValueKind::Str(command),
        },
    ]
}

/// The overall `GUI_ContextMenu_fileassoc($bEnable)` decision
/// (UniExtract.au3:7237-7273): every call uninstalls the *old* tracked
/// extensions first, regardless of `enable` — only when enabling does a
/// fresh install (against the *current* field/checkbox values) follow.
/// On disable, `install` is `None` and nothing else is touched: the
/// stored `addassoc`/`addassocallusers` preference values are left
/// exactly as they were, ready to reactivate unchanged later — this
/// row's own manifest note's flagged quirk, reflected here by simply
/// not producing a new value for them rather than by an explicit
/// "preserve" step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAssocApplyPlan {
    pub extensions_to_uninstall: Vec<String>,
    pub new_addassocenabled: bool,
    /// `Some((new_extensions, new_all_users))` only when enabling.
    pub install: Option<(Vec<String>, bool)>,
}

/// Ports `GUI_ContextMenu_fileassoc`'s top-level dispatch
/// (UniExtract.au3:7237-7273).
pub fn resolve_file_assoc_apply_plan(
    enable: bool,
    old_extension_list: &str,
    new_extension_list: &str,
    new_all_users_checked: bool,
) -> FileAssocApplyPlan {
    FileAssocApplyPlan {
        extensions_to_uninstall: parse_extension_list(old_extension_list),
        new_addassocenabled: enable,
        install: enable.then(|| {
            (
                parse_extension_list(new_extension_list),
                new_all_users_checked,
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_leading_dot_removes_one_leading_dot_only() {
        assert_eq!(strip_leading_dot(".zip"), "zip");
        assert_eq!(strip_leading_dot("zip"), "zip");
    }

    #[test]
    fn parse_extension_list_trims_and_splits() {
        assert_eq!(
            parse_extension_list(" zip, rar ,7z"),
            vec!["zip".to_string(), "rar".to_string(), "7z".to_string()]
        );
    }

    #[test]
    fn parse_extension_list_empty_string_is_empty_list() {
        assert!(parse_extension_list("").is_empty());
    }

    #[test]
    fn registry_scope_matches_all_users_flag() {
        assert_eq!(resolve_registry_scope(true), "HKLM");
        assert_eq!(resolve_registry_scope(false), "HKCU");
    }

    #[test]
    fn should_remove_association_only_when_progid_found() {
        assert!(should_remove_association(Some("UniExtract.zip")));
        assert!(!should_remove_association(None));
    }

    #[test]
    fn install_writes_match_source_sequence_exactly() {
        let writes = build_install_writes(
            r"HKCU\SOFTWARE\Classes\",
            ".zip",
            "zip",
            "UniExtract zip",
            r"C:\UniExtract.exe",
        );
        assert_eq!(
            writes,
            vec![
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\.zip".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str("zip".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip\DefaultIcon\".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str(r"C:\UniExtract.exe,0".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip\shell\open".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str("UniExtract zip".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip\shell\open".to_string(),
                    value_name: "Icon".to_string(),
                    value: RegistryValueKind::ExpandStr(r"C:\UniExtract.exe,0".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip\shell\open\command\".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str(r#""C:\UniExtract.exe" "%1""#.to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str("UniExtract zip".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip".to_string(),
                    value_name: "Icon".to_string(),
                    value: RegistryValueKind::ExpandStr(r"C:\UniExtract.exe,0".to_string()),
                },
                FileAssocWrite {
                    key: r"HKCU\SOFTWARE\Classes\zip\command".to_string(),
                    value_name: String::new(),
                    value: RegistryValueKind::Str(r#""C:\UniExtract.exe" "%1""#.to_string()),
                },
            ]
        );
    }

    /// Parity test: every call uninstalls the old list, disable stops
    /// there.
    #[test]
    fn apply_plan_disable_only_uninstalls_and_leaves_install_none() {
        let plan = resolve_file_assoc_apply_plan(false, "zip,rar", "7z", true);
        assert_eq!(
            plan.extensions_to_uninstall,
            vec!["zip".to_string(), "rar".to_string()]
        );
        assert!(!plan.new_addassocenabled);
        assert_eq!(plan.install, None);
    }

    /// Parity test: enabling also produces a fresh install plan from the
    /// *current* field/checkbox values, independent of the old list.
    #[test]
    fn apply_plan_enable_produces_install_plan_from_current_values() {
        let plan = resolve_file_assoc_apply_plan(true, "zip,rar", "7z, cab", false);
        assert_eq!(
            plan.extensions_to_uninstall,
            vec!["zip".to_string(), "rar".to_string()]
        );
        assert!(plan.new_addassocenabled);
        assert_eq!(
            plan.install,
            Some((vec!["7z".to_string(), "cab".to_string()], false))
        );
    }
}
