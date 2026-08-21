//! Pre-extraction entry gates: three early checks that short-circuit
//! straight to a batch queue, a scan-only report, or a specific
//! termination status before any real extraction work starts.
//!
//! ```autoit
//! ; Prevent multiple instances to avoid errors
//! $hMutex = _Singleton($name & " " & $sVersion, 1)
//! If $hMutex = 0 And $extract Then
//!     AddToBatch()
//!     terminate($STATUS_SILENT)
//! EndIf
//!
//! StartExtraction()
//!
//! Func StartExtraction()
//!     If _IsDirectory($file) Then
//!         GUI_Batch_AddDirectory($file)
//!         terminate($STATUS_BATCH)
//!     EndIf
//!
//!     ; Display file information and terminate if scan only mode
//!     If Not $extract Then
//!         FileScan_MediaInfo()
//!         terminate($STATUS_FILEINFO, $filenamefull, $fileext)
//!     EndIf
//!
//!     FilenameParse($file)
//!     ValidateOutputDirectory()
//!     ; ...
//! EndFunc
//! ```
//!
//! All three gates decide *whether* to short-circuit and, where the
//! result is a single fixed status, *which* exit status that produces
//! (C016's contract, already `DONE`) — the real work each branch
//! performs (`_Singleton`'s OS mutex, `GUI_Batch_AddDirectory`'s
//! per-file enumeration, `AddToBatch`'s queue-file write,
//! `FileScan_MediaInfo`'s scan, C045) is out of scope here, matching the
//! codebase's usual caller-supplied-boolean seam (e.g.
//! `plugin::resolve_plugin_ini_with`). `GUI_Batch_AddDirectory`'s `GUI_`
//! prefix marks it as deferred-GUI-subsystem work (manifest row D001),
//! the same convention `type_override::TypeOverride::PromptForType`
//! already follows for `GUI_MethodSelectList`; the underlying queuing
//! mechanism it would use is `batch::build_command_line` (C147, already
//! `DONE`).

use crate::status::Status;

/// C015: whether a second invocation — one that finds the single-instance
/// mutex already held, and would otherwise extract — queues itself into
/// the batch instead of running concurrently, ported from
/// `If $hMutex = 0 And $extract Then AddToBatch(); terminate($STATUS_SILENT); EndIf`.
/// `mutex_acquired` is caller-supplied (a real `_Singleton`-equivalent OS
/// mutex acquisition); `will_extract` is `$extract` (C003's scan-only
/// mode leaves this `false`). Returns the resulting status when this
/// gate fires, matching `$STATUS_SILENT` — exit code 0, per
/// `status::exit_code`.
pub fn second_instance_gate(mutex_acquired: bool, will_extract: bool) -> Option<Status> {
    if !mutex_acquired && will_extract {
        Some(Status::Silent)
    } else {
        None
    }
}

/// C014: whether a directory input routes to "queue every file within as
/// a batch" instead of being treated as a single input file — the
/// directory itself is never extracted or reported as an error — ported
/// from `StartExtraction()`'s `If _IsDirectory($file) Then
/// GUI_Batch_AddDirectory($file); terminate($STATUS_BATCH); EndIf`.
/// `is_directory` is caller-supplied (a real filesystem check). Returns
/// the resulting status when this gate fires, matching `$STATUS_BATCH`
/// — exit code 0, per `status::exit_code`.
pub fn directory_input_gate(is_directory: bool) -> Option<Status> {
    if is_directory {
        Some(Status::Batch)
    } else {
        None
    }
}

/// C152: whether scan-only mode short-circuits straight to a file-info
/// report instead of reaching the extraction dispatcher, ported from
/// `StartExtraction()`'s `If Not $extract Then FileScan_MediaInfo();
/// terminate($STATUS_FILEINFO, $filenamefull, $fileext); EndIf`.
/// `will_extract` is `$extract` (C003's scan-only mode leaves this
/// `false`). Returns whether the gate fires, not the `Status::FileInfo`
/// value itself — that variant also carries `silent_mode` and
/// `filetype_identified` (C153/C154, already `DONE`), which this gate
/// doesn't have; the caller builds the actual status once it knows
/// those. `FileScan_MediaInfo`'s media-info scan is C045 (REQUIRED,
/// separate), out of scope here.
pub fn scan_only_gate(will_extract: bool) -> bool {
    !will_extract
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C015: mutex already held and the run
    /// would extract → queues as a second instance (`Status::Silent`,
    /// exit code 0).
    #[test]
    fn second_instance_queues_when_mutex_held_and_would_extract() {
        assert_eq!(second_instance_gate(false, true), Some(Status::Silent));
    }

    /// Parity test for capability C015: the mutex was acquired (this is
    /// the only instance) — no gate, regardless of extract mode.
    #[test]
    fn second_instance_gate_does_not_fire_when_mutex_acquired() {
        assert_eq!(second_instance_gate(true, true), None);
        assert_eq!(second_instance_gate(true, false), None);
    }

    /// Parity test for capability C015: `$extract` is false (C003's
    /// scan-only mode) — the source's `And $extract` condition means a
    /// second scan-only instance is *not* queued, matching
    /// `$hMutex = 0 And $extract` requiring both.
    #[test]
    fn second_instance_gate_does_not_fire_in_scan_only_mode() {
        assert_eq!(second_instance_gate(false, false), None);
    }

    /// Parity test for capability C014: a directory input queues as a
    /// batch (`Status::Batch`, exit code 0).
    #[test]
    fn directory_input_queues_as_batch() {
        assert_eq!(directory_input_gate(true), Some(Status::Batch));
    }

    /// Parity test for capability C014: a plain file input doesn't
    /// trigger this gate.
    #[test]
    fn directory_input_gate_does_not_fire_for_a_file() {
        assert_eq!(directory_input_gate(false), None);
    }

    /// Parity test for capability C152: scan-only mode (`$extract =
    /// False`) fires this gate.
    #[test]
    fn scan_only_gate_fires_when_not_extracting() {
        assert!(scan_only_gate(false));
    }

    /// Parity test for capability C152: a normal extraction run doesn't
    /// trigger this gate.
    #[test]
    fn scan_only_gate_does_not_fire_when_extracting() {
        assert!(!scan_only_gate(true));
    }
}
