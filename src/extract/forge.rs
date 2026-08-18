//! InstallForge (wraps `7z.exe` + base64 path-rename logic).

use super::{Invocation, WindowMode};

/// Builds the conditional inner-archive unpack invocation
/// UniExtract2's `Case $TYPE_FORGE` (UniExtract.au3:2546) makes when the
/// installer's primary 7-Zip extraction produced a gzip-compressed inner
/// archive (`FileExists($tempoutdir & $filename)`): `<program> x "<tmp>"`,
/// run in `tempoutdir` with the window hidden.
///
/// **Not modeled here:** the recursive `extract($TYPE_7Z, -1, "", True,
/// False)` dispatch that runs first (UniExtract.au3:2543) — composite/
/// recursive dispatch, capability C054, not yet ported — nor the
/// `FileExists`/`_FileDelete` staging around this call, nor the trailing
/// `RenameBase64PathNames`/`MoveFiles` steps (see [`decide_rename`] for
/// the base64-rename half of this capability). All separate runtime
/// behavior, not part of building this one invocation.
pub fn inner_archive_invocation(program: &str, tmp: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), tmp.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// What `RenameBase64PathNames` (UniExtract.au3:3997-4023) does with one
/// directory-listing entry, as a pure decision over its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameDecision {
    /// `$sName == "empty.empty"` (UniExtract.au3:4008-4011): a
    /// placeholder InstallForge leaves for an otherwise-empty directory —
    /// delete it, nothing to rename.
    Delete,
    /// The name decoded successfully — rename to this decoded name.
    Rename(String),
    /// The name isn't `"empty.empty"` and doesn't decode as base64 — the
    /// source's `_Base64Decode` call sets `@error` and the loop
    /// `ContinueLoop`s, leaving the entry untouched.
    Skip,
}

/// Ports `RenameBase64PathNames`'s per-entry decision (UniExtract.au3:
/// 4006-4018) as a pure function of the entry's name — the actual
/// directory listing, deletion, and rename/move calls (`_FileListToArray`,
/// `_FileDelete`, `MovePath`, and the recursion into renamed
/// subdirectories) are the caller's job, the same "decision vs. I/O"
/// boundary `outdir::decide_outdir_outcome` already draws.
pub fn decide_rename(name: &str) -> RenameDecision {
    if name == "empty.empty" {
        return RenameDecision::Delete;
    }
    match base64_decode_utf16le(name) {
        Some(new_name) => RenameDecision::Rename(new_name),
        None => RenameDecision::Skip,
    }
}

/// Ports AutoIt's `_Base64Decode($sInput)` (UniExtract.au3:4728-4750) the
/// way `RenameBase64PathNames` calls it — no explicit `$eEncoding`
/// argument, so it takes the function's own default, `$SB_UTF16LE`:
/// decode `s` as standard base64 (matching `Crypt32.dll`'s
/// `CryptStringToBinary` with the `CRYPT_STRING_BASE64` flag this
/// function uses), then interpret the decoded bytes as UTF-16LE text.
///
/// Returns `None` where the source's `_Base64Decode` sets `@error` (an
/// empty input is the one exception — the source returns `""` for that
/// case directly, without attempting to decode anything, reproduced here
/// as `Some(String::new())`). An odd number of decoded bytes has no
/// direct source equivalent to point to (`BinaryToString` doesn't itself
/// error on this), but can't represent whole UTF-16 code units, so it's
/// treated as a decode failure here rather than silently truncating.
pub fn base64_decode_utf16le(s: &str) -> Option<String> {
    let bytes = base64_decode_bytes(s)?;
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(String::from_utf16_lossy(&units))
}

fn base64_value(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Hand-rolled standard base64 (RFC 4648) decoder, matching
/// `CryptStringToBinary`'s `CRYPT_STRING_BASE64` flag's tolerance of
/// embedded whitespace. No `base64` crate dependency — this migration's
/// no-new-dependency policy (stop-and-ask) makes hand-rolling the smaller
/// cost here, the same reasoning already used for `batch`'s multipart-
/// archive pattern matching (C147).
fn base64_decode_bytes(s: &str) -> Option<Vec<u8>> {
    let filtered: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if filtered.is_empty() {
        return Some(Vec::new());
    }
    if !filtered.len().is_multiple_of(4) {
        return None;
    }

    let chunks: Vec<&[u8]> = filtered.chunks_exact(4).collect();
    let last = chunks.len() - 1;
    let mut out = Vec::with_capacity(chunks.len() * 3);

    for (i, chunk) in chunks.iter().enumerate() {
        let pad = chunk.iter().filter(|&&b| b == b'=').count();
        if pad > 2 || (pad > 0 && i != last) {
            return None;
        }
        if chunk[..4 - pad].contains(&b'=') {
            return None;
        }

        let mut vals = [0u8; 4];
        for (j, &b) in chunk.iter().enumerate() {
            vals[j] = if b == b'=' { 0 } else { base64_value(b)? };
        }
        let n = (vals[0] as u32) << 18
            | (vals[1] as u32) << 12
            | (vals[2] as u32) << 6
            | (vals[3] as u32);
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C119: the inner-archive invocation
    /// matches UniExtract.au3:2546's effective `7z.exe x "<tmp>"` call.
    #[test]
    fn inner_archive_invocation_matches_source() {
        let inv = inner_archive_invocation(
            r"C:\UniExtract\bin\7z.exe",
            r"C:\Temp\installer_tmp\payload.gz",
            r"C:\Temp\installer_tmp",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\Temp\installer_tmp\payload.gz".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\Temp\installer_tmp");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test: the base64 decoder matches the well-known RFC 4648
    /// test vector "Man" -> "TWFu" at the raw-byte level.
    #[test]
    fn base64_decode_bytes_matches_rfc4648_vector() {
        assert_eq!(base64_decode_bytes("TWFu"), Some(vec![b'M', b'a', b'n']));
    }

    /// Parity test: decoding a base64 string of UTF-16LE bytes reproduces
    /// the original text, matching `_Base64Decode`'s default
    /// `$SB_UTF16LE` encoding. "RABhAHQAYQA=" is the base64 encoding of
    /// "Data"'s UTF-16LE bytes (44 00 61 00 74 00 61 00).
    #[test]
    fn base64_decode_utf16le_decodes_utf16le_text() {
        assert_eq!(
            base64_decode_utf16le("RABhAHQAYQA="),
            Some("Data".to_string())
        );
    }

    /// Parity test: an empty input decodes to an empty string, matching
    /// `_Base64Decode`'s own `If $sInput == "" Then Return ""` early exit.
    #[test]
    fn base64_decode_utf16le_empty_input_returns_empty_string() {
        assert_eq!(base64_decode_utf16le(""), Some(String::new()));
    }

    /// Parity test: invalid base64 input fails to decode, matching
    /// `_Base64Decode`'s `@error` path.
    #[test]
    fn base64_decode_utf16le_rejects_invalid_input() {
        assert_eq!(base64_decode_utf16le("not valid base64!!"), None);
    }

    /// Parity test for capability C119: `RenameBase64PathNames`'s
    /// "empty.empty" marker is recognized as a delete, not a rename.
    #[test]
    fn decide_rename_recognizes_empty_marker() {
        assert_eq!(decide_rename("empty.empty"), RenameDecision::Delete);
    }

    /// Parity test for capability C119: a valid base64-encoded name
    /// decodes to a rename decision.
    #[test]
    fn decide_rename_decodes_valid_base64_name() {
        assert_eq!(
            decide_rename("RABhAHQAYQA="),
            RenameDecision::Rename("Data".to_string())
        );
    }

    /// Parity test for capability C119: a name that isn't the empty
    /// marker and doesn't decode as base64 is skipped, matching the
    /// source's `ContinueLoop` on `@error`.
    #[test]
    fn decide_rename_skips_undecodable_name() {
        assert_eq!(decide_rename("not valid base64!!"), RenameDecision::Skip);
    }
}
