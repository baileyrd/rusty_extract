//! Per-extractor timeout handling — capability C174. Verified: there is
//! no global, systematic timeout mechanism anywhere in the source's
//! `extract()` dispatch (~70 `Case`s). `$Timeout` (a user-configurable
//! preference, `UniExtract.au3:151,744-746`, defaulting to 60 seconds)
//! is only ever referenced from roughly 15 scattered call sites across
//! the whole ~8200-line source, most of which just `ExitLoop` a polling
//! loop on expiry without any explicit termination — an implicit,
//! per-case behavior difference, not a shared mechanism.
//!
//! `$TYPE_ARC_CONV`'s case (UniExtract.au3:2398-2399) is the cleanest,
//! most explicit example, and the one this capability's citation names
//! as representative:
//!
//! ```autoit
//! Local $hWnd = WinWait("arc_conv", "", $Timeout)
//! If $hWnd == 0 Then terminate($STATUS_TIMEOUT, $file, $arctype, $arcdisp)
//! ```
//!
//! `WinWait(..., $Timeout)` is real Win32 GUI automation (the deferred
//! GUI subsystem, manifest row D001) — not modeled here. What's
//! portable, and what this module exists to make explicit and testable,
//! is the one-line decision right after it: a `WinWait` call that
//! returns a zero window handle timed out, and this particular case
//! terminates on that outcome — unlike most of the other ~14 `$Timeout`
//! call sites in the source, which don't.

use crate::status::Status;

/// Ports `$TYPE_ARC_CONV`'s timeout check (UniExtract.au3:2399):
/// `WinWait`'s return value of `0` means the wait timed out without the
/// window ever appearing.
pub fn arc_conv_wait_timed_out(window_handle: i64) -> bool {
    window_handle == 0
}

/// What `$TYPE_ARC_CONV` does once it knows whether the wait timed out.
pub fn arc_conv_timeout_outcome(window_handle: i64) -> Option<Status> {
    if arc_conv_wait_timed_out(window_handle) {
        Some(Status::Timeout)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_handle_is_a_timeout() {
        assert!(arc_conv_wait_timed_out(0));
    }

    #[test]
    fn nonzero_handle_is_not_a_timeout() {
        assert!(!arc_conv_wait_timed_out(12345));
    }

    #[test]
    fn timeout_outcome_terminates_only_on_zero_handle() {
        assert_eq!(arc_conv_timeout_outcome(0), Some(Status::Timeout));
        assert_eq!(arc_conv_timeout_outcome(98765), None);
    }
}
