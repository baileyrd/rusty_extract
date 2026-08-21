//! Recursive/composite dispatch — the `extract()` completion contract
//! that makes a recursive `extract($otherType, ...)` call behave as a
//! plain function call (returning a value to its caller) instead of
//! terminating the whole process, feeding capabilities C054/C181.
//!
//! ```autoit
//! Func extract($arctype, $arcdisp = 0, $additionalParameters = "", $returnSuccess = False, $returnFail = False)
//!     ; ... build $tempoutdir, run the Switch $arctype case, which may
//!     ; itself call extract($otherType, ..., $returnSuccess, $returnFail)
//!     ; recursively ...
//!
//!     Switch $success
//!         Case $RESULT_SUCCESS
//!         Case $RESULT_NOFREESPACE
//!             terminate($STATUS_NOFREESPACE)
//!         Case $RESULT_FAILED
//!         Case $RESULT_CANCELED
//!         Case $RESULT_UNKNOWN
//!             ; C171's heuristic reclassifies this to $RESULT_FAILED, or
//!             ; (arctype="ace" and fileext="exe") bails out with a bare
//!             ; `Return False` right here, bypassing everything below —
//!             ; result_heuristic::UnknownResultOutcome::AceExeEarlyAbort.
//!     EndSwitch
//!
//!     If $success = $RESULT_FAILED Then
//!         If Not $returnFail Then terminate($STATUS_FAILED, $file, $arctype, $arcdisp)
//!         $success = $RESULT_UNKNOWN
//!         Return 0
//!     EndIf
//!
//!     If Not $returnSuccess Then terminate($STATUS_SUCCESS, $filenamefull, $arctype, $arcdisp)
//!     $success = $RESULT_UNKNOWN
//!     Return 1
//! EndFunc
//! ```
//!
//! **This is the one function in the whole source, called both as the
//! top-level entry point and recursively from inside its own `Switch
//! $arctype`.** A "recursive" extractor case (C054's six call sites,
//! C181's citations) is nothing more than calling this same function
//! again with `$returnSuccess`/`$returnFail` set so the inner call
//! returns a plain success/failure value instead of exiting — this
//! module ports exactly that completion contract, the piece every
//! recursive call site needs and none of them can supply on its own.
//!
//! **Scope — the completion decision only.** [`resolve_completion`]
//! takes an already-classified [`ExtractionResult`] (`$success` after
//! `result_heuristic::resolve_unknown_result` has resolved any
//! `$RESULT_UNKNOWN` — its `AceExeEarlyAbort` output bypasses this
//! function entirely, matching the source's own bare `Return False`)
//! and the caller's `return_success`/`return_fail` flags, and decides
//! whether to terminate the process or hand a plain boolean back to
//! whichever extractor case made the (possibly recursive) call. It does
//! not build `$tempoutdir`, run the `Switch $arctype` case itself, or
//! actually re-invoke a resolved case — this crate's dispatcher
//! (`extract::dispatch`, C049) resolves *which* module handles a type;
//! wiring a full call → run → evaluate → recurse loop is a larger,
//! not-yet-built orchestration layer (`dispatch`'s own doc comment
//! already flags this), out of scope for this decision function.
//!
//! **Preserved quirk — `$RESULT_CANCELED` takes the success path.** The
//! `Switch $success` block's `Case $RESULT_CANCELED` is empty, so a
//! canceled run never gets reclassified to `$RESULT_FAILED` — the
//! `If $success = $RESULT_FAILED` check that follows is simply false
//! for it, and it falls through to the *same* branch `$RESULT_SUCCESS`
//! takes (`If Not $returnSuccess Then terminate($STATUS_SUCCESS, ...)`).
//! [`ExtractionResult::Canceled`] reproduces this exactly rather than
//! treating cancellation as its own outcome.
//!
//! **Preserved quirk — `$RESULT_NOFREESPACE` always terminates.** Unlike
//! every other result, its `terminate($STATUS_NOFREESPACE)` call sits
//! *inside* the `Switch $success` block itself, before the
//! `$returnSuccess`/`$returnFail` gating is even reached — a recursive
//! call with `$returnFail = True` still can't survive a no-free-space
//! result from the case it just ran.

use crate::status::Status;

/// `$success`'s classified state by the time `extract()` reaches its
/// completion contract — `$RESULT_UNKNOWN` isn't a variant here because
/// `result_heuristic::resolve_unknown_result` has already resolved it
/// into one of these (or its own `AceExeEarlyAbort`, which bypasses this
/// module entirely).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionResult {
    Success,
    NoFreeSpace,
    Failed,
    Canceled,
}

/// What `extract()`'s completion contract decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionOutcome {
    /// Terminate the whole process with this status — either because the
    /// relevant `return_*` flag is `false`, or unconditionally for
    /// [`ExtractionResult::NoFreeSpace`].
    Terminate(Status),
    /// A recursive call returns this value to whichever extractor case
    /// invoked it, instead of terminating: `true` for
    /// [`ExtractionResult::Success`]/[`ExtractionResult::Canceled`],
    /// `false` for [`ExtractionResult::Failed`].
    Return(bool),
}

/// Ports `extract()`'s completion contract (UniExtract.au3:3408-3441,
/// minus the `$RESULT_UNKNOWN` heuristic itself — see the module doc
/// comment).
pub fn resolve_completion(
    result: ExtractionResult,
    return_success: bool,
    return_fail: bool,
) -> CompletionOutcome {
    match result {
        ExtractionResult::NoFreeSpace => CompletionOutcome::Terminate(Status::NoFreeSpace),
        ExtractionResult::Failed => {
            if return_fail {
                CompletionOutcome::Return(false)
            } else {
                CompletionOutcome::Terminate(Status::Failed)
            }
        }
        ExtractionResult::Success | ExtractionResult::Canceled => {
            if return_success {
                CompletionOutcome::Return(true)
            } else {
                CompletionOutcome::Terminate(Status::Success)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capabilities C054/C181: a top-level (non-recursive)
    /// call — `return_success`/`return_fail` both `false`, the source's
    /// own defaults — always terminates rather than returning.
    #[test]
    fn top_level_call_always_terminates() {
        assert_eq!(
            resolve_completion(ExtractionResult::Success, false, false),
            CompletionOutcome::Terminate(Status::Success)
        );
        assert_eq!(
            resolve_completion(ExtractionResult::Failed, false, false),
            CompletionOutcome::Terminate(Status::Failed)
        );
    }

    /// Parity test for capabilities C054/C181: a fully-recursive call
    /// (`return_success = true, return_fail = true`, e.g. `Case
    /// $TYPE_ACTUAL`'s `extract($TYPE_7Z, -1, "", True, True)`) returns a
    /// plain boolean either way, never terminating.
    #[test]
    fn fully_recursive_call_always_returns() {
        assert_eq!(
            resolve_completion(ExtractionResult::Success, true, true),
            CompletionOutcome::Return(true)
        );
        assert_eq!(
            resolve_completion(ExtractionResult::Failed, true, true),
            CompletionOutcome::Return(false)
        );
    }

    /// Parity test for capabilities C054/C181: a partially-recursive call
    /// (`return_success = true, return_fail = false`, e.g. `Case
    /// $TYPE_FORGE`'s `extract($TYPE_7Z, -1, "", True, False)` or `Case
    /// $TYPE_RAI`'s `extract($TYPE_INNO, $arcdisp, "", True)`) survives a
    /// success but still terminates the whole process on failure.
    #[test]
    fn partially_recursive_call_terminates_only_on_failure() {
        assert_eq!(
            resolve_completion(ExtractionResult::Success, true, false),
            CompletionOutcome::Return(true)
        );
        assert_eq!(
            resolve_completion(ExtractionResult::Failed, true, false),
            CompletionOutcome::Terminate(Status::Failed)
        );
    }

    /// Parity test for capabilities C054/C181: `NoFreeSpace` always
    /// terminates, even for a fully-recursive call that would otherwise
    /// survive both success and failure.
    #[test]
    fn no_free_space_always_terminates_even_when_fully_recursive() {
        assert_eq!(
            resolve_completion(ExtractionResult::NoFreeSpace, true, true),
            CompletionOutcome::Terminate(Status::NoFreeSpace)
        );
    }

    /// Parity test for capabilities C054/C181: `Canceled` takes the exact
    /// same path as `Success` — the preserved quirk documented at module
    /// level.
    #[test]
    fn canceled_takes_the_same_path_as_success() {
        assert_eq!(
            resolve_completion(ExtractionResult::Canceled, false, false),
            resolve_completion(ExtractionResult::Success, false, false),
        );
        assert_eq!(
            resolve_completion(ExtractionResult::Canceled, true, true),
            resolve_completion(ExtractionResult::Success, true, true),
        );
    }
}
