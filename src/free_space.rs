//! Free-space check: ports the pure decision core of `HasFreeSpace()`
//! (UniExtract.au3:3782-3808) — capability C179, **partial**. Measuring
//! the drive's free space (`DriveSpaceFree`) and the source file's size
//! (`FileGetSize`) is real filesystem I/O, and the source's interactive
//! abort/retry/ignore prompt is a `MsgBox` GUI dialog — deferred under
//! the same GUI-subsystem boundary (manifest row D001) as every other
//! interactive prompt in this port. This module ports only: whether the
//! drive has enough free space, and whether that should terminate the
//! run outright (silent mode) or hand off to the interactive prompt this
//! module doesn't implement.
//!
//! **Scope note:** the source's own preliminary step — walking `$sPath`
//! up to its nearest existing directory ancestor via a `While Not
//! _IsDirectory($sPath)` loop before ever calling `DriveSpaceFree` — is
//! real filesystem I/O entangled with path manipulation and isn't ported
//! here; the caller is expected to resolve a real, existing directory
//! path before calling [`measure_free_space`]. The `MsgBox` call itself
//! (`$iTopmost + $MB_ICONWARNING + $MB_ABORTRETRYIGNORE`) is real GUI,
//! deferred under manifest row D001 — but [`decide_prompt_action`] now
//! covers what happens *once a response is obtained*, including a
//! genuinely easy-to-miss finding: the source's own `Switch` has no
//! `Case` for Ignore (or any unexpected `MsgBox` return value), so
//! choosing Ignore silently falls through with no action at all,
//! letting extraction continue despite the insufficient-space warning.
//! Because producing the response is still unmodeled, this capability's
//! manifest row stays `REQUIRED`, not `DONE`.

/// One free-space measurement, already rounded the way the source
/// rounds it: `Round(DriveSpaceFree($sPath), 2)` for `free_space_mb`,
/// `Round(FileGetSize($file) / 1048576, 2) * $fModifier` for
/// `needed_mb` (the multiplication happens *after* rounding the
/// megabyte conversion, not before), and `Round(Abs(free - needed), 2)`
/// for `difference_mb`.
pub struct FreeSpaceMeasurement {
    pub free_space_mb: f64,
    pub needed_mb: f64,
    pub difference_mb: f64,
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Ports the arithmetic of `HasFreeSpace()`'s body
/// (UniExtract.au3:3791-3796), given an already-measured
/// `drive_free_mb` (`DriveSpaceFree($sPath)`) and `file_size_bytes`
/// (`FileGetSize($file)`) — both real I/O the caller performs first.
pub fn measure_free_space(
    drive_free_mb: f64,
    file_size_bytes: f64,
    modifier: f64,
) -> FreeSpaceMeasurement {
    let free_space_mb = round2(drive_free_mb);
    let needed_mb = round2(file_size_bytes / 1_048_576.0) * modifier;
    let difference_mb = round2((free_space_mb - needed_mb).abs());
    FreeSpaceMeasurement {
        free_space_mb,
        needed_mb,
        difference_mb,
    }
}

/// Ports `If $freeSpace < $fileSize Then` (UniExtract.au3:3794) as its
/// positive form.
pub fn has_enough_free_space(measurement: &FreeSpaceMeasurement) -> bool {
    measurement.free_space_mb >= measurement.needed_mb
}

/// What `HasFreeSpace()` does once it knows whether there's enough
/// space (UniExtract.au3:3783,3798-3806).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeSpaceOutcome {
    /// The check is disabled (`$bOptCheckFreeSpace` false) or there's
    /// enough space either way — extraction proceeds.
    Continue,
    /// Not enough space, and running silently: the source terminates
    /// immediately via `terminate($STATUS_FAILED, $filenamefull,
    /// $STATUS_NOFREESPACE, $sMsg)`. **The exit status passed is
    /// `$STATUS_FAILED`, not `$STATUS_NOFREESPACE`** — `$STATUS_NOFREESPACE`
    /// is stuffed into the *`$arctype`* parameter slot instead, purely
    /// for message display. This is a distinct, separate code path from
    /// the post-extraction `Case $RESULT_NOFREESPACE: terminate($STATUS_NOFREESPACE)`
    /// branch this crate's `result_heuristic` module neighbors (which
    /// *does* use the real `$STATUS_NOFREESPACE` exit status) — do not
    /// conflate the two. A caller acting on this outcome should call
    /// `terminate` with `status::Status::Failed`.
    TerminateFailedSilently,
    /// Not enough space, running interactively: the source shows an
    /// abort/retry/ignore `MsgBox`. Not implemented here — GUI, deferred
    /// under manifest row D001. A caller must supply its own prompt and
    /// interpret the result (retry re-measures, abort removes a
    /// created output directory and terminates silently, ignore falls
    /// through and continues extraction).
    PromptInteractive,
}

/// Ports the branch selection in `HasFreeSpace()`
/// (UniExtract.au3:3783,3794,3798-3806): given whether the check is
/// enabled, whether the measurement found enough space, and whether
/// this run is silent, decide what happens next.
pub fn decide_free_space_outcome(
    check_enabled: bool,
    has_enough: bool,
    silent_mode: bool,
) -> FreeSpaceOutcome {
    if !check_enabled || has_enough {
        FreeSpaceOutcome::Continue
    } else if silent_mode {
        FreeSpaceOutcome::TerminateFailedSilently
    } else {
        FreeSpaceOutcome::PromptInteractive
    }
}

/// One of the `$MB_ABORTRETRYIGNORE` `MsgBox`'s three offered choices —
/// or any other value it could return, which the source's own `Switch`
/// (UniExtract.au3:3800-3806) has no explicit `Case` for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptResponse {
    /// `$IDRETRY`.
    Retry,
    /// `$IDABORT`.
    Abort,
    /// `$IDIGNORE`, or any other value — the source's `Switch` handles
    /// neither, so both fall through identically.
    Other,
}

/// What `HasFreeSpace()` does once it has a prompt response in hand
/// (UniExtract.au3:3800-3806).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAction {
    /// `Return HasFreeSpace($sPath, $fModifier)` — re-run the check
    /// from scratch.
    RetryCheck,
    /// `If $createdir Then DirRemove($outdir, 0)` then
    /// `terminate($STATUS_SILENT)` — remove the output directory only
    /// if this run created it, then terminate silently either way.
    AbortAndTerminateSilently { remove_created_outdir: bool },
    /// Neither `Case` matched: the source's `Switch` falls through with
    /// no action at all, and the function returns without terminating
    /// or retrying — extraction simply continues despite the
    /// insufficient-space warning. This is the "ignore" behavior, not
    /// modeled as its own explicit branch in the source.
    ContinueWithoutAction,
}

/// Ports `HasFreeSpace()`'s response-handling `Switch`
/// (UniExtract.au3:3800-3806). `created_outdir_this_run` is `$createdir`
/// — whether the current run's own output directory didn't exist before
/// and was created for it.
pub fn decide_prompt_action(
    response: PromptResponse,
    created_outdir_this_run: bool,
) -> PromptAction {
    match response {
        PromptResponse::Retry => PromptAction::RetryCheck,
        PromptResponse::Abort => PromptAction::AbortAndTerminateSilently {
            remove_created_outdir: created_outdir_this_run,
        },
        PromptResponse::Other => PromptAction::ContinueWithoutAction,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decide_free_space_outcome, decide_prompt_action, has_enough_free_space, measure_free_space,
        FreeSpaceOutcome, PromptAction, PromptResponse,
    };

    /// Parity test for capability C179: the megabyte conversion rounds
    /// to 2 decimals *before* applying the modifier, matching
    /// `Round(FileGetSize($file) / 1048576, 2) * $fModifier`.
    #[test]
    fn measure_free_space_rounds_before_applying_modifier() {
        // 10 MB exactly, modifier 2 -> needed = 20.0.
        let m = measure_free_space(15.0, 10.0 * 1_048_576.0, 2.0);
        assert_eq!(m.free_space_mb, 15.0);
        assert_eq!(m.needed_mb, 20.0);
        assert_eq!(m.difference_mb, 5.0);
    }

    /// Parity test for capability C179: `free_space_mb >= needed_mb` is
    /// "enough" — the boundary case (exact equality) counts as enough,
    /// matching the source's strict `<` for the "not enough" branch.
    #[test]
    fn has_enough_free_space_boundary_is_inclusive() {
        let exact = measure_free_space(20.0, 10.0 * 1_048_576.0, 2.0);
        assert!(has_enough_free_space(&exact));

        let short = measure_free_space(19.99, 10.0 * 1_048_576.0, 2.0);
        assert!(!has_enough_free_space(&short));
    }

    /// Parity test for capability C179: the check being disabled always
    /// continues, regardless of the measurement.
    #[test]
    fn disabled_check_always_continues() {
        assert_eq!(
            decide_free_space_outcome(false, false, true),
            FreeSpaceOutcome::Continue
        );
        assert_eq!(
            decide_free_space_outcome(false, false, false),
            FreeSpaceOutcome::Continue
        );
    }

    /// Parity test for capability C179: enough space always continues,
    /// regardless of silent mode.
    #[test]
    fn enough_space_always_continues() {
        assert_eq!(
            decide_free_space_outcome(true, true, true),
            FreeSpaceOutcome::Continue
        );
        assert_eq!(
            decide_free_space_outcome(true, true, false),
            FreeSpaceOutcome::Continue
        );
    }

    /// Parity test for capability C179: not enough space in silent mode
    /// terminates immediately.
    #[test]
    fn not_enough_space_silent_mode_terminates() {
        assert_eq!(
            decide_free_space_outcome(true, false, true),
            FreeSpaceOutcome::TerminateFailedSilently
        );
    }

    /// Parity test for capability C179: not enough space interactively
    /// hands off to the (unimplemented) prompt.
    #[test]
    fn not_enough_space_interactive_prompts() {
        assert_eq!(
            decide_free_space_outcome(true, false, false),
            FreeSpaceOutcome::PromptInteractive
        );
    }

    #[test]
    fn retry_response_re_runs_the_check() {
        assert_eq!(
            decide_prompt_action(PromptResponse::Retry, true),
            PromptAction::RetryCheck
        );
        assert_eq!(
            decide_prompt_action(PromptResponse::Retry, false),
            PromptAction::RetryCheck
        );
    }

    /// Parity test for capability C179: abort only removes the output
    /// directory if this run actually created it.
    #[test]
    fn abort_response_removes_outdir_only_if_created_this_run() {
        assert_eq!(
            decide_prompt_action(PromptResponse::Abort, true),
            PromptAction::AbortAndTerminateSilently {
                remove_created_outdir: true
            }
        );
        assert_eq!(
            decide_prompt_action(PromptResponse::Abort, false),
            PromptAction::AbortAndTerminateSilently {
                remove_created_outdir: false
            }
        );
    }

    /// Parity test for capability C179: the source's own `Switch` has
    /// no `Case` for Ignore -- it falls through with no action,
    /// silently continuing extraction despite insufficient space.
    #[test]
    fn ignore_response_takes_no_action_and_continues() {
        assert_eq!(
            decide_prompt_action(PromptResponse::Other, true),
            PromptAction::ContinueWithoutAction
        );
        assert_eq!(
            decide_prompt_action(PromptResponse::Other, false),
            PromptAction::ContinueWithoutAction
        );
    }
}
