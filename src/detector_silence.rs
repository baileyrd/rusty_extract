//! Third-party detector tool silencing: PEiD and Exeinfo PE both pop up
//! their own GUI/prompts by default, which would hang an automated scan —
//! `FileScan_Peid`/`OpenExeInfo` force both tools into non-interactive
//! mode via their own registry keys before running them, backing up
//! whatever was there first so `CloseExeInfo`/`FileScan_Peid`'s own tail
//! can restore it afterward. Both follow the identical shape:
//!
//! ```autoit
//! Func FileScan_Peid($sType, $analyze = 1)
//!     Local $bHasRegKey = True
//!     Local Const $key = "HKCU\Software\PEiD"
//!     Local $exsig = RegRead($key, "ExSig")
//!     If @error Then $bHasRegKey = False
//!     Local $loadplugins = RegRead($key, "LoadPlugins")
//!     Local $stayontop = RegRead($key, "StayOnTop")
//!     RegWrite($key, "ExSig", "REG_DWORD", 1)
//!     RegWrite($key, "LoadPlugins", "REG_DWORD", 0)
//!     RegWrite($key, "StayOnTop", "REG_DWORD", 0)
//!     ; ... extraction/detection runs here ...
//!     If $bHasRegKey Then
//!         RegWrite($key, "ExSig", "REG_DWORD", $exsig)
//!         RegWrite($key, "LoadPlugins", "REG_DWORD", $loadplugins)
//!         RegWrite($key, "StayOnTop", "REG_DWORD", $stayontop)
//!     Else
//!         RegDelete($key)
//!     EndIf
//! EndFunc
//! ```
//!
//! `OpenExeInfo`/`CloseExeInfo` (`HKCU\Software\ExEi-pe`) follow the exact
//! same backup/overwrite/restore-or-delete pattern over 9 values instead
//! of 3.
//!
//! **Scope — decision policy only, no registry I/O.** This module doesn't
//! call `RegRead`/`RegWrite`/`RegDelete` itself (that needs real Win32
//! registry access this crate doesn't have FFI for yet); it only decides,
//! from a caller-supplied backup read, what the restore plan is —
//! [`restore_plan`] takes the backup as plain data, the same
//! dependency-injection split already used by
//! `extract::plugin::resolve_plugin_ini_with`.
//!
//! **Preserved quirk — only the first value's read failure is checked.**
//! Both source functions check `@error` after reading only the *first*
//! value (`ExSig`/`ExeError`) to decide `$bHasRegKey`; every other
//! value's `RegRead` result is used whether or not it errored (AutoIt's
//! `RegRead` returns `""` on failure, which a subsequent
//! `RegWrite(..., "REG_DWORD", "")` would coerce to `0`). This port
//! reproduces that exactly: [`restore_plan`]'s `existing` slice pairs
//! each value with an `Option<i64>` (`None` = that individual read
//! failed), but only `existing[0]`'s `None`-ness decides
//! [`RestorePlan::Delete`] vs. [`RestorePlan::Restore`] — any other
//! `None` in the slice restores as `0`, not as "still missing."

/// `HKCU\Software\PEiD`'s three silenced values, in the order the source
/// backs them up/writes them/restores them.
pub const PEID_KEY: &str = r"HKCU\Software\PEiD";
pub const PEID_SILENCE_VALUES: &[(&str, i64)] =
    &[("ExSig", 1), ("LoadPlugins", 0), ("StayOnTop", 0)];

/// `HKCU\Software\ExEi-pe`'s nine silenced values, in the order the
/// source backs them up/writes them/restores them.
pub const EXEINFO_KEY: &str = r"HKCU\Software\ExEi-pe";
pub const EXEINFO_SILENCE_VALUES: &[(&str, i64)] = &[
    ("ExeError", 1),
    ("Scan", 1),
    ("AllwaysOnTop", 0),
    ("Skin", 0xFFFF_FFFFu32 as i64),
    ("Shell_integr", 0),
    ("Log", 0xFFFF_FFFFu32 as i64),
    ("Big_GUI", 0),
    ("Lang", 0xFFFF_FFFFu32 as i64),
    ("closeExEi_whenExtRun", 0),
];

/// What to do once the detector tool has finished running, ported from
/// `FileScan_Peid`'s/`CloseExeInfo`'s tail (see module doc comment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestorePlan {
    /// The key existed before (the *first* backed-up value's read
    /// succeeded) — write these name/value pairs back, in order.
    Restore(Vec<(&'static str, i64)>),
    /// The key didn't exist before (the first read failed) — delete it
    /// entirely rather than restoring individual values.
    Delete,
}

/// Computes the restore plan from `existing`, the caller-supplied backup
/// read: one `(name, value)` pair per silenced value, in the same order
/// as [`PEID_SILENCE_VALUES`]/[`EXEINFO_SILENCE_VALUES`], where `value`
/// is `None` when that individual `RegRead` failed. Only `existing[0]`'s
/// `None`-ness selects [`RestorePlan::Delete`] — see the module doc
/// comment's "preserved quirk" note for why every other `None` restores
/// as `0` instead.
pub fn restore_plan(existing: &[(&'static str, Option<i64>)]) -> RestorePlan {
    match existing.first() {
        None | Some((_, None)) => RestorePlan::Delete,
        Some(_) => RestorePlan::Restore(
            existing
                .iter()
                .map(|(name, value)| (*name, value.unwrap_or(0)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C036: the PEiD/Exeinfo silence-value
    /// sets match the source's `RegWrite` calls exactly, in order.
    #[test]
    fn silence_value_sets_match_source() {
        assert_eq!(
            PEID_SILENCE_VALUES,
            &[("ExSig", 1), ("LoadPlugins", 0), ("StayOnTop", 0)]
        );
        assert_eq!(
            EXEINFO_SILENCE_VALUES,
            &[
                ("ExeError", 1),
                ("Scan", 1),
                ("AllwaysOnTop", 0),
                ("Skin", 0xFFFF_FFFFi64),
                ("Shell_integr", 0),
                ("Log", 0xFFFF_FFFFi64),
                ("Big_GUI", 0),
                ("Lang", 0xFFFF_FFFFi64),
                ("closeExEi_whenExtRun", 0),
            ]
        );
    }

    /// Parity test for capability C036: when the first backed-up value's
    /// read succeeded, the key existed — restores every value, coercing
    /// any individually-failed read to `0` (matching AutoIt's
    /// `RegRead` failure returning `""`, coerced to `0` by a subsequent
    /// `REG_DWORD` write).
    #[test]
    fn restore_plan_restores_when_key_existed() {
        let existing = [
            ("ExSig", Some(1)),
            ("LoadPlugins", None),
            ("StayOnTop", Some(0)),
        ];
        assert_eq!(
            restore_plan(&existing),
            RestorePlan::Restore(vec![("ExSig", 1), ("LoadPlugins", 0), ("StayOnTop", 0)])
        );
    }

    /// Parity test for capability C036: when the *first* backed-up
    /// value's read failed, the key didn't exist — deletes it rather
    /// than restoring, regardless of whether later reads in the slice
    /// happened to succeed.
    #[test]
    fn restore_plan_deletes_when_key_did_not_exist() {
        let existing = [
            ("ExSig", None),
            ("LoadPlugins", Some(0)),
            ("StayOnTop", Some(0)),
        ];
        assert_eq!(restore_plan(&existing), RestorePlan::Delete);
    }

    /// Parity test for capability C036: an empty backup slice (the
    /// degenerate case, not reachable from either real call site but
    /// worth pinning down) deletes rather than panicking.
    #[test]
    fn restore_plan_deletes_for_empty_backup() {
        assert_eq!(restore_plan(&[]), RestorePlan::Delete);
    }
}
