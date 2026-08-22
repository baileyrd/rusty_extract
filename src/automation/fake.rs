//! Test double for [`super::GuiAutomation`]: records every call it
//! receives and returns caller-scripted results, so the orchestration
//! functions in `automation` can be tested without a real window to drive
//! — the exact role `extract::runner::FakeExtractorRunner` already plays
//! for plain process spawning.
//!
//! `Sleep`/`TimerInit`/`TimerDiff` run against a virtual clock instead of
//! the wall clock: `sleep(ms)` advances it by `ms`, `elapsed_ms` reports
//! the difference since `timer_init` — so a polling loop's timeout path
//! is deterministically testable without an actual multi-second wait.

use std::collections::HashMap;

use super::{ControlHandle, GuiAutomation, TimerHandle, WindowHandle};
use crate::extract::{Invocation, WindowMode};

/// One recorded call, in the order [`FakeGuiAutomation`] received it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    RegReadDword(String, String),
    RegWriteDword(String, String, i64),
    RegDeleteKey(String),
    Run(Invocation),
    WinWait(String, u64),
    WinSetState(WindowHandle, WindowMode),
    WinClose(WindowHandle),
    WinCloseByTitle(String),
    WinExists(String),
    ControlClick(WindowHandle, String),
    ControlSend(WindowHandle, String, String),
    ControlGetText(WindowHandle, String),
    ControlGetHandle(WindowHandle, String),
    ListboxFindString(ControlHandle, String, bool),
    MouseMove(i32, i32),
    Sleep(u64),
    TimerInit,
    ElapsedMs(TimerHandle),
    ProcessClose(String),
    FileExists(String),
}

#[derive(Default)]
pub struct FakeGuiAutomation {
    calls: Vec<Call>,
    registry: HashMap<(String, String), i64>,
    win_wait_results: HashMap<String, Option<WindowHandle>>,
    control_get_text_results: HashMap<(WindowHandle, String), String>,
    control_get_handle_results: HashMap<(WindowHandle, String), Option<ControlHandle>>,
    listbox_find_results: HashMap<(ControlHandle, String), i32>,
    win_exists_results: HashMap<String, bool>,
    file_exists_call_counts: HashMap<String, u32>,
    file_exists_appears_after: HashMap<String, u32>,
    clock_ms: u64,
}

impl FakeGuiAutomation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.clone()
    }

    /// Pre-seeds a registry value so `reg_read_dword` returns `Some(value)`
    /// for it; a value never seeded reads back as `None` (a missing
    /// key/value, matching `RegRead`'s `@error` behavior).
    pub fn set_reg_dword(&mut self, key: &str, value_name: &str, value: i64) {
        self.registry
            .insert((key.to_string(), value_name.to_string()), value);
    }

    /// Scripts `win_wait`'s result for `title_or_spec`. Unscripted titles
    /// default to `None` (timeout).
    pub fn script_win_wait(&mut self, title_or_spec: &str, result: Option<WindowHandle>) {
        self.win_wait_results
            .insert(title_or_spec.to_string(), result);
    }

    /// Scripts `control_get_handle`'s result for `(window, control_spec)`.
    /// Unscripted pairs default to `None` (control not found).
    pub fn script_control_handle(
        &mut self,
        window: WindowHandle,
        control_spec: &str,
        result: Option<ControlHandle>,
    ) {
        self.control_get_handle_results
            .insert((window, control_spec.to_string()), result);
    }

    /// Scripts `control_get_text`'s result for `(window, control_spec)`.
    pub fn script_control_text(&mut self, window: WindowHandle, control_spec: &str, text: &str) {
        self.control_get_text_results
            .insert((window, control_spec.to_string()), text.to_string());
    }

    /// Scripts `listbox_find_string`'s result for `(control, needle)`.
    /// Unscripted pairs default to `-1` (not found), matching the source's
    /// own sentinel.
    pub fn script_listbox_find(&mut self, control: ControlHandle, needle: &str, result: i32) {
        self.listbox_find_results
            .insert((control, needle.to_string()), result);
    }

    /// Scripts `win_exists`'s result for `title_or_spec`. Unscripted
    /// titles default to `false`.
    pub fn script_win_exists(&mut self, title_or_spec: &str, result: bool) {
        self.win_exists_results
            .insert(title_or_spec.to_string(), result);
    }

    /// Scripts `file_exists`'s result for `path`: the `polls_before`-th
    /// and every later call returns `true`; every call before that
    /// returns `false`. An unscripted path always returns `false` — a
    /// file that never appears, letting a polling loop's timeout path be
    /// tested deterministically.
    pub fn script_file_appears_after(&mut self, path: &str, polls_before: u32) {
        self.file_exists_appears_after
            .insert(path.to_string(), polls_before);
    }
}

impl GuiAutomation for FakeGuiAutomation {
    fn reg_read_dword(&mut self, key: &str, value_name: &str) -> Option<i64> {
        self.calls
            .push(Call::RegReadDword(key.to_string(), value_name.to_string()));
        self.registry
            .get(&(key.to_string(), value_name.to_string()))
            .copied()
    }

    fn reg_write_dword(&mut self, key: &str, value_name: &str, value: i64) {
        self.calls.push(Call::RegWriteDword(
            key.to_string(),
            value_name.to_string(),
            value,
        ));
        self.registry
            .insert((key.to_string(), value_name.to_string()), value);
    }

    fn reg_delete_key(&mut self, key: &str) {
        self.calls.push(Call::RegDeleteKey(key.to_string()));
        self.registry.retain(|(k, _), _| k != key);
    }

    fn run(&mut self, invocation: &Invocation) {
        self.calls.push(Call::Run(invocation.clone()));
    }

    fn win_wait(&mut self, title_or_spec: &str, timeout_ms: u64) -> Option<WindowHandle> {
        self.calls
            .push(Call::WinWait(title_or_spec.to_string(), timeout_ms));
        self.win_wait_results
            .get(title_or_spec)
            .copied()
            .unwrap_or(None)
    }

    fn win_set_state(&mut self, window: WindowHandle, mode: WindowMode) {
        self.calls.push(Call::WinSetState(window, mode));
    }

    fn win_close(&mut self, window: WindowHandle) {
        self.calls.push(Call::WinClose(window));
    }

    fn win_close_by_title(&mut self, title_or_spec: &str) {
        self.calls
            .push(Call::WinCloseByTitle(title_or_spec.to_string()));
    }

    fn win_exists(&mut self, title_or_spec: &str) -> bool {
        self.calls.push(Call::WinExists(title_or_spec.to_string()));
        self.win_exists_results
            .get(title_or_spec)
            .copied()
            .unwrap_or(false)
    }

    fn control_click(&mut self, window: WindowHandle, control_spec: &str) {
        self.calls
            .push(Call::ControlClick(window, control_spec.to_string()));
    }

    fn control_send(&mut self, window: WindowHandle, control_spec: &str, keys: &str) {
        self.calls.push(Call::ControlSend(
            window,
            control_spec.to_string(),
            keys.to_string(),
        ));
    }

    fn control_get_text(&mut self, window: WindowHandle, control_spec: &str) -> String {
        self.calls
            .push(Call::ControlGetText(window, control_spec.to_string()));
        self.control_get_text_results
            .get(&(window, control_spec.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    fn control_get_handle(
        &mut self,
        window: WindowHandle,
        control_spec: &str,
    ) -> Option<ControlHandle> {
        self.calls
            .push(Call::ControlGetHandle(window, control_spec.to_string()));
        self.control_get_handle_results
            .get(&(window, control_spec.to_string()))
            .copied()
            .unwrap_or(None)
    }

    fn listbox_find_string(&mut self, control: ControlHandle, needle: &str, exact: bool) -> i32 {
        self.calls
            .push(Call::ListboxFindString(control, needle.to_string(), exact));
        self.listbox_find_results
            .get(&(control, needle.to_string()))
            .copied()
            .unwrap_or(-1)
    }

    fn mouse_move(&mut self, x: i32, y: i32) {
        self.calls.push(Call::MouseMove(x, y));
    }

    fn sleep(&mut self, ms: u64) {
        self.calls.push(Call::Sleep(ms));
        self.clock_ms += ms;
    }

    fn timer_init(&mut self) -> TimerHandle {
        self.calls.push(Call::TimerInit);
        TimerHandle(self.clock_ms)
    }

    fn elapsed_ms(&mut self, since: TimerHandle) -> u64 {
        self.calls.push(Call::ElapsedMs(since));
        self.clock_ms.saturating_sub(since.0)
    }

    fn process_close(&mut self, name: &str) {
        self.calls.push(Call::ProcessClose(name.to_string()));
    }

    fn file_exists(&mut self, path: &str) -> bool {
        self.calls.push(Call::FileExists(path.to_string()));
        let count = self
            .file_exists_call_counts
            .entry(path.to_string())
            .or_insert(0);
        *count += 1;
        match self.file_exists_appears_after.get(path) {
            Some(threshold) => *count >= *threshold,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unscripted_registry_value_reads_as_none() {
        let mut fake = FakeGuiAutomation::new();
        assert_eq!(fake.reg_read_dword(r"HKCU\Foo", "Bar"), None);
    }

    #[test]
    fn seeded_registry_value_reads_back() {
        let mut fake = FakeGuiAutomation::new();
        fake.set_reg_dword(r"HKCU\Foo", "Bar", 42);
        assert_eq!(fake.reg_read_dword(r"HKCU\Foo", "Bar"), Some(42));
    }

    #[test]
    fn reg_delete_clears_every_value_under_the_key() {
        let mut fake = FakeGuiAutomation::new();
        fake.set_reg_dword(r"HKCU\Foo", "A", 1);
        fake.set_reg_dword(r"HKCU\Foo", "B", 2);
        fake.reg_delete_key(r"HKCU\Foo");
        assert_eq!(fake.reg_read_dword(r"HKCU\Foo", "A"), None);
        assert_eq!(fake.reg_read_dword(r"HKCU\Foo", "B"), None);
    }

    #[test]
    fn virtual_clock_advances_only_via_sleep() {
        let mut fake = FakeGuiAutomation::new();
        let timer = fake.timer_init();
        assert_eq!(fake.elapsed_ms(timer), 0);
        fake.sleep(200);
        fake.sleep(300);
        assert_eq!(fake.elapsed_ms(timer), 500);
    }

    #[test]
    fn unscripted_win_wait_times_out() {
        let mut fake = FakeGuiAutomation::new();
        assert_eq!(fake.win_wait("Some Window", 1_000), None);
    }

    #[test]
    fn calls_are_recorded_in_order() {
        let mut fake = FakeGuiAutomation::new();
        fake.mouse_move(1, 2);
        fake.sleep(10);
        assert_eq!(fake.calls(), vec![Call::MouseMove(1, 2), Call::Sleep(10)]);
    }
}
