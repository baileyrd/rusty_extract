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
- C114: Actual Installer inner-blob handling (`extract::actual`).
- C115: Advanced Installer self-extraction (`extract::ai`).
- C104 (partial): ffmpeg audio conversion, video-convert, and stream probe invocations (`extract::ffmpeg`); `$TYPE_VIDEO` per-stream extraction not yet ported, row stays REQUIRED.
- C169, C170: run-log write policy (`run_log::should_append_error_log`, `run_log::build_error_log_line`, `run_log::should_save_log`).
- C117: Netopsystems FEAD self-extraction (`extract::fead`).
- C173: batch continues past an ordinary per-item failure, stops only on
  `$STATUS_SILENT` (`batch::should_continue_batch`).
- C171: generic success/failure fallback heuristic — output-directory
  size/mtime comparison used when an extractor case never explicitly
  reports success (`result_heuristic::resolve_unknown_result`).
- C153: scan-only full-detail output — concatenates every scanner's
  result into one report, with or without centered dashed headers
  (`filetype_report::format_filetype_results`).
- C154: scan-only silent-mode file output — the file-scan log entry
  format appended per item in a batch run
  (`filetype_report::build_scan_log_entry`).
- C179 (partial): free-space check arithmetic and silent-mode
  termination decision (`free_space::measure_free_space`,
  `decide_free_space_outcome`); interactive abort/retry/ignore prompt
  not yet ported, row stays REQUIRED.
- C165: per-run log file naming — `<logdir>YYYY-MM-DD_HH-MM-SS_[STATUS_]<name>.<ext>.log`,
  status omitted on success (`run_log::build_log_file_name`).
- C162: generic password-failure detection via output-text matching,
  including the `_StringGetLine($sLog, -1)` off-by-one fallback quirk
  (`log_eval::is_password_failure`).
- C167: output-log evaluation (`log_eval::evaluate_log`, full
  `EvaluateLog()` `ElseIf` chain) and warning extraction
  (`log_eval::parse_warnings`, `ParseWarnings()`).
- C164: debug-line format — `<datetime>:<msec>\t<msg>\r\n`
  (`run_log::build_debug_line`).
- C155 (partial): post-extraction cleanup mode gating, action
  selection, path resolution, and wildcard-target classification
  (`cleanup` module); wildcard expansion and the real filesystem calls
  not yet ported, row stays REQUIRED.
- C092: UnRAR extractor integration (`extract::rar`).
- C084: lessmsi extractor integration (`extract::lessmsi`).
- C085: jsMSIx extractor integration (`extract::jsmsix`).
- C086: MsiX extractor integration, shared across `$TYPE_MSI`/`$TYPE_MSM`/`$TYPE_MSP`
  (`extract::msix`).
- C087: msiexec administrative-install fallback (`extract::msiexec`).
- C160: archive password-list trial decision policy — probe/protected
  detection and password-loop first-match search, used by 7z, DGCA, and
  RAR extraction (`password_search`).
- C148: batch-item-per-process spawning — pops the batch queue and
  relaunches the current executable with the next item's arguments,
  chaining driven entirely by each process's own exit (`batch_runner`).
- C159: unicode-relocation reversion decision — given the relocation
  mode a run left behind, decides whether to move the working copy
  back, recycle it, or do nothing (`unicode_relocation`).
- C156: unconditional temp-outdir cleanup decision — a still-present
  per-run temp output directory is always removed, success or failure
  (`outdir::should_remove_temp_outdir`).
- C100: ttarchext extractor integration (`extract::ttarch`).
- C105: Visionaire Engine v3 two-pass extraction invocations
  (`extract::visionaire3`).
- C073: helpdeco extractor integration, RTF reconstruction pass
  (`extract::helpdeco`).
- C076: IsXunpack extractor integration (`extract::isxunpack`).
- C091: RAIU extractor integration (`extract::raiu`).
- C074: innounp/innoextract primary/fallback pair (`extract::inno`).
- C001: positional file argument resolution and existence validation
  (`file_arg::resolve_file_argument_path`, `file_arg::validate_file_argument`).
- C002, C003: destination-argument routing and scan-only mode
  (`dest_arg::parse_destination_argument`).
- C006: `/type[=value]` override routing
  (`type_override::parse_type_override`).
- C011: `/batch` flag detection (`cli::has_batch_flag`).
- C014, C015: directory-input and second-instance entry gates
  (`entry_gate::directory_input_gate`, `entry_gate::second_instance_gate`).
- C017: `language` preference resolution (`prefs::resolve_language`).
- C090: PeaZip extractor integration, added directly to `extract::table`.
- C161: ACE excluded from the password-list trial, made explicit
  (`password_search::PASSWORD_TRIAL_EXTRACTOR_TYPES`).
- C143: no centralized overwrite policy, verified across all 48
  `extract::table` formats in one sweep.
- C098: swfextract extractor integration, added directly to
  `extract::table`.
- C152: scan-only-mode short-circuit (`entry_gate::scan_only_gate`).
- C036: third-party detector tool silencing, PEiD/Exeinfo PE registry
  backup/restore decision logic (`detector_silence::restore_plan`).
- C099: ThinApp/Thinstall extractor integration (`extract::thinapp`).
- C172: undifferentiated failure messaging, pinned down as a documented
  quirk (`failure_message::FAILURE_MESSAGE_KEY`).
- C053: manual disambiguation selection policy and the five real
  candidate lists it dispatches (`method_select`).
- C075: InstallShield CAB fallback chain — unshield with `-O` retry,
  then a C053-disambiguated choice of is6comp/is5comp/iscab
  (`extract::iscab`).
- C150: verified is6comp's blocking-`RunWait`-with-no-crash-guard batch
  risk is still present, structurally inherent to
  `extract::runner::CommandExtractorRunner`.
- C155: completed post-extraction cleanup's wildcard-expansion decision
  logic (`cleanup::split_wildcard_target`), closing the module's last
  documented gap.
- C054/C181 (partial): recursive-dispatch completion contract
  (`extract::completion::resolve_completion`), completing the recursion
  piece for `extract::actual`/`extract::forge`/`extract::raiu`.
- C054 (partial): `$TYPE_ZIP`'s recursive dispatch (`extract::zip`),
  finding its fallback path runs unconditionally whenever reached.
- C054 (partial): `$TYPE_MSCF`'s recursive dispatch and cab-extraction
  invocation (`extract::mscf`); its own `RipExeInfo` fallback stays
  GUI-blocked, same as C069.
- C054: completed `$TYPE_UNITYPACKAGE`'s recursive dispatch
  (`extract::unity`) — all 6 cited call sites now covered, capability
  marked `DONE`.
- C181: `$TYPE_CTAR`'s same-tool nested-archive loop (`extract::ctar`)
  — a genuinely different mechanism from `extract()` recursion, capability
  marked `DONE`.
### Changed
- Collapsed 43 single-invocation extractor modules (`extract::ace`,
  `extract::kgb`, `extract::rar`, etc.) into one data-driven table,
  `extract::table` — same `Invocation` output for the same inputs, no
  behavior change. Module paths named in earlier entries above for the
  affected capabilities (C057, C058, C062, C063, C065, C067, C068, C070,
  C072, C076, C078, C079-C088, C092-C097, C100, C102, C103, C107-C113,
  C115-C118, C120, C146) now live in `extract::table` instead (PR
  [#360](https://github.com/baileyrd/rusty_extract/pull/360)).
- Collapsed the 18 `def/*.ini`-only wrapper modules (`extract::alz`,
  `extract::arc`, `extract::adf`, etc. — capabilities C059, C060,
  C122-C137) into one table-driven regression test,
  `extract::plugin_defs_test`; none had production callers of their own
  (PR [#361](https://github.com/baileyrd/rusty_extract/pull/361)).
- Folded `extract::freearc` (C071) and `extract::uharc` (C101), missed in
  the first collapse, into `extract::table` (PR
  [#362](https://github.com/baileyrd/rusty_extract/pull/362)).
- `capability-manifest.md`: updated the **Evidence** test-path citation for
  all 63 capabilities affected by the three consolidations above.
### Fixed
- CI now runs on `windows-latest` (was `ubuntu-latest`) and triggers on pushes
  to this repo's actual default branch — this is a Windows-only parity port
  that needs a Windows runner to build/test at all.
- `IniFile::parse` reports skipped/malformed lines instead of silently
  dropping them (unix-philosophy audit finding F1).
- Retracted PR #365's claim of having "verified against the live source"
  and corrected C002/C003's AutoIt line citations — that verification
  trusted `WebFetch`'s line-number reporting on a large file, which
  turned out not to be reliable. Reverted to the original citations; the
  ported behavior itself was unaffected.
### Security

<!-- ## [0.1.0] - YYYY-MM-DD
### Added
- Initial release -->
