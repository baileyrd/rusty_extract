//! Win32 GUI-automation infrastructure: the seam between a ported
//! orchestration function (e.g. [`open_exeinfo`]/[`close_exeinfo`]/
//! [`rip_exeinfo`], C069) and actually driving a real third-party window —
//! `Run`/`WinWait`/`ControlClick`/`ControlSend`/`ControlGetText`/registry
//! access/etc. Mirrors the split `extract::runner::ExtractorRunner`
//! already established for plain process spawning: a trait
//! ([`GuiAutomation`]), a real Win32-backed implementation
//! ([`win32::Win32GuiAutomation`], Windows-only), and a test double
//! ([`fake::FakeGuiAutomation`]) that records every call so orchestration
//! logic can be tested without a real window to drive.
//!
//! **What this does and doesn't prove.** [`fake::FakeGuiAutomation`] lets
//! [`open_exeinfo`]/[`close_exeinfo`]/[`rip_exeinfo`]'s own *decision
//! logic* — which registry values get backed up and in what order, which
//! command line launches Exeinfo PE, which control gets clicked, which key
//! sequence gets sent, how the listbox-polling loop reacts to a scripted
//! sequence of search results — be verified against the source line by
//! line, the same confidence level every other parity test in this crate
//! has. It does **not** prove [`win32::Win32GuiAutomation`] actually drives
//! the real Exeinfo PE window correctly: that would need a live interactive
//! Windows desktop with the real, licensed `exeinfope.exe` running, which
//! doesn't exist in this environment or on the CI runner (headless
//! `windows-latest`, see `RELEASE_NOTES.md`). The Win32 backend is written
//! to the Win32 API's and AutoIt's own documented semantics as carefully as
//! this port can manage, but it carries a strictly weaker guarantee than
//! every `Invocation`-based module in this crate — flagged here rather than
//! glossed over.
//!
//! **Scope of the trait itself.** Covers exactly the primitives the cited
//! call sites need (`OpenExeInfo`/`RipExeInfo`/`CloseExeInfo`,
//! UniExtract.au3:1822-1917): registry read/write/delete, process spawn,
//! window wait/show/close/exists, control click/send/get-text/get-handle,
//! one listbox query, mouse move, sleep/timer, and process-close. Not a
//! general-purpose AutoIt-window-automation engine — extending it for a
//! different call site's needs (a different control-spec shape, a
//! different registry value type) is expected as those call sites get
//! ported, not pre-built speculatively.

pub mod control_spec;
pub mod fake;
pub mod keys;
#[cfg(windows)]
pub mod win32;

use crate::detector_silence::{self, RestorePlan};
use crate::extract::{Invocation, WindowMode};

/// Opaque handle to a located top-level window (`WinWait`'s non-zero
/// return value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub isize);

/// Opaque handle to a control within a window (`ControlGetHandle`'s return
/// value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlHandle(pub isize);

/// Opaque handle to a running timer (`TimerInit`'s return value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerHandle(pub u64);

/// The Win32 primitives `OpenExeInfo`/`RipExeInfo`/`CloseExeInfo` (and,
/// eventually, `FileScan_Peid`'s scan and the other deferred-GUI call
/// sites) drive. See the module doc comment for what a test against
/// [`fake::FakeGuiAutomation`] does and doesn't prove.
pub trait GuiAutomation {
    /// `RegRead($key, $value_name)`, coerced to a number the way this
    /// crate's callers always use it (every silenced value here is a
    /// `REG_DWORD`) — `None` on `@error` (the key/value didn't exist).
    fn reg_read_dword(&mut self, key: &str, value_name: &str) -> Option<i64>;
    /// `RegWrite($key, $value_name, "REG_DWORD", $value)`.
    fn reg_write_dword(&mut self, key: &str, value_name: &str, value: i64);
    /// `RegDelete($key)`.
    fn reg_delete_key(&mut self, key: &str);

    /// `Run(...)` — spawns `invocation` without waiting for it to exit
    /// (unlike `extract::runner::ExtractorRunner::run`, which blocks for
    /// the extractor's final output): GUI automation needs to interact
    /// with the launched process's window while it's still running.
    fn run(&mut self, invocation: &Invocation);

    /// `WinWait($title_or_spec, "", $timeout_ms)` — `None` on timeout
    /// (the source's `0` return).
    fn win_wait(&mut self, title_or_spec: &str, timeout_ms: u64) -> Option<WindowHandle>;
    /// `WinSetState($window, "", $mode)`.
    fn win_set_state(&mut self, window: WindowHandle, mode: WindowMode);
    /// `WinClose($window)`.
    fn win_close(&mut self, window: WindowHandle);
    /// `WinExists($title_or_spec)`.
    fn win_exists(&mut self, title_or_spec: &str) -> bool;

    /// `ControlClick($window, "", $control_spec)`.
    fn control_click(&mut self, window: WindowHandle, control_spec: &str);
    /// `ControlSend($window, "", $control_spec, $keys)`.
    fn control_send(&mut self, window: WindowHandle, control_spec: &str, keys: &str);
    /// `ControlGetText($window, "", $control_spec)`.
    fn control_get_text(&mut self, window: WindowHandle, control_spec: &str) -> String;
    /// `ControlGetHandle($window, "", $control_spec)` — `None` if no
    /// matching control was found.
    fn control_get_handle(
        &mut self,
        window: WindowHandle,
        control_spec: &str,
    ) -> Option<ControlHandle>;

    /// `_GUICtrlListBox_FindString($control, $needle, $exact)` — the
    /// source's own `-1` "not found" sentinel, not an `Option`, since
    /// callers (`RipExeInfo`) compare it against `-1`/`0` directly.
    fn listbox_find_string(&mut self, control: ControlHandle, needle: &str, exact: bool) -> i32;

    /// `MouseMove($x, $y, 0)`.
    fn mouse_move(&mut self, x: i32, y: i32);
    /// `Sleep($ms)`.
    fn sleep(&mut self, ms: u64);
    /// `TimerInit()`.
    fn timer_init(&mut self) -> TimerHandle;
    /// `TimerDiff($handle)`.
    fn elapsed_ms(&mut self, since: TimerHandle) -> u64;
    /// `ProcessClose($name)`.
    fn process_close(&mut self, name: &str);
}

/// The `$aReturn` array `OpenExeInfo` builds (UniExtract.au3:1823-1834):
/// `$aReturn[0]`'s window-title constant and `$aReturn[1]`'s key-name
/// constant aren't carried here (they're always
/// `EXEINFO_WINDOW_TITLE`/`detector_silence::EXEINFO_KEY`); `backed_up`
/// holds the same `$aReturn[3..11]` values `CloseExeInfo` restores from —
/// captured once, here, not re-read later, matching the source passing
/// the whole `$aReturn` array through rather than re-querying the
/// registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExeInfoHandle {
    pub window: WindowHandle,
    pub key_existed: bool,
    pub backed_up: Vec<(&'static str, Option<i64>)>,
}

const EXEINFO_WINDOW_TITLE: &str = "Exeinfo PE";

/// Ports `OpenExeInfo` (UniExtract.au3:1822-1850): backs up
/// `detector_silence::EXEINFO_KEY`'s 9 values, forces them all to their
/// silenced values, launches `exeinfope` against `file`, waits for its
/// window, then hides it.
///
/// `bindir` is `Run`'s working directory (UniExtract.au3:1845's `$bindir`
/// argument); `timeout_ms` is `$Timeout`.
pub fn open_exeinfo<A: GuiAutomation>(
    automation: &mut A,
    exeinfope: &str,
    file: &str,
    bindir: &str,
    timeout_ms: u64,
) -> ExeInfoHandle {
    let backed_up: Vec<(&'static str, Option<i64>)> = detector_silence::EXEINFO_SILENCE_VALUES
        .iter()
        .map(|(name, _)| {
            (
                *name,
                automation.reg_read_dword(detector_silence::EXEINFO_KEY, name),
            )
        })
        .collect();
    let key_existed = matches!(backed_up.first(), Some((_, Some(_))));

    for (name, value) in detector_silence::EXEINFO_SILENCE_VALUES {
        automation.reg_write_dword(detector_silence::EXEINFO_KEY, name, *value);
    }

    automation.run(&Invocation {
        program: exeinfope.to_string(),
        args: vec![file.to_string()],
        working_dir: bindir.to_string(),
        window: WindowMode::Minimized,
    });
    let window = automation
        .win_wait(EXEINFO_WINDOW_TITLE, timeout_ms)
        .unwrap_or(WindowHandle(0));
    automation.win_set_state(window, WindowMode::Hidden);

    ExeInfoHandle {
        window,
        key_existed,
        backed_up,
    }
}

/// Ports `CloseExeInfo` (UniExtract.au3:1897-1917): closes the window,
/// then restores or deletes `detector_silence::EXEINFO_KEY` per
/// `detector_silence::restore_plan`'s decision over `handle.backed_up` —
/// reusing that existing C036 logic rather than re-deriving it.
pub fn close_exeinfo<A: GuiAutomation>(automation: &mut A, handle: &ExeInfoHandle) {
    automation.win_close(handle.window);
    match detector_silence::restore_plan(&handle.backed_up) {
        RestorePlan::Restore(values) => {
            for (name, value) in values {
                automation.reg_write_dword(detector_silence::EXEINFO_KEY, name, value);
            }
        }
        RestorePlan::Delete => automation.reg_delete_key(detector_silence::EXEINFO_KEY),
    }
}

const RIP_BUTTON_CONTROL: &str = "[CLASS:TBitBtn; INSTANCE:16]";
const RIP_RESULT_WINDOW: &str = "[CLASS:TSViewer]";
const RIP_RESULT_LISTBOX: &str = "TListBox1";
const RIP_END_OF_FILE_MARKERS: [&str; 2] = ["--- End of file ---", "-- End of file --"];
const RIP_NOT_FOUND_MARKER: &str = "--- Not found , sorry ---";

/// Ports `RipExeInfo` (UniExtract.au3:1861-1896): opens Exeinfo PE against
/// the (already caller-relocated) file at `tempoutdir_file`, drives its
/// MSI-rip command via `command` (a key sequence like
/// `mscf::RIP_EXEINFO_KEY_SEQUENCE`/`wise::RIP_EXEINFO_KEY_SEQUENCE`),
/// polls the results listbox until it reports completion or `timeout_ms`
/// elapses, then closes Exeinfo PE and reports whether a match was found.
///
/// **The polling loop preserves two source quirks exactly**: both
/// "End of file" spellings are checked (the second only when the first
/// doesn't match, UniExtract.au3:1878-1879), and the timeout check happens
/// *after* both `_GUICtrlListBox_FindString` calls each iteration
/// (UniExtract.au3:1880) — a timeout on the very last poll still gets to
/// see that poll's result before exiting the loop, matching the source's
/// `Until`-at-the-bottom `While` shape reproduced here as a `loop`.
///
/// The `_FileMove` relocation before/after (moving `file` to
/// `tempoutdir_file` and back to `filedir`) is real filesystem I/O, left
/// to the caller — this function only drives the Exeinfo PE window.
pub fn rip_exeinfo<A: GuiAutomation>(
    automation: &mut A,
    exeinfope: &str,
    tempoutdir_file: &str,
    bindir: &str,
    command: &str,
    timeout_ms: u64,
) -> bool {
    let handle = open_exeinfo(automation, exeinfope, tempoutdir_file, bindir, timeout_ms);

    let window = automation
        .win_wait(EXEINFO_WINDOW_TITLE, timeout_ms)
        .unwrap_or(handle.window);
    automation.mouse_move(0, 0);
    automation.control_click(window, RIP_BUTTON_CONTROL);
    automation.control_send(window, RIP_BUTTON_CONTROL, &format!("{command}{{ENTER}}"));

    let result_window = automation
        .win_wait(RIP_RESULT_WINDOW, timeout_ms)
        .unwrap_or(WindowHandle(0));
    let listbox = automation
        .control_get_handle(result_window, RIP_RESULT_LISTBOX)
        .unwrap_or(ControlHandle(0));

    let timer = automation.timer_init();
    loop {
        automation.sleep(200);
        let mut found = automation.listbox_find_string(listbox, RIP_END_OF_FILE_MARKERS[0], true);
        if found < 0 {
            found = automation.listbox_find_string(listbox, RIP_END_OF_FILE_MARKERS[1], true);
        }
        if found >= 0 || automation.elapsed_ms(timer) > timeout_ms {
            break;
        }
    }

    let success = automation.listbox_find_string(listbox, RIP_NOT_FOUND_MARKER, true) == -1;

    close_exeinfo(automation, &handle);

    success
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Call, FakeGuiAutomation};

    #[test]
    fn open_exeinfo_backs_up_writes_and_launches_when_key_existed() {
        let mut fake = FakeGuiAutomation::new();
        for (name, _) in detector_silence::EXEINFO_SILENCE_VALUES {
            fake.set_reg_dword(detector_silence::EXEINFO_KEY, name, 7);
        }
        fake.script_win_wait(EXEINFO_WINDOW_TITLE, Some(WindowHandle(42)));

        let handle = open_exeinfo(
            &mut fake,
            r"C:\bin\exeinfope.exe",
            r"C:\downloads\file.exe",
            r"C:\bin",
            5_000,
        );

        assert!(handle.key_existed);
        assert_eq!(handle.window, WindowHandle(42));
        assert!(fake.calls().iter().any(|c| matches!(
            c,
            Call::Run(inv) if inv.program == r"C:\bin\exeinfope.exe"
                && inv.args == vec![r"C:\downloads\file.exe".to_string()]
                && inv.working_dir == r"C:\bin"
                && inv.window == WindowMode::Minimized
        )));
        assert!(fake
            .calls()
            .contains(&Call::WinSetState(WindowHandle(42), WindowMode::Hidden)));
        // Every silenced value got forced to its override, in order.
        let writes: Vec<_> = fake
            .calls()
            .into_iter()
            .filter_map(|c| match c {
                Call::RegWriteDword(key, name, value) if key == detector_silence::EXEINFO_KEY => {
                    Some((name, value))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            writes,
            detector_silence::EXEINFO_SILENCE_VALUES
                .iter()
                .map(|(name, value)| (name.to_string(), *value))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn open_exeinfo_detects_missing_key_from_first_value() {
        let mut fake = FakeGuiAutomation::new();
        // No values set -- every RegRead returns None, matching a missing key.
        let handle = open_exeinfo(&mut fake, "exeinfope.exe", "file.exe", "C:\\bin", 1_000);
        assert!(!handle.key_existed);
    }

    #[test]
    fn close_exeinfo_restores_when_key_existed() {
        let mut fake = FakeGuiAutomation::new();
        let backed_up: Vec<(&'static str, Option<i64>)> = detector_silence::EXEINFO_SILENCE_VALUES
            .iter()
            .map(|(name, _)| (*name, Some(3)))
            .collect();
        let handle = ExeInfoHandle {
            window: WindowHandle(1),
            key_existed: true,
            backed_up,
        };

        close_exeinfo(&mut fake, &handle);

        assert!(fake.calls().contains(&Call::WinClose(WindowHandle(1))));
        assert!(!fake
            .calls()
            .iter()
            .any(|c| matches!(c, Call::RegDeleteKey(_))));
        let restores = fake
            .calls()
            .into_iter()
            .filter(|c| matches!(c, Call::RegWriteDword(..)))
            .count();
        assert_eq!(restores, detector_silence::EXEINFO_SILENCE_VALUES.len());
    }

    #[test]
    fn close_exeinfo_deletes_when_key_did_not_exist() {
        let mut fake = FakeGuiAutomation::new();
        let backed_up: Vec<(&'static str, Option<i64>)> = detector_silence::EXEINFO_SILENCE_VALUES
            .iter()
            .map(|(name, _)| (*name, None))
            .collect();
        let handle = ExeInfoHandle {
            window: WindowHandle(1),
            key_existed: false,
            backed_up,
        };

        close_exeinfo(&mut fake, &handle);

        assert!(fake.calls().contains(&Call::RegDeleteKey(
            detector_silence::EXEINFO_KEY.to_string()
        )));
    }

    #[test]
    fn rip_exeinfo_reports_success_when_end_of_file_found_and_not_marked_missing() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(EXEINFO_WINDOW_TITLE, Some(WindowHandle(1)));
        fake.script_win_wait(RIP_RESULT_WINDOW, Some(WindowHandle(2)));
        fake.script_control_handle(WindowHandle(2), RIP_RESULT_LISTBOX, Some(ControlHandle(9)));
        // First poll already finds the primary "End of file" marker.
        fake.script_listbox_find(ControlHandle(9), RIP_END_OF_FILE_MARKERS[0], 5);
        fake.script_listbox_find(ControlHandle(9), RIP_NOT_FOUND_MARKER, -1);

        let success = rip_exeinfo(
            &mut fake,
            "exeinfope.exe",
            r"C:\temp\scratch\file.exe",
            "C:\\bin",
            "{DOWN}{DOWN}{DOWN}",
            5_000,
        );

        assert!(success);
    }

    #[test]
    fn rip_exeinfo_falls_back_to_the_alternate_end_of_file_spelling() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(EXEINFO_WINDOW_TITLE, Some(WindowHandle(1)));
        fake.script_win_wait(RIP_RESULT_WINDOW, Some(WindowHandle(2)));
        fake.script_control_handle(WindowHandle(2), RIP_RESULT_LISTBOX, Some(ControlHandle(9)));
        // The primary spelling never matches; the alternate one does.
        fake.script_listbox_find(ControlHandle(9), RIP_END_OF_FILE_MARKERS[0], -1);
        fake.script_listbox_find(ControlHandle(9), RIP_END_OF_FILE_MARKERS[1], 3);
        fake.script_listbox_find(ControlHandle(9), RIP_NOT_FOUND_MARKER, -1);

        let success = rip_exeinfo(
            &mut fake,
            "exeinfope.exe",
            r"C:\temp\scratch\file.exe",
            "C:\\bin",
            "{DOWN}",
            5_000,
        );

        assert!(success);
    }

    #[test]
    fn rip_exeinfo_reports_failure_when_not_found_marker_present() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(EXEINFO_WINDOW_TITLE, Some(WindowHandle(1)));
        fake.script_win_wait(RIP_RESULT_WINDOW, Some(WindowHandle(2)));
        fake.script_control_handle(WindowHandle(2), RIP_RESULT_LISTBOX, Some(ControlHandle(9)));
        fake.script_listbox_find(ControlHandle(9), RIP_END_OF_FILE_MARKERS[0], 5);
        fake.script_listbox_find(ControlHandle(9), RIP_NOT_FOUND_MARKER, 0);

        let success = rip_exeinfo(
            &mut fake,
            "exeinfope.exe",
            r"C:\temp\scratch\file.exe",
            "C:\\bin",
            "{DOWN}",
            5_000,
        );

        assert!(!success);
    }

    /// Parity test for capability C069: when neither "End of file" spelling
    /// ever appears, the loop exits on timeout (the virtual clock in
    /// `FakeGuiAutomation` advances by `sleep`'s argument every iteration)
    /// rather than looping forever.
    #[test]
    fn rip_exeinfo_gives_up_after_timeout_when_no_marker_ever_appears() {
        let mut fake = FakeGuiAutomation::new();
        fake.script_win_wait(EXEINFO_WINDOW_TITLE, Some(WindowHandle(1)));
        fake.script_win_wait(RIP_RESULT_WINDOW, Some(WindowHandle(2)));
        fake.script_control_handle(WindowHandle(2), RIP_RESULT_LISTBOX, Some(ControlHandle(9)));
        fake.script_listbox_find(ControlHandle(9), RIP_NOT_FOUND_MARKER, -1);
        // Both "End of file" markers stay unscripted -> default -1 forever.

        let success = rip_exeinfo(
            &mut fake,
            "exeinfope.exe",
            r"C:\temp\scratch\file.exe",
            "C:\\bin",
            "{DOWN}",
            1_000,
        );

        // Not found marker also absent (-1 default): success stays true,
        // matching the source's own `== -1` check -- this test's real
        // assertion is that it terminates at all rather than hanging.
        assert!(success);
        let sleeps = fake
            .calls()
            .iter()
            .filter(|c| matches!(c, Call::Sleep(_)))
            .count();
        assert!(
            sleeps >= 5,
            "expected the loop to poll several times before timing out, got {sleeps}"
        );
    }
}
