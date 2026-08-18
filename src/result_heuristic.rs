//! Generic success/failure fallback heuristic: ports the `$RESULT_UNKNOWN`
//! arm of the success-evaluation `Switch $success` in `extract()`
//! (UniExtract.au3:3415-3430) — capability C171.
//!
//! `$success` is a separate, in-progress-extraction result value
//! (`$RESULT_SUCCESS`/`$RESULT_NOFREESPACE`/`$RESULT_FAILED`/
//! `$RESULT_CANCELED`/`$RESULT_UNKNOWN`, UniExtract.au3 constants near the
//! `$STATUS_*` block this crate's `status::Status` already ports) — not to
//! be confused with that terminal `$STATUS_*` value. Most extractor cases
//! explicitly set `$success` to one of the first four; `$RESULT_UNKNOWN`
//! is what's left in place when a case never bothered, and this is the
//! fallback used to resolve it: infer success or failure by comparing the
//! output directory's size and modification time against a snapshot taken
//! before extraction started.

/// Ports the body of the `Case $RESULT_UNKNOWN` arm
/// (UniExtract.au3:3424-3429). Capturing `initdirsize`
/// (`_DirGetSize($outdir)`, UniExtract.au3:2283) and `dirmtime`
/// (`FileGetTime($outdir, 0, 1)`, UniExtract.au3:3971) before extraction,
/// and `current_dirsize`/`current_dirmtime` after, is real filesystem
/// I/O — the caller's job; this function is the pure decision over
/// already-known values.
///
/// `initdirsize == -1` reproduces the source's own sentinel: `_DirGetSize`
/// (a wrapper, not the AutoIt builtin) returns its `$return` default of
/// `-1` when measuring a drive-root output directory with more than 4 GB
/// already in use would be too expensive to bother with — the source's
/// `$initdirsize > -1` guard exists specifically to skip the
/// size-comparison half of the heuristic in that case, falling through to
/// the mtime comparison alone.
///
/// The two conditions are combined with `Or`: no growth in output
/// directory size (only checked when `initdirsize` is a real
/// measurement) **or** an unchanged modification time. Either one alone
/// is enough to conclude the extraction produced nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownResultOutcome {
    /// Neither the size nor the mtime check found evidence of output;
    /// `$success` is reclassified `$RESULT_FAILED`.
    Failed,
    /// The heuristic found evidence of output (or couldn't rule it out);
    /// `$success` is left at `$RESULT_UNKNOWN`, which the caller's next
    /// check (`If $success = $RESULT_FAILED`) treats the same as success.
    TreatAsSuccess,
    /// `$arctype = "ace" And $fileext = "exe"` special case
    /// (UniExtract.au3:3427): the source `Return False`s out of the
    /// entire `extract()` function right here, bypassing the normal
    /// `terminate()`/success flow this heuristic otherwise feeds into —
    /// not merely "not failed."
    AceExeEarlyAbort,
}

pub fn resolve_unknown_result(
    initdirsize: i64,
    current_dirsize: i64,
    dirmtime_before: i64,
    dirmtime_after: i64,
    arctype: &str,
    fileext: &str,
) -> UnknownResultOutcome {
    let no_growth = initdirsize > -1 && current_dirsize <= initdirsize;
    let mtime_unchanged = dirmtime_after == dirmtime_before;

    if no_growth || mtime_unchanged {
        if arctype == "ace" && fileext == "exe" {
            return UnknownResultOutcome::AceExeEarlyAbort;
        }
        return UnknownResultOutcome::Failed;
    }

    UnknownResultOutcome::TreatAsSuccess
}

#[cfg(test)]
mod tests {
    use super::{resolve_unknown_result, UnknownResultOutcome};

    /// Parity test for capability C171: directory size unchanged (and
    /// mtime changed, so only the size half triggers) resolves to
    /// `Failed`.
    #[test]
    fn no_size_growth_resolves_to_failed() {
        assert_eq!(
            resolve_unknown_result(1000, 1000, 111, 222, "zip", "zip"),
            UnknownResultOutcome::Failed
        );
        assert_eq!(
            resolve_unknown_result(1000, 500, 111, 222, "zip", "zip"),
            UnknownResultOutcome::Failed
        );
    }

    /// Parity test for capability C171: unchanged mtime alone (even with
    /// size growth) resolves to `Failed` — the two conditions are `Or`'d.
    #[test]
    fn unchanged_mtime_alone_resolves_to_failed() {
        assert_eq!(
            resolve_unknown_result(1000, 5000, 111, 111, "zip", "zip"),
            UnknownResultOutcome::Failed
        );
    }

    /// Parity test for capability C171: size growth and mtime change
    /// together mean the heuristic doesn't trigger — left as
    /// `TreatAsSuccess`.
    #[test]
    fn growth_and_mtime_change_treated_as_success() {
        assert_eq!(
            resolve_unknown_result(1000, 5000, 111, 222, "zip", "zip"),
            UnknownResultOutcome::TreatAsSuccess
        );
    }

    /// Parity test for capability C171: `initdirsize == -1` (the
    /// `_DirGetSize` too-expensive-to-measure sentinel) skips the size
    /// check entirely, regardless of `current_dirsize` — only the mtime
    /// comparison can still trigger `Failed`.
    #[test]
    fn negative_one_initdirsize_skips_size_check() {
        assert_eq!(
            resolve_unknown_result(-1, 0, 111, 222, "zip", "zip"),
            UnknownResultOutcome::TreatAsSuccess
        );
        assert_eq!(
            resolve_unknown_result(-1, 0, 111, 111, "zip", "zip"),
            UnknownResultOutcome::Failed
        );
    }

    /// Parity test for capability C171: the `ace`+`exe` special case
    /// bails out with `AceExeEarlyAbort` instead of `Failed`, even though
    /// the same no-growth condition holds.
    #[test]
    fn ace_exe_case_early_aborts_instead_of_failing() {
        assert_eq!(
            resolve_unknown_result(1000, 1000, 111, 222, "ace", "exe"),
            UnknownResultOutcome::AceExeEarlyAbort
        );
    }

    /// Parity test for capability C171: the `ace`+`exe` carve-out only
    /// applies when the heuristic would otherwise have flagged failure —
    /// a genuinely successful ace/exe run still treats as success, not an
    /// early abort.
    #[test]
    fn ace_exe_case_with_evidence_of_output_treats_as_success() {
        assert_eq!(
            resolve_unknown_result(1000, 5000, 111, 222, "ace", "exe"),
            UnknownResultOutcome::TreatAsSuccess
        );
    }
}
