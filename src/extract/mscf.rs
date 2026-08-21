//! MSCF (`$TYPE_MSCF`) — Microsoft Cabinet self-extracting installer:
//! try 7-Zip first, recursively; on failure, fall back to Exeinfo PE's
//! MSI-ripping automation and extract whatever `.cab` files it surfaces.
//!
//! ```autoit
//! Case $TYPE_MSCF
//!     $oldfiles = ReturnFiles($outdir)
//!     extract($TYPE_7Z, -1, "", False, True)
//!
//!     ; If 7z fails, remove useless files and extract cab files from installer
//!     MoveFiles($outdir, $tempoutdir, False, $oldfiles, True, False)
//!     DirRemove($tempoutdir, True)
//!     Sleep(1000)
//!
//!     If RipExeInfo($tempoutdir, "{DOWN}{DOWN}{DOWN}{DOWN}{DOWN}{RIGHT}{DOWN}{DOWN}{DOWN}") Then
//!         Local $aFiles = _FileListToArrayRec($tempoutdir, "*.cab", $FLTAR_FILES, $FLTAR_RECUR, $FLTAR_NOSORT, $FLTAR_FULLPATH)
//!         If Not @error Then
//!             For $i = 1 To $aFiles[0]
//!                 Cout("Extracting cab file " & $aFiles[$i])
//!                 _Run($7z & ' x "' & $aFiles[$i] & '"', $tempoutdir, @SW_HIDE, True, True, True, False)
//!                 If $success == $RESULT_SUCCESS Then Cleanup($aFiles[$i])
//!             Next
//!         EndIf
//!
//!         MoveFiles($tempoutdir, $outdir, False, "", True, True)
//!         Local $aCleanup[] = ["resource.dat", "cp*.bin", "*.cab"]
//!         Cleanup($aCleanup)
//!         $success = $RESULT_UNKNOWN
//!     Else
//!         $success = $RESULT_FAILED
//!     EndIf
//! ```
//!
//! **The recursive call's own failure is what makes the rest of this
//! `Case` reachable at all.** `extract($TYPE_7Z, -1, "", False, True)`
//! (UniExtract.au3:2816) uses `return_success = false, return_fail =
//! true` — the same shape as `extract::zip`'s recursive call. Per
//! `extract::completion` (C054/C181): a *successful* recursive 7-Zip
//! extraction terminates the whole process right there — the entire
//! rest of this `Case` (`MoveFiles`/`DirRemove`/`RipExeInfo`/the
//! cab-extraction loop) is unreachable on success, matching the
//! source's own comment ("If 7z fails, remove useless files..."). A
//! *failed* recursive extraction always returns `false`, which this
//! `Case` doesn't even check — unlike `Case $TYPE_ZIP`'s explicit `If
//! Not extract(...) Then`, there's no conditional here at all, because
//! the termination side effect alone already guarantees everything
//! below only runs on failure.
//!
//! **Scope — the fallback path is genuinely GUI-blocked.** `RipExeInfo`
//! (UniExtract.au3:1861-1896, already investigated for C069) drives
//! Exeinfo PE via real Win32 window/control automation (`WinWait`,
//! `MouseMove`, `ControlClick`, `ControlSend`, polling a list box via
//! `_GUICtrlListBox_FindString`) — this crate has no Win32
//! GUI-automation FFI, so whether the fallback even finds an MSI to rip
//! can't be determined here. What's ported: the recursive-dispatch
//! shape above, the per-cab-file extraction invocation
//! ([`cab_extract_invocation`]), the `RipExeInfo` key sequence this call
//! site uses ([`RIP_EXEINFO_KEY_SEQUENCE`]), and the cleanup target list
//! ([`SUCCESS_CLEANUP_TARGETS`]) — real filesystem I/O (`ReturnFiles`,
//! `MoveFiles`, `DirRemove`, `Sleep`, `_FileListToArrayRec`, `Cleanup`)
//! and `RipExeInfo` itself stay out of scope. Manifest row stays
//! `REQUIRED`.

use super::{Invocation, WindowMode};

/// The literal keystroke sequence this call site sends `RipExeInfo`
/// (UniExtract.au3:2823) — five Down-arrow presses, then Right, then
/// three more Down — the keyboard navigation Exeinfo PE's UI needs to
/// reach its MSI-rip command for this installer type's dialog layout.
/// Pinned down as data even though `RipExeInfo` itself (real Win32 GUI
/// automation, C069) isn't implemented here.
pub const RIP_EXEINFO_KEY_SEQUENCE: &str =
    "{DOWN}{DOWN}{DOWN}{DOWN}{DOWN}{RIGHT}{DOWN}{DOWN}{DOWN}";

/// Builds the per-`.cab`-file extraction invocation the source runs
/// inside its `For $i = 1 To $aFiles[0]` loop (UniExtract.au3:2827):
/// `<program> x "<cab_file>"`, run in `tempoutdir` with the window
/// hidden.
pub fn cab_extract_invocation(program: &str, cab_file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), cab_file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// The final cleanup targets (UniExtract.au3:2833) once every
/// discovered `.cab` has been extracted — real `Cleanup(...)` execution
/// stays out of scope (see module doc comment).
pub const SUCCESS_CLEANUP_TARGETS: &[&str] = &["resource.dat", "cp*.bin", "*.cab"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rip_exeinfo_key_sequence_matches_source() {
        assert_eq!(
            RIP_EXEINFO_KEY_SEQUENCE,
            "{DOWN}{DOWN}{DOWN}{DOWN}{DOWN}{RIGHT}{DOWN}{DOWN}{DOWN}"
        );
    }

    #[test]
    fn cab_extract_invocation_matches_source() {
        let inv = cab_extract_invocation(
            r"C:\UniExtract\bin\7z.exe",
            r"C:\temp\scratch\data1.cab",
            r"C:\temp\scratch",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\temp\scratch\data1.cab".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\temp\scratch");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn success_cleanup_targets_match_source() {
        assert_eq!(
            SUCCESS_CLEANUP_TARGETS,
            &["resource.dat", "cp*.bin", "*.cab"]
        );
    }

    /// Parity test for capabilities C054/C181: this call site's
    /// recursive extraction uses the same `(return_success=false,
    /// return_fail=true)` shape as `extract::zip`'s — success
    /// terminates the process outright, matching the source's own
    /// "If 7z fails..." comment about everything below this call.
    #[test]
    fn recursive_call_shares_zips_terminate_on_success_shape() {
        use crate::extract::completion::{resolve_completion, CompletionOutcome, ExtractionResult};
        use crate::status::Status;

        assert_eq!(
            resolve_completion(ExtractionResult::Success, false, true),
            CompletionOutcome::Terminate(Status::Success)
        );
        assert_eq!(
            resolve_completion(ExtractionResult::Failed, false, true),
            CompletionOutcome::Return(false)
        );
    }
}
