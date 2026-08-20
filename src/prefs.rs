//! Persisted preference values from `UniExtract.ini`'s `[UniExtract
//! Preferences]` section, read at startup by `ReadPrefs()`
//! (UniExtract.au3:697-791) via the generic `LoadPref()` helper
//! (UniExtract.au3:825-841). Each preference is its own capability; this
//! module ports the ones whose resolved value follows a rule non-trivial
//! enough to need a dedicated function rather than a plain default.

/// Ports `LoadPref($STATUS_TIMEOUT, $Timeout)` followed by the two lines
/// immediately after it (UniExtract.au3:744-746), together the whole of
/// capability C026.
///
/// `stored_seconds` is `LoadPref`'s normal-path result: the `timeout` ini
/// value already coerced through `_Max(Int($return), -1)` — `Some(n)`.
/// `None` represents `LoadPref`'s *error* path (the `timeout` key is
/// missing or unreadable): that branch never assigns its `ByRef $value`
/// parameter, so `$Timeout` keeps whatever it already held — the
/// `Global $Timeout = 60000` declaration at UniExtract.au3:151, in
/// *milliseconds* — and this function still multiplies it by 1000 as if
/// it were seconds, same as the source does. That's a genuine unit-mismatch
/// quirk in UniExtract2 (not something this port invented or fixed): a
/// truly first-run process with no `timeout` key ends up with a
/// 60,000,000ms (~16.7 hour) extraction timeout, not the 60 seconds the
/// `Global` declaration's own comment implies. Every other path — any
/// stored value, however small or negative — clamps to the intended
/// 60-second default once converted to ms and found under 10 seconds.
pub fn resolve_timeout_ms(stored_seconds: Option<i64>) -> i64 {
    let value_before_multiply = stored_seconds.unwrap_or(60_000);
    let timeout_ms = value_before_multiply * 1000;
    if timeout_ms < 10_000 {
        60_000
    } else {
        timeout_ms
    }
}

/// Mirrors UniExtract2's `$OPTION_*` enum (UniExtract.au3:97:
/// `Enum $OPTION_KEEP, $OPTION_DELETE, $OPTION_ASK, $OPTION_MOVE`).
/// Shared by two preferences — `deletesourcefile` (C024) and `cleanup`
/// (C033) — though only Keep/Ask/Delete are ever offered for
/// `deletesourcefile` through the GUI (UniExtract.au3:6393-6395); `Move`
/// is representable here because `LoadPref` stores whatever integer was
/// in the ini without validating it against the option's own UI, and
/// [`should_delete_source_file`] (C158) treats it exactly like `Keep`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteSourceFileOption {
    #[default]
    Keep,
    Delete,
    Ask,
    Move,
}

/// C024: parses the `deletesourcefile` preference's raw ini integer the
/// way `LoadPref`'s `_Max(Int($return), -1)` clamp plus AutoIt's `Enum`
/// numbering does — 1=Delete, 2=Ask, 3=Move, anything else (including a
/// missing/unreadable key, `LoadPref`'s error path) falls back to the
/// `Global $eOptDeleteSourceFile = $OPTION_KEEP` default
/// (UniExtract.au3:150).
pub fn parse_delete_source_file_option(raw: Option<i64>) -> DeleteSourceFileOption {
    match raw {
        Some(1) => DeleteSourceFileOption::Delete,
        Some(2) => DeleteSourceFileOption::Ask,
        Some(3) => DeleteSourceFileOption::Move,
        _ => DeleteSourceFileOption::Keep,
    }
}

/// C033: parses the `cleanup` preference's raw ini integer using the same
/// `$OPTION_*` numbering as [`parse_delete_source_file_option`], but with
/// a different fallback for a missing/unreadable/out-of-range value:
/// `Global $iCleanup = $OPTION_MOVE` (UniExtract.au3:162), not
/// `$OPTION_KEEP`. In practice `cleanup` only ever gets *written* as
/// `Delete` or `Move` through the GUI (a single checkbox, `$iCleanup =
/// _IsChecked(...) ? $OPTION_DELETE : $OPTION_MOVE`,
/// UniExtract.au3:6525) — `Keep`/`Ask` are representable here purely
/// because `LoadPref` parses whatever integer is in the ini without
/// validating it against that.
pub fn parse_cleanup_option(raw: Option<i64>) -> DeleteSourceFileOption {
    match raw {
        Some(0) => DeleteSourceFileOption::Keep,
        Some(1) => DeleteSourceFileOption::Delete,
        Some(2) => DeleteSourceFileOption::Ask,
        Some(3) => DeleteSourceFileOption::Move,
        _ => DeleteSourceFileOption::Move,
    }
}

/// C158: ports the deletion condition inside `terminate()`'s
/// `$STATUS_SUCCESS` case (UniExtract.au3:4204):
/// `$eOptDeleteSourceFile = $OPTION_DELETE Or ($eOptDeleteSourceFile =
/// $OPTION_ASK And Not $silentmode And Prompt(32 + 4, 'FILE_DELETE',
/// $file))`. `user_confirmed_delete` stands in for the `Prompt(...)` call
/// (the GUI confirmation dialog itself is out of scope, deferred GUI
/// subsystem, manifest row D001) — AutoIt's `And` short-circuits, so the
/// source never actually shows that prompt in silent mode, matching this
/// function's `!silent_mode &&` guard before it's consulted.
pub fn should_delete_source_file(
    option: DeleteSourceFileOption,
    silent_mode: bool,
    user_confirmed_delete: bool,
) -> bool {
    match option {
        DeleteSourceFileOption::Delete => true,
        DeleteSourceFileOption::Ask => !silent_mode && user_confirmed_delete,
        DeleteSourceFileOption::Keep | DeleteSourceFileOption::Move => false,
    }
}

/// C035: resolves which password-list file path `_FindArchivePassword`
/// reads from (UniExtract.au3:726,4855-4860). The default is
/// `$settingsdir\passwords.txt`; if reading it fails (`FileReadToArray`
/// setting `@error`), the source falls back to `@ScriptDir\passwords.txt`
/// instead. `settingsdir_password_file_readable` stands in for that read
/// attempt's success — the actual file I/O is out of scope for this pure
/// path-selection function.
pub fn password_list_path(
    settingsdir: &str,
    script_dir: &str,
    settingsdir_password_file_readable: bool,
) -> String {
    if settingsdir_password_file_readable {
        format!("{settingsdir}\\passwords.txt")
    } else {
        format!("{script_dir}\\passwords.txt")
    }
}

/// C021: ports `WriteHist`'s move-to-front/dedupe/cap-at-10 semantics
/// (UniExtract.au3:857-869), expressed as the resulting ordered list a
/// subsequent `ReadHist` (UniExtract.au3:844-854) would observe.
///
/// `WriteHist` writes `new_item` to ini key `"0"` unconditionally, then
/// walks the *old* history (as `ReadHist` returns it — already
/// non-empty-only, in slot order) for at most 9 entries, re-writing each
/// to its own original ini key except any entry equal to `new_item`,
/// which it deletes instead of rewriting. Two quirks fall out of that,
/// both preserved here:
///
/// - **A dedup match leaves a hole, not a shift.** Deleting a mid-list
///   duplicate's ini key doesn't pull later entries forward to fill the
///   gap — but `ReadHist` skips empty slots when reconstructing the list,
///   so the hole is invisible to every consumer that only ever reads
///   history through `ReadHist`, which is every consumer in the source.
///   This function models that externally observable list, not
///   `WriteHist`'s raw ini key layout.
/// - **The 9-entry scan is positional, not count-of-survivors.** The loop
///   stops after considering the old list's first 9 entries regardless of
///   whether one of them got dropped as a duplicate — it never reaches
///   into a 10th old entry to backfill the slot a dedup match freed up.
///   A duplicate found within the first 9 old entries therefore shrinks
///   the resulting list below 10, rather than keeping it topped up.
pub fn push_history(existing: &[String], new_item: &str) -> Vec<String> {
    let mut result = vec![new_item.to_string()];
    for item in existing.iter().take(9) {
        if item != new_item {
            result.push(item.clone());
        }
    }
    result
}

/// Ports `LoadPref`'s int-preference path (`$bInt = True`, the default,
/// UniExtract.au3:825-841) as applied to a simple 0/1-valued preference
/// read as a boolean. UniExtract2 treats any nonzero integer as truthy in
/// AutoIt's `If $var Then` checks, so `LoadPref`'s `_Max(Int($return),
/// $iMin)` clamp — which can only ever push a stray negative ini value up
/// to -1, itself still nonzero/truthy — never changes the boolean outcome
/// for any of the ten preferences below, so this function only needs to
/// model the missing-key fallback. `raw` is `LoadPref`'s normal-path
/// result (the ini value, as a bool); `None` represents its error path
/// (key missing/unreadable), where `$value` keeps whatever it already
/// held — each preference's own `Global` declaration's default, passed as
/// `default_when_missing`.
pub fn resolve_bool_pref(raw: Option<bool>, default_when_missing: bool) -> bool {
    raw.unwrap_or(default_when_missing)
}

/// C020: `batchenabled` preference default (`Global $batchEnabled = 0`,
/// UniExtract.au3:140) — persisted flag driving batch-queue continuation
/// logic on process exit.
pub const BATCHENABLED_DEFAULT: bool = false;

/// C022: `appendext` preference default (`Global $appendext = 0`,
/// UniExtract.au3:143) — controls whether an extension is appended to
/// extracted output.
pub const APPENDEXT_DEFAULT: bool = false;

/// C023: `warnexecute` preference default (`Global $bOptWarnExecute = 1`,
/// UniExtract.au3:144) — warn before running/executing self-extracting
/// content.
pub const WARNEXECUTE_DEFAULT: bool = true;

/// C025: `freespacecheck` preference default
/// (`Global $bOptCheckFreeSpace = 1`, UniExtract.au3:145) — enable/disable
/// a disk-space check before extraction.
pub const FREESPACECHECK_DEFAULT: bool = true;

/// C027: `keepoutputdir` preference default
/// (`Global $bOptLockOutputDirectory = 0`, UniExtract.au3:163).
pub const KEEPOUTPUTDIR_DEFAULT: bool = false;

/// C028: `log` preference default (`Global $bOptCreateLog = 0`,
/// UniExtract.au3:159) — enable/disable a per-extraction debug log file;
/// overridable per-run by `/nolog` (C008).
pub const LOG_DEFAULT: bool = false;

/// C029: `extract` preference default (`Global $extract = 1`,
/// UniExtract.au3:166) — persisted default for extract-vs-scan-only;
/// overridden per-run by `/scan` (C003).
pub const EXTRACT_DEFAULT: bool = true;

/// C030: `unicodecheck` preference default (`Global $checkUnicode = 1`,
/// UniExtract.au3:167) — enables detection/handling of non-ASCII
/// filenames requiring temp rename.
pub const UNICODECHECK_DEFAULT: bool = true;

/// C031: `extractvideotrack` preference default
/// (`Global $bOptExtractVideo = 1`, UniExtract.au3:168) — controls
/// whether video-track extraction is attempted for media files.
pub const EXTRACTVIDEOTRACK_DEFAULT: bool = true;

/// C032: `silentmode` preference default (`Global $silentmode = 0`,
/// UniExtract.au3:165) — persisted default for silent mode, independent
/// of the per-run `/silent` flag (C007).
pub const SILENTMODE_DEFAULT: bool = false;

/// C034: `BatchRecurse` preference default — recurse into subdirectories
/// when batch-adding a directory (C014). Read directly via `IniRead` with
/// its own default argument (UniExtract.au3:6611: `Local Static $bRecurse
/// = Number(IniRead($prefs, "UniExtract Preferences", "BatchRecurse",
/// 1))`) rather than through the generic `LoadPref` helper every other
/// preference in this module uses — no `SavePref` write-back on a missing
/// key, and the read only happens once per process (`Local Static`).
/// Observably, though, `IniRead`'s own default argument resolves a
/// missing/unreadable key to `1` (true) the same way
/// [`resolve_bool_pref`]'s `default_when_missing` parameter does, so this
/// preference reuses that function rather than duplicating its shape.
pub const BATCHRECURSE_DEFAULT: bool = true;

/// Approximates AutoIt's `_PathFull(path, base)` UDF: not defined anywhere
/// in this port's source checkout (an external/bundled include this repo
/// doesn't carry), so this models its well-established, standard meaning
/// — a relative `path` resolves against `base`; an already-absolute path
/// (a drive letter, `C:...`, or a UNC share, `\\...`) is returned
/// unchanged. `resolve_batchqueue_path`/`resolve_filescanlogfile_path`
/// (C018/C019) use it directly; `pub(crate)` since `extract::unity`
/// (C121) needs the same approximation for its asset-destination
/// resolution.
pub(crate) fn resolve_relative_path(path: &str, base: &str) -> String {
    let bytes = path.as_bytes();
    let is_absolute = (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || path.starts_with(r"\\");
    if is_absolute {
        path.to_string()
    } else {
        format!("{base}\\{path}")
    }
}

/// C018: ports the `batchqueue` preference's default-and-override
/// resolution (UniExtract.au3:721,729-730): `LoadPref("batchqueue",
/// $batchQueue, False)` (string mode: a present value is used verbatim,
/// a missing/unreadable key leaves `$batchQueue` at its `Global` default,
/// `$settingsdir & "\batch.queue"`), followed by `If $batchQueue Then
/// $batchQueue = _PathFull($batchQueue, $settingsdir)` — AutoIt's truthy
/// check on the *value itself*, not `@error`, so this resolution step
/// runs on the default too (a no-op here since the default is already
/// absolute) and is skipped only for the one edge case where the ini
/// explicitly sets an empty `batchqueue=` value.
pub fn resolve_batchqueue_path(raw: Option<&str>, settingsdir: &str) -> String {
    let value = raw
        .map(str::to_string)
        .unwrap_or_else(|| format!("{settingsdir}\\batch.queue"));
    if value.is_empty() {
        value
    } else {
        resolve_relative_path(&value, settingsdir)
    }
}

/// C019: ports the `filescanlogfile` preference's default-and-override
/// resolution (UniExtract.au3:722,725,731-732): `LoadPref("filescanlogfile",
/// $fileScanLogFile, False)` (string mode, same missing-key-keeps-default
/// semantics as C018 — the default is `$logdir & "filescan.txt"`, i.e.
/// `$settingsdir\log\filescan.txt`), followed by `If Not @error Then
/// $fileScanLogFile = _PathFull(...)` — unlike C018, this checks
/// `LoadPref`'s error flag rather than the value's truthiness, so the
/// resolution step is skipped whenever the default is kept (harmless
/// either way since that default is already absolute) and only ever runs
/// on a value actually read from the ini.
pub fn resolve_filescanlogfile_path(raw: Option<&str>, settingsdir: &str) -> String {
    match raw {
        Some(v) => resolve_relative_path(v, settingsdir),
        None => format!("{settingsdir}\\log\\filescan.txt"),
    }
}

/// C017: resolves the `language` preference — `LoadPref("language",
/// $language, False)` followed by:
/// ```autoit
/// If Not HasTranslation($language) Then
///     $language = _WinAPI_GetLocaleInfo(_WinAPI_GetSystemDefaultUILanguage(), $LOCALE_SENGLANGUAGE)
///     If Not HasTranslation($language) Then $language = _GetOSLanguage()
///     If Not HasTranslation($language) Then $language = "English"
///     SavePref('language', $language)
/// EndIf
/// ```
/// `stored` is the already-loaded preference value (`None` when
/// `LoadPref` reports missing/unreadable, matching every other
/// `LoadPref`-backed resolver in this module). `has_translation` is
/// caller-supplied — a real check against installed `lang/*.ini` files;
/// full translation catalogs beyond a default English set are out of
/// scope (manifest row D006). `os_ui_language` and `os_language` are the
/// two OS-locale candidates the source tries in order
/// (`_WinAPI_GetLocaleInfo(_WinAPI_GetSystemDefaultUILanguage(), ...)`,
/// then `_GetOSLanguage()`), caller-supplied since both are real OS
/// calls. Persisting the resolved value (`SavePref`) is the caller's job
/// — this function only decides what the value should be.
pub fn resolve_language(
    stored: Option<&str>,
    has_translation: impl Fn(&str) -> bool,
    os_ui_language: &str,
    os_language: &str,
) -> String {
    if let Some(lang) = stored {
        if has_translation(lang) {
            return lang.to_string();
        }
    }
    if has_translation(os_ui_language) {
        return os_ui_language.to_string();
    }
    if has_translation(os_language) {
        return os_language.to_string();
    }
    "English".to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_cleanup_option, parse_delete_source_file_option, password_list_path, push_history,
        resolve_batchqueue_path, resolve_bool_pref, resolve_filescanlogfile_path, resolve_language,
        resolve_timeout_ms, should_delete_source_file, DeleteSourceFileOption, APPENDEXT_DEFAULT,
        BATCHENABLED_DEFAULT, BATCHRECURSE_DEFAULT, EXTRACTVIDEOTRACK_DEFAULT, EXTRACT_DEFAULT,
        FREESPACECHECK_DEFAULT, KEEPOUTPUTDIR_DEFAULT, LOG_DEFAULT, SILENTMODE_DEFAULT,
        UNICODECHECK_DEFAULT, WARNEXECUTE_DEFAULT,
    };

    /// Parity test for capability C026: a normally-stored preference value
    /// (in seconds, already through LoadPref's `_Max(Int(x), -1)` clamp)
    /// converts to milliseconds, matching `$Timeout *= 1000`
    /// (UniExtract.au3:745).
    #[test]
    fn stored_seconds_convert_to_milliseconds() {
        assert_eq!(resolve_timeout_ms(Some(60)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(30)), 30_000);
        assert_eq!(resolve_timeout_ms(Some(120)), 120_000);
    }

    /// Any stored value under 10 seconds (once converted to ms) — the
    /// "minimum enforced 10s" half of C026 — resets to the 60-second
    /// default rather than being clamped up to 10s (UniExtract.au3:746:
    /// `If $Timeout < 10000 Then $Timeout = 60000`).
    #[test]
    fn values_under_ten_seconds_reset_to_the_sixty_second_default() {
        assert_eq!(resolve_timeout_ms(Some(9)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(1)), 60_000);
        assert_eq!(resolve_timeout_ms(Some(0)), 60_000);
        // LoadPref's own `_Max(Int($return), -1)` clamp means the lowest
        // value that can ever reach this function is -1.
        assert_eq!(resolve_timeout_ms(Some(-1)), 60_000);
    }

    /// Exactly 10 seconds is the boundary: `< 10000`, not `<= 10000`, so
    /// 10000ms survives unchanged.
    #[test]
    fn exactly_ten_seconds_is_not_reset() {
        assert_eq!(resolve_timeout_ms(Some(10)), 10_000);
    }

    /// A missing/unreadable `timeout` preference key hits LoadPref's error
    /// branch, which never assigns `$Timeout` — so the pre-call value (the
    /// 60000-millisecond `Global` default) survives, gets misinterpreted
    /// as seconds by the unconditional `*= 1000`, and comes out far larger
    /// than the intended 60-second default. A real, preserved quirk.
    #[test]
    fn missing_preference_key_reproduces_the_sixty_million_millisecond_quirk() {
        assert_eq!(resolve_timeout_ms(None), 60_000_000);
    }

    /// Parity test for capability C024: the `$OPTION_*` enum's AutoIt
    /// numbering (`Keep`=0, `Delete`=1, `Ask`=2, `Move`=3), and a
    /// missing/unreadable/out-of-range value falling back to the
    /// `$OPTION_KEEP` default, exactly as `LoadPref`'s error path (never
    /// assigning its `ByRef` output) leaves `$eOptDeleteSourceFile` at its
    /// `Global` declaration's value.
    #[test]
    fn delete_source_file_option_parses_autoit_enum_numbering() {
        assert_eq!(
            parse_delete_source_file_option(Some(1)),
            DeleteSourceFileOption::Delete
        );
        assert_eq!(
            parse_delete_source_file_option(Some(2)),
            DeleteSourceFileOption::Ask
        );
        assert_eq!(
            parse_delete_source_file_option(Some(3)),
            DeleteSourceFileOption::Move
        );
        assert_eq!(
            parse_delete_source_file_option(Some(0)),
            DeleteSourceFileOption::Keep
        );
        assert_eq!(
            parse_delete_source_file_option(Some(99)),
            DeleteSourceFileOption::Keep
        );
        assert_eq!(
            parse_delete_source_file_option(None),
            DeleteSourceFileOption::Keep
        );
    }

    /// Parity test for capability C033: same `$OPTION_*` numbering as
    /// `deletesourcefile` (C024), but a missing/unreadable/out-of-range
    /// value falls back to `Move`, matching `Global $iCleanup =
    /// $OPTION_MOVE` (UniExtract.au3:162) rather than `$OPTION_KEEP`.
    #[test]
    fn cleanup_option_parses_autoit_enum_numbering_with_move_fallback() {
        assert_eq!(parse_cleanup_option(Some(0)), DeleteSourceFileOption::Keep);
        assert_eq!(
            parse_cleanup_option(Some(1)),
            DeleteSourceFileOption::Delete
        );
        assert_eq!(parse_cleanup_option(Some(2)), DeleteSourceFileOption::Ask);
        assert_eq!(parse_cleanup_option(Some(3)), DeleteSourceFileOption::Move);
        assert_eq!(parse_cleanup_option(Some(99)), DeleteSourceFileOption::Move);
        assert_eq!(parse_cleanup_option(None), DeleteSourceFileOption::Move);
    }

    /// Parity test for capability C035: the default `$settingsdir` path is
    /// used when it's readable; `_FindArchivePassword` falls back to
    /// `@ScriptDir\passwords.txt` only if reading the default fails
    /// (UniExtract.au3:4855-4860).
    #[test]
    fn password_list_path_matches_source_fallback() {
        assert_eq!(
            password_list_path(
                r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract",
                r"C:\Program Files\UniExtract",
                true
            ),
            r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract\passwords.txt"
        );
        assert_eq!(
            password_list_path(
                r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract",
                r"C:\Program Files\UniExtract",
                false
            ),
            r"C:\Program Files\UniExtract\passwords.txt"
        );
    }

    /// Shared behavior test for `resolve_bool_pref`: a present ini value
    /// always wins; a missing/unreadable key falls back to whatever
    /// default is passed in, matching `LoadPref`'s error path leaving
    /// `$value` untouched.
    #[test]
    fn resolve_bool_pref_prefers_raw_value_over_default() {
        assert!(resolve_bool_pref(Some(true), false));
        assert!(!resolve_bool_pref(Some(false), true));
        assert!(resolve_bool_pref(None, true));
        assert!(!resolve_bool_pref(None, false));
    }

    /// Parity test for capability C020: `batchenabled` defaults to `false`
    /// (`Global $batchEnabled = 0`, UniExtract.au3:140).
    #[test]
    fn batchenabled_preference_default_matches_source() {
        assert!(!resolve_bool_pref(None, BATCHENABLED_DEFAULT));
    }

    /// Parity test for capability C022: `appendext` defaults to `false`
    /// (`Global $appendext = 0`, UniExtract.au3:143).
    #[test]
    fn appendext_preference_default_matches_source() {
        assert!(!resolve_bool_pref(None, APPENDEXT_DEFAULT));
    }

    /// Parity test for capability C023: `warnexecute` defaults to `true`
    /// (`Global $bOptWarnExecute = 1`, UniExtract.au3:144).
    #[test]
    fn warnexecute_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, WARNEXECUTE_DEFAULT));
    }

    /// Parity test for capability C025: `freespacecheck` defaults to
    /// `true` (`Global $bOptCheckFreeSpace = 1`, UniExtract.au3:145).
    #[test]
    fn freespacecheck_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, FREESPACECHECK_DEFAULT));
    }

    /// Parity test for capability C027: `keepoutputdir` defaults to
    /// `false` (`Global $bOptLockOutputDirectory = 0`,
    /// UniExtract.au3:163).
    #[test]
    fn keepoutputdir_preference_default_matches_source() {
        assert!(!resolve_bool_pref(None, KEEPOUTPUTDIR_DEFAULT));
    }

    /// Parity test for capability C028: `log` defaults to `false`
    /// (`Global $bOptCreateLog = 0`, UniExtract.au3:159).
    #[test]
    fn log_preference_default_matches_source() {
        assert!(!resolve_bool_pref(None, LOG_DEFAULT));
    }

    /// Parity test for capability C029: `extract` defaults to `true`
    /// (`Global $extract = 1`, UniExtract.au3:166).
    #[test]
    fn extract_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, EXTRACT_DEFAULT));
    }

    /// Parity test for capability C030: `unicodecheck` defaults to `true`
    /// (`Global $checkUnicode = 1`, UniExtract.au3:167).
    #[test]
    fn unicodecheck_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, UNICODECHECK_DEFAULT));
    }

    /// Parity test for capability C031: `extractvideotrack` defaults to
    /// `true` (`Global $bOptExtractVideo = 1`, UniExtract.au3:168).
    #[test]
    fn extractvideotrack_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, EXTRACTVIDEOTRACK_DEFAULT));
    }

    /// Parity test for capability C032: `silentmode` defaults to `false`
    /// (`Global $silentmode = 0`, UniExtract.au3:165).
    #[test]
    fn silentmode_preference_default_matches_source() {
        assert!(!resolve_bool_pref(None, SILENTMODE_DEFAULT));
    }

    /// Parity test for capability C018: a missing/unreadable `batchqueue`
    /// key falls back to `$settingsdir\batch.queue`; a present relative
    /// value resolves against `settingsdir`; a present absolute value is
    /// kept as-is; an explicit empty value skips resolution entirely
    /// (AutoIt's `If $batchQueue Then` truthy check).
    #[test]
    fn batchqueue_path_matches_source_default_and_override() {
        assert_eq!(
            resolve_batchqueue_path(None, r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract"),
            r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract\batch.queue"
        );
        assert_eq!(
            resolve_batchqueue_path(
                Some("custom.queue"),
                r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract"
            ),
            r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract\custom.queue"
        );
        assert_eq!(
            resolve_batchqueue_path(Some(r"D:\queues\batch.queue"), r"C:\settings"),
            r"D:\queues\batch.queue"
        );
        assert_eq!(resolve_batchqueue_path(Some(""), r"C:\settings"), "");
    }

    /// Parity test for capability C019: a missing/unreadable
    /// `filescanlogfile` key falls back to
    /// `$settingsdir\log\filescan.txt` (skipping `_PathFull` entirely,
    /// matching the source's `If Not @error Then` gate rather than a
    /// truthiness check); a present relative value resolves against
    /// `settingsdir`; a present absolute value is kept as-is.
    #[test]
    fn filescanlogfile_path_matches_source_default_and_override() {
        assert_eq!(
            resolve_filescanlogfile_path(None, r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract"),
            r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract\log\filescan.txt"
        );
        assert_eq!(
            resolve_filescanlogfile_path(
                Some("scan-results.txt"),
                r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract"
            ),
            r"C:\Users\me\AppData\Roaming\Bioruebe\UniExtract\scan-results.txt"
        );
        assert_eq!(
            resolve_filescanlogfile_path(Some(r"D:\logs\filescan.txt"), r"C:\settings"),
            r"D:\logs\filescan.txt"
        );
    }

    /// Parity test for capability C017: a stored value with an installed
    /// translation is kept as-is, without consulting either OS-locale
    /// candidate.
    #[test]
    fn resolve_language_keeps_a_valid_stored_value() {
        let installed = ["German"];
        assert_eq!(
            resolve_language(
                Some("German"),
                |lang| installed.contains(&lang),
                "French",
                "French",
            ),
            "German"
        );
    }

    /// Parity test for capability C017: a missing or invalid stored value
    /// falls through to the OS UI-language candidate when it has a
    /// translation.
    #[test]
    fn resolve_language_falls_back_to_os_ui_language() {
        let installed = ["Spanish"];
        assert_eq!(
            resolve_language(None, |lang| installed.contains(&lang), "Spanish", "German",),
            "Spanish"
        );
        assert_eq!(
            resolve_language(
                Some("NotARealLanguage"),
                |lang| installed.contains(&lang),
                "Spanish",
                "German",
            ),
            "Spanish"
        );
    }

    /// Parity test for capability C017: when the OS UI-language candidate
    /// also has no translation, falls through to the second OS-locale
    /// candidate.
    #[test]
    fn resolve_language_falls_back_to_second_os_language_candidate() {
        let installed = ["Italian"];
        assert_eq!(
            resolve_language(
                None,
                |lang| installed.contains(&lang),
                "NotInstalled",
                "Italian",
            ),
            "Italian"
        );
    }

    /// Parity test for capability C017: when nothing matches — stored,
    /// nor either OS-locale candidate — falls all the way back to the
    /// literal `"English"` default.
    #[test]
    fn resolve_language_defaults_to_english_when_nothing_matches() {
        assert_eq!(
            resolve_language(None, |_| false, "NotInstalled", "AlsoNotInstalled"),
            "English"
        );
    }

    /// Parity test for capability C034: `BatchRecurse` defaults to `true`,
    /// matching `IniRead`'s own default argument
    /// (UniExtract.au3:6611: `IniRead(..., "BatchRecurse", 1)`).
    #[test]
    fn batchrecurse_preference_default_matches_source() {
        assert!(resolve_bool_pref(None, BATCHRECURSE_DEFAULT));
    }

    /// Parity test for capability C021: a brand-new item is prepended to
    /// an existing history list, with the rest carried over unchanged.
    #[test]
    fn push_history_prepends_new_item() {
        let existing = vec!["B".to_string(), "C".to_string()];
        assert_eq!(
            push_history(&existing, "A"),
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }

    /// Parity test for capability C021: re-using an item already in the
    /// history moves it to the front instead of appearing twice — the
    /// hole `WriteHist` leaves in the raw ini is invisible here because
    /// `ReadHist` skips empty slots (UniExtract.au3:849-850).
    #[test]
    fn push_history_deduplicates_and_moves_to_front() {
        let existing = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ];
        assert_eq!(
            push_history(&existing, "C"),
            vec![
                "C".to_string(),
                "A".to_string(),
                "B".to_string(),
                "D".to_string(),
                "E".to_string(),
            ]
        );
    }

    /// Parity test for capability C021: the 9-entry scan over the old
    /// list is positional, not count-of-survivors — a duplicate among the
    /// first 9 old entries shrinks the result below 10 rather than
    /// reaching into a 10th old entry to backfill it.
    #[test]
    fn push_history_ten_entry_cap_does_not_backfill_a_deduped_slot() {
        let existing: Vec<String> = ('a'..='j').map(|c| c.to_string()).collect(); // 10 entries: a..j
        let result = push_history(&existing, "e");
        // new "e" + old entries a,b,c,d,f,g,h,i (9 scanned, "e" dropped as
        // duplicate, "j" never reached) = 9 entries, not 10.
        assert_eq!(
            result,
            vec!["e", "a", "b", "c", "d", "f", "g", "h", "i"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    /// Parity test for capability C021: an empty history list just yields
    /// the new item alone.
    #[test]
    fn push_history_from_empty_history() {
        assert_eq!(push_history(&[], "A"), vec!["A".to_string()]);
    }

    /// Parity test for capability C158: `$OPTION_DELETE` always deletes;
    /// `$OPTION_ASK` deletes only outside silent mode and only if the
    /// (out-of-scope) confirmation prompt returned true; `$OPTION_KEEP`
    /// and `$OPTION_MOVE` never delete (UniExtract.au3:4204).
    #[test]
    fn should_delete_source_file_matches_source_condition() {
        assert!(should_delete_source_file(
            DeleteSourceFileOption::Delete,
            false,
            false
        ));
        assert!(should_delete_source_file(
            DeleteSourceFileOption::Delete,
            true,
            false
        ));

        assert!(should_delete_source_file(
            DeleteSourceFileOption::Ask,
            false,
            true
        ));
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Ask,
            false,
            false
        ));
        // Silent mode short-circuits before the (out-of-scope) prompt is
        // ever consulted, matching AutoIt's `And` short-circuit.
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Ask,
            true,
            true
        ));

        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Keep,
            false,
            true
        ));
        assert!(!should_delete_source_file(
            DeleteSourceFileOption::Move,
            false,
            true
        ));
    }
}
