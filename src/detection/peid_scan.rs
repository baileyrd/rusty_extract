//! PEiD scan orchestration (`FileScan_Peid`'s scan half, UniExtract.au3:
//! 1284-1311) — the other half of capability C044; the dispatch table
//! that runs on its result is already `DONE` as
//! [`detection::peid_dispatch::classify`].
//!
//! ```autoit
//! Func FileScan_Peid($sType, $analyze = 1)
//!     Local $sFileType = "", $bHasRegKey = True
//!     Local Const $key = "HKCU\Software\PEiD"
//!
//!     ; Backup existing PEiD options
//!     Local $exsig = RegRead($key, "ExSig")
//!     If @error Then $bHasRegKey = False
//!     Local $loadplugins = RegRead($key, "LoadPlugins")
//!     Local $stayontop = RegRead($key, "StayOnTop")
//!
//!     ; Set PEiD options
//!     RegWrite($key, "ExSig", "REG_DWORD", 1)
//!     RegWrite($key, "LoadPlugins", "REG_DWORD", 0)
//!     RegWrite($key, "StayOnTop", "REG_DWORD", 0)
//!
//!     ; Analyze file
//!     Run($peid & ' -' & $sType & ' "' & $file & '"', $bindir, @SW_HIDE)
//!     WinWait("PEiD v")
//!     Local $TimerStart = TimerInit()
//!     While ($sFileType = "") Or ($sFileType = "Scanning...")
//!         Sleep(100)
//!         $sFileType = ControlGetText("PEiD v", "", "Edit2")
//!         If TimerDiff($TimerStart) > $Timeout Then ExitLoop
//!     WEnd
//!     WinClose("PEiD v")
//!
//!     ; Restore previous PEiD options
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
//! Built on C069's `automation::GuiAutomation` infrastructure, same
//! honesty caveat: fake-backed tests prove [`peid_scan`]'s decision logic
//! against the source line-by-line, not that the real Win32 backend
//! drives an actual PEiD window. Reuses `detector_silence::PEID_KEY`/
//! `PEID_SILENCE_VALUES`/`restore_plan` (C036) for the registry
//! backup/restore rather than re-deriving it.
//!
//! **A genuine, preserved hang-risk quirk**: unlike every other
//! `WinWait` call this port has ported so far (`OpenExeInfo`'s/
//! `RipExeInfo`'s own, which pass `$Timeout` explicitly), `WinWait("PEiD
//! v")` here passes **no timeout argument at all** — AutoIt's documented
//! default is `0`, meaning "wait indefinitely". If PEiD's window never
//! appears, the source hangs forever right there, before the polling
//! loop (which *does* respect `$Timeout`) is ever reached. [`peid_scan`]
//! reproduces this by waiting with `u64::MAX` rather than the caller's
//! `timeout_ms` for that one call — the closest a concrete-`u64` API can
//! get to "no timeout" — not a bug to "fix" into a bounded wait.
//!
//! **`$sFileType = "Scanning..."` uses AutoIt's default `=`
//! case-insensitive comparison**, the same `StringCompareMode` default
//! already documented throughout this crate (`cli`/`dest_arg`/`outdir`/
//! `type_override`/`extract::sevenzip`).

use crate::automation::{GuiAutomation, WindowHandle};
use crate::detector_silence::{self, RestorePlan};
use crate::extract::{Invocation, WindowMode};

const PEID_WINDOW_TITLE: &str = "PEiD v";
const PEID_RESULT_CONTROL: &str = "Edit2";

/// The two placeholder states `Edit2` shows before PEiD has a real
/// result (UniExtract.au3:1305): empty, or the literal `"Scanning..."`
/// text, checked case-insensitively (bare `=`).
fn is_scan_placeholder(text: &str) -> bool {
    text.is_empty() || text.eq_ignore_ascii_case("Scanning...")
}

/// Ports `FileScan_Peid`'s scan (UniExtract.au3:1284-1311): backs up and
/// silences `detector_silence::PEID_KEY`'s 3 values, launches `peid`
/// against `file` with `scan_type` (`-<scan_type>`), waits (with no
/// timeout — see module doc comment) for its window, polls `Edit2` until
/// a real result appears or `timeout_ms` elapses, closes the window, then
/// restores or deletes the registry key per
/// `detector_silence::restore_plan`.
pub fn peid_scan<A: GuiAutomation>(
    automation: &mut A,
    peid: &str,
    scan_type: &str,
    file: &str,
    bindir: &str,
    timeout_ms: u64,
) -> String {
    let backed_up: Vec<(&'static str, Option<i64>)> = detector_silence::PEID_SILENCE_VALUES
        .iter()
        .map(|(name, _)| {
            (
                *name,
                automation.reg_read_dword(detector_silence::PEID_KEY, name),
            )
        })
        .collect();

    for (name, value) in detector_silence::PEID_SILENCE_VALUES {
        automation.reg_write_dword(detector_silence::PEID_KEY, name, *value);
    }

    automation.run(&Invocation {
        program: peid.to_string(),
        args: vec![format!("-{scan_type}"), file.to_string()],
        working_dir: bindir.to_string(),
        window: WindowMode::Hidden,
    });
    let window = automation
        .win_wait(PEID_WINDOW_TITLE, u64::MAX)
        .unwrap_or(WindowHandle(0));

    let timer = automation.timer_init();
    let mut file_type;
    loop {
        automation.sleep(100);
        file_type = automation.control_get_text(window, PEID_RESULT_CONTROL);
        if !is_scan_placeholder(&file_type) || automation.elapsed_ms(timer) > timeout_ms {
            break;
        }
    }
    automation.win_close(window);

    match detector_silence::restore_plan(&backed_up) {
        RestorePlan::Restore(values) => {
            for (name, value) in values {
                automation.reg_write_dword(detector_silence::PEID_KEY, name, value);
            }
        }
        RestorePlan::Delete => automation.reg_delete_key(detector_silence::PEID_KEY),
    }

    file_type
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::fake::{Call, FakeGuiAutomation};

    #[test]
    fn scan_placeholder_matches_empty_and_scanning_case_insensitively() {
        assert!(is_scan_placeholder(""));
        assert!(is_scan_placeholder("Scanning..."));
        assert!(is_scan_placeholder("SCANNING..."));
        assert!(!is_scan_placeholder("Microsoft Visual C++"));
    }

    #[test]
    fn peid_scan_launches_with_scan_type_flag_and_backs_up_registry() {
        let mut fake = FakeGuiAutomation::new();
        for (name, _) in detector_silence::PEID_SILENCE_VALUES {
            fake.set_reg_dword(detector_silence::PEID_KEY, name, 9);
        }
        fake.script_win_wait(PEID_WINDOW_TITLE, Some(WindowHandle(7)));
        fake.script_control_text(WindowHandle(7), PEID_RESULT_CONTROL, "Microsoft Visual C++");

        let result = peid_scan(
            &mut fake,
            r"C:\bin\peid.exe",
            "hard",
            r"C:\downloads\setup.exe",
            r"C:\bin",
            5_000,
        );

        assert_eq!(result, "Microsoft Visual C++");
        assert!(fake.calls().iter().any(|c| matches!(
            c,
            Call::Run(inv) if inv.program == r"C:\bin\peid.exe"
                && inv.args == vec!["-hard".to_string(), r"C:\downloads\setup.exe".to_string()]
                && inv.working_dir == r"C:\bin"
                && inv.window == WindowMode::Hidden
        )));
        // The initial WinWait passes no timeout from the source's own
        // WinWait("PEiD v") call -- modeled as u64::MAX, not timeout_ms.
        assert!(fake
            .calls()
            .contains(&Call::WinWait(PEID_WINDOW_TITLE.to_string(), u64::MAX)));
        let restores = fake
            .calls()
            .into_iter()
            .filter(
                |c| matches!(c, Call::RegWriteDword(key, ..) if key == detector_silence::PEID_KEY),
            )
            .count();
        // 3 writes to silence + 3 writes to restore.
        assert_eq!(restores, detector_silence::PEID_SILENCE_VALUES.len() * 2);
    }

    #[test]
    fn peid_scan_deletes_key_when_it_did_not_exist_before() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(PEID_WINDOW_TITLE, Some(WindowHandle(1)));
        fake.script_control_text(WindowHandle(1), PEID_RESULT_CONTROL, "upx");

        peid_scan(&mut fake, "peid.exe", "x", "file.exe", "C:\\bin", 1_000);

        assert!(fake
            .calls()
            .contains(&Call::RegDeleteKey(detector_silence::PEID_KEY.to_string())));
    }

    #[test]
    fn peid_scan_closes_the_window_after_getting_a_result() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(PEID_WINDOW_TITLE, Some(WindowHandle(3)));
        fake.script_control_text(WindowHandle(3), PEID_RESULT_CONTROL, "aspack");

        peid_scan(&mut fake, "peid.exe", "x", "file.exe", "C:\\bin", 1_000);

        assert!(fake.calls().contains(&Call::WinClose(WindowHandle(3))));
    }

    /// Parity test for capability C044: the polling loop keeps going
    /// while `Edit2` shows a placeholder, and gives up on timeout.
    #[test]
    fn peid_scan_polls_past_the_scanning_placeholder() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(PEID_WINDOW_TITLE, Some(WindowHandle(1)));
        // FakeGuiAutomation always returns the same scripted text for a
        // given (window, control) pair, so this test only exercises the
        // "placeholder never clears, times out" path -- distinct from
        // the "resolves immediately" case already covered above.
        fake.script_control_text(WindowHandle(1), PEID_RESULT_CONTROL, "Scanning...");

        let result = peid_scan(&mut fake, "peid.exe", "x", "file.exe", "C:\\bin", 500);

        assert_eq!(result, "Scanning...");
        let sleeps = fake
            .calls()
            .iter()
            .filter(|c| matches!(c, Call::Sleep(_)))
            .count();
        assert!(
            sleeps >= 5,
            "expected several polling iterations before timeout, got {sleeps}"
        );
    }
}
