//! MediaInfo scan — informational only, used in scan-only mode display,
//! never drives extraction dispatch (`FileScan_MediaInfo`,
//! UniExtract.au3:1054-1093) — capability C045.
//!
//! ```autoit
//! Func FileScan_MediaInfo()
//!     Local $sFileType = ""
//!     ; ... MediaInfo_New/Open/Inform/Delete via DllCall(MediaInfo.dll) ...
//!     ; $aReturn[0] now holds MediaInfo_Inform's raw CRLF-delimited text
//!
//!     ; Return if file is not a media file
//!     $aReturn = StringSplit($aReturn[0], @CRLF, 2)
//!     If UBound($aReturn) < 10 Then Return _DeleteTrayMessageBox()
//!
//!     ; Format returned string to align in message box
//!     For $i in $aReturn
//!         Local $aSplit = StringSplit($i, " : ", 2+1)
//!         If @error Then
//!             If Not StringIsSpace($i) Then $sFileType &= @CRLF & "[" & $i & "]" & @CRLF
//!             ContinueLoop
//!         EndIf
//!         $sType = StringStripWS($aSplit[0], 4+2+1)
//!         If $sType == "Complete name" Then ContinueLoop
//!         $sFileType &= StringFormat("%-24s%s\r\n", $sType, StringStripWS($aSplit[1], 4+2+1))
//!     Next
//!
//!     _FiletypeAdd("MediaInfo", $sFileType)
//!     _DeleteTrayMessageBox()
//! EndFunc
//! ```
//!
//! **The DLL scan itself is now ported too**, PR [#416](https://github.com/baileyrd/rusty_extract/pull/416):
//! `dlllib::scan_media_info` (built on `dlllib`'s new DLL-calling
//! infrastructure, the same trait/fake/real-Win32 split
//! `automation::GuiAutomation` already established for window
//! automation) calls `MediaInfo_New`/`_Open`/`_Inform`/`_Delete` and
//! hands its raw output straight to [`format_media_info`] — not
//! duplicated here.
//!
//! **A genuinely easy-to-miss AutoIt semantic**: `StringSplit($aReturn[0],
//! @CRLF, 2)` passes only `$STR_NOCOUNT` (2), *not* `$STR_ENTIRESPLIT`
//! (1) — so `@CRLF` (`"\r\n"`) is **not** treated as one two-character
//! delimiter. It's treated as a *set* of individual delimiter
//! characters, splitting at every lone `\r` *and* every lone `\n`. Since
//! a real `\r\n` line ending contains both, this produces an empty
//! string between them for every line boundary — roughly double the
//! real line count, not the real line count itself. [`format_media_info`]
//! reproduces this exactly (`str::split(['\r', '\n'])`) rather than the
//! more "obvious" `split("\r\n")`, so the `< 10` threshold and the
//! per-line loop both see the same element count and empty-string
//! entries the source does. Those spurious empty entries turn out
//! harmless in the visible output: each one fails the `" : "` split,
//! and `StringIsSpace("")` is true, so they're silently skipped by the
//! same rule that skips genuinely blank lines — but the *threshold
//! check* still operates on the doubled count, which is the source's
//! real behavior, not an approximation of it.
//!
//! **`$sType == "Complete name"` is case-sensitive** (`==`) — the one
//! exact-case comparison in this function, unlike the case-insensitive
//! `StringIsSpace`/formatting logic around it.
//!
//! **A quiet truncation, preserved rather than "fixed"**: `StringSplit`
//! on `" : "` with `$STR_ENTIRESPLIT` still splits on *every*
//! occurrence in the line, but the source only ever reads
//! `$aSplit[0]`/`$aSplit[1]` — a line with more than one `" : "`
//! substring silently loses everything after the second occurrence.
//! [`format_media_info`] keeps only the first two parts the same way.

/// Ports `StringStripWS($s, 4+2+1)`: strip leading and trailing
/// whitespace, then collapse any run of internal whitespace down to a
/// single space.
fn strip_ws(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_was_space = false;
    for c in trimmed.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
            }
            prev_was_space = true;
        } else {
            out.push(c);
            prev_was_space = false;
        }
    }
    out
}

/// Ports `FileScan_MediaInfo`'s formatting pass (UniExtract.au3:1073-
/// 1089). `raw` is `MediaInfo_Inform`'s raw output text (real DLL call,
/// left to the caller — see the module doc comment). Returns `None`
/// when the source would treat this as "not a media file" and return
/// early without formatting anything.
pub fn format_media_info(raw: &str) -> Option<String> {
    let lines: Vec<&str> = raw.split(['\r', '\n']).collect();
    if lines.len() < 10 {
        return None;
    }

    let mut out = String::new();
    for line in lines {
        let parts: Vec<&str> = line.split(" : ").collect();
        if parts.len() < 2 {
            if !line.trim().is_empty() {
                out.push_str("\r\n[");
                out.push_str(line);
                out.push_str("]\r\n");
            }
            continue;
        }

        let key = strip_ws(parts[0]);
        if key == "Complete name" {
            continue;
        }
        let value = strip_ws(parts[1]);
        out.push_str(&format!("{key:<24}{value}\r\n"));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media_info_sample() -> String {
        // 12 real lines, each "Key : Value" -- comfortably over the
        // 10-line threshold once run through the CRLF-charset split
        // (which roughly doubles the element count).
        [
            "General",
            "Complete name : movie.mkv",
            "Format : Matroska",
            "File size : 1.2 GiB",
            "Duration : 1 h 32 min",
            "Video",
            "Format : AVC",
            "Width : 1920 pixels",
            "Height : 1080 pixels",
            "Audio",
            "Format : AAC",
            "Channel(s) : 2 channels",
        ]
        .join("\r\n")
    }

    #[test]
    fn short_output_is_treated_as_not_a_media_file() {
        assert_eq!(format_media_info("Format : data\r\nGeneral"), None);
    }

    /// Parity test for capability C045: the CRLF-charset split (not a
    /// literal "\r\n" split) means the effective element count is
    /// roughly double the real line count -- a handful of real lines
    /// alone doesn't clear the threshold, even though it would under a
    /// naive line-count reading of "< 10 lines".
    #[test]
    fn threshold_reflects_the_crlf_charset_split_not_real_line_count() {
        // 6 real lines -> 11 elements after CRLF-charset splitting
        // (5 internal boundaries each contribute one empty entry),
        // clearing the "< 10" threshold despite looking short.
        let six_lines = ["A : 1", "B : 2", "C : 3", "D : 4", "E : 5", "F : 6"].join("\r\n");
        assert!(format_media_info(&six_lines).is_some());

        // 4 real lines -> 7 elements, stays under the threshold.
        let four_lines = ["A : 1", "B : 2", "C : 3", "D : 4"].join("\r\n");
        assert_eq!(format_media_info(&four_lines), None);
    }

    #[test]
    fn skips_complete_name_field() {
        let result = format_media_info(&media_info_sample()).unwrap();
        assert!(!result.contains("Complete name"));
        assert!(!result.contains("movie.mkv"));
    }

    #[test]
    fn formats_key_left_padded_to_24_columns() {
        let result = format_media_info(&media_info_sample()).unwrap();
        assert!(result.contains(&format!("{:<24}Matroska\r\n", "Format")));
    }

    /// Parity test for capability C045: a header line with no " : "
    /// separator (e.g. a section name like "General") gets
    /// bracket-wrapped, not silently dropped.
    #[test]
    fn lines_without_a_separator_are_bracket_wrapped() {
        let result = format_media_info(&media_info_sample()).unwrap();
        assert!(result.contains("[General]"));
        assert!(result.contains("[Video]"));
        assert!(result.contains("[Audio]"));
    }

    /// Parity test for capability C045: `StringIsSpace` treats an empty
    /// string as whitespace-only, so the spurious empty entries the
    /// CRLF-charset split produces are silently skipped, not
    /// bracket-wrapped as `"[]"`.
    #[test]
    fn empty_entries_are_silently_skipped_not_bracket_wrapped() {
        let result = format_media_info(&media_info_sample()).unwrap();
        assert!(!result.contains("[]"));
    }

    /// Parity test for capability C045: a line with more than one
    /// `" : "` occurrence keeps only the first two parts -- everything
    /// after the second is silently dropped, matching the source's own
    /// `$aSplit[1]`-only read.
    #[test]
    fn extra_separator_occurrences_are_silently_truncated() {
        let sample = [
            "Header",
            "Time : 12 : 30 : 00",
            "B : 2",
            "C : 3",
            "D : 4",
            "E : 5",
            "F : 6",
            "G : 7",
        ]
        .join("\r\n");
        let result = format_media_info(&sample).unwrap();
        assert!(result.contains(&format!("{:<24}12\r\n", "Time")));
        assert!(!result.contains("30"));
    }

    #[test]
    fn key_value_are_stripped_and_collapsed() {
        let sample = [
            "Header",
            "Odd   Key   Name : some   value",
            "B : 2",
            "C : 3",
            "D : 4",
            "E : 5",
            "F : 6",
            "G : 7",
        ]
        .join("\r\n");
        let result = format_media_info(&sample).unwrap();
        assert!(result.contains(&format!("{:<24}some value\r\n", "Odd Key Name")));
    }

    /// Parity test for capability C045: `dlllib::scan_media_info`'s raw
    /// output feeds directly into `format_media_info`, end to end.
    #[test]
    fn composes_with_dlllib_scan_media_info() {
        use crate::dlllib::fake::FakeMediaInfoLibrary;
        use crate::dlllib::scan_media_info;

        let mut lib = FakeMediaInfoLibrary::new();
        lib.script_inform(&media_info_sample());

        let raw = scan_media_info(&mut lib, r"C:\downloads\movie.mkv").unwrap();

        assert!(format_media_info(&raw).is_some());
    }
}
