# Changelog

All notable changes to this repo are documented here.
Format: Added / Changed / Deprecated / Removed / Fixed / Security, newest first.

## [Unreleased]
### Added
- Repo governance scaffold (README, ARCHITECTURE, CONTRIBUTING, CODE_OF_CONDUCT,
  SECURITY, RELEASE_NOTES, ADR seed, PR/issue templates, Rust CI) via `repo-config`.
- Minimal Cargo skeleton (single crate: `src/lib.rs` + `src/main.rs`).
- `capability-manifest.md`: full step-1 inventory of UniExtract2's core
  detection/extraction-engine surface (194 rows).
- C047: extension-based fallback dispatch (`ExtensionRegistry`, ported
  `def/registry.ini`, a minimal `IniFile` parser).
- C093: RGSS Decryptor extractor integration (`src/extract` module,
  `Invocation`/`WindowMode`, `extract::rgss`).
- C048: blind 7-Zip probe fallback (`detection::sevenzip_probe`).
- C094: unrpa extractor integration (`extract::rpa`).
- C049: central extractor dispatcher (`extract::dispatch`).
- C095: sfarkxtc extractor integration (`extract::sfark`).
- C050: Case-Else plugin-ini resolution (`extract::plugin`).
- C096: extsis extractor integration (`extract::extsis`).
- C051: detector-to-plugin mapping (`detection::detector_mapping`).
- C081: lzop extractor integration (`extract::lzop`).
- C080: lzip extractor integration (`extract::lzip`).
- C079: KGB Archiver extractor integration (`extract::kgb`).
- C071: FreeArc extractor integration (`extract::freearc`).
- C062: BCM extractor integration (`extract::bcm`).
- C065: chdman extractor integration (`extract::chdman`).
- C102: uif2iso extractor integration (`extract::uif`).
- C072: FSB extractor integration (`extract::fsb`).
- C078: unisz extractor integration (`extract::isz`).
- C067: cicdec extractor integration (`extract::cic`).
- C070: xor invocation for Ghost Installer overlay decode (`extract::xor`).
- C057: acefile extractor integration (`extract::ace`).
- C068: GARbro extractor integration (`extract::garbro`).
- C112: upx invocation for packed-executable unpack (`extract::upx`).
- C109: Info-ZIP UnZip fallback invocation (`extract::unzip`).
- C110: unzoo extractor integration (`extract::zoo`).
- C111: zpaq extractor integration (`extract::zpaq`).
- C089: Xpdf tools extractor integration (`extract::pdf`).
- C082: unlzx extractor integration (`extract::lzx`).
- C083: demoleition / MoleBox extractor integration (`extract::mole`).
- C108: WolfDec extractor integration (`extract::wolf`).
- C103: umodel extractor integration (`extract::unreal`).
- C107: dark / WiX Toolset extractor integration (`extract::wix`).
- C058: AspackDie invocation for packed-executable unpack (`extract::aspack`).
### Changed
### Fixed
- CI now runs on `windows-latest` (was `ubuntu-latest`) and triggers on pushes
  to this repo's actual default branch — this is a Windows-only parity port
  that needs a Windows runner to build/test at all.
- `IniFile::parse` reports skipped/malformed lines instead of silently
  dropping them (unix-philosophy audit finding F1).
### Security

<!-- ## [0.1.0] - YYYY-MM-DD
### Added
- Initial release -->
