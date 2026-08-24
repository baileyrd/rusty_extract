//! Real Win32-backed [`super::GuiAutomation`] implementation.
//!
//! Windows-only (`#[cfg(windows)]`) — this file doesn't compile as part of
//! the crate on any other target, and the Linux dev/CI environment this
//! port is otherwise developed on can't run it at all. Compiled and
//! type-checked against the `x86_64-pc-windows-gnu` target during
//! development (no dependency on the MSVC toolchain/linker), but never
//! executed against a real window here — see `super`'s module doc comment
//! for exactly what that does and doesn't prove. CI (`windows-latest`) can
//! build this but has no interactive desktop session either, so even a
//! green CI run only proves it compiles and links, not that it drives a
//! real Exeinfo PE window correctly.

use std::collections::HashMap;
use std::time::Instant;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_DWORD,
    REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_DOWN, VK_RETURN, VK_RIGHT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowW, GetClassNameW, GetWindowTextW,
    GetWindowThreadProcessId, PostMessageW, SendMessageW, SetCursorPos, SetForegroundWindow,
    ShowWindow, BM_CLICK, SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWMINIMIZED, SW_SHOWNORMAL, WM_CHAR,
    WM_KEYDOWN, WM_KEYUP,
};

use super::control_spec::{parse_control_spec, ControlSpec};
use super::keys::{parse_key_sequence, KeyToken};
use super::{ControlHandle, GuiAutomation, TimerHandle, WindowHandle};
use crate::extract::{Invocation, WindowMode};

/// Real Win32-backed automation. Every handle this returns is a live OS
/// resource for the lifetime of the process — there is no cleanup-on-drop
/// here (matching AutoIt, which never explicitly closes window/control
/// handles either; they're just integers).
#[derive(Default)]
pub struct Win32GuiAutomation {
    timers: HashMap<u64, Instant>,
    next_timer_id: u64,
    file_positions: HashMap<String, u64>,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn window_mode_to_show_cmd(mode: WindowMode) -> SHOW_WINDOW_CMD {
    match mode {
        WindowMode::Hidden => SW_HIDE,
        WindowMode::Minimized => SW_SHOWMINIMIZED,
        WindowMode::Show => SW_SHOWNORMAL,
    }
}

/// Splits `"HKCU\Software\ExEi-pe"` into its root [`HKEY`] and subkey path.
/// `HKCU`/`HKLM` cover this port's original call sites; `HKCR` was added
/// for C202 (`GUI_ContextMenu_OK`'s context-menu registration, which
/// writes under `HKEY_CLASSES_ROOT`). Any other root falls back to
/// `HKEY_CURRENT_USER` rather than panicking, since a malformed key
/// string here would otherwise crash automation entirely over a value
/// this crate doesn't control the shape of.
fn parse_reg_key(key: &str) -> (HKEY, String) {
    match key.split_once('\\') {
        Some(("HKCU", rest)) => (HKEY_CURRENT_USER, rest.to_string()),
        Some(("HKLM", rest)) => (HKEY_LOCAL_MACHINE, rest.to_string()),
        Some(("HKCR", rest)) => (HKEY_CLASSES_ROOT, rest.to_string()),
        Some((_, rest)) => (HKEY_CURRENT_USER, rest.to_string()),
        None => (HKEY_CURRENT_USER, key.to_string()),
    }
}

/// Shared by [`GuiAutomation::reg_write_string`] and
/// [`GuiAutomation::reg_write_expand_string`] -- identical except for
/// which `REG_VALUE_TYPE` the value is tagged with.
fn write_string_value(key: &str, value_name: &str, value: &str, reg_type: REG_VALUE_TYPE) {
    let (root, subkey) = parse_reg_key(key);
    let subkey_wide = to_wide(&subkey);
    let value_name_wide = to_wide(value_name);
    let mut hkey = HKEY::default();
    let opened = unsafe {
        RegCreateKeyExW(
            root,
            PCWSTR(subkey_wide.as_ptr()),
            Some(0),
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut hkey,
            None,
        )
    };
    if opened != windows::Win32::Foundation::WIN32_ERROR(0) {
        return;
    }
    // Includes the trailing NUL `to_wide` already appends -- REG_SZ's
    // own documented convention, matching AutoIt's `RegWrite`.
    let value_wide = to_wide(value);
    let value_bytes = unsafe {
        std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2)
    };
    unsafe {
        let _ = RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            Some(0),
            reg_type,
            Some(value_bytes),
        );
        let _ = RegCloseKey(hkey);
    }
}

/// Finds a top-level window either by its literal title (`FindWindowW`
/// with a `NULL` class) or, for a `[CLASS:name]`-style spec, by class
/// name — the two shapes this port's cited call sites use for `WinWait`.
fn find_window(title_or_spec: &str) -> Option<HWND> {
    let hwnd = if let Some(ControlSpec { class_name, .. }) = title_or_spec
        .strip_prefix('[')
        .and(parse_control_spec(title_or_spec))
    {
        let class_wide = to_wide(&class_name);
        unsafe { FindWindowW(PCWSTR(class_wide.as_ptr()), PCWSTR::null()) }
    } else {
        let title_wide = to_wide(title_or_spec);
        unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title_wide.as_ptr())) }
    };
    hwnd.ok().filter(|h| !h.is_invalid())
}

struct EnumState {
    class_name: String,
    remaining_instances: u32,
    found: Option<HWND>,
}

struct FindByPidState {
    pid: u32,
    found: Option<HWND>,
}

unsafe extern "system" fn enum_window_by_pid_proc(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows::core::BOOL {
    let state = &mut *(lparam.0 as *mut FindByPidState);
    let mut owner_pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut owner_pid));
    }
    if owner_pid == state.pid {
        state.found = Some(hwnd);
        return windows::core::BOOL(0); // stop enumerating
    }
    windows::core::BOOL(1) // continue
}

unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    let class = String::from_utf16_lossy(&buf[..len as usize]);
    if class == state.class_name {
        state.remaining_instances -= 1;
        if state.remaining_instances == 0 {
            state.found = Some(hwnd);
            return windows::core::BOOL(0); // stop enumerating
        }
    }
    windows::core::BOOL(1) // continue
}

/// Resolves a control spec (`[CLASS:name; INSTANCE:n]` or `ClassNameNN`)
/// to the `instance`-th child window of `parent` whose class name matches
/// exactly, via `EnumChildWindows` — the closest real-API equivalent of
/// AutoIt's own control-resolution engine for these two spec shapes.
fn resolve_control(parent: HWND, spec: &str) -> Option<HWND> {
    let ControlSpec {
        class_name,
        instance,
    } = parse_control_spec(spec)?;
    let mut state = EnumState {
        class_name,
        remaining_instances: instance.max(1),
        found: None,
    };
    unsafe {
        let _ = EnumChildWindows(
            Some(parent),
            Some(enum_child_proc),
            LPARAM(&mut state as *mut EnumState as isize),
        );
    }
    state.found
}

impl GuiAutomation for Win32GuiAutomation {
    fn reg_read_dword(&mut self, key: &str, value_name: &str) -> Option<i64> {
        let (root, subkey) = parse_reg_key(key);
        let subkey_wide = to_wide(&subkey);
        let value_wide = to_wide(value_name);
        let mut hkey = HKEY::default();
        let opened = unsafe {
            windows::Win32::System::Registry::RegOpenKeyExW(
                root,
                PCWSTR(subkey_wide.as_ptr()),
                Some(0),
                windows::Win32::System::Registry::KEY_READ,
                &mut hkey,
            )
        };
        if opened != windows::Win32::Foundation::WIN32_ERROR(0) {
            return None;
        }
        let mut data: u32 = 0;
        let mut data_len: u32 = std::mem::size_of::<u32>() as u32;
        let mut value_type = REG_VALUE_TYPE(0);
        let result = unsafe {
            RegQueryValueExW(
                hkey,
                PCWSTR(value_wide.as_ptr()),
                None,
                Some(&mut value_type),
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_len),
            )
        };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        if result == windows::Win32::Foundation::WIN32_ERROR(0) && value_type == REG_DWORD {
            Some(data as i64)
        } else {
            None
        }
    }

    fn reg_write_dword(&mut self, key: &str, value_name: &str, value: i64) {
        let (root, subkey) = parse_reg_key(key);
        let subkey_wide = to_wide(&subkey);
        let value_wide = to_wide(value_name);
        let mut hkey = HKEY::default();
        let opened = unsafe {
            RegCreateKeyExW(
                root,
                PCWSTR(subkey_wide.as_ptr()),
                Some(0),
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                None,
                &mut hkey,
                None,
            )
        };
        if opened != windows::Win32::Foundation::WIN32_ERROR(0) {
            return;
        }
        let data = value as u32;
        unsafe {
            let _ = RegSetValueExW(
                hkey,
                PCWSTR(value_wide.as_ptr()),
                Some(0),
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &data as *const u32 as *const u8,
                    4,
                )),
            );
            let _ = RegCloseKey(hkey);
        }
    }

    fn reg_write_string(&mut self, key: &str, value_name: &str, value: &str) {
        write_string_value(key, value_name, value, REG_SZ);
    }

    fn reg_write_expand_string(&mut self, key: &str, value_name: &str, value: &str) {
        write_string_value(key, value_name, value, REG_EXPAND_SZ);
    }

    fn reg_read_string(&mut self, key: &str, value_name: &str) -> Option<String> {
        let (root, subkey) = parse_reg_key(key);
        let subkey_wide = to_wide(&subkey);
        let value_wide = to_wide(value_name);
        let mut hkey = HKEY::default();
        let opened = unsafe {
            windows::Win32::System::Registry::RegOpenKeyExW(
                root,
                PCWSTR(subkey_wide.as_ptr()),
                Some(0),
                windows::Win32::System::Registry::KEY_READ,
                &mut hkey,
            )
        };
        if opened != windows::Win32::Foundation::WIN32_ERROR(0) {
            return None;
        }
        let mut value_type = REG_VALUE_TYPE(0);
        let mut data_len: u32 = 0;
        let sized = unsafe {
            RegQueryValueExW(
                hkey,
                PCWSTR(value_wide.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut data_len),
            )
        };
        if sized != windows::Win32::Foundation::WIN32_ERROR(0)
            || (value_type != REG_SZ && value_type != REG_EXPAND_SZ)
        {
            unsafe {
                let _ = RegCloseKey(hkey);
            }
            return None;
        }
        let mut buffer: Vec<u16> = vec![0; (data_len as usize).div_ceil(2)];
        let result = unsafe {
            RegQueryValueExW(
                hkey,
                PCWSTR(value_wide.as_ptr()),
                None,
                None,
                Some(buffer.as_mut_ptr() as *mut u8),
                Some(&mut data_len),
            )
        };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        if result != windows::Win32::Foundation::WIN32_ERROR(0) {
            return None;
        }
        // Trim the trailing NUL terminator(s) REG_SZ/REG_EXPAND_SZ data
        // includes, matching `RegRead`'s own string return.
        while buffer.last() == Some(&0) {
            buffer.pop();
        }
        Some(String::from_utf16_lossy(&buffer))
    }

    fn reg_delete_key(&mut self, key: &str) {
        // `RegDeleteKeyW` (the API this used before this fix) refuses to
        // delete a key that still has subkeys -- it is *not* recursive,
        // unlike AutoIt's own `RegDelete()`, which deletes a key and
        // everything under it in one call (verified against AutoIt's
        // documented behavior while porting C204, which relies on
        // exactly this to remove a ProgID key's `\DefaultIcon`/
        // `\shell\open\command`/`\command` subkeys in one `RegDelete`
        // call). `RegDeleteTreeW` is the real recursive-delete API this
        // needs -- every prior caller of `reg_delete_key` (C069's
        // Exeinfo PE registry backup/restore, C201-C203's context-menu
        // verb keys, which also have their own `\command` subkey) gets
        // this fix too, not just C204's new call sites.
        let (root, subkey) = parse_reg_key(key);
        let subkey_wide = to_wide(&subkey);
        unsafe {
            let _ = RegDeleteTreeW(root, PCWSTR(subkey_wide.as_ptr()));
        }
    }

    fn run(&mut self, invocation: &Invocation) -> u32 {
        // `_MakeCommand`'s own bindir-prefixing isn't modeled (same scope
        // note as every `Invocation`-consuming module in this crate) --
        // `invocation.program` is spawned directly via `std::process`
        // rather than raw `CreateProcessW`, matching
        // `extract::runner::CommandExtractorRunner`'s own choice for the
        // same reason: no behavioral difference for this port's purposes,
        // far less unsafe surface. The spawned `Child` is intentionally
        // dropped without waiting -- matching `Run()`'s own fire-and-
        // forget semantics, the process keeps running after this returns.
        std::process::Command::new(&invocation.program)
            .args(&invocation.args)
            .current_dir(&invocation.working_dir)
            .spawn()
            .map(|child| child.id())
            .unwrap_or(0)
    }

    fn win_wait(&mut self, title_or_spec: &str, timeout_ms: u64) -> Option<WindowHandle> {
        let deadline = Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            if let Some(hwnd) = find_window(title_or_spec) {
                return Some(WindowHandle(hwnd.0 as isize));
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    fn win_set_state(&mut self, window: WindowHandle, mode: WindowMode) {
        let hwnd = HWND(window.0 as *mut _);
        unsafe {
            let _ = ShowWindow(hwnd, window_mode_to_show_cmd(mode));
        }
    }

    fn win_close(&mut self, window: WindowHandle) {
        let hwnd = HWND(window.0 as *mut _);
        unsafe {
            let _ = PostMessageW(
                Some(hwnd),
                windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }

    fn win_close_by_title(&mut self, title_or_spec: &str) {
        if let Some(hwnd) = find_window(title_or_spec) {
            unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }

    fn win_exists(&mut self, title_or_spec: &str) -> bool {
        find_window(title_or_spec).is_some()
    }

    fn control_click(&mut self, window: WindowHandle, control_spec: &str) {
        let hwnd = HWND(window.0 as *mut _);
        if let Some(control) = resolve_control(hwnd, control_spec) {
            unsafe {
                let _ = SendMessageW(control, BM_CLICK, Some(WPARAM(0)), Some(LPARAM(0)));
            }
        }
    }

    fn control_send(&mut self, window: WindowHandle, control_spec: &str, keys: &str) {
        let hwnd = HWND(window.0 as *mut _);
        let Some(control) = resolve_control(hwnd, control_spec) else {
            return;
        };
        for token in parse_key_sequence(keys) {
            let vk = match token {
                KeyToken::Down => Some(VK_DOWN.0),
                KeyToken::Right => Some(VK_RIGHT.0),
                KeyToken::Enter => Some(VK_RETURN.0),
                KeyToken::Literal(_) => None,
            };
            unsafe {
                if let Some(vk) = vk {
                    let _ = SendMessageW(
                        control,
                        WM_KEYDOWN,
                        Some(WPARAM(vk as usize)),
                        Some(LPARAM(0)),
                    );
                    let _ = SendMessageW(
                        control,
                        WM_KEYUP,
                        Some(WPARAM(vk as usize)),
                        Some(LPARAM(0)),
                    );
                } else if let KeyToken::Literal(ch) = token {
                    let _ =
                        SendMessageW(control, WM_CHAR, Some(WPARAM(ch as usize)), Some(LPARAM(0)));
                }
            }
        }
    }

    fn control_get_text(&mut self, window: WindowHandle, control_spec: &str) -> String {
        let hwnd = HWND(window.0 as *mut _);
        let Some(control) = resolve_control(hwnd, control_spec) else {
            return String::new();
        };
        let mut buf = [0u16; 1024];
        let len = unsafe { GetWindowTextW(control, &mut buf) };
        String::from_utf16_lossy(&buf[..len.max(0) as usize])
    }

    fn control_get_handle(
        &mut self,
        window: WindowHandle,
        control_spec: &str,
    ) -> Option<ControlHandle> {
        let hwnd = HWND(window.0 as *mut _);
        resolve_control(hwnd, control_spec).map(|h| ControlHandle(h.0 as isize))
    }

    fn listbox_find_string(&mut self, control: ControlHandle, needle: &str, exact: bool) -> i32 {
        const LB_FINDSTRING: u32 = 0x018F;
        const LB_FINDSTRINGEXACT: u32 = 0x01A2;
        let hwnd = HWND(control.0 as *mut _);
        let needle_wide = to_wide(needle);
        let message = if exact {
            LB_FINDSTRINGEXACT
        } else {
            LB_FINDSTRING
        };
        let result: LRESULT = unsafe {
            SendMessageW(
                hwnd,
                message,
                Some(WPARAM(usize::MAX)), // search from the start, like AutoIt's own -1
                Some(LPARAM(needle_wide.as_ptr() as isize)),
            )
        };
        result.0 as i32
    }

    fn mouse_move(&mut self, x: i32, y: i32) {
        unsafe {
            let _ = SetCursorPos(x, y);
        }
    }

    fn sleep(&mut self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    fn timer_init(&mut self) -> TimerHandle {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.insert(id, Instant::now());
        TimerHandle(id)
    }

    fn elapsed_ms(&mut self, since: TimerHandle) -> u64 {
        self.timers
            .get(&since.0)
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    fn process_close(&mut self, name: &str) {
        // No direct Win32 equivalent worth the extra unsafe surface here --
        // shells out to `taskkill`, the same approach AutoIt's own
        // `ProcessClose` reduces to under the hood for a name (as opposed
        // to a PID).
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", name, "/F"])
            .output();
    }

    fn file_exists(&mut self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn process_exists(&mut self, pid: u32) -> bool {
        unsafe {
            match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(handle) => {
                    let _ = windows::Win32::Foundation::CloseHandle(handle);
                    true
                }
                Err(_) => false,
            }
        }
    }

    fn win_get_by_pid(&mut self, pid: u32) -> Option<WindowHandle> {
        let mut state = FindByPidState { pid, found: None };
        unsafe {
            let _ = EnumWindows(
                Some(enum_window_by_pid_proc),
                LPARAM(&mut state as *mut FindByPidState as isize),
            );
        }
        state.found.map(|h| WindowHandle(h.0 as isize))
    }

    fn win_set_state_by_title(&mut self, title_or_spec: &str, mode: WindowMode) {
        if let Some(hwnd) = find_window(title_or_spec) {
            unsafe {
                let _ = ShowWindow(hwnd, window_mode_to_show_cmd(mode));
            }
        }
    }

    fn win_activate(&mut self, window: WindowHandle) {
        let hwnd = HWND(window.0 as *mut _);
        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
    }

    fn read_file_incremental(&mut self, path: &str) -> String {
        use std::io::{Read, Seek, SeekFrom};
        let Ok(mut file) = std::fs::File::open(path) else {
            return String::new();
        };
        let pos = *self.file_positions.get(path).unwrap_or(&0);
        if file.seek(SeekFrom::Start(pos)).is_err() {
            return String::new();
        }
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf);
        let new_pos = file.stream_position().unwrap_or(pos);
        self.file_positions.insert(path.to_string(), new_pos);
        buf
    }

    fn read_file_from_start(&mut self, path: &str) -> String {
        self.file_positions.remove(path);
        std::fs::read_to_string(path).unwrap_or_default()
    }

    fn dir_size_bytes(&mut self, path: &str) -> u64 {
        fn walk(dir: &std::path::Path) -> u64 {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return 0;
            };
            let mut total = 0u64;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        total += walk(&path);
                    } else {
                        total += metadata.len();
                    }
                }
            }
            total
        }
        walk(std::path::Path::new(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hkcu_root() {
        assert_eq!(
            parse_reg_key(r"HKCU\Software\ExEi-pe").1,
            r"Software\ExEi-pe"
        );
    }
}
