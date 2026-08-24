//! Helper-file download and install: ports the decision logic in
//! `_UpdateHelpers` (UniExtract.au3:5480-5553) that downloads and applies
//! updated helper/program files once `CheckUpdate`/`CheckUpdateHelpers`
//! (C207/C206) have determined an update exists.
//!
//! This capability covers only the pure decision/progress-math logic. The
//! real network download (`InetGet`), file deletion/creation, and progress
//! dialog updates are I/O the caller performs, driven by the outcomes these
//! functions return. Per-entry "does this need updating" and "is this a
//! directory" reuse [`crate::update_index::decide_file_needs_update`],
//! [`crate::update_index::is_directory_entry`], and
//! [`crate::update_index::is_self_path`] rather than re-deriving them —
//! `_UpdateHelpers` calls the exact same `_UpdateFileCompare` the source
//! uses, so sharing one Rust function here guarantees the two callers can't
//! drift out of parity with each other the way independently-coded AutoIt
//! logic could.
//!
//! **Verified divergence, preserved rather than "fixed"**: the *recursion*
//! decision for a differing directory entry is coded independently in
//! `CheckUpdateHelpers` and `_UpdateHelpers`, and the two don't actually
//! match. `CheckUpdateHelpers` (C206's `decide_helper_check_step`) treats a
//! *missing* subdirectory as sufficient proof an update exists and returns
//! immediately without recursing into it. `_UpdateHelpers`, ported here,
//! always creates the directory first (if missing) and then fetches its
//! subdirectory index to recurse — it never short-circuits on "missing
//! implies update". This is a real DRY gap in the source: a probe (does an
//! update exist?) and an apply (install it) path that look like they should
//! share one recursion rule, but don't.

use crate::update_index::is_self_path;

/// Ports the overall-progress-bar math (UniExtract.au3:5498-5499):
/// `$ret = (($i + 1) / $iSize) * 100; $iProgress = $ret > $iProgress ?
/// $ret : $iProgress + 0.2`. **Verified quirk, preserved rather than
/// "fixed"**: when the straightforward ratio doesn't exceed the
/// already-displayed progress (e.g. a directory entry expands the total
/// file count, making the ratio's denominator grow faster than its
/// numerator), the bar is nudged forward by a fixed 0.2 anyway — a purely
/// cosmetic increment with no relationship to real progress, just to keep
/// the bar visibly moving.
pub fn advance_overall_progress(current_progress: f64, index: usize, total: usize) -> f64 {
    let ratio = ((index + 1) as f64 / total as f64) * 100.0;
    if ratio > current_progress {
        ratio
    } else {
        current_progress + 0.2
    }
}

/// Ports the per-file download progress percentage (UniExtract.au3:5539):
/// `Int($iBytesReceived / $a[1] * 100)`. AutoIt's `Int()` truncates toward
/// zero, matched here by an `as i64` cast on the same floating-point
/// computation rather than rounding.
pub fn download_progress_percent(bytes_received: i64, total_bytes: i64) -> i64 {
    (bytes_received as f64 / total_bytes as f64 * 100.0) as i64
}

/// Ports `If Not FileExists($sPath) Then DirCreate($sPath)`
/// (UniExtract.au3:5511).
pub fn should_create_directory(exists: bool) -> bool {
    !exists
}

/// The outcome of fetching a differing directory entry's own index
/// (UniExtract.au3:5513-5519), once [`should_create_directory`] has been
/// acted on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryExpansionOutcome {
    /// The subdirectory's index was fetched; append its rows to the file
    /// list being walked and keep going.
    Expanded,
    /// The fetch failed (`Not IsArray($aReturn)`): the overall run is
    /// marked unsuccessful, but the loop moves on to the next entry rather
    /// than aborting.
    FetchFailed,
}

pub fn resolve_directory_expansion_outcome(
    index_fetch_succeeded: bool,
) -> DirectoryExpansionOutcome {
    if index_fetch_succeeded {
        DirectoryExpansionOutcome::Expanded
    } else {
        DirectoryExpansionOutcome::FetchFailed
    }
}

/// Ports the `$success` accumulation across the whole loop: it starts
/// `True` and a single failing entry (a directory whose index fetch fails,
/// UniExtract.au3:5515, or a file whose download errors,
/// UniExtract.au3:5535) latches it to `False` for the rest of the run — a
/// per-file failure never aborts the loop (via a double-level
/// `ContinueLoop 2` for downloads) and never gets "un-latched" by a later
/// success. **Verified quirk, preserved rather than "fixed"**: regardless
/// of the final `$success` value, `SendStats("UpdateHelpers", 1)` is sent
/// unconditionally once the loop ends (UniExtract.au3:5547) — a "success"
/// telemetry ping even when files failed to update.
pub fn accumulate_success(current: bool, this_step_succeeded: bool) -> bool {
    current && this_step_succeeded
}

/// Ports the two guards `_UpdateHelpers` checks before branching on
/// file-vs-directory (UniExtract.au3:5505,5507): skip the running
/// executable itself (`local_path` is the caller's already-joined
/// `@ScriptDir & "\" & $a[0]`), then skip any entry that doesn't need
/// updating (per [`crate::update_index::decide_file_needs_update`]).
pub fn should_process_entry(local_path: &str, script_full_path: &str, needs_update: bool) -> bool {
    !is_self_path(local_path, script_full_path) && needs_update
}

#[cfg(test)]
mod tests {
    use super::{
        accumulate_success, advance_overall_progress, download_progress_percent,
        resolve_directory_expansion_outcome, should_create_directory, should_process_entry,
        DirectoryExpansionOutcome,
    };

    #[test]
    fn overall_progress_uses_ratio_when_it_advances() {
        // 1 of 4 done -> 25%, starting from 0.
        assert!((advance_overall_progress(0.0, 0, 4) - 25.0).abs() < f64::EPSILON);
    }

    /// The verified quirk: when the ratio doesn't exceed the current
    /// displayed progress, nudge forward by a fixed 0.2 instead.
    #[test]
    fn overall_progress_nudges_forward_when_ratio_does_not_advance() {
        // Already at 50%, and a newly-expanded total makes the ratio for
        // this same index drop to 10% -- the bar still creeps forward.
        let advanced = advance_overall_progress(50.0, 0, 10);
        assert!((advanced - 50.2).abs() < f64::EPSILON);
    }

    #[test]
    fn download_progress_truncates_toward_zero() {
        assert_eq!(download_progress_percent(50, 200), 25);
        // 33.33...% truncates to 33, not rounds to 33.
        assert_eq!(download_progress_percent(1, 3), 33);
        assert_eq!(download_progress_percent(999, 1000), 99);
    }

    #[test]
    fn directory_created_only_when_missing() {
        assert!(should_create_directory(false));
        assert!(!should_create_directory(true));
    }

    #[test]
    fn directory_expansion_outcome_matches_fetch_result() {
        assert_eq!(
            resolve_directory_expansion_outcome(true),
            DirectoryExpansionOutcome::Expanded
        );
        assert_eq!(
            resolve_directory_expansion_outcome(false),
            DirectoryExpansionOutcome::FetchFailed
        );
    }

    /// The verified quirk: once latched to failure, a later successful
    /// entry never restores success.
    #[test]
    fn success_latches_to_false_and_never_recovers() {
        let mut success = true;
        success = accumulate_success(success, true);
        assert!(success);
        success = accumulate_success(success, false);
        assert!(!success);
        success = accumulate_success(success, true);
        assert!(!success);
    }

    #[test]
    fn entry_processed_only_when_not_self_and_needs_update() {
        assert!(should_process_entry("file.txt", "app.exe", true));
        assert!(!should_process_entry("file.txt", "app.exe", false));
        assert!(!should_process_entry("app.exe", "app.exe", true));
    }
}
