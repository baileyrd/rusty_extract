//! Scan-only file-type report formatting: ports `_FiletypeGet()`
//! (UniExtract.au3:5292-5313, capability C153) and the silent-mode
//! file-scan log entry format it feeds into
//! (UniExtract.au3:4139-4142, capability C154). Each detector that ran
//! (TrID, Unix `file`, Exeinfo PE, PEiD, MediaInfo) appends its own
//! `($sScanner, $sType)` pair to the source's `$aFiletype` array via
//! `_FiletypeAdd()`; [`format_filetype_results`] concatenates all of
//! them into one report string, either as a single unlabeled block
//! (`$bHeader = False`, used to build the plain `$sFileType` value
//! `terminate()` and [`build_scan_log_entry`]'s silent-mode scan log
//! both consume) or with a centered dashed header per scanner
//! (`$bHeader = True`, used for on-screen display).

/// One scanner's contribution to the report: the scanner's display name
/// and the type text it produced. Mirrors one row of the source's
/// `$aFiletype[i][0..1]` array.
pub struct ScannerResult<'a> {
    pub scanner: &'a str,
    pub type_text: &'a str,
}

/// Ports `_FiletypeGet($bHeader = True, $iWidth = 50)`
/// (UniExtract.au3:5292-5313).
///
/// Entries are joined by a blank line (`@CRLF & @CRLF`), matching the
/// source's `If $return <> "" Then $return &= @CRLF & @CRLF` guard before
/// each entry after the first. With `with_header = false`, each entry
/// contributes only its `type_text` — no scanner name appears anywhere.
///
/// With `with_header = true` and `width > 0`, the scanner name is
/// centered between two runs of `-` characters sized to
/// `Floor((width - len(" name ")) / 2)` each — using AutoIt's `Floor`
/// (rounds toward negative infinity), so a name longer than `width`
/// yields a negative padding length, which reproduces as no dashes at
/// all (the standard `_StringRepeat` UDF's own guard against a
/// non-positive repeat count). `width <= 0` skips padding entirely and
/// uses the bare scanner name as the header, with no surrounding spaces.
pub fn format_filetype_results(entries: &[ScannerResult], with_header: bool, width: i64) -> String {
    let mut result = String::new();

    for entry in entries {
        if !result.is_empty() {
            result.push_str("\r\n\r\n");
        }

        if !with_header {
            result.push_str(entry.type_text);
            continue;
        }

        let header = if width > 0 {
            let padded_name = format!(" {} ", entry.scanner);
            let diff = width - padded_name.chars().count() as i64;
            let padding_len = diff.div_euclid(2);
            let padding = if padding_len > 0 {
                "-".repeat(padding_len as usize)
            } else {
                String::new()
            };
            format!("{padding}{padded_name}{padding}")
        } else {
            entry.scanner.to_string()
        };

        result.push_str(&header);
        result.push_str("\r\n\r\n");
        result.push_str(entry.type_text);
    }

    result
}

/// Ports the `$STATUS_FILEINFO`/silent-mode branch of `terminate()`
/// (UniExtract.au3:4139-4142) — capability C154: the block appended to
/// the file-scan log when a scan-only run finishes in silent mode.
/// `filetype` is exactly `_FiletypeGet(False)`'s output — in this crate,
/// [`format_filetype_results`] called with `with_header = false`.
///
/// The 60-dash separator is a literal from the source, not a computed
/// width — unrelated to the `width` parameter [`format_filetype_results`]
/// takes. Opening `$fileScanLogFile` in append mode
/// (`$FO_CREATEPATH + $FO_APPEND`) is real filesystem I/O and the
/// caller's job; every scanned item in a batch run appends its own block
/// to the same file, which is how results accumulate across the run —
/// this function only builds the one block for a single item.
pub fn build_scan_log_entry(file: &str, filetype: &str) -> String {
    format!("{file}\r\n\r\n{filetype}\r\n{}\r\n", "-".repeat(60))
}

#[cfg(test)]
mod tests {
    use super::{build_scan_log_entry, format_filetype_results, ScannerResult};

    /// Parity test for capability C153: `with_header = false` joins raw
    /// type text only, no scanner names, entries separated by a blank
    /// line.
    #[test]
    fn no_header_joins_type_text_only() {
        let entries = [
            ScannerResult {
                scanner: "TrID",
                type_text: "ZIP archive",
            },
            ScannerResult {
                scanner: "file",
                type_text: "Zip archive data",
            },
        ];
        assert_eq!(
            format_filetype_results(&entries, false, 50),
            "ZIP archive\r\n\r\nZip archive data"
        );
    }

    /// Parity test for capability C153: a single entry with no header
    /// produces just its type text, no leading separator.
    #[test]
    fn no_header_single_entry_has_no_leading_separator() {
        let entries = [ScannerResult {
            scanner: "TrID",
            type_text: "ZIP archive",
        }];
        assert_eq!(format_filetype_results(&entries, false, 50), "ZIP archive");
    }

    /// Parity test for capability C153: an empty entry list produces an
    /// empty string regardless of header/width settings.
    #[test]
    fn empty_entries_produce_empty_string() {
        assert_eq!(format_filetype_results(&[], true, 50), "");
        assert_eq!(format_filetype_results(&[], false, 50), "");
    }

    /// Parity test for capability C153: `with_header = true` centers the
    /// scanner name between dashes using `Floor((width - len(name)) /
    /// 2)` padding on each side.
    #[test]
    fn header_centers_scanner_name_with_floor_division() {
        let entries = [ScannerResult {
            scanner: "TrID",
            type_text: "ZIP archive",
        }];
        // " TrID " has length 6; (50 - 6) / 2 = 22 exactly.
        let dashes = "-".repeat(22);
        assert_eq!(
            format_filetype_results(&entries, true, 50),
            format!("{dashes} TrID {dashes}\r\n\r\nZIP archive")
        );
    }

    /// Parity test for capability C153: an odd remainder floors down,
    /// producing a header one character shorter than `width`, not
    /// rounded.
    #[test]
    fn header_floor_rounding_on_odd_remainder() {
        let entries = [ScannerResult {
            scanner: "7-Zip",
            type_text: "7z archive",
        }];
        // " 7-Zip " has length 7; (50 - 7) / 2 = 21.5, Floor = 21.
        let dashes = "-".repeat(21);
        assert_eq!(
            format_filetype_results(&entries, true, 50),
            format!("{dashes} 7-Zip {dashes}\r\n\r\n7z archive")
        );
    }

    /// Parity test for capability C153: a scanner name longer than
    /// `width` yields a negative `Floor` padding length, which
    /// reproduces as no dashes at all rather than a panic or a
    /// truncated name.
    #[test]
    fn header_name_longer_than_width_has_no_padding() {
        let entries = [ScannerResult {
            scanner: "A Very Long Scanner Name Indeed",
            type_text: "some type",
        }];
        assert_eq!(
            format_filetype_results(&entries, true, 10),
            " A Very Long Scanner Name Indeed \r\n\r\nsome type"
        );
    }

    /// Parity test for capability C153: `width <= 0` skips padding
    /// entirely and uses the bare scanner name, with no surrounding
    /// spaces.
    #[test]
    fn zero_width_uses_bare_scanner_name() {
        let entries = [ScannerResult {
            scanner: "TrID",
            type_text: "ZIP archive",
        }];
        assert_eq!(
            format_filetype_results(&entries, true, 0),
            "TrID\r\n\r\nZIP archive"
        );
    }

    /// Parity test for capability C153: multiple entries with headers
    /// are separated by a full blank-line gap between one entry's type
    /// text and the next entry's header.
    #[test]
    fn multiple_header_entries_are_separated() {
        let entries = [
            ScannerResult {
                scanner: "AB",
                type_text: "type one",
            },
            ScannerResult {
                scanner: "CD",
                type_text: "type two",
            },
        ];
        let result = format_filetype_results(&entries, true, 10);
        assert_eq!(
            result,
            "--- AB ---\r\n\r\ntype one\r\n\r\n--- CD ---\r\n\r\ntype two"
        );
    }

    /// Parity test for capability C154: the scan log entry format
    /// matches the source's exact concatenation — file path, blank line,
    /// filetype text, a 60-dash separator line, each `\r\n`-terminated.
    #[test]
    fn build_scan_log_entry_matches_source_format() {
        let dashes = "-".repeat(60);
        assert_eq!(
            build_scan_log_entry(r"C:\downloads\archive.zip", "ZIP archive"),
            format!("C:\\downloads\\archive.zip\r\n\r\nZIP archive\r\n{dashes}\r\n")
        );
    }

    /// Parity test for capability C154: the separator is exactly 60
    /// dashes, matching the source's literal string length.
    #[test]
    fn build_scan_log_entry_separator_is_sixty_dashes() {
        let entry = build_scan_log_entry("file.exe", "unknown type");
        let separator_line = entry.lines().nth(3).unwrap();
        assert_eq!(separator_line.len(), 60);
        assert!(separator_line.chars().all(|c| c == '-'));
    }
}
