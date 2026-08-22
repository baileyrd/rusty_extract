//! Parses the two `$sControl` spellings this port's cited call sites
//! actually use, into one common shape a [`super::GuiAutomation`] backend
//! can resolve against real child windows: AutoIt's advanced
//! `[CLASS:name; INSTANCE:n]` syntax, and the plain `ClassNameNN` shorthand
//! (e.g. `"TEdit5"`, `"TListBox1"` — the control's class name with its
//! 1-indexed instance number appended, the same convention AutoIt's Window
//! Info tool prints and the two literal `$sControl` strings in
//! `RipExeInfo`/the Ghost-Installer call site use). Both spellings name the
//! same thing: "the `instance`-th child window whose class name is exactly
//! `class_name`".
//!
//! Not a general AutoIt control-spec parser — `[TEXT:...]`, `[REGEXPCLASS:
//! ...]`, `[ID:...]`, and every other advanced-syntax key are out of scope,
//! since nothing this port has ported so far uses them.

/// A resolved control identifier: match the `instance`-th child window
/// (1-indexed, in enumeration order) whose class name is exactly
/// `class_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlSpec {
    pub class_name: String,
    pub instance: u32,
}

/// Parses `spec`, or `None` if it's empty or an advanced-syntax spec with
/// no `CLASS:` key (this port only ever needs to resolve by class).
pub fn parse_control_spec(spec: &str) -> Option<ControlSpec> {
    let trimmed = spec.trim();
    if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return parse_advanced_spec(inner);
    }
    parse_shorthand_spec(trimmed)
}

/// `"CLASS:TBitBtn; INSTANCE:16"` -> `{class_name: "TBitBtn", instance: 16}`.
/// `INSTANCE` defaults to `1` when absent, matching AutoIt's own default.
fn parse_advanced_spec(inner: &str) -> Option<ControlSpec> {
    let mut class_name = None;
    let mut instance = 1u32;
    for part in inner.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("CLASS:") {
            class_name = Some(v.trim().to_string());
        } else if let Some(v) = part.strip_prefix("INSTANCE:") {
            instance = v.trim().parse().unwrap_or(1);
        }
    }
    class_name.map(|class_name| ControlSpec {
        class_name,
        instance,
    })
}

/// `"TEdit5"` -> `{class_name: "TEdit", instance: 5}`; `"TListBox1"` ->
/// `{class_name: "TListBox", instance: 1}`; no trailing digits defaults to
/// instance `1`.
fn parse_shorthand_spec(trimmed: &str) -> Option<ControlSpec> {
    if trimmed.is_empty() {
        return None;
    }
    let digit_count = trimmed
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .count();
    let split_at = trimmed.len() - digit_count;
    let (class_part, digits) = trimmed.split_at(split_at);
    if class_part.is_empty() {
        return None;
    }
    let instance = if digits.is_empty() {
        1
    } else {
        digits.parse().unwrap_or(1)
    };
    Some(ControlSpec {
        class_name: class_part.to_string(),
        instance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_advanced_spec_with_instance() {
        assert_eq!(
            parse_control_spec("[CLASS:TBitBtn; INSTANCE:16]"),
            Some(ControlSpec {
                class_name: "TBitBtn".to_string(),
                instance: 16,
            })
        );
    }

    #[test]
    fn parses_advanced_spec_without_instance_defaults_to_one() {
        assert_eq!(
            parse_control_spec("[CLASS:TSViewer]"),
            Some(ControlSpec {
                class_name: "TSViewer".to_string(),
                instance: 1,
            })
        );
    }

    #[test]
    fn parses_shorthand_spec_with_trailing_digits() {
        assert_eq!(
            parse_control_spec("TEdit5"),
            Some(ControlSpec {
                class_name: "TEdit".to_string(),
                instance: 5,
            })
        );
        assert_eq!(
            parse_control_spec("TListBox1"),
            Some(ControlSpec {
                class_name: "TListBox".to_string(),
                instance: 1,
            })
        );
    }

    #[test]
    fn parses_shorthand_spec_without_trailing_digits_defaults_to_one() {
        assert_eq!(
            parse_control_spec("TBitBtn"),
            Some(ControlSpec {
                class_name: "TBitBtn".to_string(),
                instance: 1,
            })
        );
    }

    #[test]
    fn empty_spec_is_none() {
        assert_eq!(parse_control_spec(""), None);
        assert_eq!(parse_control_spec("   "), None);
    }

    #[test]
    fn advanced_spec_without_class_key_is_none() {
        assert_eq!(parse_control_spec("[INSTANCE:2]"), None);
    }
}
