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
- C146: DAA→ISO conversion invocation, no-existing-file-check quirk preserved (`extract::daa`).
- C058: AspackDie invocation for packed-executable unpack (`extract::aspack`).
- C052: `def/*.ini` plugin definition schema (`extract::plugin_config::PluginConfig`).
- C182: plugin extension point placeholder substitution (`extract::placeholder::replace_placeholders`).
- C060, C122, C123, C124, C125: `def/*.ini`-only extractor integrations (`extract::arc`, `extract::adf`, `extract::bitrock`, `extract::bsa`, `extract::godot`).
- C059: unalz (ALZip) probe + extractor integration (`detection::alz_probe`, `extract::alz`).
- C126, C127, C128, C129, C130, C131, C132, C133, C134, C135, C136, C137: `def/*.ini`-only extractor integrations (`extract::lbr`, `extract::lit`, `extract::mo`, `extract::pex`, `extract::qm`, `extract::rpgmvp`, `extract::sgb`, `extract::sim`, `extract::sit`, `extract::spoon`, `extract::utage`, `extract::uu`).
- C016: process exit code contract (`status::exit_code`).
- C026: `Timeout` preference resolution, including the preserved missing-key unit-mismatch quirk (`prefs::resolve_timeout_ms`).
- C024, C158: `deletesourcefile` preference and its deletion-on-success policy (`prefs::DeleteSourceFileOption`, `prefs::parse_delete_source_file_option`, `prefs::should_delete_source_file`).
- C033: `cleanup` preference (`prefs::parse_cleanup_option`).
- C035: password list file path resolution (`prefs::password_list_path`).
- C020, C022, C023, C025, C027, C028, C029, C030, C031, C032: simple boolean preference defaults (`prefs::resolve_bool_pref` and one `..._DEFAULT` constant per preference).
- C018, C019: `batchqueue`/`filescanlogfile` path-override preferences (`prefs::resolve_batchqueue_path`, `prefs::resolve_filescanlogfile_path`).
- C034: `BatchRecurse` preference default (`prefs::BATCHRECURSE_DEFAULT`).
- C021: `history` preference move-to-front/dedupe/cap-at-10 semantics (`prefs::push_history`).
- C007, C008, C009, C010, C012, C013: command-line flag detection, case-insensitive as AutoIt's `_ArraySearch`/`=` default to (`cli::has_silent_flag`, `cli::has_nolog_flag`, `cli::has_nostats_flag`, `cli::is_help_flag`, `cli::is_batchclear_flag`, `cli::has_close_flag`).
- C004, C005, C139, C140: output-directory `/sub`/`/last` token and relative/trailing-slash path resolution (`outdir::resolve_output_directory`, `outdir::get_last_outdir`).
- C140 (continued): `extract()`'s trailing-backslash strip/reappend cycle (`outdir::strip_trailing_backslash_for_extraction`, `outdir::reappend_trailing_backslash_after_extraction`).
- C141: drive-root output directory ambiguity, reproducing the documented "Extracting to C:/" bug (test only, using the existing C140 strip function).
- C142: output-directory creation and validation decision tree (`outdir::OutdirOutcome`, `outdir::decide_outdir_outcome`).
- C157: empty created-output-directory cleanup on failure (`outdir::should_remove_empty_created_outdir`).
- C138: output-subfolder default resolution for `/sub` (`outdir::default_output_subfolder`).
- C144: overwrite message treated as extraction success in output-log evaluation (`log_eval::is_overwrite_success_message`).
- C145: overwrite/password/no-space/new-filename prompt live-detection (`log_eval::needs_manual_input`).
- C147: batch queue file format and duplicate/multipart-archive handling (`batch::build_command_line`, `batch::should_add_to_batch`, `batch::is_multipart_archive_already_queued`).
- C148: batch queue FIFO pop mechanics (`batch::pop_batch_queue`).
- Composition root: `ExtractorRunner` port (`extract::runner::CommandExtractorRunner`,
  `extract::runner::FakeExtractorRunner`) and a real `main.rs` wiring the
  `rgss`/`ace` extractors end-to-end (output-directory resolution, dispatch,
  run, log evaluation, exit code) — the first working slice of the
  composition root `ARCHITECTURE.md` describes.
- C097: SQLite database dump extractor integration (`extract::sqlite`).
- C088: NBHextract extractor integration (`extract::nbh`).
- C101: UHARC 3-version fallback chain (`extract::uharc`).
- C061: ARJ SFX verification probe (`detection::arj_probe`).
- C063: bootimg extractor integration (`extract::bootimg`).
- C064: Windows `expand.exe` CAB/MSU invocations (`extract::expand`).
- C116: Excelsior Installer self-extraction (`extract::ei`).
- C118: SuperDAT Updater self-extraction (`extract::superdat`).
- C066: ci-extractor integration (`extract::ci`).
- C119: InstallForge 7z.exe wrapper + base64 path-rename logic (`extract::forge`).
- C120: MSCF Cab installer 7z.exe wrapper (`extract::mscf`).
- C121: Unity `.unitypackage` decoder 7z.exe wrapper + path-remapping (`extract::unity`).
- C113: arc_conv integration (`extract::arc_conv`).
- C117: Netopsystems FEAD self-extraction (`extract::fead`).
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
