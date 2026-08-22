//! Parses an AutoIt `ControlSend`/`Send` key-sequence string (e.g.
//! `"{DOWN}{DOWN}{RIGHT}{ENTER}"`) into a token list a [`super::GuiAutomation`]
//! backend can replay as real keystrokes.
//!
//! Scoped to exactly the tokens this port's own cited call sites actually
//! use — `{DOWN}`, `{RIGHT}`, `{ENTER}` — verified by grepping every
//! `RipExeInfo(...)` key-sequence argument in the source
//! (`mscf::RIP_EXEINFO_KEY_SEQUENCE`, `wise::RIP_EXEINFO_KEY_SEQUENCE`, and
//! `Case $TYPE_SWFEXE`'s own literal). AutoIt's full `Send` syntax supports
//! many more brace tokens (`{F1}`, `{ALT}`, `{SHIFT down}`, etc.) — not
//! implemented here, since nothing this port has ported so far needs them;
//! an unrecognized brace token falls back to sending its contents as plain
//! literal characters rather than panicking or silently dropping it.

/// One keystroke this port knows how to replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyToken {
    Down,
    Right,
    Enter,
    /// A plain character, sent as typed text (not a special key).
    Literal(char),
}

/// Parses `keys` into a sequence of [`KeyToken`]s, left to right.
pub fn parse_key_sequence(keys: &str) -> Vec<KeyToken> {
    let mut tokens = Vec::new();
    let mut chars = keys.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '{' {
            tokens.push(KeyToken::Literal(c));
            continue;
        }
        let mut name = String::new();
        let mut closed = false;
        for c2 in chars.by_ref() {
            if c2 == '}' {
                closed = true;
                break;
            }
            name.push(c2);
        }
        if !closed {
            // Unterminated brace: reproduce what was collected literally.
            tokens.push(KeyToken::Literal('{'));
            for ch in name.chars() {
                tokens.push(KeyToken::Literal(ch));
            }
            continue;
        }
        match name.as_str() {
            "DOWN" => tokens.push(KeyToken::Down),
            "RIGHT" => tokens.push(KeyToken::Right),
            "ENTER" => tokens.push(KeyToken::Enter),
            other => {
                for ch in other.chars() {
                    tokens.push(KeyToken::Literal(ch));
                }
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mscf_key_sequence() {
        assert_eq!(
            parse_key_sequence(crate::extract::mscf::RIP_EXEINFO_KEY_SEQUENCE),
            vec![
                KeyToken::Down,
                KeyToken::Down,
                KeyToken::Down,
                KeyToken::Down,
                KeyToken::Down,
                KeyToken::Right,
                KeyToken::Down,
                KeyToken::Down,
                KeyToken::Down,
            ]
        );
    }

    #[test]
    fn parses_wise_key_sequence() {
        assert_eq!(
            parse_key_sequence(crate::extract::wise::RIP_EXEINFO_KEY_SEQUENCE),
            vec![KeyToken::Down, KeyToken::Down, KeyToken::Down]
        );
    }

    #[test]
    fn parses_enter_suffix() {
        assert_eq!(
            parse_key_sequence("{DOWN}{ENTER}"),
            vec![KeyToken::Down, KeyToken::Enter]
        );
    }

    #[test]
    fn literal_characters_pass_through() {
        assert_eq!(
            parse_key_sequence("ab"),
            vec![KeyToken::Literal('a'), KeyToken::Literal('b')]
        );
    }

    #[test]
    fn unrecognized_brace_token_falls_back_to_literal_characters() {
        assert_eq!(
            parse_key_sequence("{F1}"),
            vec![KeyToken::Literal('F'), KeyToken::Literal('1'),]
        );
    }

    #[test]
    fn unterminated_brace_is_reproduced_literally() {
        assert_eq!(
            parse_key_sequence("{DOWN"),
            vec![
                KeyToken::Literal('{'),
                KeyToken::Literal('D'),
                KeyToken::Literal('O'),
                KeyToken::Literal('W'),
                KeyToken::Literal('N'),
            ]
        );
    }
}
