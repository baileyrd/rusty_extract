//! Positional destination-argument routing: ports `ParseCommandLine()`'s
//! `$iArgs > 1` block (UniExtract.au3:635-646):
//!
//! ```autoit
//! If $iArgs > 1 Then
//!     ; Scan only
//!     If $cmdline[2] = "/scan" Then
//!         $extract = False
//!         $bOptCreateLog = False
//!     Else ; Outdir specified
//!         $outdir = $cmdline[2]
//!         If $outdir <> "/sub" And $outdir <> "/last" Then $outdir = _PathFull($outdir)
//!         $bOptOpenOutDir = 0
//!     EndIf
//! EndIf
//! ```
//!
//! Both the `=`/`<>` string comparisons above are case-insensitive by this
//! script's default `StringCompareMode` (the same rule `cli`'s module doc
//! comment documents for its own flag checks) — `/SCAN`, `/Scan`, and
//! `/scan` are all the same token here.
//!
//! **Not modeled:** `$bOptOpenOutDir = 0` (suppressing "open the output
//! folder when done" for a context-menu invocation) — a deferred-GUI-
//! subsystem concern (manifest row D001), not part of this decision.

use crate::file_arg::resolve_file_argument_path;

/// C002/C003: how the destination argument (`$cmdline[2]`) routes,
/// ported from the block quoted in the module doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationArgument {
    /// `$iArgs <= 1`: no second positional argument at all. The caller
    /// falls back to whatever default output directory applies elsewhere
    /// (`outdir::default_output_subfolder`, C138); scan mode is not
    /// entered.
    Absent,
    /// C003: `$cmdline[2] = "/scan"` — scan-only mode. Carries the two
    /// flags the source sets (UniExtract.au3:638-639) as plain booleans
    /// rather than leaving them implicit: extraction is skipped entirely,
    /// and no per-run log file is created for this run, independent of
    /// the persisted `log` preference (C028).
    ScanOnly { extract: bool, create_log: bool },
    /// A real destination value. `"/sub"`/`"/last"` pass through
    /// unresolved — the tokens `outdir::resolve_output_directory`
    /// (C004/C005) itself recognizes. Anything else is pre-resolved to a
    /// full path via the same `_PathFull` the file argument uses (C001,
    /// [`resolve_file_argument_path`]) *before*
    /// `outdir::resolve_output_directory` ever sees it — a separate
    /// resolution pass from that function's own relative-path handling,
    /// matching the source's two-stage `_PathFull`-then-
    /// `ValidateOutputDirectory` shape.
    Outdir(String),
}

/// Routes `second_arg` (`$cmdline[2]`, absent when `$iArgs <= 1`) the same
/// way the source's `$iArgs > 1` block does, resolving a plain destination
/// value against `cwd` (mirroring `_PathFull`'s common-case behavior —
/// see [`resolve_file_argument_path`]'s doc comment for the one gap that
/// doesn't reproduce).
pub fn parse_destination_argument(second_arg: Option<&str>, cwd: &str) -> DestinationArgument {
    let Some(arg) = second_arg else {
        return DestinationArgument::Absent;
    };
    if arg.eq_ignore_ascii_case("/scan") {
        return DestinationArgument::ScanOnly {
            extract: false,
            create_log: false,
        };
    }
    if arg.eq_ignore_ascii_case("/sub") || arg.eq_ignore_ascii_case("/last") {
        return DestinationArgument::Outdir(arg.to_string());
    }
    DestinationArgument::Outdir(resolve_file_argument_path(arg, cwd))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test: `$iArgs <= 1` — no destination argument at all.
    #[test]
    fn absent_when_no_second_argument() {
        assert_eq!(
            parse_destination_argument(None, r"C:\downloads"),
            DestinationArgument::Absent
        );
    }

    /// Parity test for capability C003: `/scan` (any case) enters
    /// scan-only mode with both flags false.
    #[test]
    fn scan_token_enters_scan_only_mode_case_insensitively() {
        for token in ["/scan", "/SCAN", "/Scan"] {
            assert_eq!(
                parse_destination_argument(Some(token), r"C:\downloads"),
                DestinationArgument::ScanOnly {
                    extract: false,
                    create_log: false,
                }
            );
        }
    }

    /// Parity test for capability C002: `/sub`/`/last` (any case) pass
    /// through unresolved rather than being run through `_PathFull`.
    #[test]
    fn sub_and_last_tokens_pass_through_unresolved() {
        assert_eq!(
            parse_destination_argument(Some("/sub"), r"C:\downloads"),
            DestinationArgument::Outdir("/sub".to_string())
        );
        assert_eq!(
            parse_destination_argument(Some("/LAST"), r"C:\downloads"),
            DestinationArgument::Outdir("/LAST".to_string())
        );
    }

    /// Parity test for capability C002: any other value is pre-resolved
    /// to a full path via the same `_PathFull` logic the file argument
    /// uses — drive-absolute and UNC pass through, a plain relative value
    /// joins onto `cwd`.
    #[test]
    fn other_values_are_path_full_resolved_against_cwd() {
        assert_eq!(
            parse_destination_argument(Some("output"), r"C:\downloads"),
            DestinationArgument::Outdir(r"C:\downloads\output".to_string())
        );
        assert_eq!(
            parse_destination_argument(Some(r"D:\already\full"), r"C:\downloads"),
            DestinationArgument::Outdir(r"D:\already\full".to_string())
        );
        assert_eq!(
            parse_destination_argument(Some(r"\\server\share\out"), r"C:\downloads"),
            DestinationArgument::Outdir(r"\\server\share\out".to_string())
        );
    }
}
