//! Extension-based fallback dispatch: `def/registry.ini`'s `[Extensions]`
//! section, consulted last, after every signature-based detector has failed
//! to identify a file.

use crate::ini::IniFile;
use std::fmt;

/// The bundled `def/registry.ini`, ported verbatim from the source repo.
pub const DEFAULT_REGISTRY_INI: &str = include_str!("../../def/registry.ini");

#[derive(Debug)]
pub struct MissingSectionError(String);

impl fmt::Display for MissingSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ini file has no [{}] section", self.0)
    }
}

impl std::error::Error for MissingSectionError {}

/// Extension → extractor-type-stem lookup table, built from `[Extensions]`.
///
/// A `[Extensions]` line's key may list several extensions separated by
/// commas, all mapping to the same extractor stem (the source's `CheckExt`
/// splits the raw key on `,` before comparing) — `entries` keeps each
/// `(extensions, stem)` pair as parsed, in file order, so the first matching
/// line wins on lookup, same as the source.
pub struct ExtensionRegistry {
    entries: Vec<(Vec<String>, String)>,
}

impl ExtensionRegistry {
    /// Parses `ini_text` and indexes its `[Extensions]` section.
    ///
    /// [`IniFile::parse`] also reports lines it couldn't parse as
    /// `[Section]`/`key=value`; this constructor discards that list since
    /// its only caller so far ([`Self::default_registry`]) parses the
    /// bundled, ported-verbatim `def/registry.ini`, which has none. A future
    /// caller loading user-editable `def/*.ini` plugin files should call
    /// [`IniFile::parse`] directly and inspect the skipped lines itself
    /// rather than going through this discarding wrapper.
    pub fn parse(ini_text: &str) -> Result<Self, MissingSectionError> {
        let (ini, _skipped) = IniFile::parse(ini_text);
        let section = ini
            .section("Extensions")
            .ok_or_else(|| MissingSectionError("Extensions".to_string()))?;

        let entries = section
            .iter()
            .map(|(key, value)| {
                let extensions = key.split(',').map(|s| s.trim().to_string()).collect();
                (extensions, value.clone())
            })
            .collect();

        Ok(ExtensionRegistry { entries })
    }

    /// The registry built from the bundled `def/registry.ini`. Fails only if
    /// the bundled file itself is malformed — a build-time asset, not
    /// something a caller can hit in practice, but surfaced as `Result`
    /// rather than panicking so this stays true even if the file changes.
    pub fn default_registry() -> Result<Self, MissingSectionError> {
        Self::parse(DEFAULT_REGISTRY_INI)
    }

    /// Looks up the extractor-type stem for a file extension, matching the
    /// source's `CheckExt`: comparison is case-insensitive (the source
    /// lowercases the extension before this point, so this mirrors that
    /// rather than depending on the caller having already done so), and
    /// `ext` is expected without a leading `.`.
    pub fn lookup_by_extension(&self, ext: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(extensions, _)| extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
            .map(|(_, stem)| stem.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C047: every `[Extensions]` mapping in the
    /// bundled `def/registry.ini` (ported verbatim from UniExtract2)
    /// resolves through `lookup_by_extension` to the same extractor stem
    /// `CheckExt` (UniExtract.au3:2174-2190) would select.
    #[test]
    fn resolves_every_extension_in_the_bundled_registry() {
        let registry = ExtensionRegistry::default_registry().unwrap();
        let expected = [
            ("arc", "arc"),
            ("ba2", "bsa"),
            ("bsa", "bsa"),
            ("fsb", "fsb"),
            ("lit", "lit"),
            ("mo", "mo"),
            ("msi", "msi"),
            ("pex", "pex"),
            ("qm", "qm"),
            ("rpgmvp", "rpgmvp"),
            ("sgbpack", "sgb"),
            ("sit", "sit"),
            ("sitx", "sit"),
            ("ttarch", "ttarch"),
            ("ttarch2", "ttarch"),
            ("utage", "utage"),
            ("uu", "uu"),
            ("uue", "uu"),
            ("wolf", "wolf"),
            ("xx", "uu"),
            ("xxe", "uu"),
        ];
        for (ext, stem) in expected {
            assert_eq!(
                registry.lookup_by_extension(ext),
                Some(stem),
                "extension {ext:?} should resolve to {stem:?}"
            );
        }
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let registry = ExtensionRegistry::default_registry().unwrap();
        assert_eq!(registry.lookup_by_extension("BSA"), Some("bsa"));
        assert_eq!(registry.lookup_by_extension("Sitx"), Some("sit"));
    }

    #[test]
    fn unknown_extension_returns_none() {
        let registry = ExtensionRegistry::default_registry().unwrap();
        assert_eq!(registry.lookup_by_extension("zip"), None);
        assert_eq!(registry.lookup_by_extension(""), None);
    }

    #[test]
    fn comma_separated_key_maps_every_listed_extension() {
        let registry = ExtensionRegistry::parse("[Extensions]\nfoo,bar = baz\n").unwrap();
        assert_eq!(registry.lookup_by_extension("foo"), Some("baz"));
        assert_eq!(registry.lookup_by_extension("bar"), Some("baz"));
    }

    #[test]
    fn missing_extensions_section_is_an_error() {
        assert!(ExtensionRegistry::parse("[Trid]\nfoo=bar\n").is_err());
    }
}
