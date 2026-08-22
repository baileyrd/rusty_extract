//! Real Win32-backed [`super::TridLibrary`]/[`super::MediaInfoLibrary`]
//! implementations.
//!
//! Windows-only (`#[cfg(windows)]`), same as `automation::win32` — this
//! file doesn't compile on any other target, is type-checked against the
//! `x86_64-pc-windows-gnu` target during development (this environment
//! can't build native Windows binaries), and is never executed against
//! the real DLLs here. See `super`'s module doc comment for exactly what
//! that does and doesn't prove.
//!
//! **`GetProcAddress` + a hand-written function-pointer type per
//! export**, not a generic dynamic-call shim — see `super`'s module doc
//! comment for why this is the right level of specificity rather than a
//! shortcut. Every exported function here follows the `extern "system"`
//! (`stdcall` on x86, the platform default on x64) calling convention —
//! AutoIt's own `DllCall` default when no `cdecl:` prefix is given, which
//! none of these type strings use.
//!
//! **A documented assumption, not verifiable without `TrIDLib.dll`'s own
//! (non-public) contract**: `TrID_GetInfo`'s output buffer is allocated
//! here at a fixed 4096 bytes. AutoIt's own `DllCall(..., "str", "")`
//! passing an empty string doesn't specify a buffer size either — this
//! matches common real-world usage of the library, but isn't something
//! this port can confirm without the actual DLL and its own
//! documentation.
//!
//! **ANSI marshalling for TrIDLib's `"str"` parameters**: AutoIt's `str`
//! `DllCall` type is ANSI (`LPSTR`), the same marshalling choice already
//! found for C178 (`extract::trid_scan::trid_dll_string_marshalling`) —
//! non-ASCII paths are lossy here, matching the source's own limitation,
//! not a gap this port introduces.

use std::ffi::CString;

use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

use super::{MediaInfoHandle, MediaInfoLibrary, TridLibrary};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn to_ansi(s: &str) -> Vec<u8> {
    s.bytes().chain(std::iter::once(0)).collect()
}

/// Reads a null-terminated ANSI buffer back into a `String`, lossily
/// (matching AutoIt's own ANSI `"str"` type).
fn from_ansi_buf(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// Reads a null-terminated UTF-16 string from a raw pointer MediaInfo
/// itself owns (not freed here — matches `MediaInfo_Inform`'s documented
/// contract of returning a library-owned buffer).
unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf16_lossy(slice)
}

unsafe fn get_proc<F: Copy>(module: HMODULE, name: &str) -> Option<F> {
    let cname = CString::new(name).ok()?;
    let addr = unsafe { GetProcAddress(module, PCSTR(cname.as_ptr() as *const u8)) }?;
    Some(unsafe { std::mem::transmute_copy(&addr) })
}

type FnLoadDefsPack = unsafe extern "system" fn(*const i8) -> i32;
type FnSubmitFileA = unsafe extern "system" fn(*const i8) -> i32;
type FnAnalyze = unsafe extern "system" fn() -> i32;
type FnGetInfo = unsafe extern "system" fn(i32, i32, *mut i8) -> i32;

/// Real `TrIDLib.dll` binding. `new` returns `None` if the DLL itself
/// couldn't be loaded (`DllOpen`'s `-1`/`@error`).
pub struct Win32TridLibrary {
    module: HMODULE,
}

impl Win32TridLibrary {
    pub fn new(dll_path: &str) -> Option<Self> {
        let wide = to_wide(dll_path);
        let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }.ok()?;
        Some(Self { module })
    }
}

impl Drop for Win32TridLibrary {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.module);
        }
    }
}

impl TridLibrary for Win32TridLibrary {
    fn load_defs_pack(&mut self, bindir: &str) -> Option<u32> {
        let f: FnLoadDefsPack = unsafe { get_proc(self.module, "TrID_LoadDefsPack") }?;
        let bindir_ansi = to_ansi(bindir);
        let result = unsafe { f(bindir_ansi.as_ptr() as *const i8) };
        u32::try_from(result).ok()
    }

    fn submit_file(&mut self, file: &str) {
        if let Some(f) = unsafe { get_proc::<FnSubmitFileA>(self.module, "TrID_SubmitFileA") } {
            let file_ansi = to_ansi(file);
            unsafe {
                f(file_ansi.as_ptr() as *const i8);
            }
        }
    }

    fn analyze(&mut self) {
        if let Some(f) = unsafe { get_proc::<FnAnalyze>(self.module, "TrID_Analyze") } {
            unsafe {
                f();
            }
        }
    }

    fn result_count(&mut self) -> i64 {
        let Some(f) = (unsafe { get_proc::<FnGetInfo>(self.module, "TrID_GetInfo") }) else {
            return 0;
        };
        let mut buf = [0i8; 4096];
        unsafe { f(1, 0, buf.as_mut_ptr()) as i64 }
    }

    fn result_type(&mut self, index: u32) -> Option<String> {
        let f: FnGetInfo = unsafe { get_proc(self.module, "TrID_GetInfo") }?;
        let mut buf = [0u8; 4096];
        let result = unsafe { f(2, index as i32, buf.as_mut_ptr() as *mut i8) };
        if result < 0 {
            None
        } else {
            Some(from_ansi_buf(&buf))
        }
    }

    fn result_extension(&mut self, index: u32) -> Option<String> {
        let f: FnGetInfo = unsafe { get_proc(self.module, "TrID_GetInfo") }?;
        let mut buf = [0u8; 4096];
        let result = unsafe { f(3, index as i32, buf.as_mut_ptr() as *mut i8) };
        if result < 0 {
            None
        } else {
            Some(from_ansi_buf(&buf))
        }
    }
}

type FnMediaInfoNew = unsafe extern "system" fn() -> *mut core::ffi::c_void;
type FnMediaInfoOpen = unsafe extern "system" fn(*mut core::ffi::c_void, *const u16) -> usize;
type FnMediaInfoInform = unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> *const u16;
type FnMediaInfoDelete = unsafe extern "system" fn(*mut core::ffi::c_void);

/// Real `MediaInfo.dll` binding.
pub struct Win32MediaInfoLibrary {
    module: HMODULE,
    instances: std::collections::HashMap<u64, *mut core::ffi::c_void>,
    next_handle: u64,
}

impl Win32MediaInfoLibrary {
    pub fn new(dll_path: &str) -> Option<Self> {
        let wide = to_wide(dll_path);
        let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }.ok()?;
        Some(Self {
            module,
            instances: std::collections::HashMap::new(),
            next_handle: 0,
        })
    }
}

impl Drop for Win32MediaInfoLibrary {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.module);
        }
    }
}

impl MediaInfoLibrary for Win32MediaInfoLibrary {
    fn open(&mut self, file: &str) -> Option<MediaInfoHandle> {
        let new_fn: FnMediaInfoNew = unsafe { get_proc(self.module, "MediaInfo_New") }?;
        let open_fn: FnMediaInfoOpen = unsafe { get_proc(self.module, "MediaInfo_Open") }?;
        let instance = unsafe { new_fn() };
        if instance.is_null() {
            return None;
        }
        let file_wide = to_wide(file);
        unsafe {
            open_fn(instance, file_wide.as_ptr());
        }
        let handle = MediaInfoHandle(self.next_handle);
        self.next_handle += 1;
        self.instances.insert(handle.0, instance);
        Some(handle)
    }

    fn inform(&mut self, handle: MediaInfoHandle) -> String {
        let Some(&instance) = self.instances.get(&handle.0) else {
            return String::new();
        };
        let Some(inform_fn) =
            (unsafe { get_proc::<FnMediaInfoInform>(self.module, "MediaInfo_Inform") })
        else {
            return String::new();
        };
        let ptr = unsafe { inform_fn(instance, 0) };
        unsafe { from_wide_ptr(ptr) }
    }

    fn close(&mut self, handle: MediaInfoHandle) {
        let Some(instance) = self.instances.remove(&handle.0) else {
            return;
        };
        if let Some(delete_fn) =
            unsafe { get_proc::<FnMediaInfoDelete>(self.module, "MediaInfo_Delete") }
        {
            unsafe {
                delete_fn(instance);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_roundtrip_stops_at_null_terminator() {
        let mut buf = [0u8; 16];
        buf[..5].copy_from_slice(b"hello");
        assert_eq!(from_ansi_buf(&buf), "hello");
    }

    #[test]
    fn to_ansi_null_terminates() {
        assert_eq!(to_ansi("hi"), vec![b'h', b'i', 0]);
    }
}
