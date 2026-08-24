//! Explorer context-menu registration dialog — capability C201. Ports
//! the pure decisions inside `GUI_ContextMenu` (UniExtract.au3:7038-
//! 7141), `GUI_ContextMenu_ChangePic` (UniExtract.au3:7144-7147),
//! `GUI_ContextMenu_OK` (UniExtract.au3:7150-7213), and
//! `GUI_ContextMenu_activate` (UniExtract.au3:7216-7234).
//!
//! **Not wired to a real window or real registry I/O** — same treatment
//! as every other dialog this migration phase has ported so far. No
//! self-elevation/UAC prompt exists in the source either: all-users
//! checkboxes are just disabled (with a tooltip) unless already running
//! elevated, a real behavior this module's `resolve_context_menu_activate`
//! reflects (`is_admin: bool` as an input, not something this port could
//! itself elevate to obtain).

/// The per-verb + aggregate result of scanning every shell verb's
/// registration state at window-construction time
/// (UniExtract.au3:7084-7097 for the simple-mode registry paths,
/// 7102-7115 for the cascading ones under `\Uniextract\Shell\`, gated on
/// `_IsWin7OrNewer()`). Both scans share this exact same shape and
/// per-verb-overwrite semantics — only the registry path prefix differs,
/// a real-wiring concern, not a pure-logic one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellRegistrationScan {
    /// Whether *any* verb was found registered in either scope — drives
    /// `$CM_Checkbox_enabled`'s checked state.
    pub enabled: bool,
    /// Which scope the "all users" toggle should reflect.
    /// **Verified quirk this row's own manifest note already flags**:
    /// this isn't "any verb is an all-users registration" — it's
    /// whichever scope the *last* matching verb in iteration order
    /// happened to be in, because `$CM_Checkbox_allusers`'s single
    /// checked/unchecked state gets unconditionally overwritten on every
    /// verb that matches *either* scope, all-users first then per-user
    /// (UniExtract.au3:7085-7096). If both scopes have stale
    /// registrations simultaneously, the per-user check silently wins
    /// the displayed toggle regardless of which verb it belongs to.
    pub all_users: bool,
    /// Per-verb checked state, in the same order as the input.
    pub checked_verbs: Vec<bool>,
}

/// Ports the per-verb scan loop shared by the simple-mode
/// (UniExtract.au3:7084-7097) and cascading-mode (UniExtract.au3:7102-
/// 7115) registration checks. `registrations[i]` is `(all_users_registered,
/// per_user_registered)` for verb `i` — both already-resolved `_RegExists`
/// results, real registry I/O the caller performs.
pub fn resolve_shell_registration_scan(registrations: &[(bool, bool)]) -> ShellRegistrationScan {
    let mut enabled = false;
    let mut all_users = false;
    let mut checked_verbs = vec![false; registrations.len()];

    for (i, &(all_users_registered, per_user_registered)) in registrations.iter().enumerate() {
        if all_users_registered {
            all_users = true;
            checked_verbs[i] = true;
            enabled = true;
        }
        if per_user_registered {
            all_users = false;
            checked_verbs[i] = true;
            enabled = true;
        }
    }

    ShellRegistrationScan {
        enabled,
        all_users,
        checked_verbs,
    }
}

/// Ports `GUI_ContextMenu_ChangePic`'s asset filename resolution
/// (UniExtract.au3:7145).
pub fn resolve_context_menu_picture_filename(cascading_checked: bool) -> &'static str {
    if cascading_checked {
        "ContextMenu_Cascading.png"
    } else {
        "ContextMenu_Simple.png"
    }
}

/// Ports the "if enabling and nothing is checked yet, check everything"
/// fallback that appears twice in the source, identically: once in
/// `GUI_ContextMenu_activate` (UniExtract.au3:7224) and again
/// independently in `GUI_ContextMenu_OK` (UniExtract.au3:7161) right
/// before writing the registry.
pub fn should_force_check_all_verbs(enabled: bool, any_verb_checked: bool) -> bool {
    enabled && !any_verb_checked
}

/// `GUI_ContextMenu_activate`'s full enable/disable cascade
/// (UniExtract.au3:7216-7234), as what each control's state *should be*
/// rather than the source's own incremental `GUICtrlSetState` deltas.
/// `None` marks a control this function doesn't touch at all, matching
/// the source's own conditionals precisely: the all-users checkboxes are
/// only ever touched `If IsAdmin()`, and the cascading radio only `If
/// _IsWin7OrNewer()` -- a non-admin or pre-Win7 run leaves those controls
/// exactly as they already were, not forced to any particular state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateResult {
    pub allusers_checkbox_enabled: Option<bool>,
    pub simple_radio_enabled: bool,
    pub verb_checkboxes_enabled: bool,
    pub force_check_all_verbs: bool,
    pub cascading_radio_enabled: Option<bool>,
    pub allusers2_checkbox_enabled: Option<bool>,
    pub add_input_enabled: bool,
}

/// Ports `GUI_ContextMenu_activate` (UniExtract.au3:7216-7234).
/// `any_verb_checked` must reflect the verb checkboxes' state *before*
/// this call, matching the source reading `_IsAnyChecked($CM_Checkbox)`
/// after already applying `bEnabled` to them but before the fallback
/// check -- since `_SetState` only ever sets *enabled/disabled*, never
/// checked/unchecked, that read is unaffected by the enable/disable step
/// immediately before it.
pub fn resolve_context_menu_activate(
    enabled_checked: bool,
    is_admin: bool,
    is_win7_or_newer: bool,
    any_verb_checked: bool,
    add_assoc_checked: bool,
) -> ActivateResult {
    ActivateResult {
        allusers_checkbox_enabled: is_admin.then_some(enabled_checked),
        simple_radio_enabled: enabled_checked,
        verb_checkboxes_enabled: enabled_checked,
        force_check_all_verbs: should_force_check_all_verbs(enabled_checked, any_verb_checked),
        cascading_radio_enabled: is_win7_or_newer.then_some(enabled_checked),
        allusers2_checkbox_enabled: if add_assoc_checked {
            is_admin.then_some(true)
        } else {
            Some(false)
        },
        add_input_enabled: add_assoc_checked,
    }
}

/// Which registry subtree a checked "Simple"/"Cascading" radio should
/// write to (`GUI_ContextMenu_OK`, UniExtract.au3:7168,7184): the
/// cascading branch additionally requires Windows 7 or newer, since
/// cascading menus aren't supported on older Windows -- `None` when
/// neither condition is met (the enabled checkbox was on, but somehow
/// neither radio's condition holds, e.g. cascading selected on an old
/// OS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationMode {
    Simple,
    Cascading,
    None,
}

/// Ports the `If _IsChecked($CM_Simple_Radio) Then ... ElseIf
/// $bIsWin7OrNewer And _IsChecked($CM_Cascading_Radio) Then ...`
/// dispatch (UniExtract.au3:7168-7184).
pub fn resolve_registration_mode(
    is_win7_or_newer: bool,
    simple_checked: bool,
    cascading_checked: bool,
) -> RegistrationMode {
    if simple_checked {
        RegistrationMode::Simple
    } else if is_win7_or_newer && cascading_checked {
        RegistrationMode::Cascading
    } else {
        RegistrationMode::None
    }
}

/// Ports the file-association input's auto-uncheck gate
/// (UniExtract.au3:7205): an empty extension list forces the "add file
/// association" checkbox off, overriding whatever it was actually set
/// to.
pub fn resolve_add_checkbox_state(raw_checked: bool, input_is_empty: bool) -> bool {
    raw_checked && !input_is_empty
}

/// Ports the confirmed-and-changed gate on actually applying a new file
/// association (UniExtract.au3:7207-7208): the dangerous-action confirm
/// must be accepted, *and* either this wasn't already enabled, or the
/// extension list genuinely changed from what's already stored --
/// re-confirming and re-applying an unchanged, already-active
/// association is a no-op.
pub fn should_apply_file_assoc_after_confirmation(
    user_confirmed_dangerous_prompt: bool,
    was_previously_enabled: bool,
    stored_assoc: &str,
    current_input: &str,
) -> bool {
    user_confirmed_dangerous_prompt && (!was_previously_enabled || stored_assoc != current_input)
}

/// Ports the `Else` branch's removal gate (UniExtract.au3:7209-7210):
/// the checkbox ending up unchecked (after [`resolve_add_checkbox_state`]'s
/// own auto-uncheck is already applied) only removes an association that
/// was actually previously enabled -- never a no-op removal.
pub fn should_remove_file_assoc(
    add_checkbox_effective_checked: bool,
    was_previously_enabled: bool,
) -> bool {
    !add_checkbox_effective_checked && was_previously_enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_all_users_only() {
        let scan = resolve_shell_registration_scan(&[(true, false)]);
        assert_eq!(
            scan,
            ShellRegistrationScan {
                enabled: true,
                all_users: true,
                checked_verbs: vec![true]
            }
        );
    }

    #[test]
    fn scan_per_user_only() {
        let scan = resolve_shell_registration_scan(&[(false, true)]);
        assert_eq!(
            scan,
            ShellRegistrationScan {
                enabled: true,
                all_users: false,
                checked_verbs: vec![true]
            }
        );
    }

    /// Parity test for the verified quirk: when a single verb is
    /// registered in *both* scopes, the per-user check (evaluated
    /// second) wins the displayed "all users" toggle.
    #[test]
    fn scan_both_scopes_registered_per_user_wins() {
        let scan = resolve_shell_registration_scan(&[(true, true)]);
        assert!(!scan.all_users);
        assert!(scan.enabled);
    }

    /// Parity test: the *last* verb (in iteration order) with any match
    /// determines the final "all users" toggle, not the first.
    #[test]
    fn scan_last_matching_verb_wins_the_scope_toggle() {
        let scan = resolve_shell_registration_scan(&[(true, false), (false, true), (true, false)]);
        assert!(scan.all_users, "last match (index 2) was all-users");
        assert_eq!(scan.checked_verbs, vec![true, true, true]);
    }

    #[test]
    fn scan_nothing_registered() {
        let scan = resolve_shell_registration_scan(&[(false, false), (false, false)]);
        assert!(!scan.enabled);
        assert!(!scan.all_users);
        assert_eq!(scan.checked_verbs, vec![false, false]);
    }

    #[test]
    fn picture_filename_matches_mode() {
        assert_eq!(
            resolve_context_menu_picture_filename(true),
            "ContextMenu_Cascading.png"
        );
        assert_eq!(
            resolve_context_menu_picture_filename(false),
            "ContextMenu_Simple.png"
        );
    }

    #[test]
    fn force_check_all_verbs_only_when_enabling_with_none_checked() {
        assert!(should_force_check_all_verbs(true, false));
        assert!(!should_force_check_all_verbs(true, true));
        assert!(!should_force_check_all_verbs(false, false));
    }

    #[test]
    fn activate_admin_touches_allusers_checkbox() {
        let result = resolve_context_menu_activate(true, true, true, true, false);
        assert_eq!(result.allusers_checkbox_enabled, Some(true));
    }

    #[test]
    fn activate_non_admin_leaves_allusers_checkbox_untouched() {
        let result = resolve_context_menu_activate(true, false, true, true, false);
        assert_eq!(result.allusers_checkbox_enabled, None);
    }

    #[test]
    fn activate_pre_win7_leaves_cascading_radio_untouched() {
        let result = resolve_context_menu_activate(true, true, false, true, false);
        assert_eq!(result.cascading_radio_enabled, None);
    }

    #[test]
    fn activate_win7_touches_cascading_radio() {
        let result = resolve_context_menu_activate(true, true, true, true, false);
        assert_eq!(result.cascading_radio_enabled, Some(true));
    }

    #[test]
    fn activate_forces_all_verbs_checked_when_enabling_with_none_checked() {
        let result = resolve_context_menu_activate(true, true, true, false, false);
        assert!(result.force_check_all_verbs);
    }

    #[test]
    fn activate_add_assoc_unchecked_disables_allusers2_and_input() {
        let result = resolve_context_menu_activate(true, true, true, true, false);
        assert_eq!(result.allusers2_checkbox_enabled, Some(false));
        assert!(!result.add_input_enabled);
    }

    #[test]
    fn activate_add_assoc_checked_admin_enables_allusers2_and_input() {
        let result = resolve_context_menu_activate(true, true, true, true, true);
        assert_eq!(result.allusers2_checkbox_enabled, Some(true));
        assert!(result.add_input_enabled);
    }

    #[test]
    fn activate_add_assoc_checked_non_admin_leaves_allusers2_untouched() {
        let result = resolve_context_menu_activate(true, false, true, true, true);
        assert_eq!(result.allusers2_checkbox_enabled, None);
        assert!(result.add_input_enabled);
    }

    #[test]
    fn registration_mode_simple_wins_when_checked() {
        assert_eq!(
            resolve_registration_mode(true, true, true),
            RegistrationMode::Simple
        );
    }

    #[test]
    fn registration_mode_cascading_requires_win7() {
        assert_eq!(
            resolve_registration_mode(true, false, true),
            RegistrationMode::Cascading
        );
        assert_eq!(
            resolve_registration_mode(false, false, true),
            RegistrationMode::None
        );
    }

    #[test]
    fn registration_mode_none_when_neither_checked() {
        assert_eq!(
            resolve_registration_mode(true, false, false),
            RegistrationMode::None
        );
    }

    #[test]
    fn add_checkbox_forced_off_when_input_empty() {
        assert!(!resolve_add_checkbox_state(true, true));
        assert!(resolve_add_checkbox_state(true, false));
        assert!(!resolve_add_checkbox_state(false, false));
    }

    #[test]
    fn apply_file_assoc_requires_confirmation() {
        assert!(!should_apply_file_assoc_after_confirmation(
            false, false, "", "zip"
        ));
    }

    #[test]
    fn apply_file_assoc_when_not_previously_enabled() {
        assert!(should_apply_file_assoc_after_confirmation(
            true, false, "", "zip"
        ));
    }

    #[test]
    fn apply_file_assoc_when_extension_list_changed() {
        assert!(should_apply_file_assoc_after_confirmation(
            true, true, "zip", "zip;rar"
        ));
    }

    /// Parity test: re-confirming an unchanged, already-enabled
    /// association is a no-op.
    #[test]
    fn apply_file_assoc_skipped_when_unchanged() {
        assert!(!should_apply_file_assoc_after_confirmation(
            true, true, "zip", "zip"
        ));
    }

    #[test]
    fn remove_file_assoc_only_when_was_enabled() {
        assert!(should_remove_file_assoc(false, true));
        assert!(!should_remove_file_assoc(false, false));
        assert!(!should_remove_file_assoc(true, true));
    }
}
