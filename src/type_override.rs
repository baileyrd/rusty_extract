//! `/type[=value]` override routing: ports `ParseCommandLine()`'s
//! `$cmdline[3]`-driven block (C006, UniExtract.au3:652-682,420 per
//! `capability-manifest.md`):
//!
//! ```autoit
//! If $iArgs > 2 And StringLeft($cmdline[3], 5) = "/type" Then
//!     Local $aReturn = _FileListToArray($defdir, "*.ini", 1)
//!     ; ... (build the list of valid type names: def/*.ini stems, minus
//!     ; registry.ini, plus the hardcoded $aExtractionTypes, deduped and
//!     ; sorted) ...
//!     $sArcTypeOverride = StringTrimLeft($cmdline[3], 6)
//!     If StringLen($sArcTypeOverride) > 0 Then
//!         If _ArrayBinarySearch($aReturn, $sArcTypeOverride) < 0 Then
//!             Local $tmp = StringRight($sArcTypeOverride, 1)
//!             If StringIsInt($tmp) Then $sMethodSelectOverride = ""
//!             While StringLen($sArcTypeOverride) > 0 And StringIsInt($tmp)
//!                 $sMethodSelectOverride = $tmp & $sMethodSelectOverride
//!                 $sArcTypeOverride = StringTrimRight($sArcTypeOverride, 1)
//!                 $tmp = StringRight($sArcTypeOverride, 1)
//!             WEnd
//!         EndIf
//!     Else
//!         $sArcTypeOverride = GUI_MethodSelectList($aReturn, ...)
//!         If $sArcTypeOverride < 0 Then terminate($STATUS_SILENT)
//!     EndIf
//! EndIf
//! ```
//!
//! This module ports the routing/parsing decision only — building the
//! candidate type-name list involves real filesystem I/O
//! (`_FileListToArray($defdir, "*.ini", ...)`), so [`parse_type_override`]
//! takes it as a caller-supplied `known_types` slice, the same seam
//! `plugin::resolve_plugin_ini_with` uses for its own existence check.
//! `GUI_MethodSelectList` itself (the candidate-list prompt) is the
//! deferred GUI subsystem, manifest row D001 — [`TypeOverride::PromptForType`]
//! only signals that this routing reaches that branch.
//!
//! `StringLeft($cmdline[3], 5) = "/type"` is case-insensitive by this
//! script's default `StringCompareMode`, the same rule already documented
//! on `cli`'s and `dest_arg`'s module doc comments.

/// C006: how the `/type[=value]` override argument (`$cmdline[3]`)
/// routes, ported from the block quoted in the module doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeOverride {
    /// No third positional argument, or it doesn't start with `/type`
    /// (case-insensitive): no override in effect.
    None,
    /// A value was given after `=`. Passes through unchanged — either
    /// because it's a recognized type name, or because it isn't but has
    /// no trailing-digit suffix to peel off (see
    /// [`ArcTypeWithMethodSelect`](TypeOverride::ArcTypeWithMethodSelect)).
    /// The source never touches `$sMethodSelectOverride` in this branch,
    /// so a caller should leave its own method-select state at whatever
    /// default already applies.
    ArcType(String),
    /// The full value wasn't a recognized type name, and its trailing
    /// characters are ASCII digits — unconditionally peeled off as
    /// `method_select` (a candidate-method index, feeding C053's
    /// disambiguation) regardless of whether the remaining `arctype`
    /// prefix is itself recognized. **Deliberately preserved as written**:
    /// the source never re-validates the peeled remainder against the
    /// known-types list, so this port doesn't either.
    ArcTypeWithMethodSelect {
        arctype: String,
        method_select: String,
    },
    /// Bare `/type` with nothing after `=`: the source presents a GUI
    /// candidate list (`GUI_MethodSelectList`) — deferred GUI subsystem,
    /// manifest row D001. Cancelling that prompt terminates silently
    /// (`$STATUS_SILENT`, exit code 0); building and driving the prompt
    /// itself is out of scope for this row.
    PromptForType,
}

/// Routes `third_arg` (`$cmdline[3]`, absent when `$iArgs <= 2`) the same
/// way the source's `/type` block does. `known_types` is the
/// caller-built candidate list (def/*.ini stems plus the hardcoded
/// extraction types) that a real value is checked against;
/// case-insensitive, matching `_ArrayBinarySearch`'s default comparison
/// mode.
pub fn parse_type_override(third_arg: Option<&str>, known_types: &[&str]) -> TypeOverride {
    let Some(arg) = third_arg else {
        return TypeOverride::None;
    };
    let Some(prefix) = arg.get(..5) else {
        return TypeOverride::None;
    };
    if !prefix.eq_ignore_ascii_case("/type") {
        return TypeOverride::None;
    }

    // StringTrimLeft($cmdline[3], 6): everything after "/type=" (6 chars).
    let value = arg.get(6..).unwrap_or("");
    if value.is_empty() {
        return TypeOverride::PromptForType;
    }

    if known_types.iter().any(|t| t.eq_ignore_ascii_case(value)) {
        return TypeOverride::ArcType(value.to_string());
    }

    let arctype = value.trim_end_matches(|c: char| c.is_ascii_digit());
    if arctype.len() == value.len() {
        // No trailing digit at all -- nothing to peel.
        return TypeOverride::ArcType(value.to_string());
    }
    TypeOverride::ArcTypeWithMethodSelect {
        arctype: arctype.to_string(),
        method_select: value[arctype.len()..].to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: &[&str] = &["ace", "kgb", "rar", "inno2"];

    /// Parity test: no third argument at all, or one that isn't `/type`.
    #[test]
    fn none_when_absent_or_not_a_type_flag() {
        assert_eq!(parse_type_override(None, KNOWN), TypeOverride::None);
        assert_eq!(
            parse_type_override(Some("archive.zip"), KNOWN),
            TypeOverride::None
        );
        assert_eq!(parse_type_override(Some("/typ"), KNOWN), TypeOverride::None);
    }

    /// Parity test for capability C006: bare `/type` with no value routes
    /// to the GUI candidate-list prompt.
    #[test]
    fn bare_type_flag_prompts() {
        assert_eq!(
            parse_type_override(Some("/type"), KNOWN),
            TypeOverride::PromptForType
        );
        assert_eq!(
            parse_type_override(Some("/type="), KNOWN),
            TypeOverride::PromptForType
        );
    }

    /// Parity test for capability C006: `/TYPE=` (any case) is recognized
    /// the same as `/type=`.
    #[test]
    fn type_flag_is_case_insensitive() {
        assert_eq!(
            parse_type_override(Some("/TYPE=ace"), KNOWN),
            TypeOverride::ArcType("ace".to_string())
        );
    }

    /// Parity test for capability C006: a recognized type name passes
    /// through unchanged, even when it ends in a digit that could
    /// otherwise be peeled (the known-types check runs first).
    #[test]
    fn recognized_type_passes_through_even_with_trailing_digit() {
        assert_eq!(
            parse_type_override(Some("/type=inno2"), KNOWN),
            TypeOverride::ArcType("inno2".to_string())
        );
    }

    /// Parity test for capability C006: an unrecognized value with no
    /// trailing digit passes through unchanged too.
    #[test]
    fn unrecognized_type_with_no_trailing_digit_passes_through() {
        assert_eq!(
            parse_type_override(Some("/type=unknownformat"), KNOWN),
            TypeOverride::ArcType("unknownformat".to_string())
        );
    }

    /// Parity test for capability C006: an unrecognized value with a
    /// trailing digit run has it peeled into `method_select`, without
    /// re-checking whether the remaining prefix is itself recognized —
    /// `"kgb2"` peels to `arctype: "kgb"` even though `"kgb"` happens to
    /// be known; `"unknownformat3"` peels to an unrecognized prefix too,
    /// matching the source's unconditional peel.
    #[test]
    fn unrecognized_type_with_trailing_digits_peels_method_select() {
        assert_eq!(
            parse_type_override(Some("/type=kgb2"), KNOWN),
            TypeOverride::ArcTypeWithMethodSelect {
                arctype: "kgb".to_string(),
                method_select: "2".to_string(),
            }
        );
        assert_eq!(
            parse_type_override(Some("/type=unknownformat3"), KNOWN),
            TypeOverride::ArcTypeWithMethodSelect {
                arctype: "unknownformat".to_string(),
                method_select: "3".to_string(),
            }
        );
        assert_eq!(
            parse_type_override(Some("/type=foo123"), KNOWN),
            TypeOverride::ArcTypeWithMethodSelect {
                arctype: "foo".to_string(),
                method_select: "123".to_string(),
            }
        );
    }
}
