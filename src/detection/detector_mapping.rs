//! `def/registry.ini` detector-to-plugin mapping: the `[Trid]`/`[File]`/
//! `[Exeinfo]` sections, bridging a detector's textual output to a plugin
//! `.ini` stem for the `def/*.ini` fallback (C050).

use crate::detection::registry::DEFAULT_REGISTRY_INI;
use crate::ini::IniFile;

/// Parsed `[Trid]`/`[File]`/`[Exeinfo]` sections from `def/registry.ini`.
pub struct DetectorMapping {
    trid: Vec<(String, String)>,
    file: Vec<(String, String)>,
    exeinfo: Vec<(String, String)>,
}

impl DetectorMapping {
    /// Parses `ini_text`'s `[Trid]`/`[File]`/`[Exeinfo]` sections. A missing
    /// section is treated as empty (matches every future lookup against it
    /// finding nothing), not an error — the source's own `UserDefCompare`
    /// (UniExtract.au3:1804-1819) tolerates a load failure the same way,
    /// logging and continuing rather than stopping detection.
    pub fn parse(ini_text: &str) -> Self {
        let (ini, _skipped) = IniFile::parse(ini_text);
        let section_owned = |name: &str| ini.section(name).unwrap_or(&[]).to_vec();
        DetectorMapping {
            trid: section_owned("Trid"),
            file: section_owned("File"),
            exeinfo: section_owned("Exeinfo"),
        }
    }

    /// The mapping built from the bundled `def/registry.ini`.
    pub fn default_mapping() -> Self {
        Self::parse(DEFAULT_REGISTRY_INI)
    }

    /// Resolves TrID's output text to the plugin stem it matches — see
    /// [`resolve`] for the matching rule.
    pub fn resolve_trid(&self, detector_output: &str) -> Option<&str> {
        resolve(&self.trid, detector_output)
    }

    /// Resolves the Unix `file` tool's output text to the plugin stem it
    /// matches — see [`resolve`] for the matching rule.
    pub fn resolve_file(&self, detector_output: &str) -> Option<&str> {
        resolve(&self.file, detector_output)
    }

    /// Resolves Exeinfo PE's output text to the plugin stem it matches —
    /// see [`resolve`] for the matching rule.
    pub fn resolve_exeinfo(&self, detector_output: &str) -> Option<&str> {
        resolve(&self.exeinfo, detector_output)
    }
}

/// Ports `UserDefCompare`'s matching rule (UniExtract.au3:1815-1817):
/// substring search — a row matches when `detector_output` *contains* the
/// row's value — first match in file order wins.
///
/// The source's own loop has no early exit after a match: it keeps
/// scanning every remaining row even after calling `extract()` for one.
/// That's not a second dispatch in practice, though — `extract()` always
/// ends in `terminate()`, which exits the process, so no later iteration in
/// the same call ever actually runs. Returning only the first match here is
/// the faithful port of what's externally observable, not a simplification
/// of the source's loop structure.
fn resolve<'a>(section: &'a [(String, String)], detector_output: &str) -> Option<&'a str> {
    section
        .iter()
        .find(|(_, value)| detector_output.contains(value.as_str()))
        .map(|(key, _)| key.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C051: resolves real `[Trid]` entries from
    /// the bundled `def/registry.ini` the way `UserDefCompare` would,
    /// including a detector output that's a superset of the matched value
    /// (the source uses `StringInStr`, a substring search, not equality).
    #[test]
    fn resolves_trid_output_by_substring_match() {
        let mapping = DetectorMapping::default_mapping();
        assert_eq!(
            mapping.resolve_trid("100.0% (.ADF) Amiga Disk image File (5000/1)"),
            Some("adf")
        );
        assert_eq!(mapping.resolve_trid("Godot Engine package"), Some("godot"));
    }

    #[test]
    fn resolves_file_tool_output_by_substring_match() {
        let mapping = DetectorMapping::default_mapping();
        assert_eq!(
            mapping.resolve_file("Amiga DOS disk, boot block for OFS filesystem"),
            Some("adf")
        );
    }

    #[test]
    fn resolves_exeinfo_output_by_substring_match() {
        let mapping = DetectorMapping::default_mapping();
        assert_eq!(
            mapping.resolve_exeinfo("BitRock InstallBuilder installer detected"),
            Some("bitrock")
        );
    }

    #[test]
    fn unmatched_output_returns_none() {
        let mapping = DetectorMapping::default_mapping();
        assert_eq!(
            mapping.resolve_trid("Some completely unrelated file type"),
            None
        );
    }

    #[test]
    fn first_matching_row_in_file_order_wins_when_multiple_rows_match() {
        // Synthetic data, since the bundled registry.ini's near-duplicate
        // rows (e.g. [Trid]'s four "lbr" entries) all share the same key,
        // which can't distinguish "first row wins" from "any row wins".
        // Here two DIFFERENT keys could both match the same output; the
        // earlier row in file order must win, per UserDefCompare's plain
        // top-to-bottom For loop.
        let mapping = DetectorMapping::parse("[Trid]\nfoo=Archive\nbar=Archive data\n");
        assert_eq!(mapping.resolve_trid("Archive data, version 2"), Some("foo"));
    }

    #[test]
    fn missing_section_resolves_to_none_rather_than_erroring() {
        let mapping = DetectorMapping::parse("[Extensions]\nuu=uu\n");
        assert_eq!(mapping.resolve_trid("anything"), None);
        assert_eq!(mapping.resolve_file("anything"), None);
        assert_eq!(mapping.resolve_exeinfo("anything"), None);
    }
}
