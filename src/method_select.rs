//! Manual disambiguation: when automatic testing can't pick a single
//! extractor for a format, `GUI_MethodSelect` presents the candidates for
//! the user to choose — the GUI dialog itself is deferred (manifest row
//! D001), but the *data* (which candidates exist, per format) and the
//! *selection policy* (override wins, then silent-mode auto-pick, then
//! the dialog) are core and ported here.
//!
//! ```autoit
//! Func GUI_MethodSelect($aData, $arcdisp)
//!     If $sMethodSelectOverride > 0 Then
//!         Cout("Method select override active, selected choice " & $sMethodSelectOverride)
//!         Return $sMethodSelectOverride
//!     EndIf
//!
//!     ; Auto choose first extraction method in silent mode
//!     If $silentmode Then
//!         Cout("Extractor selected automatically - run again in normal mode if not extracted correctly")
//!         Return 1
//!     EndIf
//!     ; ... GUI dialog (out of scope, D001) ...
//! EndFunc
//! ```
//!
//! `$sMethodSelectOverride` (default `0`) is only ever a run of digits
//! (never re-set to a bare `""`, per `$cmdline[3]`'s `/type` parsing) or
//! that default — the same value `type_override::TypeOverride::
//! ArcTypeWithMethodSelect.method_select` already carries once C006 peels
//! it off. `None` here means the default `0` (no override in effect);
//! `Some(digits)` means C006 peeled a method-select suffix.
//!
//! **The five call sites**, each `$aData`/`$aOptions` verbatim except for
//! dropping element `0` (the header-only string, never a selectable
//! choice) and each candidate's localized label text — only the
//! `t(radio_label_key, method_key)` pair survives, since the label text
//! itself is translation, not data:
//!
//! ```autoit
//! ; $TYPE_ISCAB (UniExtract.au3:2663)
//! ["InstallShield Cabinet " & t('TERM_ARCHIVE'), t('METHOD_EXTRACTION_RADIO', "is6comp"), t('METHOD_EXTRACTION_RADIO', "is5comp"), t('METHOD_EXTRACTION_RADIO', "iscab")]
//!
//! ; $TYPE_ISEXE (UniExtract.au3:2705)
//! ["InstallShield " & t('TERM_INSTALLER'), t('METHOD_EXTRACTION_RADIO', 'isxunpack'), t('METHOD_SWITCH_RADIO', 'InstallShield /b'), t('METHOD_NOT_INSTALLER_RADIO', "InstallShield")]
//!
//! ; MSI, lessmsi-failure fallback (UniExtract.au3:2854)
//! ['MSI ' & t('TERM_INSTALLER'), t('METHOD_EXTRACTION_RADIO', 'jsMSI Unpacker'), t('METHOD_EXTRACTION_RADIO', 'MsiX'), t('METHOD_EXTRACTION_RADIO', 'MSI TC Packer'), t('METHOD_ADMIN_RADIO', 'MSI')]
//!
//! ; $TYPE_MSP (UniExtract.au3:2893)
//! ["MSP " & t('TERM_PACKAGE'), t('METHOD_EXTRACTION_RADIO', "7-Zip"), t('METHOD_EXTRACTION_RADIO', "MSI TC Packer"), t('METHOD_EXTRACTION_RADIO', "MsiX")]
//!
//! ; $TYPE_WISE, Wise-installer failure fallback (UniExtract.au3:3337)
//! ['Wise ' & t('TERM_INSTALLER'), t('METHOD_UNPACKER_RADIO', 'Wise UNpacker'), t('METHOD_SWITCH_RADIO', 'Wise Installer /x'), t('METHOD_EXTRACTION_RADIO', 'Wise MSI'), t('METHOD_EXTRACTION_RADIO', 'Unzip'), t('METHOD_NOT_INSTALLER_RADIO', "Wise")]
//! ```

/// One candidate in a disambiguation list: the AutoIt `t(...)` call's own
/// two arguments, kept as raw data rather than resolved to a localized
/// string — `radio_label_key` is the radio-button's label-format key
/// (`METHOD_EXTRACTION_RADIO`/`METHOD_SWITCH_RADIO`/`METHOD_ADMIN_RADIO`/
/// `METHOD_UNPACKER_RADIO`/`METHOD_NOT_INSTALLER_RADIO`), `method_key` is
/// the specific method/tool name substituted into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodCandidate {
    pub radio_label_key: &'static str,
    pub method_key: &'static str,
}

const fn candidate(radio_label_key: &'static str, method_key: &'static str) -> MethodCandidate {
    MethodCandidate {
        radio_label_key,
        method_key,
    }
}

/// `$TYPE_ISCAB`'s candidate list (UniExtract.au3:2663), 1-indexed by
/// `$iChoice` starting at `ISCAB_CANDIDATES[0]` for choice `1`.
pub const ISCAB_CANDIDATES: &[MethodCandidate] = &[
    candidate("METHOD_EXTRACTION_RADIO", "is6comp"),
    candidate("METHOD_EXTRACTION_RADIO", "is5comp"),
    candidate("METHOD_EXTRACTION_RADIO", "iscab"),
];

/// `$TYPE_ISEXE`'s candidate list (UniExtract.au3:2705).
pub const ISEXE_CANDIDATES: &[MethodCandidate] = &[
    candidate("METHOD_EXTRACTION_RADIO", "isxunpack"),
    candidate("METHOD_SWITCH_RADIO", "InstallShield /b"),
    candidate("METHOD_NOT_INSTALLER_RADIO", "InstallShield"),
];

/// The MSI lessmsi-failure fallback candidate list (UniExtract.au3:2854).
pub const MSI_CANDIDATES: &[MethodCandidate] = &[
    candidate("METHOD_EXTRACTION_RADIO", "jsMSI Unpacker"),
    candidate("METHOD_EXTRACTION_RADIO", "MsiX"),
    candidate("METHOD_EXTRACTION_RADIO", "MSI TC Packer"),
    candidate("METHOD_ADMIN_RADIO", "MSI"),
];

/// `$TYPE_MSP`'s candidate list (UniExtract.au3:2893).
pub const MSP_CANDIDATES: &[MethodCandidate] = &[
    candidate("METHOD_EXTRACTION_RADIO", "7-Zip"),
    candidate("METHOD_EXTRACTION_RADIO", "MSI TC Packer"),
    candidate("METHOD_EXTRACTION_RADIO", "MsiX"),
];

/// The `$TYPE_WISE` Wise-installer-failure fallback candidate list
/// (UniExtract.au3:3337).
pub const WISE_CANDIDATES: &[MethodCandidate] = &[
    candidate("METHOD_UNPACKER_RADIO", "Wise UNpacker"),
    candidate("METHOD_SWITCH_RADIO", "Wise Installer /x"),
    candidate("METHOD_EXTRACTION_RADIO", "Wise MSI"),
    candidate("METHOD_EXTRACTION_RADIO", "Unzip"),
    candidate("METHOD_NOT_INSTALLER_RADIO", "Wise"),
];

/// What `GUI_MethodSelect` resolves to before ever reaching its GUI
/// dialog (UniExtract.au3:7499-7508).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodSelectionOutcome {
    /// `$sMethodSelectOverride > 0` — the override wins outright,
    /// carrying its 1-indexed choice number.
    Overridden(u32),
    /// No override, running silently — auto-picks choice `1`.
    AutoFirstCandidate,
    /// No override, running interactively — the radio-button GUI dialog,
    /// out of scope here (deferred GUI subsystem, manifest row D001).
    PromptInteractive,
}

/// Ports `GUI_MethodSelect`'s pre-dialog branch selection
/// (UniExtract.au3:7500-7508). `method_select_override` is `None` for
/// `$sMethodSelectOverride`'s default `0` (no override), or the peeled
/// digit string `type_override::TypeOverride::ArcTypeWithMethodSelect`
/// carries once C006 has parsed it. A `Some` value that fails to parse
/// as a positive `u32`, or parses to `0`, reproduces the source's
/// `$sMethodSelectOverride > 0` check being false the same way an
/// all-zero digit run (e.g. `"00"`) would in its own numeric coercion.
pub fn decide_method_selection(
    method_select_override: Option<&str>,
    silent_mode: bool,
) -> MethodSelectionOutcome {
    if let Some(digits) = method_select_override {
        if let Ok(choice) = digits.parse::<u32>() {
            if choice > 0 {
                return MethodSelectionOutcome::Overridden(choice);
            }
        }
    }
    if silent_mode {
        MethodSelectionOutcome::AutoFirstCandidate
    } else {
        MethodSelectionOutcome::PromptInteractive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C053: an active override
    /// (`$sMethodSelectOverride > 0`) wins regardless of silent mode.
    #[test]
    fn override_wins_regardless_of_silent_mode() {
        assert_eq!(
            decide_method_selection(Some("2"), false),
            MethodSelectionOutcome::Overridden(2)
        );
        assert_eq!(
            decide_method_selection(Some("2"), true),
            MethodSelectionOutcome::Overridden(2)
        );
    }

    /// Parity test for capability C053: no override, silent mode
    /// auto-picks choice `1`.
    #[test]
    fn silent_mode_without_override_auto_picks_first() {
        assert_eq!(
            decide_method_selection(None, true),
            MethodSelectionOutcome::AutoFirstCandidate
        );
    }

    /// Parity test for capability C053: no override, interactive mode
    /// reaches the (unimplemented) GUI dialog.
    #[test]
    fn interactive_mode_without_override_prompts() {
        assert_eq!(
            decide_method_selection(None, false),
            MethodSelectionOutcome::PromptInteractive
        );
    }

    /// Parity test for capability C053: an all-zero-digit override
    /// (`"00"`) reproduces `$sMethodSelectOverride > 0` being false, the
    /// same as no override at all.
    #[test]
    fn all_zero_override_is_treated_as_unset() {
        assert_eq!(
            decide_method_selection(Some("00"), true),
            MethodSelectionOutcome::AutoFirstCandidate
        );
        assert_eq!(
            decide_method_selection(Some("00"), false),
            MethodSelectionOutcome::PromptInteractive
        );
    }

    /// Parity test for capability C053: the five candidate lists match
    /// the source verbatim, in order.
    #[test]
    fn candidate_lists_match_source() {
        assert_eq!(
            ISCAB_CANDIDATES,
            &[
                candidate("METHOD_EXTRACTION_RADIO", "is6comp"),
                candidate("METHOD_EXTRACTION_RADIO", "is5comp"),
                candidate("METHOD_EXTRACTION_RADIO", "iscab"),
            ]
        );
        assert_eq!(
            ISEXE_CANDIDATES,
            &[
                candidate("METHOD_EXTRACTION_RADIO", "isxunpack"),
                candidate("METHOD_SWITCH_RADIO", "InstallShield /b"),
                candidate("METHOD_NOT_INSTALLER_RADIO", "InstallShield"),
            ]
        );
        assert_eq!(
            MSI_CANDIDATES,
            &[
                candidate("METHOD_EXTRACTION_RADIO", "jsMSI Unpacker"),
                candidate("METHOD_EXTRACTION_RADIO", "MsiX"),
                candidate("METHOD_EXTRACTION_RADIO", "MSI TC Packer"),
                candidate("METHOD_ADMIN_RADIO", "MSI"),
            ]
        );
        assert_eq!(
            MSP_CANDIDATES,
            &[
                candidate("METHOD_EXTRACTION_RADIO", "7-Zip"),
                candidate("METHOD_EXTRACTION_RADIO", "MSI TC Packer"),
                candidate("METHOD_EXTRACTION_RADIO", "MsiX"),
            ]
        );
        assert_eq!(
            WISE_CANDIDATES,
            &[
                candidate("METHOD_UNPACKER_RADIO", "Wise UNpacker"),
                candidate("METHOD_SWITCH_RADIO", "Wise Installer /x"),
                candidate("METHOD_EXTRACTION_RADIO", "Wise MSI"),
                candidate("METHOD_EXTRACTION_RADIO", "Unzip"),
                candidate("METHOD_NOT_INSTALLER_RADIO", "Wise"),
            ]
        );
    }
}
