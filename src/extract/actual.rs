//! Actual Installer inner-blob handling (reuses `unzip.exe` + `7z.exe`,
//! plus embedded `aisetup.ini` rename manifest).
//!
//! ```autoit
//! If Not extract($TYPE_7Z, -1, "", True, True) Then
//!     Cout("Failed to extract files")
//!     $success = $RESULT_FAILED
//! ElseIf Not IsArray($aFiles) Then
//!     Cout("Failed to read file names")
//! Else
//!     ; ... rename loop, see sanitize_destination_filename/resolve_rename ...
//! EndIf
//! ```
//!
//! The payload extraction is a **fully recursive** `extract()` call
//! (`return_success = true, return_fail = true`, `$arcdisp = -1`
//! suppresses its own tray progress box) — per `extract::completion`
//! (C054/C181), it always returns a plain boolean rather than
//! terminating, which [`decide_post_recursion_action`] then branches on
//! exactly as the source's `If Not extract(...) Then ... ElseIf ...`
//! does.

use super::{Invocation, WindowMode};

/// Builds the metadata-extraction invocation UniExtract2's `Case
/// $TYPE_ACTUAL` (UniExtract.au3:2355) makes: `<program> "<file>"`, run
/// in `tempoutdir` with the window minimized (`@SW_MINIMIZE`, explicit).
/// This pulls out `aisetup.ini`, the rename manifest
/// [`sanitize_destination_filename`]/[`resolve_rename`] consume — not the
/// actual installer payload, which the recursive `extract($TYPE_7Z, -1,
/// "", True, True)` dispatch handles (see module doc comment).
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

/// What `Case $TYPE_ACTUAL` does once the recursive `extract($TYPE_7Z,
/// -1, "", True, True)` call has returned (UniExtract.au3:2362-2382).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostRecursionAction {
    /// The recursive extraction returned `false` — this case's own
    /// `$success` is set to `$RESULT_FAILED`.
    MarkFailed,
    /// It returned `true`, but `aisetup.ini`'s `[Files]` section didn't
    /// parse as an array — nothing to rename, `$success` is left
    /// untouched.
    SkipRename,
    /// It returned `true` and the files array is valid — run the rename
    /// loop ([`resolve_rename`]/[`sanitize_destination_filename`], one
    /// call per entry).
    RunRenameLoop,
}

/// Ports the branch selection quoted in the module doc comment. `succeeded`
/// is the recursive `extract($TYPE_7Z, ...)` call's own return value —
/// per `extract::completion::resolve_completion`, always a `Return`
/// (never a `Terminate`) since this call site uses `return_success =
/// true, return_fail = true`.
pub fn decide_post_recursion_action(
    succeeded: bool,
    files_array_is_valid: bool,
) -> PostRecursionAction {
    if !succeeded {
        PostRecursionAction::MarkFailed
    } else if !files_array_is_valid {
        PostRecursionAction::SkipRename
    } else {
        PostRecursionAction::RunRenameLoop
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

    /// Parity test for capabilities C054/C114/C181: the recursive
    /// extraction failing marks this case's own result failed, regardless
    /// of the files array.
    #[test]
    fn post_recursion_marks_failed_when_recursive_extraction_failed() {
        assert_eq!(
            decide_post_recursion_action(false, true),
            PostRecursionAction::MarkFailed
        );
        assert_eq!(
            decide_post_recursion_action(false, false),
            PostRecursionAction::MarkFailed
        );
    }

    /// Parity test for capabilities C054/C114/C181: a succeeded recursive
    /// extraction with no valid files array skips the rename loop without
    /// marking failure.
    #[test]
    fn post_recursion_skips_rename_when_files_array_invalid() {
        assert_eq!(
            decide_post_recursion_action(true, false),
            PostRecursionAction::SkipRename
        );
    }

    /// Parity test for capabilities C054/C114/C181: a succeeded recursive
    /// extraction with a valid files array runs the rename loop.
    #[test]
    fn post_recursion_runs_rename_loop_when_files_array_valid() {
        assert_eq!(
            decide_post_recursion_action(true, true),
            PostRecursionAction::RunRenameLoop
        );
    }
}
