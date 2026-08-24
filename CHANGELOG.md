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
- C056 (partial): 7-Zip integration (`extract::sevenzip`) — main
  extraction, error/password classification, and the full RPM/Debian/
  gzip-family post-extraction branch tree; SFX-splitter branch stays
  GUI-blocked, same as C069/C106.
- C077 (partial): QuickBMS + WCX plugin fan-out (`extract::qbms`) —
  InstallExplorer/ISO/TotalObserver probe-then-classify detectors and
  the shared extraction case; `BmsExtract`'s SQLite-backed `.bms`
  lookup stays unmodeled, same ambiguity as C055.
- C046: extension-based pre-check (`detection::initial_ext_check`,
  ports `InitialCheckExt`) — every routing target already had a home
  from earlier capabilities, so this is purely the pre-scan
  order/grouping decision; preserves the reversed `CheckIso()`/7-Zip
  probe call order between the two disk-image extension groups.
  Capability marked `DONE`.
- C037: top-level detection cascade order (`detection::cascade`, ports
  `StartExtraction()`'s step order after `InitialCheckExt`) — found
  that an exe/dll file in extract mode is delegated entirely to
  `IsExe()`, which never returns control in that mode, so none of
  `StartExtraction()`'s other steps ever run for it. Capability marked
  `DONE`.
- C043: Exeinfo PE match dispatch table (`detection::exeinfo_dispatch`,
  ~45 cases) — matched top to bottom exactly as the source orders it;
  every literal needle string verified present in the exact cited
  source range before writing tests. Capability marked `DONE`.
- C041: Unix `file` tool match dispatch table
  (`detection::file_dispatch`, ~25 cases plus its trailing not-packed/
  not-supported checks) — found that `"POSIX tar archive"` is
  unreachable in practice, shadowed by the earlier `"ar archive"`
  case (a genuine source quirk, preserved rather than fixed).
  Capability marked `DONE`.
- C039: TrID match dispatch table (`detection::trid_dispatch`, 92
  `Case` clauses, the largest of the three detector dispatch tables)
  — preserved the table's one case-sensitive comparison and two
  dead-code quirks (`"null bytes"` shadowed; the generic `Executable`
  case correctly never misroutes ELF binaries). Capability marked
  `DONE`.
- C040: Unix `file` tool secondary detector (`detection::unixfile_scan`)
  — output cleanup and post-scan branch; extract mode hands off
  entirely to `detection::file_dispatch::classify` (C041). Capability
  marked `DONE`.
- C044 (partial): PEiD match dispatch table (`detection::peid_dispatch`,
  20 cases) — the actual PEiD scan is real Win32 GUI automation, the
  same blocker already found for C069/C106/C056's SFX splitter; stays
  `REQUIRED`.
- C042 (partial): Exeinfo PE scan orchestration
  (`detection::exeinfo_scan`) — found the extract-mode scan is a plain
  command-line invocation, not GUI automation; the scan-only-mode GUI
  path and its corrupted-log retry stay unmodeled. Stays `REQUIRED`.
- C045 (partial): MediaInfo scan formatting
  (`detection::mediainfo_scan`) — found `StringSplit`'s missing
  `$STR_ENTIRESPLIT` flag treats `@CRLF` as a character set, roughly
  doubling the element count the not-a-media-file threshold checks
  against; reproduced exactly. The `MediaInfo.dll` calls themselves
  stay unmodeled (missing-FFI blocker, same as C038). Stays
  `REQUIRED`.
- C038 (partial): TrID scan orchestration (`detection::trid_scan`) —
  found extract mode is the DLL-blocked path here while scan-only
  mode is the portable command-line path, the reverse of C042's
  split. `TridLib_Analyse`/`TridLib_GetType` and `FetchStdout` itself
  stay unmodeled. Stays `REQUIRED`.
- C175/C176: non-ASCII and UNC-path input relocation
  (`unicode_relocation::plan_relocation`) — verified `$sRegExAscii`
  precisely (whitelists 20 accented Western-European letters, not
  just ASCII) and preserved a real interaction between the two
  capabilities: UNC relocation only applies when the unicode check
  didn't already compute a destination. Both capabilities marked
  `DONE`.
- C177: unicode-move bookkeeping loss on nested re-entry
  (`unicode_relocation::start_extraction_reentry_resets_unicode_mode`)
  — verified still present: `unpack()`'s post-unpack re-scan
  re-enters `StartExtraction()`, whose first statement unconditionally
  resets `$iUnicodeMode`, discarding the outer run's relocation
  bookkeeping. Documented and made testable, not fixed. Capability
  marked `DONE`.
- C178: TrID UNC-path detection reliability
  (`detection::trid_scan::trid_dll_string_marshalling`) — verified
  still present: every string parameter into `TrIDLib.dll` is
  marshalled as ANSI ("str"), never wide ("wstr"), consistent with
  the documented UNC-path detection-failure report. Capability marked
  `DONE`.
- C179 (partial): free-space prompt response handling
  (`free_space::decide_prompt_action`) — found the source's own
  `Switch` has no `Case` for Ignore, silently continuing extraction
  despite insufficient space. Extends PR #318's arithmetic/silent-mode
  coverage; still `REQUIRED` (the `MsgBox` call itself is unmodeled).
  Also backfilled `capability-manifest.md`'s C179 row, which had gone
  unupdated since PR #318.
- C149: batch stall on blocking user-input prompts
  (`batch::needs_user_input`) — verified still present: the tee-log
  polling loop has no timeout, so an unattended run blocked on a
  detected prompt stalls the whole batch chain indefinitely.
  Capability marked `DONE`.
- C151: batch-completion summary
  (`batch_runner::decide_batch_completion_actions`) — the "queue
  empty" branch `pop_and_relaunch_next_batch_item`'s own doc comment
  had flagged as missing; documents a real distinction from
  `terminate()`'s own separate keep-open relaunch condition.
  Capability marked `DONE`.
- C174: per-extractor timeout handling (`extractor_timeout`) —
  verified no global timeout mechanism exists (~15 scattered
  `$Timeout` sites out of ~70 extractor cases); ported the one clean
  representative example (`$TYPE_ARC_CONV`). Capability marked
  `DONE`.
- C166 (partial): teelog dual-output mechanism (`teelog`) — tee-pipe
  command composition, the fold-into-run-log gate, and the no-tee
  branch's own "reveal window once after 60s of no growth" heuristic;
  preserved the `$bPatternSearch > -1` numeric-comparison quirk. Stays
  `REQUIRED` (process/GUI/file I/O unmodeled).
- C104: ffmpeg per-stream extraction (`extract::ffmpeg`) — completes
  the capability's remaining `$TYPE_VIDEO` gap; found the `$iStreams`
  off-by-one (WMA shortcut fires for up to two real streams) and the
  dash-stripping asymmetry between `_MakeFFmpegCommand` and the
  image-sequence branch. Capability marked `DONE`.
- C106 (partial): Wise Installer 4-method fallback (`extract::wise`) —
  primary invocation, primary-result routing, five-choice dispatch
  (reusing C053's `method_select::WISE_CANDIDATES`), and invocation
  builders for choices 1/2/4 plus the completion-BAT path; choice 3's
  MSI rip stays GUI-blocked (`RipExeInfo`), row stays `REQUIRED`.
- C055/C180: game-archive BMS-script lookup (`bms` module,
  `extract::qbms::gaup_probe_invocation`) — resolved `_SQLite_GetTable`'s
  once-blocking array-shape question against AutoIt's own official
  documentation; ported `CheckGame`'s row-count gate/candidate sort,
  `GUI_MethodSelectList`'s override/silent/prompt dispatch, and
  `BmsExtract`'s script-test classification. C180's "hang risk" resolves
  to the already-`DONE` C026/C150 findings, not a new mechanism. Both
  capabilities marked `DONE`.
- C077: completed QuickBMS + WCX plugin fan-out's remaining two sites
  (`CheckGame`'s GAUP probe, `BmsExtract`) now that C055/C180 resolved
  the `_SQLite_GetTable` blocker; all 6 sites covered. Capability marked
  `DONE`.
- C069: new `automation` module — a `GuiAutomation` trait (mirroring
  `extract::runner::ExtractorRunner`'s real/fake split) covering the
  Win32 primitives `OpenExeInfo`/`RipExeInfo`/`CloseExeInfo` need, a
  real `Win32GuiAutomation` backend (new `windows` crate dependency,
  scoped to `cfg(windows)` only), a `FakeGuiAutomation` test double, and
  the ported orchestration functions themselves. Explicitly documented:
  fake-backed tests verify the decision logic but nothing proves the
  real Win32 backend drives an actual Exeinfo PE window, since no live
  Windows desktop with the real tool exists in this environment or on
  CI. Capability marked `DONE` on that basis.
- C042: completed the scan-only-mode GUI path
  (`detection::exeinfo_scan::scan_via_gui`), built on C069's new
  automation infrastructure — the last unported piece of this
  capability. Capability marked `DONE`.
- C044: completed the PEiD scan (`detection::peid_scan::peid_scan`),
  built on C069's automation infrastructure; found and preserved a
  genuine hang-risk quirk (`WinWait("PEiD v")` has no timeout, unlike
  every other `WinWait` call this port has ported so far). Capability
  marked `DONE`.
- C056: completed the 7z SFX-splitter branch
  (`extract::sevenzip::sfx_splitter_extract`), built on C069's
  automation infrastructure; added `win_close_by_title`/`file_exists`
  to `GuiAutomation` for it. Capability marked `DONE`.
- C106: completed choice 3's MSI rip (`extract::wise::wise_msi_rip`), a
  thin wrapper over C069's `automation::rip_exeinfo`. Capability marked
  `DONE`.
- C038/C045: new `dlllib` module — `TridLibrary`/`MediaInfoLibrary`
  traits, `FakeTridLibrary`/`FakeMediaInfoLibrary` test doubles, real
  `Win32TridLibrary`/`Win32MediaInfoLibrary` backends
  (`LoadLibraryW`/`GetProcAddress`), and the ported orchestration
  functions (`tridlib_load`/`tridlib_analyse`/`tridlib_analyse_simple`,
  `scan_media_info`) — the DLL-calling equivalent of C069's `automation`
  module. Found and preserved a reentry-guard quirk in `TridLib_Load`.
  Both capabilities marked `DONE`.
- C166: completed the teelog dual-output mechanism
  (`teelog::run_with_tee`/`teelog::run_without_tee`), built on C069's
  automation infrastructure, extended with the process-polling/
  file-reading primitives (`process_exists`/`win_get_by_pid`/
  `read_file_incremental`/`read_file_from_start`/`dir_size_bytes`/
  `win_set_state_by_title`/`win_activate`) this capability's own
  streaming-process needs introduced. Found and preserved a genuine bug:
  the tee branch's needs-input reveal calls `WinSetState` with the
  spawned process's PID instead of the resolved window handle, a silent
  no-op in the source itself. Capability marked `DONE`.
- Started migration phase 2 (GUI/tray/updater/telemetry/uninstall):
  inventoried UniExtract2's own AutoIt GUI shell into 35 new capability
  rows, C183-C217 (`capability-manifest.md`); D001-D005/D007-D012
  retired as superseded, D006 (translation catalogs) stays deferred.
- C183 (partial): new `gui` module (`egui`/`eframe`, new dependency) —
  `gui::layout`/`gui::theme`/`gui::window_state` port `CreateGUI`'s
  layout math, theme/high-contrast detection (reusing C069's
  `automation::GuiAutomation` registry read), and position-save gating;
  `gui::app::MainWindow` is a real, compiling `eframe::App` window
  shell, launched from `main()` on zero CLI args. Position/size
  persistence, DPI scaling, and literal widget fidelity remain — stays
  `REQUIRED`.
- C184 (partial): `gui::tray` decision logic plus a real tray icon and
  menu (`gui::tray_icon_shell::TrayHandle`, new `tray-icon` dependency)
  polled from `MainWindow::update`. Verified `TraySetClick(8)` against
  AutoIt's own docs (right-click-down only, matching `tray-icon`'s own
  default). `Tray_ShowHide`/`Tray_Exit`'s real actions aren't wired to a
  running helper process yet — stays `REQUIRED`.
- C185 (partial): `gui::tray_status_box` decision logic (fullscreen/
  no-status-box gates, character-counted truncation, taskbar-relative
  position resolution incl. the `$pos[0] = $pos[1]` top-dock quirk)
  plus a real popup via `egui`'s multi-viewport support, wired to real
  desktop/active-window/taskbar Win32 queries. Fade-in is animated,
  fade-out is not. Preference persistence not yet wired — stays
  `REQUIRED`.
- C186 (partial): `gui::file_input` decision logic (`FilenameParse`
  parsing reusing C138's `outdir::default_output_subfolder`, `GUI_OK_
  Set`'s validate-and-resolve path, the two distinct auto-fill gates)
  plus real native file-open/folder-picker dialogs via a new `rfd`
  dependency. Moved `main`'s private `split_file_path` into `outdir`
  as a shared public function. Single-file selection only; the
  "invalid file" dialog and `%VAR%` expansion aren't wired — stays
  `REQUIRED`.
- C187 (partial): `gui::drag_drop` ports `GUI_Drop`'s per-item dispatch
  (skip/expand-directory/populate-only/populate-and-queue);
  `WM_DROPFILES_UNICODE_FUNC` isn't ported since `egui`'s own native
  drag-drop input already supersedes it. Wired for real via `eframe`'s
  drop-file input; only the single-file case is handled (multi-file/
  directory route into the not-yet-built batch queue, C188) — stays
  `REQUIRED`.
- C188 (partial): `gui::batch_queue` ports the Batch button's Add/Run/
  error overload, the `BatchRecurse` clamp, real recursive directory
  listing, and the queue-edit dialog's pure decisions, reusing the
  already-DONE C147/C148 `crate::batch` queue mechanics rather than
  re-deriving them. Wired an in-memory batch queue for real, completing
  C187's deferred `AddDirectory`/`PopulateAndQueue` drop cases. Real
  batch execution, the queue-edit window, and queue persistence aren't
  wired — stays `REQUIRED`.
- C189 (partial): `warn_execute` ports `Warn_Execute`'s dispatch
  (warning-disabled always proceeds; a shown-and-declined dialog aborts
  and cleans up only a `$createdir`-created output directory, mirroring
  C179's `free_space::decide_prompt_action` shape) and the "don't ask
  again" persistence decision. Verified a real `Opt("GUIOnEventMode")`
  restore bug in the source, moot under `egui`'s architecture. The real
  confirm/cancel dialog isn't wired — its ~13 call sites are all inside
  the extraction dispatch table, which the GUI doesn't drive yet — stays
  `REQUIRED`.
- C190 (partial): `gui::prefs_dialog` ports `GUI_Prefs_OK`'s real
  quirks (history-delete-on-uncheck, update-interval preset table
  folding in D003, send-stats/nightly-update change-only network
  gates, delete-source-file radio resolution, scan-only save-skip
  parameter, topmost ex-style). Wired a "Lock output directory"
  checkbox and "Always on top" (via `ViewportCommand::WindowLevel`,
  superseding the source's window-recreate workaround) for real. The
  Preferences dialog window itself, plus `GUI_Silent`/`GUI_KeepOpen`,
  aren't wired — stays `REQUIRED`.
- C191 (partial): `gui::first_start` ports the wizard's page-navigation
  state machine as a pure function of the current page
  (Prev-visibility, Next/Finish relabeling, per-page action button) and
  the missing-translation branch's decision shape. Deliberately not
  wired to any real ID-clear/exit call (needs explicit sign-off per
  this row's own note) or a real wizard window (its pages link to
  not-yet-real C190/C192 dialogs and need translation-catalog
  infrastructure, D006) — stays `REQUIRED`.
- C192 (partial): `gui::plugin_manager` ports the install-mechanism
  dispatch, the overloaded Select/Finish button's guard, per-selection
  button display (preserving the `@Compiled`-gated installed-check
  quirk), the required-file wildcard-skip quirk, and the single-vs-
  multi-file copy destination resolution. Corrected an earlier
  inventory note: the previously-flagged `.7z` extension-check "bug"
  doesn't exist (`".7z"` is 3 characters, not 4). The plugin manager
  window itself isn't wired — stays `REQUIRED`.
- C193 (partial): `gui::prompt` ports `Prompt`'s full dispatch
  (silent-mode auto-affirm, load-bearing for several already-DONE
  capabilities' `user_confirmed_*` stand-ins) and `CustomPrompt`'s full
  dispatch (the persisted Always/Never short-circuit and button-to-
  setting mapping). `_IsChecked`/`_IsAnyChecked`/`_SetState` aren't
  ported — moot Win32 control-state plumbing once a checkbox is a plain
  `bool`. Neither dialog has a real window — stays `REQUIRED`.
- C194 (partial): `gui::error_dialogs` ports the shared silent-mode
  no-op gate, the copy-selection-or-all decision (unified rather than
  reproducing the source's own inline duplication), the two dialogs'
  different scrollbar line-count thresholds, and the unknown-ext
  dialog's dynamic sizing based on scan results. Neither dialog has a
  real window — stays `REQUIRED`.
- C195 (partial): `gui::log_actions` ports the open-last-log gate and
  its alphabetical-not-chronological "most recent" selection, the
  log-menu-item update gate and MB-size rounding, and the
  password-file touch-if-missing gate. None of it is wired to real I/O
  yet (no settings-directory resolution plumbed into the GUI) — stays
  `REQUIRED`.
- C196 (partial): `gui::stats` ports the status-to-INI-key mapping, the
  archive-type counter's success-only gate, the four-bucket status
  categorization, the confirmed ≥10-distinct-keys display gate, and the
  top-9-by-count filter. Found and preserved a new quirk: four status
  keys (`movefailed`/`nofreespace`/`missingpart`/`trayexit`) fall
  through the source's own classification switch and get miscounted as
  archive types. The pie-chart rendering itself is out of scope — stays
  `REQUIRED`.
- C197 (partial): `gui::about` ports the high-contrast logo asset swap
  — the one real decision in `GUI_About`. Everything else in this row
  is static composition or plain I/O. Not wired to a real window —
  stays `REQUIRED`.
- C198: verified no pure decision logic exists to port (entirely static
  text blocked on D006 plus one unconditional `ShellExecute`) — no
  source changes, stays `REQUIRED`.
- C199 (partial): `gui::update_prompt` ports the changelog-fetch
  failure-text fallback; the Yes/No result reuses C193's boolean
  dialog-outcome shape. Flagged, not resolved, an open design question
  on blocking-vs-async fetch for whoever wires the real dialog. Not
  wired to a real window — stays `REQUIRED`.
- C200 (partial): `gui::missing_helper` ports `HasPlugin`'s existence
  check and dispatch, the FFmpeg dialog's license-acceptance gate and
  file-picker early-out, the fragile FFmpeg-binary validation heuristic
  (preserved exactly), and the hardlink-then-copy-fallback install
  dispatch (documented as a deliberate architectural choice). Not
  wired to a real window or the real network calls — stays `REQUIRED`.
- C201 (partial): `gui::context_menu` ports the per-verb registration
  scan (confirming the flagged all-users/per-user toggle quirk), the
  Simple/Cascading picture swap, the full enable/disable activate
  cascade, the force-check-all-verbs fallback, the registration-mode
  dispatch, and the file-association sub-flow's gates. Corrects this
  row's function list (`GUI_ContextMenu_OK` was omitted). Not wired to
  a real window or registry I/O — stays `REQUIRED`.
- C202 (partial): extended `automation` with `GuiAutomation::reg_write_string`
  (`REG_SZ` writes) and `HKEY_CLASSES_ROOT` root support, as this row's
  own note required. Verified only `REG_SZ` writes are actually needed
  (no `REG_EXPAND_SZ` call site exists). `gui::context_menu_registry`
  ports the simple/cascading write plans and the full-wipe removal
  list. Not wired to a real window — stays `REQUIRED`.
- C203 (partial): substantially already covered by C202's
  `resolve_removal_targets` (same function, same PR). Adds the one
  remaining gap: `should_teardown_file_assoc_before_rebuild`, the
  unconditional file-association teardown before any re-registration.
  Stays `REQUIRED`.
- C204 (partial): extended `automation` with `reg_read_string`/
  `reg_write_expand_string`. **Fixed a real bug**: `reg_delete_key`
  used `RegDeleteKeyW`, which refuses to delete a key with subkeys
  (unlike AutoIt's own recursive `RegDelete()`); switched to
  `RegDeleteTreeW`, fixing every prior caller (C069, C201-C203) too,
  not just this row's own new ones. `gui::file_assoc` ports the
  extension-list parsing, the install write plan (preserving the
  "two overlapping association styles" verbatim), and the top-level
  enable/disable dispatch. Stays `REQUIRED`.
- C205 (partial): `cli::match_update_verb` ports the `/update`/
  `/updatehelper`/`/updatehelpers`/`/afterupdate` dispatch, preserving
  the verified case-sensitivity inconsistency between `/updatehelper`
  and `/updatehelpers`. Not wired to the real updater (D003) — stays
  `REQUIRED`.
- C206 (partial): new `update_index` module ports `_UpdateGetIndex`'s
  wire-format parsing, `_UpdateGetSize`'s plugin-binary exclusion
  logic, `_UpdateFileCompare`'s diff decision, and
  `CheckUpdateHelpers`'s skip/update-found/recurse branching. Found a
  quirk beyond the manifest note: AutoIt's `FileGetSize` returns `-1`
  for a missing file with no existence check before the subtraction,
  so a missing excluded plugin binary inflates the computed size by
  one byte. Not wired to real network/file I/O (D003) — stays
  `REQUIRED`.
- C207 (partial): new `update_orchestration` module ports `CheckUpdate`'s
  decision tree (interval debounce, mode-gated main-executable compare,
  relaunch parameters, the mode-override-after-decline quirk, the
  helper-update apply/relaunch-elevated branch, the "up to date" gate),
  threading the `GUI_UpdatePrompt`/`Prompt` seams through as plain
  booleans. Verified that declining the main-executable update also
  silently suppresses the helper-files and FFmpeg checks for that run.
  Not wired to real I/O (`Exit`/`ShellExecute`/`SendStats`) — stays
  `REQUIRED`.
- C208 (partial): new `update_helpers` module ports `_UpdateHelpers`'s
  progress math and directory/success decisions, reusing
  `update_index`'s `decide_file_needs_update`/`is_self_path` (C206)
  directly. Found a divergence beyond the manifest's DRY-gap note:
  `CheckUpdateHelpers` and `_UpdateHelpers` don't actually agree on
  directory recursion — one treats a missing subdirectory as proof of
  an update, the other always recurses into it regardless. Not wired
  to real I/O (`InetGet`/`_FileDelete`/`DirCreate`) — stays `REQUIRED`.
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
