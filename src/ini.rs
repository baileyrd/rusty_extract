//! Minimal parser for the `def/*.ini` family of files UniExtract2 ships
//! (`registry.ini`'s `[Trid]`/`[File]`/`[Exeinfo]`/`[Extensions]` sections,
//! and the per-plugin `[Plugin]` definition files). Deliberately hand-rolled
//! rather than a crate dependency: the format is a small, fixed subset of
//! INI (`;` comments, `[Section]` headers, `key=value` lines, no nesting,
//! no interpolation) and every consumer needs section order and duplicate
//! keys preserved, which most INI crates collapse into a map.

/// One `key=value` line, in file order. Duplicate keys within a section are
/// kept as separate entries (`def/registry.ini`'s `[Trid]` section relies on
/// this — e.g. `lbr` appears four times with different values).
pub type Entries = Vec<(String, String)>;

/// A parsed ini file: an ordered list of `(section name, entries)`.
#[derive(Debug, Default, Clone)]
pub struct IniFile {
    sections: Vec<(String, Entries)>,
}

impl IniFile {
    /// Parses `text`. Lines starting with `;` (after leading whitespace) are
    /// comments; blank lines are ignored; a line not matching `[Section]` or
    /// `key=value` is skipped rather than treated as an error, matching
    /// AutoIt's own `IniReadSection` tolerance for stray lines.
    pub fn parse(text: &str) -> Self {
        let mut sections: Vec<(String, Entries)> = Vec::new();
        let mut current: Option<Entries> = None;
        let mut current_name = String::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(entries) = current.take() {
                    sections.push((std::mem::take(&mut current_name), entries));
                }
                current_name = name.trim().to_string();
                current = Some(Vec::new());
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                if let Some(entries) = current.as_mut() {
                    entries.push((key.trim().to_string(), value.trim().to_string()));
                }
            }
        }
        if let Some(entries) = current {
            sections.push((current_name, entries));
        }

        IniFile { sections }
    }

    /// Returns a section's entries in file order, or `None` if the section
    /// doesn't exist. Section-name lookup is case-insensitive to match
    /// AutoIt's `IniReadSection`.
    pub fn section(&self, name: &str) -> Option<&[(String, String)]> {
        self.sections
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, entries)| entries.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_in_order_with_duplicate_keys() {
        let ini = IniFile::parse(
            "; comment\n[Trid]\nlbr=CrLZH compressed\nlbr=Crunch compressed archive\n\n[Extensions]\narc=arc\n",
        );
        let trid = ini.section("Trid").expect("Trid section present");
        assert_eq!(
            trid,
            &[
                ("lbr".to_string(), "CrLZH compressed".to_string()),
                ("lbr".to_string(), "Crunch compressed archive".to_string()),
            ]
        );
        let ext = ini
            .section("Extensions")
            .expect("Extensions section present");
        assert_eq!(ext, &[("arc".to_string(), "arc".to_string())]);
    }

    #[test]
    fn section_lookup_is_case_insensitive_and_missing_returns_none() {
        let ini = IniFile::parse("[Extensions]\nuu=uu\n");
        assert!(ini.section("extensions").is_some());
        assert!(ini.section("Nope").is_none());
    }
}
