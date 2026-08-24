//! Context-menu registry write/apply — capability C202. Ports the
//! registry-write mechanics inside `GUI_ContextMenu_OK`
//! (UniExtract.au3:7150-7213, the write half not already covered by
//! C201's dialog/dispatch logic) and `GUI_ContextMenu_remove`
//! (UniExtract.au3:7309-7324) — the full remove-then-rebuild this row's
//! own description calls for, rather than an incremental diff.
//!
//! **Required extending `automation` first, as this row's own manifest
//! note calls for**: `GuiAutomation::reg_write_string` (`REG_SZ`
//! writes) and `HKEY_CLASSES_ROOT` support in
//! `automation::win32::parse_reg_key` — both added alongside this
//! module rather than as a separate row, matching the note's own
//! instruction.
//!
//! **Not wired to a real window.** This module produces the *plan* --
//! which keys and values to write or remove, for a caller to execute
//! against a real [`crate::automation::GuiAutomation`] -- rather than
//! executing it itself, so it stays real, cross-platform-testable Rust
//! even though nothing calls it from a live dialog yet.
//!
//! `reguser`/`reg_all`/`reg_current` parameters throughout this module
//! are full registry path prefixes already ending in `\`, exactly
//! matching the source's own `$regall`/`$regcurrent` constants
//! (UniExtract.au3:301-302: `$regall = "HKCR" & $reg64 & "\*\shell\"`,
//! `$regcurrent = "HKCU" & $reg64 & "\Software\Classes\*\shell\"`) --
//! callers concatenate a bare verb name directly onto them, with no
//! separator of their own to add.

/// One shell verb's registry-relevant fields (`$CM_Shells[i]`'s columns
/// 0-3, UniExtract.au3's own shell-verb table): the bare verb name used
/// as the registry key name directly under `reguser` (e.g.
/// `"UniExtract.Extract"`), the translated menu label, the command-line
/// suffix appended after `"<script>" "%1"`, and an optional
/// `MultiSelectModel` value (simple mode only -- cascading mode
/// hardcodes `"Player"` for the whole submenu instead,
/// UniExtract.au3:7190).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellVerb {
    pub key_name: String,
    pub label: String,
    pub command_suffix: String,
    pub multi_select_model: Option<String>,
}

/// One registry write: `RegWrite($key, $value_name, "REG_SZ", $value)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryWrite {
    pub key: String,
    pub value_name: String,
    pub value: String,
}

fn command_line(script_path: &str, verb: &ShellVerb) -> String {
    format!("\"{script_path}\" \"%1\"{}", verb.command_suffix)
}

/// Ports the simple-mode registry-write loop (UniExtract.au3:7170-7181):
/// one key per *checked* verb, directly under `reguser`. The icon value
/// is only written on Windows 7 or newer (UniExtract.au3:7179) -- icons
/// in Explorer context menus predate Windows 7, but this specific
/// registration mechanism for them doesn't, matching the source exactly
/// rather than writing it unconditionally.
pub fn build_simple_mode_writes(
    script_path: &str,
    reguser: &str,
    verbs: &[ShellVerb],
    checked: &[bool],
    is_win7_or_newer: bool,
    icon_value: &str,
) -> Vec<RegistryWrite> {
    let mut writes = Vec::new();
    for (verb, &is_checked) in verbs.iter().zip(checked) {
        if !is_checked {
            continue;
        }
        let key = format!("{reguser}{}", verb.key_name);
        writes.push(RegistryWrite {
            key: key.clone(),
            value_name: String::new(),
            value: verb.label.clone(),
        });
        writes.push(RegistryWrite {
            key: format!("{key}\\command"),
            value_name: String::new(),
            value: command_line(script_path, verb),
        });
        if let Some(model) = &verb.multi_select_model {
            writes.push(RegistryWrite {
                key: key.clone(),
                value_name: "MultiSelectModel".to_string(),
                value: model.clone(),
            });
        }
        if is_win7_or_newer {
            writes.push(RegistryWrite {
                key,
                value_name: "Icon".to_string(),
                value: icon_value.to_string(),
            });
        }
    }
    writes
}

/// Ports the cascading-mode registry-write block (UniExtract.au3:7186-
/// 7200): a parent `"uniextract"` key describing the submenu itself,
/// followed by one child key per *checked* verb under
/// `"uniextract\Shell\"`. Unlike simple mode, every verb's icon is
/// written unconditionally (UniExtract.au3:7198, no `$bIsWin7OrNewer`
/// guard) -- an asymmetry with [`build_simple_mode_writes`] that looks
/// inconsistent in isolation but is actually equivalent in practice: this
/// whole branch is only ever reached when Windows 7+ is already
/// confirmed (`GUI_ContextMenu_OK`'s own `ElseIf $bIsWin7OrNewer And
/// ...` guard, UniExtract.au3:7184), so the missing guard here changes
/// nothing real.
pub fn build_cascading_mode_writes(
    script_path: &str,
    reguser: &str,
    verbs: &[ShellVerb],
    checked: &[bool],
    icon_value: &str,
) -> Vec<RegistryWrite> {
    let parent_key = format!("{reguser}uniextract");
    let mut writes = vec![
        RegistryWrite {
            key: parent_key.clone(),
            value_name: "MUIVerb".to_string(),
            value: "Universal Extractor".to_string(),
        },
        RegistryWrite {
            key: parent_key.clone(),
            value_name: "Icon".to_string(),
            value: icon_value.to_string(),
        },
        RegistryWrite {
            key: parent_key.clone(),
            value_name: "SubCommands".to_string(),
            value: String::new(),
        },
        RegistryWrite {
            key: parent_key,
            value_name: "MultiSelectModel".to_string(),
            value: "Player".to_string(),
        },
    ];

    for (verb, &is_checked) in verbs.iter().zip(checked) {
        if !is_checked {
            continue;
        }
        let key = format!("{reguser}uniextract\\Shell\\{}", verb.key_name);
        writes.push(RegistryWrite {
            key: key.clone(),
            value_name: String::new(),
            value: verb.label.clone(),
        });
        writes.push(RegistryWrite {
            key: format!("{key}\\command"),
            value_name: String::new(),
            value: command_line(script_path, verb),
        });
        writes.push(RegistryWrite {
            key,
            value_name: "Icon".to_string(),
            value: icon_value.to_string(),
        });
    }
    writes
}

/// Ports `GUI_ContextMenu_remove`'s full-wipe key list
/// (UniExtract.au3:7309-7324, minus the file-association removal, which
/// C201's `should_remove_file_assoc` already covers): every verb's
/// simple-mode key under *both* registry scopes, plus the cascading
/// `"uniextract"` parent key under both scopes when Windows 7+ --
/// unconditionally, not gated on existence, since deleting an
/// already-absent key is a harmless no-op in the real registry API
/// (the source's own `If _RegExists(...) Then RegDelete(...)` guard is
/// purely an optimization to skip a call that would fail anyway, not a
/// behavioral distinction worth modeling here).
pub fn resolve_removal_targets(
    reg_all: &str,
    reg_current: &str,
    verb_key_names: &[String],
    is_win7_or_newer: bool,
) -> Vec<String> {
    let mut targets = Vec::new();
    for name in verb_key_names {
        targets.push(format!("{reg_all}{name}"));
        targets.push(format!("{reg_current}{name}"));
    }
    if is_win7_or_newer {
        targets.push(format!("{reg_all}uniextract"));
        targets.push(format!("{reg_current}uniextract"));
    }
    targets
}

/// Capability C203's own remaining piece of `GUI_ContextMenu_remove`
/// (UniExtract.au3:7323): `If $addassocenabled Then
/// GUI_ContextMenu_fileassoc(0)` -- the file association is torn down
/// as part of the unconditional wipe-before-rebuild, regardless of
/// what the dialog's *new* checkbox state will turn out to be. This is
/// deliberately a separate, simpler question from C201's
/// `should_apply_file_assoc_after_confirmation`/`should_remove_file_assoc`,
/// which decide the *re-apply* step afterward, once the new state is
/// known -- conflating the two would lose the "always wipe first"
/// property this row's own description calls out.
pub fn should_teardown_file_assoc_before_rebuild(was_previously_enabled: bool) -> bool {
    was_previously_enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG_ALL: &str = r"HKCR\*\shell\";
    const REG_CURRENT: &str = r"HKCU\Software\Classes\*\shell\";

    fn sample_verbs() -> Vec<ShellVerb> {
        vec![
            ShellVerb {
                key_name: "UniExtract.Extract".to_string(),
                label: "Extract with UniExtract".to_string(),
                command_suffix: String::new(),
                multi_select_model: None,
            },
            ShellVerb {
                key_name: "UniExtract.ExtractTo".to_string(),
                label: "Extract to folder".to_string(),
                command_suffix: " /sub".to_string(),
                multi_select_model: Some("Player".to_string()),
            },
        ]
    }

    #[test]
    fn simple_mode_skips_unchecked_verbs() {
        let writes = build_simple_mode_writes(
            r"C:\UniExtract.exe",
            REG_ALL,
            &sample_verbs(),
            &[true, false],
            true,
            "icon.ico",
        );
        assert!(writes
            .iter()
            .all(|w| !w.key.contains("UniExtract.ExtractTo")));
    }

    #[test]
    fn simple_mode_writes_label_command_and_icon() {
        let writes = build_simple_mode_writes(
            r"C:\UniExtract.exe",
            REG_ALL,
            &sample_verbs()[..1],
            &[true],
            true,
            "icon.ico",
        );
        assert_eq!(
            writes,
            vec![
                RegistryWrite {
                    key: r"HKCR\*\shell\UniExtract.Extract".to_string(),
                    value_name: String::new(),
                    value: "Extract with UniExtract".to_string(),
                },
                RegistryWrite {
                    key: r"HKCR\*\shell\UniExtract.Extract\command".to_string(),
                    value_name: String::new(),
                    value: r#""C:\UniExtract.exe" "%1""#.to_string(),
                },
                RegistryWrite {
                    key: r"HKCR\*\shell\UniExtract.Extract".to_string(),
                    value_name: "Icon".to_string(),
                    value: "icon.ico".to_string(),
                },
            ]
        );
    }

    #[test]
    fn simple_mode_writes_multi_select_model_when_present() {
        let writes = build_simple_mode_writes(
            r"C:\UniExtract.exe",
            REG_ALL,
            &sample_verbs()[1..],
            &[true],
            false,
            "icon.ico",
        );
        assert!(writes
            .iter()
            .any(|w| w.value_name == "MultiSelectModel" && w.value == "Player"));
    }

    /// Parity test: on pre-Windows-7, the icon value is never written at
    /// all in simple mode.
    #[test]
    fn simple_mode_skips_icon_before_win7() {
        let writes = build_simple_mode_writes(
            r"C:\UniExtract.exe",
            REG_ALL,
            &sample_verbs()[..1],
            &[true],
            false,
            "icon.ico",
        );
        assert!(!writes.iter().any(|w| w.value_name == "Icon"));
    }

    #[test]
    fn cascading_mode_writes_parent_key_unconditionally() {
        let writes =
            build_cascading_mode_writes(r"C:\UniExtract.exe", REG_ALL, &[], &[], "icon.ico");
        assert_eq!(
            writes,
            vec![
                RegistryWrite {
                    key: r"HKCR\*\shell\uniextract".to_string(),
                    value_name: "MUIVerb".to_string(),
                    value: "Universal Extractor".to_string(),
                },
                RegistryWrite {
                    key: r"HKCR\*\shell\uniextract".to_string(),
                    value_name: "Icon".to_string(),
                    value: "icon.ico".to_string(),
                },
                RegistryWrite {
                    key: r"HKCR\*\shell\uniextract".to_string(),
                    value_name: "SubCommands".to_string(),
                    value: String::new(),
                },
                RegistryWrite {
                    key: r"HKCR\*\shell\uniextract".to_string(),
                    value_name: "MultiSelectModel".to_string(),
                    value: "Player".to_string(),
                },
            ]
        );
    }

    /// Parity test: cascading mode writes every checked verb's icon
    /// unconditionally, no Windows-version gate.
    #[test]
    fn cascading_mode_writes_child_verb_with_icon() {
        let writes = build_cascading_mode_writes(
            r"C:\UniExtract.exe",
            REG_ALL,
            &sample_verbs()[..1],
            &[true],
            "icon.ico",
        );
        let child_writes: Vec<_> = writes.iter().filter(|w| w.key.contains("Shell")).collect();
        assert_eq!(child_writes.len(), 3);
        assert!(child_writes.iter().any(|w| w.value_name == "Icon"));
    }

    #[test]
    fn removal_targets_cover_both_scopes_per_verb() {
        let targets = resolve_removal_targets(
            REG_ALL,
            REG_CURRENT,
            &["UniExtract.Extract".to_string()],
            false,
        );
        assert_eq!(
            targets,
            vec![
                r"HKCR\*\shell\UniExtract.Extract".to_string(),
                r"HKCU\Software\Classes\*\shell\UniExtract.Extract".to_string(),
            ]
        );
    }

    #[test]
    fn removal_targets_include_cascading_parent_only_on_win7() {
        let targets = resolve_removal_targets(REG_ALL, REG_CURRENT, &[], true);
        assert_eq!(
            targets,
            vec![
                r"HKCR\*\shell\uniextract".to_string(),
                r"HKCU\Software\Classes\*\shell\uniextract".to_string(),
            ]
        );

        let targets_pre_win7 = resolve_removal_targets(REG_ALL, REG_CURRENT, &[], false);
        assert!(targets_pre_win7.is_empty());
    }

    #[test]
    fn teardown_file_assoc_only_when_previously_enabled() {
        assert!(should_teardown_file_assoc_before_rebuild(true));
        assert!(!should_teardown_file_assoc_before_rebuild(false));
    }
}
