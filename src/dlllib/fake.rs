//! Test doubles for [`super::TridLibrary`]/[`super::MediaInfoLibrary`]:
//! record every call and return caller-scripted results, the same role
//! `automation::fake::FakeGuiAutomation` plays for `GuiAutomation`.

use std::collections::HashMap;

use super::{MediaInfoHandle, MediaInfoLibrary, TridLibrary};

#[derive(Default)]
pub struct FakeTridLibrary {
    load_defs_pack_result: Option<u32>,
    submitted_files: Vec<String>,
    analyze_calls: u32,
    result_count: i64,
    result_types: HashMap<u32, String>,
    result_extensions: HashMap<u32, String>,
}

impl FakeTridLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scripts `load_defs_pack`'s result. Unscripted defaults to `None`
    /// (`@error`).
    pub fn script_load_defs_pack(&mut self, count: u32) {
        self.load_defs_pack_result = Some(count);
    }

    /// Scripts `result_count`'s return value.
    pub fn script_result_count(&mut self, count: i64) {
        self.result_count = count;
    }

    /// Scripts `result_type`'s return value for `index`.
    pub fn script_result_type(&mut self, index: u32, text: &str) {
        self.result_types.insert(index, text.to_string());
    }

    /// Scripts `result_extension`'s return value for `index`.
    pub fn script_result_extension(&mut self, index: u32, text: &str) {
        self.result_extensions.insert(index, text.to_string());
    }

    pub fn submitted_files(&self) -> Vec<String> {
        self.submitted_files.clone()
    }

    pub fn analyze_call_count(&self) -> u32 {
        self.analyze_calls
    }
}

impl TridLibrary for FakeTridLibrary {
    fn load_defs_pack(&mut self, _bindir: &str) -> Option<u32> {
        self.load_defs_pack_result
    }

    fn submit_file(&mut self, file: &str) {
        self.submitted_files.push(file.to_string());
    }

    fn analyze(&mut self) {
        self.analyze_calls += 1;
    }

    fn result_count(&mut self) -> i64 {
        self.result_count
    }

    fn result_type(&mut self, index: u32) -> Option<String> {
        self.result_types.get(&index).cloned()
    }

    fn result_extension(&mut self, index: u32) -> Option<String> {
        self.result_extensions.get(&index).cloned()
    }
}

#[derive(Default)]
pub struct FakeMediaInfoLibrary {
    open_fails: bool,
    inform_text: String,
    closed: Vec<MediaInfoHandle>,
    next_handle: u64,
}

impl FakeMediaInfoLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scripts `open` to return `None`, matching a failed `MediaInfo_New`.
    pub fn script_open_failure(&mut self) {
        self.open_fails = true;
    }

    /// Scripts `inform`'s return value for every handle.
    pub fn script_inform(&mut self, text: &str) {
        self.inform_text = text.to_string();
    }

    pub fn was_closed(&self) -> bool {
        !self.closed.is_empty()
    }
}

impl MediaInfoLibrary for FakeMediaInfoLibrary {
    fn open(&mut self, _file: &str) -> Option<MediaInfoHandle> {
        if self.open_fails {
            return None;
        }
        let handle = MediaInfoHandle(self.next_handle);
        self.next_handle += 1;
        Some(handle)
    }

    fn inform(&mut self, _handle: MediaInfoHandle) -> String {
        self.inform_text.clone()
    }

    fn close(&mut self, handle: MediaInfoHandle) {
        self.closed.push(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unscripted_load_defs_pack_is_none() {
        let mut lib = FakeTridLibrary::new();
        assert_eq!(lib.load_defs_pack("C:\\bin"), None);
    }

    #[test]
    fn unscripted_result_type_is_none() {
        let mut lib = FakeTridLibrary::new();
        assert_eq!(lib.result_type(1), None);
    }

    #[test]
    fn open_succeeds_by_default_with_distinct_handles() {
        let mut lib = FakeMediaInfoLibrary::new();
        let a = lib.open("a.mkv").unwrap();
        let b = lib.open("b.mkv").unwrap();
        assert_ne!(a, b);
    }
}
