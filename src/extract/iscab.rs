//! InstallShield Cabinet (`$TYPE_ISCAB`) fallback chain: try `unshield`
//! first, and only on failure fall through to a user-disambiguated choice
//! between `is6comp`, `is5comp`, and `iscab`.
//!
//! ```autoit
//! Case $TYPE_ISCAB
//!     ; Unshield only works with UNIX-style paths
//!     Local $sPath = StringReplace($file, "\", "/")
//!     Local $sReturn = _Run($unshield & ' -D 2 -d "' & $outdir & '" x "' & $sPath & '"', $outdir)
//!     If StringInStr($sReturn, "Try unshield_file_save_old()") Then $sReturn = _Run($unshield & ' -O -D 2 -d "' & $outdir & '" x "' & $sPath & '"', $outdir)
//!
//!     If StringInStr($sReturn, "Failed to extract file") Or StringInStr($sReturn, "Failed to read header files") Then
//!         Local $aReturn = ["InstallShield Cabinet " & t('TERM_ARCHIVE'), t('METHOD_EXTRACTION_RADIO', "is6comp"), t('METHOD_EXTRACTION_RADIO', "is5comp"), t('METHOD_EXTRACTION_RADIO', "iscab")]
//!         $iChoice = GUI_MethodSelect($aReturn, $arcdisp)
//!
//!         Switch $iChoice
//!             Case 1
//!                 ; List contents of archive
//!                 Local $return = FetchStdout($is6cab & ' l "' & $file & '"', $filedir, @SW_HIDE)
//!                 $return = _StringBetween(StringRight($return, 22), " ", " file(s) total")
//!                 If Not @error Then $return = Number(StringStripWS($return[0], 8))
//!
//!                 ; If successful, extract contents of InstallShield cabs file-by-file
//!                 If $return > 0 Then
//!                     RunWait(_MakeCommand($is6cab & ' x "' & $file & '"', True), $outdir, @SW_MINIMIZE)
//!                 Else
//!                     ; Otherwise, attempt to extract with unshield
//!                     _Run($unshield & ' -d "' & $outdir & '" x "' & $file & '"', $outdir)
//!                 EndIf
//!             Case 2
//!                 HasPlugin($is5cab)
//!                 RunWait($is5cab & ' x "' & $file & '"', $outdir, @SW_MINIMIZE)
//!             Case 3
//!                 HasPlugin($iscab)
//!                 RunWait($iscab & ' "' & $file & '" -i"files.ini" -lx', $outdir, @SW_HIDE)
//!                 RunWait($iscab & ' "' & $file & '" -i"files.ini" -x', $outdir, @SW_MINIMIZE)
//!                 FileDelete($outdir & "\files.ini")
//!         EndSwitch
//!     Else
//!         Local $aCleanup[] = ["_Engine_*", "_Support_*"]
//!         Cleanup($aCleanup)
//!     EndIf
//! ```
//!
//! **Scope — invocations and routing decisions only.** `HasPlugin`
//! preconditions, `FileDelete`, and the final `Cleanup(...)` call are
//! real filesystem I/O, out of scope, matching this crate's usual split.
//! The disambiguation candidate list is C053's own
//! `method_select::ISCAB_CANDIDATES` — reused here, not duplicated. The
//! success-path cleanup targets are both plain wildcards
//! (`cleanup::classify_target` would classify each as `Wildcard`);
//! [`SUCCESS_CLEANUP_TARGETS`] just pins the two literal patterns down,
//! not the classification/expansion machinery itself.
//!
//! **Preserved quirk — the is6comp fallback drops `-D 2` and the
//! UNIX-style path.** Choice 1's unshield fallback
//! (`_Run($unshield & ' -d "' & $outdir & '" x "' & $file & '"', ...)`)
//! is *not* a repeat of the initial attempt: it omits `-D 2` and uses the
//! raw `$file` (backslashes intact), not `$sPath`'s forward-slash
//! rewrite. Reproduced exactly as two distinct invocation builders,
//! [`initial_unshield_invocation`]/[`retry_unshield_invocation`] vs.
//! [`is6comp_fallback_unshield_invocation`].

use super::{Invocation, WindowMode};

/// The success-path cleanup targets (UniExtract.au3, the `Else` branch):
/// both plain wildcards fed to `Cleanup(...)`, out of scope here (see
/// module doc comment).
pub const SUCCESS_CLEANUP_TARGETS: &[&str] = &["_Engine_*", "_Support_*"];

/// Rewrites `$file` to the UNIX-style path `unshield` requires
/// (`StringReplace($file, "\", "/")`).
pub fn unix_style_path(file: &str) -> String {
    file.replace('\\', "/")
}

/// Builds the initial `unshield` invocation: `<program> -D 2 -d
/// "<outdir>" x "<unix_path>"`, run in `outdir`. No `$show_flag` argument
/// is passed at this call site, so `_Run`'s own default (`@SW_MINIMIZE`)
/// applies — the same convention `extract::raiu`/`extract::helpdeco`
/// already document for their own bare `_Run($cmd, $dir)` calls.
pub fn initial_unshield_invocation(program: &str, outdir: &str, unix_path: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-D".to_string(),
            "2".to_string(),
            "-d".to_string(),
            outdir.to_string(),
            "x".to_string(),
            unix_path.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Whether the initial `unshield` attempt's captured output calls for a
/// retry with `-O` added: `StringInStr($sReturn, "Try
/// unshield_file_save_old()")`.
pub fn should_retry_with_dash_o(output: &str) -> bool {
    output.contains("Try unshield_file_save_old()")
}

/// Builds the `-O`-retry `unshield` invocation: identical to
/// [`initial_unshield_invocation`] but with `-O` inserted before `-D 2`.
pub fn retry_unshield_invocation(program: &str, outdir: &str, unix_path: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-O".to_string(),
            "-D".to_string(),
            "2".to_string(),
            "-d".to_string(),
            outdir.to_string(),
            "x".to_string(),
            unix_path.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Whether the (possibly `-O`-retried) `unshield` output counts as a
/// failure, routing to the disambiguation chain instead of the
/// success-path cleanup: `StringInStr($sReturn, "Failed to extract
/// file") Or StringInStr($sReturn, "Failed to read header files")`.
pub fn unshield_failed(output: &str) -> bool {
    output.contains("Failed to extract file") || output.contains("Failed to read header files")
}

/// Builds choice 1's listing invocation: `<is6comp> l "<file>"`, run in
/// `filedir` with the window hidden (`FetchStdout(..., $filedir,
/// @SW_HIDE)`).
pub fn is6comp_listing_invocation(program: &str, file: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["l".to_string(), file.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Parses the file count from `is6comp`'s listing output:
/// `_StringBetween(StringRight($return, 22), " ", " file(s) total")`,
/// then `Number(StringStripWS($return[0], 8))` (mode `8` strips *every*
/// whitespace character, not just leading/trailing). Returns `0` both
/// when the pattern isn't found and when it parses to `0` — the two
/// cases the source can't distinguish either, since `$return > 0`
/// treats `_StringBetween`'s own failure return the same as a genuine
/// zero count (AutoIt's `"" > 0` string/number coercion also lands on
/// `False`).
pub fn is6comp_file_count(listing_output: &str) -> u32 {
    let tail = string_right(listing_output, 22);
    let between = match string_between(&tail, " ", " file(s) total") {
        Some(s) => s,
        None => return 0,
    };
    let digits: String = between.chars().filter(|c| !c.is_whitespace()).collect();
    digits.parse::<u32>().unwrap_or(0)
}

/// The last `n` characters of `s`, matching AutoIt's `StringRight` — the
/// whole string if `n` exceeds its length.
fn string_right(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

/// The text between the first occurrence of `start` and the following
/// occurrence of `end`, matching `_StringBetween`'s single-match
/// (`$sStart`/`$sEnd`) shape used here — `None` if either marker isn't
/// found, matching `_StringBetween`'s `@error`.
fn string_between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let after_start = s.find(start)? + start.len();
    let end_offset = s[after_start..].find(end)?;
    Some(&s[after_start..after_start + end_offset])
}

/// Whether choice 1 extracts with `is6comp` directly, ported from `If
/// $return > 0 Then`.
pub fn should_use_is6comp_extraction(file_count: u32) -> bool {
    file_count > 0
}

/// Builds choice 1's `is6comp` extraction invocation:
/// `<is6comp> x "<file>"`, run in `outdir`, window minimized
/// (`RunWait(_MakeCommand(...), $outdir, @SW_MINIMIZE)` — the
/// `_MakeCommand` bindir-prefixing isn't modeled, matching
/// `extract::expand`'s documented precedent for the same helper).
pub fn is6comp_extract_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds choice 1's unshield fallback invocation when `is6comp`'s
/// listing found no files: `<unshield> -d "<outdir>" x "<file>"`, run in
/// `outdir`. **Not** a repeat of [`initial_unshield_invocation`] — see
/// the module doc comment's preserved-quirk note: no `-D 2`, and `file`
/// here is the raw path, not the UNIX-style rewrite.
pub fn is6comp_fallback_unshield_invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-d".to_string(),
            outdir.to_string(),
            "x".to_string(),
            file.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds choice 2's `is5comp` invocation: `<is5comp> x "<file>"`, run in
/// `outdir`, window minimized.
pub fn is5comp_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds choice 3's first `iscab` invocation (list mode):
/// `<iscab> "<file>" -i"files.ini" -lx`, run in `outdir`, window hidden.
pub fn iscab_list_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            file.to_string(),
            "-i\"files.ini\"".to_string(),
            "-lx".to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds choice 3's second `iscab` invocation (extract mode):
/// `<iscab> "<file>" -i"files.ini" -x`, run in `outdir`, window
/// minimized. The source deletes `<outdir>\files.ini` afterward
/// (`FileDelete`) — real filesystem I/O, out of scope here.
pub fn iscab_extract_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            file.to_string(),
            "-i\"files.ini\"".to_string(),
            "-x".to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_style_path_replaces_backslashes() {
        assert_eq!(
            unix_style_path(r"C:\downloads\game.cab"),
            "C:/downloads/game.cab"
        );
    }

    /// Parity test for capability C075: the initial unshield invocation
    /// matches `-D 2 -d "<outdir>" x "<unix_path>"`, minimized (no
    /// `$show_flag` passed).
    #[test]
    fn initial_unshield_invocation_matches_source() {
        let inv = initial_unshield_invocation(
            r"C:\bin\unshield.exe",
            r"C:\downloads\unpacked",
            "C:/downloads/game.cab",
        );
        assert_eq!(inv.program, r"C:\bin\unshield.exe");
        assert_eq!(
            inv.args,
            vec![
                "-D".to_string(),
                "2".to_string(),
                "-d".to_string(),
                r"C:\downloads\unpacked".to_string(),
                "x".to_string(),
                "C:/downloads/game.cab".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C075: the `-O` retry invocation adds
    /// `-O` ahead of `-D 2`, otherwise identical.
    #[test]
    fn retry_unshield_invocation_adds_dash_o() {
        let inv = retry_unshield_invocation(
            r"C:\bin\unshield.exe",
            r"C:\downloads\unpacked",
            "C:/downloads/game.cab",
        );
        assert_eq!(
            inv.args,
            vec![
                "-O".to_string(),
                "-D".to_string(),
                "2".to_string(),
                "-d".to_string(),
                r"C:\downloads\unpacked".to_string(),
                "x".to_string(),
                "C:/downloads/game.cab".to_string(),
            ]
        );
    }

    #[test]
    fn should_retry_with_dash_o_detects_marker() {
        assert!(should_retry_with_dash_o(
            "some output\nTry unshield_file_save_old()\nmore"
        ));
        assert!(!should_retry_with_dash_o("ordinary output"));
    }

    #[test]
    fn unshield_failed_detects_either_marker() {
        assert!(unshield_failed("Failed to extract file: foo.dat"));
        assert!(unshield_failed("Failed to read header files"));
        assert!(!unshield_failed("extraction complete"));
    }

    /// Parity test for capability C075: a well-formed listing tail
    /// parses the file count.
    #[test]
    fn is6comp_file_count_parses_well_formed_tail() {
        let output = format!("{}      56 file(s) total", "x".repeat(50));
        assert_eq!(is6comp_file_count(&output), 56);
    }

    /// Parity test for capability C075: output with no "file(s) total"
    /// marker parses to `0`, matching `_StringBetween`'s failure being
    /// treated the same as a genuine zero count.
    #[test]
    fn is6comp_file_count_is_zero_when_marker_missing() {
        assert_eq!(is6comp_file_count("no such marker here"), 0);
    }

    #[test]
    fn should_use_is6comp_extraction_requires_positive_count() {
        assert!(should_use_is6comp_extraction(1));
        assert!(!should_use_is6comp_extraction(0));
    }

    /// Parity test for capability C075: choice 1's unshield fallback
    /// omits `-D 2` and uses the raw (non-UNIX-style) file path — not a
    /// repeat of the initial attempt.
    #[test]
    fn is6comp_fallback_unshield_invocation_omits_d2_and_uses_raw_path() {
        let inv = is6comp_fallback_unshield_invocation(
            r"C:\bin\unshield.exe",
            r"C:\downloads\unpacked",
            r"C:\downloads\game.cab",
        );
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                r"C:\downloads\unpacked".to_string(),
                "x".to_string(),
                r"C:\downloads\game.cab".to_string(),
            ]
        );
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn is6comp_extract_invocation_matches_source() {
        let inv =
            is6comp_extract_invocation(r"C:\bin\is6comp.exe", r"C:\d\game.cab", r"C:\d\unpacked");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\d\game.cab".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\d\unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn is5comp_invocation_matches_source() {
        let inv = is5comp_invocation(r"C:\bin\is5comp.exe", r"C:\d\game.cab", r"C:\d\unpacked");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\d\game.cab".to_string()]
        );
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn iscab_list_invocation_matches_source() {
        let inv = iscab_list_invocation(r"C:\bin\iscab.exe", r"C:\d\game.cab", r"C:\d\unpacked");
        assert_eq!(
            inv.args,
            vec![
                r"C:\d\game.cab".to_string(),
                "-i\"files.ini\"".to_string(),
                "-lx".to_string(),
            ]
        );
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn iscab_extract_invocation_matches_source() {
        let inv = iscab_extract_invocation(r"C:\bin\iscab.exe", r"C:\d\game.cab", r"C:\d\unpacked");
        assert_eq!(
            inv.args,
            vec![
                r"C:\d\game.cab".to_string(),
                "-i\"files.ini\"".to_string(),
                "-x".to_string(),
            ]
        );
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    #[test]
    fn success_cleanup_targets_match_source() {
        assert_eq!(SUCCESS_CLEANUP_TARGETS, &["_Engine_*", "_Support_*"]);
    }
}
