//! Actual Installer inner-blob handling (reuses `unzip.exe` + `7z.exe`,
//! plus embedded `aisetup.ini` rename manifest).

use super::{Invocation, WindowMode};

/// Builds the metadata-extraction invocation UniExtract2's `Case
/// $TYPE_ACTUAL` (UniExtract.au3:2355) makes: `<program> "<file>"`, run
/// in `tempoutdir` with the window minimized (`@SW_MINIMIZE`, explicit).
/// This pulls out `aisetup.ini`, the rename manifest
/// [`sanitize_destination_filename`]/[`resolve_rename`] consume — not the
/// actual installer payload, which a second, recursive `extract($TYPE_7Z,
/// ...)` dispatch handles (composite/recursive dispatch, capability C054,
/// not yet ported, and not modeled here).
///
/// **Not modeled here:** the preceding `DirCreate($tempoutdir)`; reading
/// `aisetup.ini`'s `[Files]` section and the `Cleanup($tempoutdir & "*")`
/// that follows — separate runtime behavior, not part of building this
/// one invocation.
pub fn meta_invocation(program: &str, file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Ports the destination-filename sanitization the rename loop applies to
/// each raw name read from `aisetup.ini`'s `[Files]` section
/// (UniExtract.au3:2371-2374): `<` becomes `[`, `>` becomes `]`.
///
/// **A real, preserved source bug — the trailing `?`-truncation "always
/// fires" quirk.** The source's own guard is `Local $iPos =
/// StringInStr($sDestination, "?") ... If $iPos > -1 Then $sDestination =
/// StringLeft($sDestination, $iPos - 1)`. `StringInStr` returns `0` (not
/// `-1`) when the substring isn't found, so `$iPos > -1` is true
/// unconditionally — the author evidently meant `<> 0` or `> 0`. When a
/// `?` genuinely is present, `$iPos - 1` correctly truncates everything
/// before it; when it *isn't*, `$iPos` is `0`, so `StringLeft($s, -1)`
/// runs instead — AutoIt's negative-count form of `StringLeft`, meaning
/// "all but the last N characters" — which silently drops the last
/// character of every renamed file that has no `?` in its name. This is a
/// genuine bug in the source, not something this port introduces, and
/// it's preserved here exactly rather than "fixed" into the evidently
/// intended `?`-only truncation.
pub fn sanitize_destination_filename(raw_name: &str) -> String {
    let replaced = raw_name.replace('<', "[").replace('>', "]");
    match replaced.find('?') {
        Some(idx) => replaced[..idx].to_string(),
        None => {
            let mut chars: Vec<char> = replaced.chars().collect();
            chars.pop();
            chars.into_iter().collect()
        }
    }
}

/// Ports one iteration of the Files-section rename loop
/// (UniExtract.au3:2367-2380): resolves both the source path
/// (`outdir\<source_key>`, the name `unzip.exe` actually extracted the
/// file under) and the sanitized destination path
/// (`outdir\<sanitized raw_name>`, the name `aisetup.ini` says it should
/// have).
pub fn resolve_rename(outdir: &str, source_key: &str, raw_name: &str) -> (String, String) {
    let source = format!("{outdir}\\{source_key}");
    let destination = format!("{outdir}\\{}", sanitize_destination_filename(raw_name));
    (source, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C114: the metadata invocation matches
    /// UniExtract.au3:2355's effective `unzip.exe "<file>"` call.
    #[test]
    fn meta_invocation_matches_source() {
        let inv = meta_invocation(
            r"C:\UniExtract\bin\unzip.exe",
            r"C:\downloads\installer.exe",
            r"C:\Temp\actual_tmp",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unzip.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\installer.exe".to_string()]);
        assert_eq!(inv.working_dir, r"C:\Temp\actual_tmp");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C114: `<`/`>` are replaced with `[`/`]`.
    #[test]
    fn sanitize_replaces_angle_brackets() {
        assert_eq!(sanitize_destination_filename("file<1>.txtX"), "file[1].txt");
    }

    /// Parity test for capability C114: a name containing `?` is
    /// truncated at the `?` — the evidently-intended behavior.
    #[test]
    fn sanitize_truncates_at_question_mark() {
        assert_eq!(sanitize_destination_filename("readme?.txt"), "readme");
    }

    /// Parity test for capability C114: a name with no `?` still loses
    /// its last character, reproducing the source's "always truncates"
    /// bug rather than leaving the name untouched.
    #[test]
    fn sanitize_drops_last_character_when_no_question_mark_present() {
        assert_eq!(sanitize_destination_filename("readme.txt"), "readme.tx");
    }

    /// Parity test for capability C114: full source/destination path
    /// resolution against `outdir`.
    #[test]
    fn resolve_rename_builds_source_and_destination_paths() {
        let (source, destination) = resolve_rename(r"C:\downloads\unpacked", "0001", "readme.txt");
        assert_eq!(source, r"C:\downloads\unpacked\0001");
        assert_eq!(destination, r"C:\downloads\unpacked\readme.tx");
    }
}
