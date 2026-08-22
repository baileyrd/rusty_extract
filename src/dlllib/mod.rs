//! DLL-calling infrastructure: the seam between a ported orchestration
//! function and actually calling into `TrIDLib.dll`/`MediaInfo.dll` —
//! capabilities C038/C045's remaining gap. Mirrors the split already
//! established for `extract::runner::ExtractorRunner` (plain process
//! spawning) and `automation::GuiAutomation` (Win32 window automation):
//! a trait per DLL ([`TridLibrary`], [`MediaInfoLibrary`]), a real
//! Win32-backed implementation ([`win32`], Windows-only), and a test
//! double ([`fake`]) that records every call so orchestration logic can
//! be tested without the real, licensed DLLs — neither of which exists
//! in this environment or on CI.
//!
//! **Why two small, function-specific traits instead of one generic
//! `DllCall` shim.** AutoIt's `DllCall` can invoke an arbitrary exported
//! function by name with a dynamically-typed argument list — replicating
//! that generally in Rust would need something like the `libffi` crate
//! (a real, heavier dependency implementing a runtime FFI trampoline).
//! Since only seven specific exports across two DLLs are ever needed
//! (`TrID_LoadDefsPack`/`TrID_SubmitFileA`/`TrID_Analyze`/`TrID_GetInfo`;
//! `MediaInfo_New`/`MediaInfo_Open`/`MediaInfo_Inform`/`MediaInfo_Delete`),
//! function-specific trait methods are simpler, safer, and match the
//! same specificity `automation::GuiAutomation`'s own methods already
//! have (they cover AutoIt's specific window functions, not a generic
//! Win32-syscall shim) — not a shortcut, the more consistent design.
//!
//! **What this does and doesn't prove.** Fake-backed tests verify the
//! ported orchestration functions' decision logic against the source
//! line-by-line, the same confidence every other parity test in this
//! crate has. They do **not** prove the real Win32 backend
//! ([`win32::Win32TridLibrary`]/[`win32::Win32MediaInfoLibrary`])
//! actually calls the real DLLs correctly — that needs the real,
//! licensed `TrIDLib.dll`/`MediaInfo.dll` (plus, for TrIDLib, its
//! definitions pack) loaded on a real Windows machine, which doesn't
//! exist here or on CI (headless `windows-latest`). Same caveat as
//! `automation`'s own module doc comment, not glossed over.
//!
//! **A genuine, preserved quirk found while porting `TridLib_Load`**
//! (UniExtract.au3:944-957): `$hTridDll = DllOpen(...)` sets the cache
//! variable *before* `TrID_LoadDefsPack`'s own result is checked — so a
//! failed definitions-pack load still leaves `$hTridDll` looking
//! "already loaded" to the next call's `If $hTridDll Then Return True`
//! reentry guard, silently skipping the retry that guard exists to
//! avoid. [`tridlib_load`] doesn't reproduce the caching itself (a
//! plain reentrancy guard, the same "trivial, not modeled as its own
//! decision function" call already made for `$tridfailed` elsewhere in
//! this crate) — a caller holding its own `Option<LibraryHandle>` cache
//! should be aware the source's own equivalent has this gap.

pub mod fake;
#[cfg(windows)]
pub mod win32;

/// Opaque handle to an open `TrIDLib.dll`/`MediaInfo.dll` library
/// (`DllOpen`'s return value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LibraryHandle(pub isize);

/// Opaque handle to a `MediaInfo_New()` instance (`MediaInfo_Open`/
/// `_Inform`/`_Delete` all take this as their first argument).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaInfoHandle(pub u64);

/// The `TrIDLib.dll` exports `FileScan_Trid`'s extract-mode path calls
/// (UniExtract.au3:944-1001): load the definitions pack, submit and
/// analyze one file, then read back its results one at a time.
pub trait TridLibrary {
    /// `DllCall($hTridDll, "int", "TrID_LoadDefsPack", "str", $bindir)`
    /// (UniExtract.au3:949) — `None` on `@error` (the DLL itself failed
    /// to open or the call errored); `Some(n)` for the definitions-pack
    /// count the DLL reports, whether or not it's `>= 1`.
    fn load_defs_pack(&mut self, bindir: &str) -> Option<u32>;
    /// `DllCall($hTridDll, "int", "TrID_SubmitFileA", "str", $sFile)`
    /// (UniExtract.au3:964) — return value discarded in the source.
    fn submit_file(&mut self, file: &str);
    /// `DllCall($hTridDll, "int", "TrID_Analyze")` (UniExtract.au3:965)
    /// — return value discarded in the source.
    fn analyze(&mut self);
    /// `DllCall($hTridDll, "int", "TrID_GetInfo", "int", 1, "int", 0,
    /// "str", "")` (UniExtract.au3:967): mode `1` returns the number of
    /// results the most recent [`analyze`](Self::analyze) call found.
    fn result_count(&mut self) -> i64;
    /// `DllCall($hTridDll, "int", "TrID_GetInfo", "int", 2, "int",
    /// $iIndex, "str", "")` (UniExtract.au3:991): mode `2` returns the
    /// `index`-th (1-indexed) result's type-description string. `None`
    /// on `@error`, matching `TridLib_GetType`'s own `Return SetError(1,
    /// 0, 0)`.
    fn result_type(&mut self, index: u32) -> Option<String>;
    /// `DllCall($hTridDll, "int", "TrID_GetInfo", "int", 3, "int",
    /// $iIndex, "str", "")` (UniExtract.au3:998): mode `3` returns the
    /// `index`-th result's suggested extension. `None` on `@error`,
    /// matching `TridLib_GetExtension`'s own `Return SetError(1, 0, 0)`.
    /// Unlike `TridLib_GetExtension` itself, lower-casing the result
    /// (its own `StringLower(...)`) is left to the caller — this trait
    /// method reports the DLL's raw answer.
    fn result_extension(&mut self, index: u32) -> Option<String>;
}

/// Ports `TridLib_Load`'s own success check (UniExtract.au3:944-957),
/// minus the reentrant `If $hTridDll Then Return True` cache guard (see
/// module doc comment): `$aReturn[0] < 1` maps to `false`.
pub fn tridlib_load<L: TridLibrary>(lib: &mut L, bindir: &str) -> bool {
    lib.load_defs_pack(bindir).is_some_and(|count| count >= 1)
}

/// Ports `TridLib_Analyse` (UniExtract.au3:960-969), minus the reentrant
/// `TridLib_Load()` cache check (see module doc comment): submits `file`,
/// runs the analysis, and returns the result count.
pub fn tridlib_analyse<L: TridLibrary>(lib: &mut L, file: &str) -> i64 {
    lib.submit_file(file);
    lib.analyze();
    lib.result_count()
}

/// Ports `TridLib_Analyse_Simple`'s own result-collecting loop
/// (UniExtract.au3:972-987): calls [`tridlib_analyse`], then reads back
/// every result's type string, joined the same way `_ArrayToString(...,
/// @CRLF, -1, -1, "|")` does — `@CRLF` between entries, `|` is
/// `_ArrayToString`'s 2D-row delimiter (never reached here, since each
/// entry is a plain string, not a sub-array) and so never actually
/// appears in the output. Returns `""` when `result_count()` comes back
/// less than `1`, matching the source's own early return.
pub fn tridlib_analyse_simple<L: TridLibrary>(lib: &mut L, file: &str) -> String {
    let count = tridlib_analyse(lib, file);
    if count < 1 {
        return String::new();
    }
    (1..=count)
        .filter_map(|i| lib.result_type(i as u32))
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// The `MediaInfo.dll` exports `FileScan_MediaInfo` calls
/// (UniExtract.au3:1060-1067): open the file, read its formatted info
/// text, then close it.
pub trait MediaInfoLibrary {
    /// `MediaInfo_New()` + `MediaInfo_Open($hMI[0], $file)`
    /// (UniExtract.au3:1062,1064) — `None` if the DLL itself failed to
    /// load (`$hDll == -1`, UniExtract.au3:1061, checked by the caller
    /// before this trait is even reached) or `MediaInfo_New` returned a
    /// null instance.
    fn open(&mut self, file: &str) -> Option<MediaInfoHandle>;
    /// `DllCall($hDll, "wstr", "MediaInfo_Inform", "ptr", $hMI[0], "int",
    /// 0)` (UniExtract.au3:1065).
    fn inform(&mut self, handle: MediaInfoHandle) -> String;
    /// `MediaInfo_Delete($hMI[0])` (UniExtract.au3:1067).
    fn close(&mut self, handle: MediaInfoHandle);
}

/// Ports `FileScan_MediaInfo`'s DLL sequence (UniExtract.au3:1062-1067):
/// open `file`, read its info text, close. `None` if [`MediaInfoLibrary::
/// open`] failed. The result feeds directly into
/// `detection::mediainfo_scan::format_media_info` — not duplicated here.
pub fn scan_media_info<L: MediaInfoLibrary>(lib: &mut L, file: &str) -> Option<String> {
    let handle = lib.open(file)?;
    let info = lib.inform(handle);
    lib.close(handle);
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{FakeMediaInfoLibrary, FakeTridLibrary};

    #[test]
    fn load_succeeds_when_defs_pack_count_is_at_least_one() {
        let mut lib = FakeTridLibrary::new();
        lib.script_load_defs_pack(150);
        assert!(tridlib_load(&mut lib, "C:\\bin"));
    }

    #[test]
    fn load_fails_when_dll_open_or_call_errors() {
        let mut lib = FakeTridLibrary::new();
        assert!(!tridlib_load(&mut lib, "C:\\bin"));
    }

    /// Parity test for capability C038: the source checks `$aReturn[0] <
    /// 1`, so a reported count of exactly `0` also fails.
    #[test]
    fn load_fails_when_defs_pack_count_is_zero() {
        let mut lib = FakeTridLibrary::new();
        lib.script_load_defs_pack(0);
        assert!(!tridlib_load(&mut lib, "C:\\bin"));
    }

    #[test]
    fn analyse_submits_then_analyzes_then_reads_count() {
        let mut lib = FakeTridLibrary::new();
        lib.script_result_count(3);

        let count = tridlib_analyse(&mut lib, r"C:\downloads\file.exe");

        assert_eq!(count, 3);
        assert_eq!(
            lib.submitted_files(),
            vec![r"C:\downloads\file.exe".to_string()]
        );
        assert_eq!(lib.analyze_call_count(), 1);
    }

    #[test]
    fn analyse_simple_joins_every_result_with_crlf() {
        let mut lib = FakeTridLibrary::new();
        lib.script_result_count(2);
        lib.script_result_type(1, "Win32 Executable");
        lib.script_result_type(2, "PE32 file");

        let joined = tridlib_analyse_simple(&mut lib, "file.exe");

        assert_eq!(joined, "Win32 Executable\r\nPE32 file");
    }

    #[test]
    fn analyse_simple_returns_empty_string_when_no_results() {
        let mut lib = FakeTridLibrary::new();
        lib.script_result_count(0);

        assert_eq!(tridlib_analyse_simple(&mut lib, "file.exe"), "");
    }

    /// Parity test for capability C038: a negative count (the source
    /// checks `$iResults < 1`, not `== 0`) also short-circuits to `""`.
    #[test]
    fn analyse_simple_treats_negative_count_the_same_as_zero() {
        let mut lib = FakeTridLibrary::new();
        lib.script_result_count(-1);

        assert_eq!(tridlib_analyse_simple(&mut lib, "file.exe"), "");
    }

    #[test]
    fn scan_media_info_opens_informs_and_closes() {
        let mut lib = FakeMediaInfoLibrary::new();
        lib.script_inform("Complete name : file.mkv\r\nFormat : Matroska");

        let result = scan_media_info(&mut lib, r"C:\downloads\file.mkv");

        assert_eq!(
            result,
            Some("Complete name : file.mkv\r\nFormat : Matroska".to_string())
        );
        assert!(lib.was_closed());
    }

    #[test]
    fn scan_media_info_returns_none_when_open_fails() {
        let mut lib = FakeMediaInfoLibrary::new();
        lib.script_open_failure();

        assert_eq!(scan_media_info(&mut lib, "file.mkv"), None);
    }
}
