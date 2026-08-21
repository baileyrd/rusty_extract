//! Unicode/UNC-path input relocation — C159 (reversal), C175/C176 (the
//! forward relocation decision, `MoveInputFileIfNecessary()`,
//! UniExtract.au3:2218-2266), and C177 (a known bookkeeping-loss quirk
//! on nested re-entry, UniExtract.au3:378,3603-3642,3633-3636).
//!
//! ```autoit
//! Func MoveInputFileIfNecessary()
//!     Local $bIsUnc = _WinAPI_PathIsUNC($file)
//!     Local $new = 0
//!     If $checkUnicode And (Not StringRegExp($file, $sRegExAscii, 0) Or StringLeft($filename, 2) == "--") Then
//!         If StringRegExp($filedir, $sRegExAscii, 0) Then
//!             $new = _TempFile($filedir, "Unicode_", $fileext)
//!         Else
//!             If Not StringRegExp(@TempDir, $sRegExAscii, 0) Then Return Cout("Temp directory contains unicode characters: " & @TempDir)
//!             $new = StringRegExp($filename, $sRegExAscii, 0)? @TempDir & "\" & $filenamefull: _TempFile(@TempDir, "Unicode_", $fileext)
//!         EndIf
//!     EndIf
//!
//!     If $new == 0 Then
//!         If Not $bIsUnc Then Return
//!         $new = @TempDir & "\" & $filenamefull
//!     EndIf
//!
//!     ; Multipart archive, TODO: move all parts
//!     If StringRegExp($file, ".*part\d+\.rar", 0) Or StringRegExp($fileext, "\d{3}", 0) Then Return Cout("File seems to be multipart archive, not moving")
//!
//!     HasFreeSpace($new, 2)
//!     If StringLeft($file, 1) = StringLeft($new, 1) Then
//!         ; ... _FileMove, $iUnicodeMode = $UNICODE_MOVE ...
//!     Else
//!         ; ... FileCopy, $iUnicodeMode = $UNICODE_COPY ...
//!     EndIf
//!     ; ... $oldpath/$sUnicodeName/$oldoutdir bookkeeping, FilenameParse($new) ...
//!     If Not StringRegExp($outdir, $sRegExAscii, 0) Then $outdir = $initoutdir
//! EndFunc
//! ```
//!
//! **`$sRegExAscii` is a misnomer** — it isn't a pure-ASCII check.
//! Besides `\w` (ASCII letters/digits/underscore) and a handful of
//! ASCII punctuation/symbol characters, it also explicitly whitelists
//! 20 accented Western-European Latin letters (`â ë ö ä ü î ê ô û ï á é
//! í ó ú à è ì ò ù`, both cases via the regex's `(?i)`) plus `ß`, `°`,
//! `²`, `³`. [`is_allowed_charset`] reproduces this exact character set
//! (extracted programmatically from the source's own `\Q...\E` literal
//! block, not retyped by hand, to rule out transcription error) —
//! **not** `char::is_ascii`. A filename in French, German, or several
//! other Western European languages can pass this check outright; one
//! in Cyrillic, CJK, or Greek cannot. `ß`'s uppercase form (`ẞ`,
//! U+1E9E) is deliberately left unmatched here — whether AutoIt's PCRE
//! engine folds `(?i)` case-insensitivity that far for a Latin-1
//! character with no simple uppercase mapping is genuinely unclear
//! without live-testing the interpreter, and `ẞ` is vanishingly rare in
//! real filenames, so this is called out rather than guessed at.
//!
//! **The `.rar` multipart exemption** (`".*part\d+\.rar"`, `"\d{3}"`)
//! applies *after* a destination is already computed, whether from the
//! unicode branch or the UNC-fallback branch — a multipart file is
//! exempted from relocation either way, matching the source's single
//! shared check late in the function rather than duplicating it per
//! branch.
//!
//! **A real, easy-to-miss interaction between C175 and C176**: UNC-path
//! relocation (`$bIsUnc`) only ever supplies its own destination when
//! `$new` is *still* `0` — i.e. when the unicode check either didn't
//! trigger at all, or triggered and set nothing (which can't actually
//! happen given the code shape, so in practice: didn't trigger). A file
//! that's both unicode-named **and** UNC-reached uses whatever
//! destination the unicode branch computed; UNC-ness contributes
//! nothing extra in that case. [`plan_relocation`] preserves this
//! priority exactly rather than treating the two triggers as
//! independent.
//!
//! **Not modeled**: `_WinAPI_PathIsUNC` (real Win32 API call, taken as
//! the caller-supplied `is_unc`); `_TempFile` (real filesystem I/O that
//! generates *and reserves* a random unique filename — see
//! [`Destination::GenerateUniqueName`]); `HasFreeSpace`; the actual
//! `_FileMove`/`FileCopy` call; and `FilenameParse` (re-parsing `$new`
//! into the global file-identity state).

/// Mirrors `$UNICODE_NONE`/`$UNICODE_MOVE`/`$UNICODE_COPY`
/// (UniExtract.au3:100): `Move` when the input file and its ASCII
/// replacement shared a drive letter and could be renamed in place,
/// `Copy` when they didn't and a copy was made instead — decided by
/// [`decide_relocation_mode`], consumed here on reversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeMode {
    None,
    Move,
    Copy,
}

/// What to do with the ASCII working copy of the input file on
/// reversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRevertAction {
    /// No relocation happened; nothing to revert.
    None,
    /// `_FileMove($file, $oldpath, 1)` — move the working copy back to
    /// the original unicode path.
    MoveBack,
    /// `FileRecycle($file)` — the working copy was a *copy*, not a
    /// rename, so reverting just discards it via the recycle bin.
    Recycle,
}

/// Reproduces `terminate()`'s unconditional `$iUnicodeMode` reversal
/// (UniExtract.au3:4101-4114): `If $iUnicodeMode Then ... EndIf`, run at
/// the top of every `terminate()` call — success, failure, or anything
/// else — never gated on the run's outcome, only on whether a relocation
/// happened at all. Returns the action to take on the input file, and
/// whether the output directory should be moved back to its original
/// location too (`_DirMove($outdir, $oldoutdir)`, done for both `Move`
/// and `Copy`, never for `None`).
pub fn decide_unicode_reversion(mode: UnicodeMode) -> (FileRevertAction, bool) {
    match mode {
        UnicodeMode::None => (FileRevertAction::None, false),
        UnicodeMode::Move => (FileRevertAction::MoveBack, true),
        UnicodeMode::Copy => (FileRevertAction::Recycle, true),
    }
}

/// Ports `$sRegExAscii` (UniExtract.au3:94) — every character allowed
/// without triggering relocation. See the module doc comment: this is
/// *not* an ASCII-only check.
pub fn is_allowed_charset(s: &str) -> bool {
    // The regex is anchored `^[...]+$` -- at least one character
    // required, so an empty string never matches.
    !s.is_empty() && s.chars().all(is_allowed_char)
}

fn is_allowed_char(c: char) -> bool {
    if c.is_ascii_alphanumeric() || c == '_' {
        return true; // \w, ASCII-only (PCRE without UCP)
    }
    matches!(
        c,
        ' ' | '@'
            | '!'
            | '§'
            | '$'
            | '%'
            | '&'
            | '/'
            | '\\'
            | '('
            | ')'
            | '='
            | '?'
            | ','
            | '.'
            | '-'
            | ':'
            | '+'
            | '~'
            | '\''
            | '²'
            | '³'
            | '{'
            | '['
            | ']'
            | '}'
            | '*'
            | '#'
            | '°'
            | '^'
            | 'ß'
            | 'â'
            | 'ë'
            | 'ö'
            | 'ä'
            | 'ü'
            | 'î'
            | 'ê'
            | 'ô'
            | 'û'
            | 'ï'
            | 'á'
            | 'é'
            | 'í'
            | 'ó'
            | 'ú'
            | 'à'
            | 'è'
            | 'ì'
            | 'ò'
            | 'ù'
            | 'Â'
            | 'Ë'
            | 'Ö'
            | 'Ä'
            | 'Ü'
            | 'Î'
            | 'Ê'
            | 'Ô'
            | 'Û'
            | 'Ï'
            | 'Á'
            | 'É'
            | 'Í'
            | 'Ó'
            | 'Ú'
            | 'À'
            | 'È'
            | 'Ì'
            | 'Ò'
            | 'Ù'
    )
}

/// Where a relocated file should end up, once [`plan_relocation`]
/// decides one is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// `_TempFile(dir, "Unicode_", ext)` — a randomized unique filename
    /// the caller must generate (and reserves on disk as a side
    /// effect); real filesystem I/O, not reproducible as a pure value
    /// here.
    GenerateUniqueName { dir: String, ext: String },
    /// A concrete, fully computed destination path.
    Fixed(String),
}

/// What `MoveInputFileIfNecessary` decides before ever touching the
/// filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelocationPlan {
    /// Neither the unicode check nor the UNC check triggered — leave
    /// the file where it is.
    NotNeeded,
    /// The unicode check triggered, the containing directory also
    /// isn't in the allowed charset, and `@TempDir` *itself* isn't
    /// either — the source aborts with a warning rather than
    /// relocating into an unreliable destination.
    AbortTempDirUnicode,
    /// A destination was computed, but the file looks like a multipart
    /// `.rar`/numeric-extension archive member — relocation is skipped
    /// entirely (moving one part without its siblings would break the
    /// archive).
    AbortMultipart,
    /// Relocate to this destination.
    Relocate(Destination),
}

/// Ports `MoveInputFileIfNecessary`'s destination decision
/// (UniExtract.au3:2218-2245, minus the actual move/copy and
/// bookkeeping at the end — see [`decide_relocation_mode`] for that).
#[allow(clippy::too_many_arguments)]
pub fn plan_relocation(
    check_unicode: bool,
    file: &str,
    filedir: &str,
    filename: &str,
    filenamefull: &str,
    fileext: &str,
    is_unc: bool,
    temp_dir: &str,
) -> RelocationPlan {
    let needs_unicode_check =
        check_unicode && (!is_allowed_charset(file) || filename.starts_with("--"));

    let destination = if needs_unicode_check {
        if is_allowed_charset(filedir) {
            Some(Destination::GenerateUniqueName {
                dir: filedir.to_string(),
                ext: fileext.to_string(),
            })
        } else if !is_allowed_charset(temp_dir) {
            return RelocationPlan::AbortTempDirUnicode;
        } else if is_allowed_charset(filename) {
            Some(Destination::Fixed(format!("{temp_dir}\\{filenamefull}")))
        } else {
            Some(Destination::GenerateUniqueName {
                dir: temp_dir.to_string(),
                ext: fileext.to_string(),
            })
        }
    } else {
        None
    };

    // Only reached when the unicode check above didn't set a
    // destination -- matches the source's `If $new == 0 Then` priority
    // (see module doc comment).
    let destination = match destination {
        Some(d) => d,
        None => {
            if !is_unc {
                return RelocationPlan::NotNeeded;
            }
            Destination::Fixed(format!("{temp_dir}\\{filenamefull}"))
        }
    };

    if is_multipart_exempt(file, fileext) {
        return RelocationPlan::AbortMultipart;
    }

    RelocationPlan::Relocate(destination)
}

/// Ports the multipart exemption (UniExtract.au3:2241): `file` matches
/// `.*part\d+\.rar` (case-insensitive, unanchored), or `fileext`
/// contains a run of 3+ consecutive digits anywhere (`\d{3}`,
/// unanchored — a 4-digit extension like `"0012"` still matches).
fn is_multipart_exempt(file: &str, fileext: &str) -> bool {
    contains_part_digits_rar(file) || contains_three_consecutive_digits(fileext)
}

fn contains_part_digits_rar(file: &str) -> bool {
    let lower = file.to_lowercase();
    let bytes = lower.as_bytes();
    let mut search_from = 0;
    while let Some(rel_pos) = lower[search_from..].find("part") {
        let after_part = search_from + rel_pos + 4;
        let mut digits_end = after_part;
        while digits_end < bytes.len() && bytes[digits_end].is_ascii_digit() {
            digits_end += 1;
        }
        if digits_end > after_part && lower[digits_end..].starts_with(".rar") {
            return true;
        }
        search_from += rel_pos + 1;
        if search_from >= lower.len() {
            break;
        }
    }
    false
}

fn contains_three_consecutive_digits(s: &str) -> bool {
    let mut run = 0;
    for b in s.bytes() {
        if b.is_ascii_digit() {
            run += 1;
            if run >= 3 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// Ports the drive-letter comparison that decides `Move` vs `Copy`
/// (UniExtract.au3:2248): `StringLeft($file, 1) = StringLeft($new, 1)`
/// — single `=`, case-insensitive. Called once a [`Destination`] has
/// been resolved to a concrete path (after generating a unique name if
/// [`Destination::GenerateUniqueName`] applied).
pub fn decide_relocation_mode(file: &str, new_path: &str) -> UnicodeMode {
    match (file.chars().next(), new_path.chars().next()) {
        (Some(a), Some(b)) if a.eq_ignore_ascii_case(&b) => UnicodeMode::Move,
        _ => UnicodeMode::Copy,
    }
}

/// Ports the trailing output-directory check (UniExtract.au3:2262-2265):
/// if `$outdir` isn't in the allowed charset either, it's reset to
/// `$initoutdir` by the caller.
pub fn should_reset_outdir(outdir: &str) -> bool {
    !is_allowed_charset(outdir)
}

/// Capability C177 — a known bookkeeping-loss quirk, confirmed still
/// present in the current source, not a modeling gap in this port.
///
/// `unpack()`'s post-unpack re-scan (UniExtract.au3:3633-3635, inside
/// UniExtract.au3:3603-3642) re-enters the top level of the extraction
/// cascade: `$file = $sPath` then a direct call to `StartExtraction()`
/// — the same function the *original* run started in. `StartExtraction()`'s
/// very first statement (UniExtract.au3:378, in its "Reset variables"
/// block) is `$iUnicodeMode = False`, run unconditionally on *every*
/// entry, nested or not.
///
/// So: if the outer run had already relocated its input (`$iUnicodeMode`
/// set to `Move` or `Copy` by [`plan_relocation`]/[`decide_relocation_mode`]),
/// and the user then chooses to re-scan the unpacked output, the nested
/// `StartExtraction()` call discards that bookkeeping before doing
/// anything else. By the time the process actually reaches `terminate()`,
/// only the *innermost* re-entry's `$iUnicodeMode` is visible —
/// [`decide_unicode_reversion`] sees `None` and skips reverting the
/// outer relocation entirely, silently leaving the outer run's
/// temp copy/rename uncleaned.
///
/// This function exists to make that fact explicit and testable, not
/// to work around it — under this migration's parity contract, a
/// caller composing this port's pieces into a real orchestrator must
/// replicate the reset on every `StartExtraction`-equivalent re-entry
/// point (discard `outer_mode`, always yield `None`) to stay
/// behaviorally faithful. Threading `outer_mode` through instead would
/// be a silent behavior change, not a bug fix.
pub fn start_extraction_reentry_resets_unicode_mode(_outer_mode: UnicodeMode) -> UnicodeMode {
    UnicodeMode::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_relocation_means_no_reversion() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::None),
            (FileRevertAction::None, false)
        );
    }

    #[test]
    fn move_mode_reverts_by_moving_back_and_moves_outdir() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::Move),
            (FileRevertAction::MoveBack, true)
        );
    }

    #[test]
    fn copy_mode_reverts_by_recycling_and_still_moves_outdir() {
        assert_eq!(
            decide_unicode_reversion(UnicodeMode::Copy),
            (FileRevertAction::Recycle, true)
        );
    }

    #[test]
    fn allowed_charset_accepts_plain_ascii() {
        assert!(is_allowed_charset("archive_v2 (final).zip"));
    }

    /// Parity test for capabilities C175/C176: `$sRegExAscii` explicitly
    /// whitelists Western-European accented letters, both cases, plus
    /// `ß`/`°`/`²`/`³` — a French or German filename passes without
    /// triggering relocation.
    #[test]
    fn allowed_charset_whitelists_western_european_accents() {
        assert!(is_allowed_charset("Kondensmilch_süß.txt"));
        assert!(is_allowed_charset("Café_à_Montréal_ÀÉÎ.txt"));
        assert!(is_allowed_charset("Straße_10°_Fläche_20m²_30m³.txt"));
    }

    /// Parity test for capabilities C175/C176: characters outside the
    /// whitelist (Cyrillic, CJK, ...) are treated as needing
    /// relocation, exactly the same as any other genuinely non-ASCII
    /// text.
    #[test]
    fn allowed_charset_rejects_other_scripts() {
        assert!(!is_allowed_charset("файл.txt"));
        assert!(!is_allowed_charset("ファイル.txt"));
    }

    #[test]
    fn allowed_charset_rejects_empty_string() {
        assert!(!is_allowed_charset(""));
    }

    #[test]
    fn no_check_unicode_and_no_unc_means_no_relocation() {
        assert_eq!(
            plan_relocation(
                false,
                r"C:\downloads\файл.zip",
                r"C:\downloads",
                "файл",
                "файл.zip",
                "zip",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::NotNeeded
        );
    }

    /// Parity test for capabilities C175/C176: an ASCII-directory,
    /// unicode-named file is renamed in place (same directory), not
    /// moved to the temp dir.
    #[test]
    fn unicode_filename_in_ascii_directory_generates_name_in_place() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\downloads\файл.zip",
                r"C:\downloads",
                "файл",
                "файл.zip",
                "zip",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::GenerateUniqueName {
                dir: r"C:\downloads".to_string(),
                ext: "zip".to_string(),
            })
        );
    }

    /// Parity test for capabilities C175/C176: a unicode directory with
    /// an ASCII filename keeps the original filename, just relocated
    /// into `@TempDir`.
    #[test]
    fn unicode_directory_ascii_filename_keeps_name_in_temp_dir() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\файл\archive.zip",
                r"C:\файл",
                "archive",
                "archive.zip",
                "zip",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::Fixed(r"C:\Temp\archive.zip".to_string()))
        );
    }

    /// Parity test for capabilities C175/C176: a unicode directory
    /// *and* a unicode filename generates a randomized name in the
    /// temp dir instead of reusing it.
    #[test]
    fn unicode_directory_and_filename_generates_name_in_temp_dir() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\файл\файл.zip",
                r"C:\файл",
                "файл",
                "файл.zip",
                "zip",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::GenerateUniqueName {
                dir: r"C:\Temp".to_string(),
                ext: "zip".to_string(),
            })
        );
    }

    /// Parity test for capabilities C175/C176: when `@TempDir` itself
    /// isn't in the allowed charset, the source aborts with a warning
    /// rather than relocating anywhere.
    #[test]
    fn unicode_temp_dir_aborts_relocation() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\файл\archive.zip",
                r"C:\файл",
                "archive",
                "archive.zip",
                "zip",
                false,
                r"C:\Тemp",
            ),
            RelocationPlan::AbortTempDirUnicode
        );
    }

    /// Parity test for capability C175: a filename starting with `--`
    /// triggers relocation the same way a unicode filename does, even
    /// when the whole path is otherwise plain ASCII.
    #[test]
    fn double_dash_prefixed_filename_triggers_relocation() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\downloads\--flag-like.zip",
                r"C:\downloads",
                "--flag-like",
                "--flag-like.zip",
                "zip",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::GenerateUniqueName {
                dir: r"C:\downloads".to_string(),
                ext: "zip".to_string(),
            })
        );
    }

    /// Parity test for capability C176: a UNC path with no unicode
    /// anywhere still relocates, to `@TempDir` under the same filename.
    #[test]
    fn unc_path_alone_relocates_to_temp_dir_same_name() {
        assert_eq!(
            plan_relocation(
                true,
                r"\\server\share\archive.zip",
                r"\\server\share",
                "archive",
                "archive.zip",
                "zip",
                true,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::Fixed(r"C:\Temp\archive.zip".to_string()))
        );
    }

    /// Parity test for capability C176/C175 interaction: when a file is
    /// *both* unicode-named and UNC-reached, the unicode branch's own
    /// destination wins — UNC-ness contributes nothing extra.
    #[test]
    fn unicode_and_unc_together_use_the_unicode_destination() {
        assert_eq!(
            plan_relocation(
                true,
                r"\\server\share\файл.zip",
                r"\\server\share",
                "файл",
                "файл.zip",
                "zip",
                true,
                r"C:\Temp",
            ),
            RelocationPlan::Relocate(Destination::GenerateUniqueName {
                dir: r"\\server\share".to_string(),
                ext: "zip".to_string(),
            })
        );
    }

    /// Parity test for capabilities C175/C176: a multipart `.rar`
    /// member is exempted from relocation even though it would
    /// otherwise need it.
    #[test]
    fn multipart_rar_is_exempt_even_when_unicode() {
        assert_eq!(
            plan_relocation(
                true,
                r"C:\файл\файл.part2.rar",
                r"C:\файл",
                "файл.part2",
                "файл.part2.rar",
                "rar",
                false,
                r"C:\Temp",
            ),
            RelocationPlan::AbortMultipart
        );
    }

    /// Parity test for capabilities C175/C176: a `.001`-style numeric
    /// split-archive extension is exempted too, including via the UNC
    /// path.
    #[test]
    fn three_digit_extension_is_exempt_via_unc_too() {
        assert_eq!(
            plan_relocation(
                false,
                r"\\server\share\archive.001",
                r"\\server\share",
                "archive",
                "archive.001",
                "001",
                true,
                r"C:\Temp",
            ),
            RelocationPlan::AbortMultipart
        );
    }

    #[test]
    fn part_rar_pattern_requires_digits_between_part_and_rar() {
        assert!(contains_part_digits_rar(r"C:\x\file.part1.rar"));
        assert!(contains_part_digits_rar(r"C:\x\FILE.PART03.RAR"));
        assert!(!contains_part_digits_rar(r"C:\x\file.part.rar"));
        assert!(!contains_part_digits_rar(r"C:\x\participant.rar"));
    }

    #[test]
    fn three_consecutive_digits_matches_anywhere_including_longer_runs() {
        assert!(contains_three_consecutive_digits("001"));
        assert!(contains_three_consecutive_digits("0012"));
        assert!(!contains_three_consecutive_digits("z01"));
        assert!(!contains_three_consecutive_digits("exe"));
    }

    #[test]
    fn relocation_mode_is_move_when_drive_letters_match_case_insensitively() {
        assert_eq!(
            decide_relocation_mode(r"c:\downloads\file.zip", r"C:\downloads\Unicode_ab12.zip"),
            UnicodeMode::Move
        );
    }

    #[test]
    fn relocation_mode_is_copy_when_drive_letters_differ() {
        assert_eq!(
            decide_relocation_mode(r"C:\downloads\file.zip", r"D:\Temp\Unicode_ab12.zip"),
            UnicodeMode::Copy
        );
    }

    /// Parity test for capabilities C175/C176: a UNC source path (no
    /// drive letter, starts with `\`) never shares a "drive letter"
    /// with a local temp-dir destination, so it always copies.
    #[test]
    fn unc_source_always_copies() {
        assert_eq!(
            decide_relocation_mode(r"\\server\share\file.zip", r"C:\Temp\file.zip"),
            UnicodeMode::Copy
        );
    }

    #[test]
    fn outdir_reset_mirrors_the_allowed_charset_check() {
        assert!(!should_reset_outdir(r"C:\output"));
        assert!(should_reset_outdir(r"C:\output_файл"));
    }

    /// Parity test for capability C177: a nested `StartExtraction()`
    /// re-entry (via `unpack()`'s post-unpack re-scan) discards
    /// whatever `UnicodeMode` the outer run had, regardless of what it
    /// was -- the known bookkeeping-loss quirk, confirmed present in
    /// the current source.
    #[test]
    fn nested_start_extraction_reentry_discards_outer_unicode_mode() {
        for outer in [UnicodeMode::None, UnicodeMode::Move, UnicodeMode::Copy] {
            assert_eq!(
                start_extraction_reentry_resets_unicode_mode(outer),
                UnicodeMode::None
            );
        }
    }
}
