//! ZIP (`$TYPE_ZIP`): 7-Zip first, recursively, with Info-ZIP `unzip` as
//! a fallback that — per this call site's exact recursive-call
//! arguments — turns out to run unconditionally whenever it's reached
//! at all, not "conditionally" the way the source's `If` reads.
//!
//! ```autoit
//! Case $TYPE_ZIP
//!     If Not extract($TYPE_7Z, -1, $additionalParameters, False, True) Then
//!         If $arcdisp > -1 Then _CreateTrayMessageBox(t('EXTRACTING') & @CRLF & $arcdisp)
//!         _Run($zip & ' -x "' & $file & '"', $outdir, @SW_MINIMIZE, True, False)
//!     EndIf
//! ```
//!
//! The recursive `extract($TYPE_7Z, -1, $additionalParameters, False,
//! True)` call (UniExtract.au3:3385) uses `return_success = false,
//! return_fail = true` — the mirror image of `extract::forge`'s and
//! `extract::raiu`'s `(true, false)`. Per `extract::completion`
//! (C054/C181):
//! - On success, `return_success = false` means the recursive call
//!   *terminates the whole process* right there with `$STATUS_SUCCESS`
//!   — it never returns control to this `Case` at all.
//! - On failure, `return_fail = true` means it *always* returns `false`
//!   rather than terminating.
//!
//! **A genuine, non-obvious finding: `If Not extract(...) Then` is
//! effectively always true whenever it's reached at all.** Since a
//! successful recursive extraction never returns here (it terminates
//! first), the only way this line's `Then` branch is ever reached is via
//! the failure path — which always evaluates to `false`, so `Not false`
//! is always `true`. The Info-ZIP fallback below isn't conditional in
//! any meaningful sense; it runs whenever the recursive 7-Zip extraction
//! fails, and the whole `Case` has already exited the process otherwise.
//! The tests below demonstrate this directly against the shared
//! `extract::completion::resolve_completion` mechanism rather than
//! re-deriving it by hand.
//!
//! **Fallback invocation already ported.** The `_Run($zip & ' -x "' &
//! $file & '"', $outdir, @SW_MINIMIZE, True, False)` call itself is
//! `extract::table`'s `unzip` entry (C109, UniExtract.au3:3384-3388) —
//! not duplicated here.
//!
//! **Not modeled here:** `$arcdisp > -1`'s tray-message-box gate
//! decision is [`should_show_extracting_message`]; the box itself
//! (`_CreateTrayMessageBox`) is deferred GUI subsystem, manifest row
//! D001.

/// Ports `If $arcdisp > -1 Then` (UniExtract.au3:3386) — whether the
/// `EXTRACTING`-prefixed tray message shows before the fallback runs.
/// The box display itself is out of scope (see module doc comment).
pub fn should_show_extracting_message(arcdisp: i64) -> bool {
    arcdisp > -1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::completion::{resolve_completion, CompletionOutcome, ExtractionResult};
    use crate::status::Status;

    /// Parity test for capabilities C054/C181: `Case $TYPE_ZIP`'s
    /// recursive call terminates the whole process on success — it never
    /// returns control here.
    #[test]
    fn recursive_call_terminates_on_success() {
        assert_eq!(
            resolve_completion(ExtractionResult::Success, false, true),
            CompletionOutcome::Terminate(Status::Success)
        );
    }

    /// Parity test for capabilities C054/C181: on failure, the recursive
    /// call always returns `false` — never terminates — so `Not
    /// extract(...)` is always `true` whenever this line is reached at
    /// all.
    #[test]
    fn recursive_call_always_returns_false_on_failure() {
        assert_eq!(
            resolve_completion(ExtractionResult::Failed, false, true),
            CompletionOutcome::Return(false)
        );
    }

    #[test]
    fn should_show_extracting_message_requires_arcdisp_above_negative_one() {
        assert!(should_show_extracting_message(0));
        assert!(should_show_extracting_message(1));
        assert!(!should_show_extracting_message(-1));
    }
}
