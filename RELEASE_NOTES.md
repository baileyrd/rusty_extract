# Release Notes

<!--
Two variants, pick the one that fits this repo's actual unit of change:

1. No version tags yet (pre-1.0, nothing published) — track by PR instead, same way
   AISF does it: one entry per merged PR against main, reverse chronological, each
   linking to its PR and (where one exists) to the doc that covers the change in full
   detail. Use "## PR #N — <summary>" headers.

2. Actual version tags exist — use "## vX.Y.Z - YYYY-MM-DD" headers instead, each
   linking to the PRs it shipped and a compare link to the previous tag. Add an
   "### Upgrade notes" subsection under any entry with a breaking change.

Either way, keep the tone AISF's file uses: bolded category tags inline in the
bullet (**Added:** / **Changed:** / **Fixed:**), not separate subheaders per
category — and state known limitations or deliberate scope cuts plainly instead of
leaving them implied.
-->

Tracks notable changes to this repo, one entry per merged PR against `main`,
reverse chronological (no version tags yet — pre-1.0, nothing published).

---

## C197 (partial) — About dialog and website menu actions
**2026-08-24**

- **Added:** `gui::about::resolve_about_logo_filename` — the one real
  decision in this row: the high-contrast logo asset swap.
- Everything else (version/timestamp/credits display, the per-install
  GUID display, the three fixed-URL website menu actions) is static
  composition or plain I/O with no decision logic to port.
  `GUI_Close`'s multi-window active-detection trick is moot under
  `egui`'s per-viewport model.
- **Scope — not wired to a real window**, same treatment as every
  other dialog this phase has ported, doubly so since the per-install
  ID (C215) doesn't exist as GUI state yet either.
- Tests: `gui::about::tests` (1).

---

## C196 (partial) — Local usage statistics
**2026-08-24**

- **Added:** `gui::stats` — `status_ini_key` (the `Status`-to-INI-key
  string mapping), `should_increment_arctype_counter` (the archive-type
  counter's success-only gate), `classify_stats_key` (the four-bucket
  categorization), `should_show_stats` (confirmed: the ≥10 gate counts
  distinct keys, not total extraction volume), and `top_n_by_count`
  (the descending-sort-then-cap-at-9 filter).
- **New verified quirk**: the source's classification `Switch` has no
  case at all for `movefailed`/`nofreespace`/`missingpart`/`trayexit` —
  four real status keys that fall through to `Case Else` and get
  miscounted as archive types rather than landing in `Failed` or being
  excluded. Preserved exactly, not "corrected."
- **Scope — the GDI+ pie-chart rendering itself is out of scope
  entirely**, a rendering dependency this port replaces rather than a
  behavior to reproduce, so nothing here is wired to a real window.
- Tests: `gui::stats::tests` (8).

---

## C195 (partial) — Misc file/log utility actions
**2026-08-24**

- **Added:** `gui::log_actions` — `should_open_a_log`/
  `resolve_most_recent_log` (`GUI_OpenLastLog`'s early-return gate and
  its "most recent" selection, preserved exactly as the
  alphabetical-not-chronological assumption the source itself relies
  on), `should_update_log_menu_item`/`format_log_dir_size_mb`
  (`GUI_UpdateLogItem`'s main-window-exists gate and its `Round(...,2)`
  MB formatting), and `should_create_password_file` (`GUI_Password`'s
  touch-if-missing gate).
- `GUI_OpenLogDir`/`GUI_ProgDir`/`GUI_ConfigFile` are plain
  `ShellExecute` one-liners with no decision logic — not ported as
  functions.
- **Scope — none of this is wired to real I/O.** `logdir` itself
  depends on settings-directory resolution this port's GUI doesn't have
  plumbed into its own state yet, the same category of gap as every
  other still-unresolved preference path.
- Tests: `gui::log_actions::tests` (6).

---

## C194 (partial) — Error dialogs with feedback/scan integration
**2026-08-24**

- **Added:** `gui::error_dialogs` — `should_show_error_dialog` (the
  shared silent-mode no-op gate), `resolve_copy_target` (the
  copy-selection-or-all decision, unified into one function instead of
  reproducing the source's own inline duplicate of the same logic in
  two places), `needs_vertical_scrollbar` (the two dialogs' different
  line-count thresholds — 13 for the standalone file-scan dialog, 7 for
  the embedded, shorter one), and `resolve_unknown_ext_layout` (dynamic
  290px/190px sizing based on whether a signature scan found anything).
- The logo-image-as-Exeinfo-PE-launch-button and the clipboard write
  itself are real I/O with no decision logic — left as real-wiring
  concerns for whoever builds the window.
- **Scope — neither dialog is wired to a real window**, same treatment
  as C188-C193's own unwired dialogs.
- Tests: `gui::error_dialogs::tests` (8).

---

## C193 (partial) — Generic prompt/confirm dialogs
**2026-08-24**

- **Added:** `gui::prompt` — `is_affirmative_msgbox_response`/
  `decide_prompt_outcome` port `Prompt`'s full dispatch, including the
  load-bearing silent-mode auto-affirm several already-DONE
  capabilities' `user_confirmed_*` parameters already stand in for
  (`cleanup`'s C158, `batch`'s C147). `CustomPromptSetting`/
  `decide_custom_prompt_short_circuit`/`resolve_custom_prompt_button`
  port `CustomPrompt`'s full dispatch: the persisted Always/Never
  short-circuit (checked before silent mode) and the dialog's own
  button-to-setting-mutation mapping.
- `_IsChecked`/`_IsAnyChecked`/`_SetState` aren't ported as functions —
  Win32 control-state bitmask/loop plumbing that's moot once a checkbox
  is a plain `bool` bound directly by `egui::Checkbox`.
- **Scope — neither dialog is wired to a real window**, same treatment
  as C188-C192's own unwired dialogs.
- Tests: `gui::prompt::tests` (11).

---

## C192 (partial) — Plugin manager GUI
**2026-08-24**

- **Added:** `gui::plugin_manager` — `resolve_install_mechanism`
  (unpack-vs-copy dispatch), `decide_select_close_action` (the
  overloaded Select/Finish button's close-vs-prompt guard),
  `resolve_plugin_selection_display` (Download/Select-Finish button
  state per selection, preserving the `@Compiled`-gated installed-check
  quirk), `missing_required_files` (the wildcard-skip quirk), and
  `resolve_copy_plan` (single-file-renamed vs. multi-file-kept-named
  destinations, reusing C186's `parse_file_dialog_result`).
- **Correction to this capability's own earlier inventory note**: the
  previously-flagged `StringRight($sPath, 3) == ".7z"` "bug" doesn't
  exist — `".7z"` is 3 characters, not 4 as originally miscounted, so
  the comparison works correctly and `.7z` archives route to the
  7-Zip-extraction branch alongside `rar`/`zip`, same as the source
  intends.
- **Scope — the plugin manager window itself has no real UI.** A
  list-plus-description window around a hardcoded 12-entry plugin
  table with real network downloads and filesystem installs, none of
  which this port's GUI drives yet.
- Tests: `gui::plugin_manager::tests` (16).

---

## C191 (partial) — First-start wizard
**2026-08-24**

- **Added:** `gui::first_start` — the page-navigation state machine
  (`prev_button_visible`, `next_button_mode`, `resolve_action_button`)
  ported as a pure function of the *current* page, reformulating
  `GUI_FirstStart_Prev`/`_Next`/`_ShowPage`'s own delta-toggling side
  effects into one declarative function per widget — verified
  equivalent by tracing both the forward and backward traversal the
  wizard's fixed 3-page structure actually produces. Also
  `decide_missing_translation_outcome`, documenting the missing-
  translation branch's real "always exits either way, only
  conditionally restarts into an update first" shape.
- **Deliberately not wired to any real `SavePref("ID", "")`/exit call.**
  That branch is a hard-to-reverse, unusually severe response to a
  missing string — the capability manifest itself flags it as needing
  explicit sign-off before real implementation, not just porting.
- **Scope — the wizard window itself stays unwired.** Its own pages
  link out to `GUI_Prefs`/`GUI_ContextMenu` (C190/C192), neither a real
  window yet, and its text needs translation-catalog infrastructure
  (D006) this port doesn't have.
- Tests: `gui::first_start::tests` (6).

---

## C190 (partial) — Preferences dialog and instant-apply tray toggles
**2026-08-24**

- **Added:** `gui::prefs_dialog` — the real quirks inside `GUI_Prefs_OK`
  and the four standalone toggles: `decide_history_toggle_outcome`
  (unchecking "remember history" deletes the two history INI keys
  outright, not just zeroes a flag), `resolve_update_interval_display_index`/
  `resolve_update_interval_value` (the fixed preset table, folding in
  D003's `iOptUpdateInterval`, including the "Custom" selection keeping
  the previously-stored value), `decide_send_stats_command`/
  `should_check_update_after_nightly_toggle` (both only fire their
  network side effect on an actual change), `resolve_checked_delete_source_file_radio`
  (reusing C024's `parse_delete_source_file_option`),
  `should_persist_scan_only_pref` (the startup-vs-user-click save-skip
  parameter), and `resolve_topmost_ex_style`.
- `gui::app::MainWindow` wires two pieces for real: a "Lock output
  directory" checkbox (mirroring the already-consumed
  `lock_output_directory` field from C186) and "Always on top" via
  `egui`'s `ViewportCommand::WindowLevel` — applied live rather than
  replicating the source's destroy-and-recreate-the-window workaround
  (`WS_EX_TOPMOST` can't be changed on a live Win32 window handle;
  `egui` has no such limitation).
- **Scope — the Preferences dialog window itself has no real UI yet**
  (~20 controls, no real prefs-file read/write pathway to back it —
  same category of gap as C188/C189's own unwired dialogs).
  `GUI_Silent`/`GUI_KeepOpen` aren't wired either, since their real
  consumers (silent-mode extraction, batch-queue-empty self-relaunch)
  don't exist in this port's GUI yet.
- Tests: `gui::prefs_dialog::tests` (15).

---

## C189 (partial) — Pre-execution warning gate
**2026-08-24**

- **Added:** `warn_execute::decide_warn_execute_outcome` ports
  `Warn_Execute`'s dispatch — warning disabled skips the dialog and
  always proceeds; a shown-and-declined dialog aborts, cleaning up only
  a temp output directory this run itself created. Mirrors
  `free_space::decide_prompt_action`'s (C179) same `$createdir`-gated
  cleanup shape. `should_disable_warn_execute_permanently` ports the
  "don't ask again" checkbox's persistence.
- **Verified, not fixed:** `GUI_Warn_Execute` never restores
  `Opt("GUIOnEventMode")` back to 1 after its dialog closes (a real
  copy-paste-looking bug in the source), but this is moot for the port
  since `egui`'s immediate-mode loop has no equivalent toggle to get
  wrong.
- **Scope — the real confirm/cancel dialog stays unwired.** Every one of
  `Warn_Execute`'s ~13 call sites sits inside the extraction dispatch
  table, which this port's GUI doesn't drive at all yet (same gap
  blocking C188's Batch "Run" branch and the OK button's real
  extraction in C183/C186); a dialog with nothing that could trigger it
  would be dead code, not a meaningful port.
- Tests: `warn_execute::tests` (4).

---

## C188 (partial) — Batch queue management UI
**2026-08-24**

- **Added:** `gui::batch_queue` — the GUI-layer decisions on top of the
  already-DONE C147/C148 queue mechanics (`crate::batch`, reused rather
  than re-derived): `decide_batch_button_action` (the Batch button's
  three-way overload: Add/Run/error), `should_clear_output_dir_on_batch_add`,
  `resolve_batch_recurse` (the `BatchRecurse` clamp-above-1 quirk),
  `list_directory_files` (real, unsorted, recursive enumeration),
  `should_disable_batch_mode_on_ok`, `delete_queue_item`, and
  `should_show_full_text_tooltip`.
- `gui::app::MainWindow` wires an in-memory `batch_queue: Vec<String>`
  for real: the Batch button's Add branch, and — completing the gap
  C187 explicitly left open — drag-and-drop's `AddDirectory`/
  `PopulateAndQueue` cases (a dropped directory recursively queues every
  file found; one of several dropped files is populated then
  immediately queued).
- **Scope — real batch execution stays out.** The Batch button's "Run"
  branch is decided correctly but not acted on: it needs
  `GUI_Batch_OK`'s `terminate`+`crate::batch_runner` relaunch chain,
  which needs a real extractor invocation the GUI can't build yet (no
  detection cascade wired in). The queue-edit dialog (`GUI_Batch_Show`)
  has no real window yet, only its pure decisions are ported. The queue
  is in-memory only, not persisted to the real `$batchQueue` file. The
  exact-duplicate confirmation dialog (C193) isn't wired, so a duplicate
  is always silently skipped rather than prompted.
- Same honesty caveat as C069/C183-C187.
- Tests: `gui::batch_queue::tests` (11, including two real-filesystem
  tests against a temp directory).

---

## C187 (partial) — Drag-and-drop file/directory handling
**2026-08-24**

- **Added:** `gui::drag_drop` — `decide_drop_action`/`decide_drop_actions`
  port `GUI_Drop`'s full per-item dispatch: a nonexistent path is
  silently skipped; a directory always expands into the batch queue
  (C188) regardless of batch size; a file is populate-only when it's
  the *sole* dropped path, or populate-and-queue when it's one of
  several.
- **`WM_DROPFILES_UNICODE_FUNC` is not ported as its own piece** — it's
  a raw `DragQueryFileW` enumeration loop whose entire job (turning an
  OS drop event into a path list) is already done by `egui`'s own
  native drag-drop input, which supersedes the Win32 workaround rather
  than needing a parallel implementation — same class of "old
  workaround made moot by the new toolkit" as C183's DPI-scaling note
  and C185's tooltip-workaround note.
- `gui::app::MainWindow` wires this for real via `eframe`'s native
  drop-file input, with `NativeOptions`'s `with_drag_and_drop(true)`
  matching the source's own `$WS_EX_ACCEPTFILES` window ex-style.
- **Scope — only the single-file case is wired for real.** A multi-file
  drop or a dropped directory routes into the batch queue (C188), which
  doesn't exist yet; silently keeping only the last of several dropped
  files would be a worse outcome than doing nothing, the same call
  already made for C186's multi-select scope note.
- Same honesty caveat as C069/C183-C186 for the real drop-handling
  wiring.
- Tests: `gui::drag_drop::tests` (6).

---

## C186 (partial) — File/directory input, validation, output-directory auto-fill
**2026-08-23**

- **Added:** `gui::file_input` (pure decisions: `is_blank`/
  `parse_filename` — `FilenameParse`'s parsing half, reusing
  `outdir::split_file_path`/`outdir::default_output_subfolder` (C138)
  for `$initoutdir` rather than re-deriving its multi-extension
  collision-avoidance quirk — `decide_ok_set`, `should_auto_fill_
  on_file_input_changed` vs. `should_auto_fill_output_dir`, `parse_
  file_dialog_result`, `resolve_folder_picker_seed`) and real native
  file-open/folder-picker dialogs wired into `gui::app::MainWindow` via
  a new dependency, `rfd` (the closest real-API equivalent of
  `FileOpenDialog`/`FileSelectFolder` — cross-platform-buildable, so it
  type-checks on this dev environment's own host target too, not just
  `x86_64-pc-windows-gnu`).
- **Real quirk preserved**: `GUI_OnFileInputChanged` (fires on every
  keystroke in the file field) uses a simpler blank-only auto-fill gate,
  while `GUI_Drop_Parse` (fires from the file-open dialog and, once
  C187 lands, drag-and-drop) uses a fuller OR gate — blank *or*
  unlocked — reusing `prefs::KEEPOUTPUTDIR_DEFAULT` (C027) for the
  lock preference's default. These are genuinely two different gates
  for two different trigger events, not one gate applied
  inconsistently.
- **Refactor**: moved `main`'s own private `split_file_path` helper
  into `outdir` as a public function, so `main`'s CLI composition root
  and this new GUI code share one implementation instead of two
  independently-derived copies of the same split.
- **Scope — single-file selection only for now.** The file dialog's
  multi-select path routes into `GUI_Drop`'s populate-vs-auto-queue
  dispatch (C187) and potentially the batch queue (C188), neither of
  which exist yet; wiring multi-select today with nowhere for a 2nd+
  file to go would silently drop every file but the first.
- **Scope — still `REQUIRED`, not `DONE`.** The "invalid file" `MsgBox`
  isn't wired (C194's own row), so an invalid OK click is currently
  silent rather than explained; `EnvParse`'s `%VAR%` env-variable
  expansion isn't applied to either field yet.
- Same honesty caveat as C069/C183/C184/C185 for the real-dialog wiring
  specifically.
- Tests: `gui::file_input::tests` (14), plus
  `outdir::tests::split_file_path_separates_dir_stem_and_extension`
  (moved from `main.rs`'s own test suite).

---

## C185 (partial) — Tray status/progress popup
**2026-08-23**

- **Added:** `gui::tray_status_box` (pure decisions:
  `should_show_status_box`, `should_suppress_for_fullscreen`,
  `truncate_with_ellipsis`, `resolve_position`) and a real popup
  rendered via `egui`'s multi-viewport support (undecorated,
  transparent, always-on-top, no taskbar entry), wired to real
  `GetSystemMetrics`/`GetForegroundWindow`+`GetWindowRect`/
  `FindWindowW("Shell_TrayWnd")`+`GetWindowRect` calls for the
  desktop/active-window/taskbar geometry the pure functions need.
  Distinct from C166, which only ported the *text-update* call into an
  already-existing popup — this row is the popup's own creation,
  positioning, and lifecycle.
- **Real quirk preserved**: the source detects a top-docked taskbar via
  `$pos[0] = $pos[1]` (X equals Y) — which only actually holds at the
  origin `(0,0)` — ported as the same equality test rather than
  "clarified" into an explicit Y-is-zero check.
- Preserves the popup's hardcoded-dark-always background (`$bDark =
  True`, never light-themed unlike C183's main window) and achieves the
  same corner-rounding result via `egui`'s native styling rather than
  replicating `_GuiRoundCorners`'s specific Win32-region mechanism —
  same class of idiomatic adaptation as C183's DPI-scaling note.
- **Fade-in is animated** (frame-timed alpha over the same ~230ms the
  source's 23-step `Sleep`-loop takes); **fade-out is not** (the popup
  is simply removed) — a documented simplification.
- **Scope — still `REQUIRED`, not `DONE`.** Preference persistence for
  `nostatusbox`/`hidestatusboxiffullscreen`/`statusposx`/`statusposy`
  isn't wired to the real prefs file yet (C184 already wires the
  tray-menu toggle to in-memory state); fade-out isn't animated.
- Same honesty caveat as C069/C183/C184.
- Tests: `gui::tray_status_box::tests` (9).

---

## C184 (partial) — System tray icon and menu
**2026-08-23**

- **Added:** `gui::tray` (pure decisions: `hide_status_item_checked`,
  `should_hide_icon`, `decide_console_visibility_action`,
  `should_log_tray_exit`) and `gui::tray_icon_shell::TrayHandle` — a
  real tray icon and two-item menu ("Hide status" checkbox, "Exit")
  built on a new dependency, the `tray-icon` crate (`eframe`/`egui`
  itself has no tray-icon primitive; hand-rolling `Shell_NotifyIconW`
  would have to cooperate with `winit`'s already-owned event loop
  rather than run its own), polled once per frame from
  `gui::app::MainWindow::update`.
- **Verified against AutoIt's own docs, not assumed**: `TraySetClick(8)`
  is `$TRAY_CLICK_SECONDARYDOWN` — the tray menu opens on right-click-down
  only, with nothing bound to the left button — which is `tray-icon`'s
  own default Windows-backend behavior already, so no explicit
  click-mode configuration was needed.
- **Scope — still `REQUIRED`, not `DONE`.** `Tray_ShowHide`'s
  console-visibility toggle isn't wired to a real spawned helper process
  yet (the GUI doesn't launch extractions at all so far); `Tray_Exit`'s
  real sequence (`KillHelper` + `GUI_SavePosition` + the conditional
  status log) isn't wired to the Exit menu item, which currently only
  closes the window.
- Same honesty caveat as C069/C183: fake-backed/pure-function tests
  prove the decision logic; nothing proves the real tray icon renders
  or responds correctly in this environment.
- Tests: `gui::tray::tests` (5).

---

## C183 (partial) — Main window shell: layout/theme decision logic + a real egui window
**2026-08-23**

- **Added:** the `gui` module — this app's *own* GUI (not to be confused
  with `automation::GuiAutomation`, which drives *other* programs'
  windows). First capability of migration phase 2 (GUI/tray/updater/
  telemetry/uninstall, capability-manifest.md C183-C217, PR
  [#418](https://github.com/baileyrd/rusty_extract/pull/418) inventoried
  it). New dependency: `eframe`/`egui`, `#[cfg(windows)]`-gated.
  - `gui::layout`: `GetPos`'s control-chaining math, including the
    source's own undocumented RTL 0.4 X-offset factor; the `-1`
    ex-style sentinel resolution; and the "minimum window size is the
    size measured *after* `GUICreate` returns, not the nominal size
    passed in" contract, pinned by a test so a future edit can't
    quietly start enforcing the nominal size instead.
  - `gui::theme`: `_AppsUseLightTheme` (reusing C069's
    `automation::GuiAutomation::reg_read_dword` — an ordinary registry
    read, not a new capability), `_IsHighContrastMode`'s bit test, and
    `_GuiSetColor`'s three-way white-background gate (Windows 10 only,
    not high-contrast, light theme).
  - `gui::window_state`: `GUI_SavePosition`'s exists-and-preference-on
    save gate.
  - `gui::app::MainWindow`: a real, compiling `eframe::App` — window
    shell with File/Edit/Help menus, the Extract/Scan-only selector,
    file/output-directory fields, and OK/Cancel/Batch buttons — running
    real theme/high-contrast/Windows-version detection at startup
    (`SystemParametersInfoW`, `RtlGetVersion`). `main()` now launches it
    on zero CLI arguments, matching `ParseCommandLine`'s own
    `$prompt = True` branch (UniExtract.au3:588-591) — a small gap this
    session's original inventory pass missed, caught and closed while
    wiring the window up.
- **Scope — still `REQUIRED`, not `DONE`.** Real position/size
  persistence (only the save *gate* is ported, not the preference
  read/write itself), the DPI-scaling engine, corner-rounding, the GDI+
  logo-image loader, the disabled-control tooltip workaround (likely
  obsolete under egui's native support — a call for whoever finishes
  this row), and literal widget-for-widget fidelity to `CreateGUI` all
  remain. Individual menu items/buttons are inert placeholders wired up
  as their own capabilities (C184-C217) land.
- **Honesty note, same as C069/C038/C045/C166**: fake-backed/pure-function
  tests prove the decision logic; nothing here proves the real window
  actually renders or responds correctly, since no interactive Windows
  desktop exists in this environment or on headless `windows-latest` CI.
- Tests: `gui::layout::tests` (6), `gui::theme::tests` (4),
  `gui::window_state::tests` (3).

---

## C166 — Teelog dual-output mechanism (completes the capability)
**2026-08-23**

- **Added:** `teelog::run_with_tee`/`teelog::run_without_tee`, the full
  spawn-to-exit orchestration for both branches of `_Run()`
  (UniExtract.au3:4880-5008), built on C069's `automation::GuiAutomation`
  infrastructure. This capability's own streaming-process needs — the
  tee branch spawns a process, waits for it to start, waits for its live
  log file to appear, then polls that file incrementally until exit —
  required extending the trait with `process_exists`/`win_get_by_pid`/
  `read_file_incremental`/`read_file_from_start`/`dir_size_bytes`/
  `win_set_state_by_title`/`win_activate`, plus real Win32 implementations
  (`OpenProcess`, `EnumWindows`+`GetWindowThreadProcessId`,
  `ShowWindow`/`SetForegroundWindow`) in `automation::win32`.
- `teelog::decide_tee_iteration`/`teelog::TeeLoopState` port the tee
  branch's per-iteration decision exactly, including the permanent `$size
  = -1` lockout once `_PatternSearch` ever matches (nothing in the source
  ever resets it) and the same variable used with two different
  comparisons in one function (`$bPatternSearch And ...`, a truthy check,
  vs. `$bPatternSearch > -1`, a tri-state threshold check already
  documented on `decide_size_poll_action`).
- **Found and preserved a genuine bug, not fixed**: the tee branch's
  "needs user input" reveal calls `WinSetState($run, "", @SW_SHOW)` —
  passing `$run`, the spawned process's **PID**, not `$runtitle` (the
  resolved window handle `WinActivate` uses two lines later). A PID never
  matches a real window title, so this call is a silent no-op in the
  source itself; `run_with_tee` reproduces it exactly via
  `win_set_state_by_title` on the stringified PID.
- **Also preserved**: the three `Do-Until` wait loops' body-first
  semantics (a plain `while cond { ... }` would skip the body entirely
  when the condition is already true on the first check — AutoIt's
  `Do-Until` never does) and `ContinueLoop`'s skip of the trailing
  `Sleep(100)` on a needs-input iteration.
- Capability marked `DONE`. Carries the same honesty caveat as C069:
  fake-backed tests prove the orchestration logic line-by-line against
  the source, not that the real Win32 backend drives a real spawned
  process's window correctly — no such live interactive session exists in
  this environment or on CI (headless `windows-latest`).
- **Not modeled** (documented, accepted limitations): `_PatternSearch`'s
  own 4-pattern regex-based progress-text parsing (taken as a
  caller-supplied closure — the same call already made for
  `extract::ffmpeg`'s own regex-backtracking edge case) and
  `_DirGetSize`'s big-drive-root guard (one shared byte count covers both
  of the source's slightly different call sites).
- Tests: `teelog::tests` (21, up from 8).

---

## C038/C045 — DLL-calling infrastructure + TrIDLib/MediaInfo calls
**2026-08-22**

- **Added:** `dlllib` — new DLL-calling infrastructure, mirroring the
  split already established for `extract::runner::ExtractorRunner`
  (plain process spawning) and `automation::GuiAutomation` (Win32 window
  automation):
  - `dlllib::TridLibrary`/`dlllib::MediaInfoLibrary` — two small,
    function-specific traits (`load_defs_pack`/`submit_file`/`analyze`/
    `result_count`/`result_type`/`result_extension`; `open`/`inform`/
    `close`) rather than one generic `DllCall` shim — replicating
    AutoIt's fully dynamic `DllCall` would need something like the
    `libffi` crate; only 7 specific exports across two DLLs are ever
    needed, so function-specific methods are simpler and safer.
  - `dlllib::fake::{FakeTridLibrary, FakeMediaInfoLibrary}` — test
    doubles recording every call with scriptable results.
  - `dlllib::win32::{Win32TridLibrary, Win32MediaInfoLibrary}` — real,
    Windows-only (`#[cfg(windows)]`) implementations using
    `LoadLibraryW`/`GetProcAddress` plus a hand-written function-pointer
    type per export.
  - `dlllib::tridlib_load`/`tridlib_analyse`/`tridlib_analyse_simple`
    and `dlllib::scan_media_info` — the ported orchestration functions
    themselves.
- **Found and preserved a genuine quirk in `TridLib_Load`**: `$hTridDll`
  is set by `DllOpen` *before* the definitions-pack load result is
  checked, so a failed load still leaves the cache looking "already
  loaded" to the next call's reentry guard, silently skipping the retry
  it exists to provide.
- **Wired into both capabilities**: `detection::trid_scan` (C038) and
  `detection::mediainfo_scan` (C045, with a new end-to-end composition
  test proving `scan_media_info`'s output feeds directly into
  `format_media_info`). Both capabilities marked `DONE`.
- **Honesty note, not glossed over**: fake-backed tests prove the
  orchestration logic against the source line-by-line, the same
  confidence every other parity test in this crate has. They do **not**
  prove the real Win32 backend calls the real DLLs correctly — that
  needs the actual, licensed `TrIDLib.dll` (plus its definitions pack)
  and `MediaInfo.dll`, neither of which exists in this environment or
  on CI (headless `windows-latest`). Same caveat as C069's own
  `automation` module, applied to a different kind of Win32 call.
- Tests: `dlllib::tests` (9), `dlllib::fake::tests` (3),
  `dlllib::win32::tests` (2, real only on `windows-latest` CI),
  `detection::mediainfo_scan::tests::composes_with_dlllib_scan_media_info`.

---

## C106 — Wise MSI rip (completes the capability)
**2026-08-22**

- **Added:** `extract::wise::wise_msi_rip`, a thin wrapper over C069's
  `automation::rip_exeinfo` with `RIP_EXEINFO_KEY_SEQUENCE` — choice 3
  of the Wise Installer disambiguation, the last unported piece of this
  capability.
- Capability marked `DONE`. Carries the same honesty caveat as C069:
  fake-backed tests prove the delegation and its arguments are right,
  not that the real Win32 backend actually finds an MSI to rip.
- Tests: `extract::wise::tests` (16, up from 14).

---

## C056 — 7z SFX-splitter branch (completes the capability)
**2026-08-22**

- **Added:** `extract::sevenzip::sfx_splitter_extract`, built on C069's
  `automation::GuiAutomation` infrastructure: launches `7ZSplit.exe`,
  clicks its two buttons (`Button8`, `Button1`), then polls for either
  the expected script file or an error window, closing any warning
  window along the way, before closing the splitter process.
- **New trait methods on `GuiAutomation`**: `win_close_by_title` (AutoIt's
  own `WinClose` accepts a title directly, no handle needed) and
  `file_exists` — a live filesystem predicate that has to live on the
  automation seam rather than as a plain pre-computed argument, since
  this branch's polling loop interleaves a real `FileExists` check with
  `WinExists` checks on every iteration.
- Reports whether/where a script file was found as data
  (`SfxSplitterOutcome`) rather than performing the final rename
  itself, keeping the same "decide, don't mutate the filesystem" split
  used everywhere else in this crate.
- **Preserves the same no-timeout `WinWait` hang-risk quirk already
  found for C044's PEiD scan**: `WinWait("7z SFX Archives splitter")`
  passes no timeout, modeled as `u64::MAX`.
- This was the last unported piece of C056 — capability marked `DONE`.
  Carries the same honesty caveat as C069: fake-backed tests prove the
  decision logic against the source, not that the real Win32 backend
  drives an actual 7ZSplit window.
- Tests: `extract::sevenzip::tests::sfx_splitter` (5, new submodule).

---

## C044 — PEiD scan (completes the capability)
**2026-08-22**

- **Added:** `detection::peid_scan::peid_scan`, built on C069's
  `automation::GuiAutomation` infrastructure: reproduces `Run`/
  `WinWait("PEiD v")`/the `Edit2`-polling loop (`is_scan_placeholder`
  checks empty or `"Scanning..."`, case-insensitively)/`WinClose`,
  reusing `detector_silence::PEID_KEY`/`PEID_SILENCE_VALUES`/
  `restore_plan` (C036) for the registry backup/restore.
- **A genuine, preserved hang-risk quirk**: unlike `OpenExeInfo`'s/
  `RipExeInfo`'s own `WinWait` calls, PEiD's own `WinWait("PEiD v")`
  passes no timeout argument at all — AutoIt's documented default (`0`)
  waits indefinitely, so a PEiD window that never appears hangs the
  source forever right there, before the polling loop (which *does*
  respect `$Timeout`) is ever reached. Modeled as `u64::MAX` rather than
  the caller's `timeout_ms` — the closest a concrete-`u64` API can get
  to "no timeout" — not "fixed" into a bounded wait.
- This was the last unported piece of C044 (the dispatch table was
  already `DONE`) — capability marked `DONE`. Carries the same honesty
  caveat as C069: fake-backed tests prove the decision logic against
  the source, not that the real Win32 backend drives an actual PEiD
  window.
- Tests: `detection::peid_scan::tests` (5).

---

## C042 — Exeinfo PE scan-only-mode GUI path (completes the capability)
**2026-08-22**

- **Added:** `detection::exeinfo_scan::scan_via_gui`, built on C069's new
  `automation::GuiAutomation` infrastructure: reproduces `OpenExeInfo()`,
  the `TEdit6`-polling loop (`is_scan_placeholder` checks the three
  "not ready yet" markers — empty, "File too big", "Antivirus may
  slow", "File corrupted or Buffer Error" — always polling at least
  once since the source's loop condition starts trivially true),
  appending `TEdit5`'s text, then `CloseExeInfo()`.
- This was the last unported piece of C042 — capability marked `DONE`.
  Carries the same honesty caveat as C069: fake-backed tests prove the
  decision logic against the source, not that the real Win32 backend
  drives an actual Exeinfo PE window.
- Tests: `detection::exeinfo_scan::tests` (13, up from 10).

---

## C069 — GUI-automation infrastructure + Exeinfo PE resource ripping
**2026-08-22**

- **Added:** `automation` — new Win32 GUI-automation infrastructure,
  mirroring the split `extract::runner::ExtractorRunner` already
  established for plain process spawning:
  - `automation::GuiAutomation` — a trait covering the Win32 primitives
    `OpenExeInfo`/`RipExeInfo`/`CloseExeInfo` need: registry
    read/write/delete, process spawn, window wait/show/close/exists,
    control click/send/get-text/get-handle, one listbox query,
    mouse move, sleep/timer, process-close.
  - `automation::fake::FakeGuiAutomation` — a test double recording
    every call with a virtual clock, so orchestration logic is testable
    without a real window to drive.
  - `automation::win32::Win32GuiAutomation` — a real, Windows-only
    (`#[cfg(windows)]`) implementation using the new `windows` crate
    dependency (scoped to `cfg(windows)` only — no effect on
    non-Windows builds).
  - `automation::keys`/`automation::control_spec` — small pure parsers
    for AutoIt's `ControlSend` key-sequence strings and
    `[CLASS:name; INSTANCE:n]`/`ClassNameNN` control specs.
  - `automation::open_exeinfo`/`close_exeinfo`/`rip_exeinfo` — the
    ported orchestration functions themselves (UniExtract.au3:1822-1917,
    C069), reusing `detector_silence`'s existing (C036) registry
    backup/restore logic rather than re-deriving it, and preserving two
    source quirks in the listbox-polling loop: both "End of file"
    spellings are checked (the second only when the first doesn't
    match), and the timeout check happens *after* both lookups each
    iteration.
- **Honesty note, not glossed over:** the fake-backed tests verify
  `open_exeinfo`/`close_exeinfo`/`rip_exeinfo`'s decision logic against
  the source line-by-line, the same confidence every other parity test
  in this crate has. They do **not** prove `Win32GuiAutomation` actually
  drives a real Exeinfo PE window correctly — that needs a live
  interactive Windows desktop with the real, licensed `exeinfope.exe`
  running, which doesn't exist in this development environment or on
  CI (headless `windows-latest`). The Win32 backend is type-checked
  against the `x86_64-pc-windows-gnu` target during development (this
  environment can't build native Windows binaries) and will compile for
  real on CI, but a green CI run only proves it compiles and links, not
  that it works.
- Capability C069 marked `DONE` on this basis — see the manifest row's
  own honesty note for the same caveat.
- Tests: `automation::tests` (8), `automation::fake::tests` (6),
  `automation::keys::tests` (6), `automation::control_spec::tests` (6),
  `automation::win32::tests` (1, exercises for real only on the
  `windows-latest` CI runner).

---

## C055/C180 — Game-archive BMS-script lookup (SQLite ambiguity resolved)
**2026-08-22**

- **The `_SQLite_GetTable` array-shape question that blocked C055/C077/
  C180 all session is resolved against AutoIt's own official
  documentation, not guessed:** `$aResult[0]` holds `(rows+1) *
  columns` (not the row count itself); `$aResult[1..columns]` are
  column headers; data follows in row-major order after that. Both
  queries in this capability select exactly one column, so `columns`
  is always `1`.
- **Added:** `bms` module — `sql_lookup_outcome` (`CheckGame`'s
  row-count gate + `_ArraySort`), `decide_game_choice`
  (`GUI_MethodSelectList`'s override/silent/prompt branching),
  `should_attempt_bms_extraction` (`BmsExtract`'s script-test
  classification), and the two literal SQL-query builders. Added
  `extract::qbms::gaup_probe_invocation`/`is_gaup_probe_failure` for
  `CheckGame`'s GAUP probe.
- **Two genuine quirks, preserved exactly:**
  - The source's `$aReturn[0] > 1` gate looks like it means "more than
    one candidate", but `_ArrayDelete($aReturn, 1)` (removing the
    header) never touches index `0` — the surviving value is still
    `rows + 1`, so the check is really `rows > 0`, "at least one
    candidate". `sql_lookup_outcome` applies the equivalent check
    directly rather than re-deriving it from a re-inflated total.
  - `GUI_MethodSelectList`'s override indexing is shifted by one
    relative to C053's `GUI_MethodSelect`: override `1` means "not a
    game archive" (the list's first position is a standard/cancel
    entry, not a real candidate), override `2` means the first real
    candidate, and so on. An out-of-range override degrades gracefully
    to the same path plain "no override" takes, rather than failing.
- **C180 resolved by cross-reference, not new mechanism:** the GAUP
  probe's "hang risk" traces to the already-`DONE` C026 finding (an
  unset `$Timeout` preference resolves to ~16.7 hours) combined with
  C150's already-`DONE` finding (this crate's runner has no timeout
  modeling for any call site) — nothing further to port under this ID.
- A pre-existing SQL-injection property of the source (`$fileext`/
  `$sName` spliced into SQL unescaped) is preserved exactly, not
  hardened into a rewrite that would no longer match the source's own
  query text.
- Tests: `bms::tests` (18), `extract::qbms::tests::gaup_probe_*` (3).

---

## C106 — Wise Installer 4-method fallback (everything but the MSI rip)
**2026-08-21**

- **Added:** `extract::wise` — the primary `e_wise_w.exe` invocation, the
  primary-result routing (fallback disambiguation vs. running the
  completion BAT), the five-choice `$iChoice` dispatch (reusing C053's
  `method_select::WISE_CANDIDATES`/`decide_method_selection`, not
  duplicated), invocation builders for choice 1 (WUN, plus its cleanup
  patterns), choice 2 (`/x` switch), and choice 4 (unzip with a 7-Zip
  fallback), plus the completion-BAT invocation.
- **Scope note:** choice 3 (ripping an embedded MSI via Exeinfo PE)
  drives real Win32 window/control automation (`RipExeInfo`) — the same
  deferred-GUI-subsystem blocker already found for C069/C044/C054's
  `$TYPE_MSCF` fallback/C056's SFX splitter. `WiseChoice::WiseMsi`
  reports the branch without modeling what happens inside it; its
  keystroke sequence is pinned as data (`RIP_EXEINFO_KEY_SEQUENCE`).
  Manifest row stays `REQUIRED`.
- Tests: `extract::wise::tests` (14), `extract::dispatch::tests::routes_wise_to_its_own_module`.

---

## C104 — ffmpeg per-stream extraction (completes the capability)
**2026-08-21**

- **All three cited call sites now covered.** `extract::ffmpeg` already
  had `Case $TYPE_AUDIO`/`Case $TYPE_VIDEO_CONVERT`/the `$TYPE_VIDEO`
  probe invocation from an earlier PR. **Added:** `$TYPE_VIDEO`'s
  per-stream extraction — parsing ffmpeg's raw `-i` stderr, splitting
  on the literal `"Stream"` token, regex-parsing each segment's header,
  classifying each stream (image-sequence split, h264-specific
  extraction, ordinary video/audio extraction, or unrecognized
  category), and building each extraction's output filename and full
  invocation.
- **Two genuine quirks, preserved exactly:**
  - `$iStreams` (computed as `$aStreams[0] - 2`) under-counts by one
    relative to the number of segments actually processed — the WMA
    shortcut check (`$fileext == "wma" And $iStreams < 2`) as a result
    actually fires for up to *two* real streams, not fewer than two,
    despite its own name.
  - `_MakeFFmpegCommand` strips every leading `-` from the output base
    name — but the gif/apng/webp image-sequence branch bypasses
    `_MakeFFmpegCommand` entirely, so its output filename is never
    dash-stripped. A real asymmetry, not an oversight to "fix".
- **Two different case-sensitivity rules in the same block**: the
  category check and the gif/apng/webp/h264 exact-codec checks use
  `==` (case-sensitive); the wmv/mpeg/vp8/flv/wma/vorbis/pcm
  remapping uses bare `StringInStr` (case-insensitive substring).
  Preserved exactly, not unified.
- **One documented, accepted limitation**: the header regex's
  stream-index group is exactly `\d:\d` — for a real double-digit
  stream index, PCRE backtracking would still match by truncating the
  captured index; this port's hand-written parser requires exactly
  one digit each side and returns `None` instead, rather than
  hand-replicating PCRE backtracking for an edge case rare enough not
  to justify the effort.
- Parity tests: `extract::ffmpeg::tests` (23 total, 20 new).
- PR [#408](https://github.com/baileyrd/rusty_extract/pull/408).

---

## C166 (partial) — Teelog dual-output mechanism
**2026-08-21**

- **Added:** `teelog::build_run_command` — the tee-pipe command
  composition (`_MakeCommand`'s result piped through `2>&1 | <tee>
  "<logfile>"`, only when tee output is enabled);
  `should_log_teelog_output` — the fold-into-run-log gate (only
  non-empty captured output is worth logging); `decide_size_poll_action`
  — the no-tee branch's own separate mechanism, a "reveal the
  previously-hidden window once, after 60 seconds of no output-folder
  growth" heuristic.
- **Preserved quirk**: `$bPatternSearch > -1` is a numeric comparison
  in the source, not a boolean check — `0` (`False`) and `1` (`True`)
  both satisfy it the same way; only an explicit `-1` disables it.
  Modeled as `pattern_search: i32` rather than "cleaned up" into a
  `bool`.
- **Already covered elsewhere, not duplicated**: the live "needs user
  input" text scan inside the tee branch's polling loop is
  `batch::needs_user_input` (C149); the captured output's own
  classification is `log_eval::evaluate_log` (C167 family).
- **Scope — still `REQUIRED`.** Process spawning, all `Win*`/
  `Process*`/`Timer*`/`Sleep` calls, and the teelog file's own
  open/read/close/delete are real process/GUI/filesystem work, out of
  scope under the same boundary as elsewhere in this port.
- Parity tests: `teelog::tests` (8).
- PR [#407](https://github.com/baileyrd/rusty_extract/pull/407).

---

## C174 — Per-extractor timeout handling
**2026-08-21**

- **Verified**: `$Timeout` is referenced from roughly 15 scattered
  call sites across the whole ~8200-line source, out of ~70 extractor
  `Case`s in `extract()`'s dispatch — confirming there is no global,
  systematic timeout mechanism. Most of those sites just `ExitLoop` a
  polling loop on expiry without any explicit termination — a
  genuinely different, non-uniform behavior per site, not something
  to generalize into one shared mechanism.
- **Added:** `extractor_timeout::arc_conv_timeout_outcome` — ports the
  one clean, explicit example the capability's citation names as
  representative (`$TYPE_ARC_CONV`): `WinWait`'s return value of `0`
  means the wait timed out, and this case terminates with
  `$STATUS_TIMEOUT` on that outcome. `WinWait` itself stays out of
  scope, the same deferred-GUI-subsystem boundary as elsewhere in
  this port.
- Parity tests: `extractor_timeout::tests` (3).
- PR [#406](https://github.com/baileyrd/rusty_extract/pull/406).

---

## C151 — Batch-completion summary
**2026-08-21**

- **Added:** `batch_runner::decide_batch_completion_actions` — ports
  `BatchQueuePop()`'s "queue empty" branch: open the accumulated
  scan-results log only if one exists, show the error-log summary
  dialog only if `errorlog.txt` has content, and relaunch the app
  only if the keep-open option is set. This was the sibling
  `pop_and_relaunch_next_batch_item`'s own doc comment had already
  flagged as the still-missing half of `BatchQueuePop()`.
- **A real, easy-to-conflate distinction, documented**: this branch's
  keep-open relaunch is gated on `$bOptKeepOpen` alone — a different,
  simpler condition from `terminate()`'s own unrelated keep-open
  relaunch (UniExtract.au3:4238), which also requires `$cmdline[0] =
  0` and a non-`$STATUS_SILENT` status. The two call sites read
  almost identically at a glance but aren't the same check.
- Parity tests: 4 new in `batch_runner::tests`.
- PR [#405](https://github.com/baileyrd/rusty_extract/pull/405).

---

## C149 — Batch stall on blocking user-input prompts
**2026-08-21**

- **Verified still present.** `batch::should_continue_batch` (earlier
  work) already covered this capability's `UniExtract.au3:4235-4237`
  citation — the `BatchQueuePop()` continuation gate. **Added:**
  `batch::needs_user_input` — ports the "needs user input" text match
  from the tee-log polling loop (`UniExtract.au3:4930-4933`), the
  remaining `4925-4958` half of this capability's citation.
- **Confirmed the no-timeout structure**: the polling loop itself
  (`While ProcessExists($run) ... WEnd`) has no timeout or give-up
  condition anywhere in it — it polls every ~100ms for as long as the
  subprocess is alive, however long that is. An unattended run blocked
  on exactly the kind of prompt `needs_user_input` detects stalls the
  whole batch chain indefinitely, matching the documented bug. Not
  fixed — verified and made testable, the same "known quirk" treatment
  already applied to C177/C178.
- **Verification**: all 8 literal needle strings checked present,
  case-insensitively, in the exact cited source range before writing
  tests — including `" replace"`'s leading space, preserved exactly
  rather than trimmed.
- Parity tests: 5 new in `batch::tests`.
- PR [#404](https://github.com/baileyrd/rusty_extract/pull/404).

---

## C179 (partial) — Free-space prompt response handling
**2026-08-21**

- **Added:** `free_space::decide_prompt_action` — extends the
  existing partial C179 coverage (arithmetic + silent-mode decision,
  PR #318) with what `HasFreeSpace()` does once a response to its
  abort/retry/ignore `MsgBox` is obtained: Retry re-runs the check,
  Abort removes the output directory only if this run created it
  then terminates silently.
- **Finding:** the source's own `Switch` has no `Case` for Ignore —
  or any unexpected `MsgBox` return value — so choosing it silently
  falls through with no action at all, letting extraction continue
  despite the insufficient-space warning.
- **Note:** `capability-manifest.md`'s C179 row had gone unupdated
  since PR #318 landed its partial coverage — backfilled alongside
  this addition.
- **Scope — still `REQUIRED`.** The `MsgBox` call itself remains
  real GUI, deferred under manifest row D001.
- Parity tests: 3 new in `free_space::tests` (13 total).
- PR [#403](https://github.com/baileyrd/rusty_extract/pull/403).

---

## C178 — TrID UNC-path detection reliability
**2026-08-21**

- **Verified still present**: `TridLib_Load`'s `TrID_LoadDefsPack`
  call (UniExtract.au3:949) marshals its directory argument as
  `"str"` — AutoIt's ANSI (single-byte, current-codepage) string
  type, not `"wstr"` (UTF-16). `TridLib_Analyse`'s own
  `TrID_SubmitFileA` call (UniExtract.au3:964 — the "A" suffix is
  itself the Win32 ANSI-variant naming convention) marshals the
  *scanned file's own path* the same way, and every other string
  parameter into `TrIDLib.dll` across the whole wrapper is `"str"`
  too — confirmed no `"wstr"` call site exists anywhere in it. This
  is consistent with the documented UNC-path TrID-detection-failure
  report: a UNC path, or any path containing a character outside the
  process's current ANSI codepage, can be silently corrupted by this
  narrowing conversion before `TrIDLib.dll` ever sees it.
- **Added:** `detection::trid_scan::trid_dll_string_marshalling` —
  makes the finding explicit and testable. The real `DllCall`s
  themselves stay out of scope, the same missing-FFI blocker as the
  rest of this module.
- Parity test: `detection::trid_scan::tests::trid_dll_calls_use_ansi_not_wide_string_marshalling`.
- PR [#402](https://github.com/baileyrd/rusty_extract/pull/402).

---

## C177 — Unicode-move bookkeeping loss on nested re-entry
**2026-08-21**

- **Verified still present**, against a directly downloaded copy of
  the raw source: `unpack()`'s post-unpack re-scan
  (UniExtract.au3:3633-3635 — `$file = $sPath` then a direct call to
  `StartExtraction()`) re-enters the same function the outer run
  started in. `StartExtraction()`'s very first statement
  (UniExtract.au3:378, unconditional on every entry, nested or not) is
  `$iUnicodeMode = False` — discarding whatever relocation bookkeeping
  the outer run had already set up, before the inner run does
  anything else. By the time `terminate()` runs, only the innermost
  re-entry's state is visible, so the outer run's relocated temp
  copy/rename is never cleaned up.
- **Added:** `unicode_relocation::start_extraction_reentry_resets_unicode_mode`
  — makes this fact explicit and testable rather than working around
  it. Under this migration's parity contract, a caller composing this
  port's pieces into a real orchestrator must replicate the reset on
  every `StartExtraction`-equivalent re-entry point to stay
  behaviorally faithful; threading the outer `UnicodeMode` through
  instead would be a silent behavior change, not a bug fix.
- Parity test: `unicode_relocation::tests::nested_start_extraction_reentry_discards_outer_unicode_mode`.
- PR [#401](https://github.com/baileyrd/rusty_extract/pull/401).

---

## C175/C176 — Non-ASCII and UNC-path input relocation
**2026-08-21**

- **Added:** `unicode_relocation::plan_relocation` — ports
  `MoveInputFileIfNecessary`'s full destination decision (UniExtract.au3:2218-2245):
  rename-in-place for a non-ASCII-directory unicode filename,
  relocate-to-`@TempDir` (keeping the name if only the directory was
  the problem, generating a fresh one otherwise) for a non-ASCII
  directory, abort with a warning if `@TempDir` itself is non-ASCII,
  and unconditional relocation for a UNC-reached file with no unicode
  involved at all. Also adds `decide_relocation_mode` (the Move-vs-Copy
  drive-letter check) and `should_reset_outdir` (the trailing
  output-directory check), completing the state machine that
  `unicode_relocation`'s existing `decide_unicode_reversion` (C159)
  already consumes.
- **`$sRegExAscii` is a misnomer, verified precisely.** Besides ASCII
  letters/digits/underscore and ordinary ASCII punctuation, it
  explicitly whitelists 20 accented Western-European Latin letters
  (both cases) plus `ß`/`°`/`²`/`³` — extracted programmatically from
  the source's own `\Q...\E` literal block (not retyped by hand) to
  rule out transcription error. A French or German filename passes
  outright; Cyrillic, CJK, or Greek does not. `ß`'s uppercase form
  (`ẞ`, U+1E9E) is deliberately left as a documented, unresolved
  minor uncertainty rather than guessed at.
- **A real interaction between C175 and C176, preserved exactly**:
  UNC-path relocation only ever supplies its own destination when the
  unicode check didn't already compute one — a file that's both
  unicode-named and UNC-reached uses the unicode branch's destination;
  UNC-ness contributes nothing extra in that case.
- **The multipart exemption** (`.*part\d+\.rar` / a 3+-digit run in
  the extension) applies after a destination is already computed,
  from either branch — reimplemented as manual character-scanning
  helpers rather than adding a `regex` crate dependency, consistent
  with this port's existing precedent (`batch.rs`).
- **Not modeled**: `_WinAPI_PathIsUNC`, `_TempFile` (real filesystem
  I/O generating a random unique name), `HasFreeSpace`, the actual
  `_FileMove`/`FileCopy` call, and `FilenameParse`.
- Parity tests: `unicode_relocation::tests` (20 new, 23 total).
- PR [#400](https://github.com/baileyrd/rusty_extract/pull/400).

---

## C038 — TrID scan orchestration (partial)
**2026-08-21**

- **Added:** `detection::trid_scan` — covers everything around
  `tridcompare` (already ported separately as C039): the
  scan-only-mode command-line invocation (`scan_invocation`,
  including the conditional `-v` verbose flag), the extract-mode
  per-result decision (`extract_result_action` — rename-on-extension
  only for the first result, dispatch to `tridcompare` capped at the
  first 3 candidates regardless of how many TrID actually returned),
  and the scan-only-mode output-line filter
  (`should_keep_scan_only_line`).
- **A reversal from C042's split, worth flagging explicitly**: there,
  extract mode was the portable command-line path and scan-only mode
  was GUI-blocked. Here it's the opposite — extract mode calls the
  blocked `TrIDLib.dll` functions directly (`TridLib_Analyse`/
  `TridLib_GetType`), while scan-only mode shells out to `trid.exe`
  via `FetchStdout`, whose command-line construction is fully
  portable.
- **Scope — the DLL calls and process execution stay out of this
  port.** `TridLib_Analyse`/`TridLib_GetType` are the same
  missing-FFI-infrastructure blocker already found for C045's
  MediaInfo calls; `FetchStdout` itself is real process execution.
  Manifest row C038 stays `REQUIRED`; this PR covers the decision
  logic and invocation-building around both paths.
- Parity tests: `detection::trid_scan::tests` (8).
- PR [#399](https://github.com/baileyrd/rusty_extract/pull/399).

---

## C045 — MediaInfo scan formatting (partial)
**2026-08-21**

- **Added:** `detection::mediainfo_scan::format_media_info` — ports
  `FileScan_MediaInfo`'s formatting pass exactly, given
  `MediaInfo_Inform`'s raw output text as input.
- **Genuinely easy-to-miss finding**: `StringSplit($x, @CRLF, 2)`
  omits `$STR_ENTIRESPLIT`, so `@CRLF` is treated as a *set* of
  individual delimiter characters — splitting at every lone `\r` and
  every lone `\n` separately, not the two-character `"\r\n"` sequence
  as one delimiter. A real `\r\n` line ending contains both, so this
  produces an empty string at every line boundary, roughly doubling
  the element count the `"< 10 lines"` threshold checks against.
  Reproduced exactly via `str::split(['\r', '\n'])` rather than the
  more "obvious" `split("\r\n")`. The spurious empty entries turn out
  harmless in the *formatted output* — they fail the `" : "` split
  and are silently skipped the same way genuinely blank lines are —
  but the threshold check genuinely operates on the doubled count.
- **Also preserved**: the case-sensitive `"Complete name"` field
  exclusion (the one exact-case comparison in an otherwise
  case-insensitive function), and a quiet truncation — a line with
  more than one `" : "` occurrence keeps only the first two split
  parts, silently dropping the rest, matching the source's own
  `$aSplit[1]`-only read.
- **Scope — the DLL scan itself stays out of this port.**
  `MediaInfo_New`/`_Open`/`_Inform`/`_Delete` are `DllCall`s into
  `MediaInfo.dll` — the same missing-FFI-infrastructure blocker
  already found for C038's TrIDLib. Manifest row C045 stays
  `REQUIRED`; this PR covers the formatting half only.
- Parity tests: `detection::mediainfo_scan::tests` (8).
- PR [#398](https://github.com/baileyrd/rusty_extract/pull/398).

---

## C042 — Exeinfo PE scan orchestration (partial)
**2026-08-21**

- **Added:** `detection::exeinfo_scan` — ports everything in
  `FileScan_ExeInfo` around its dispatch `Select` (already ported
  separately as C043): the extract-mode scan invocation, the
  corrupted/too-big/not-exe-or-dll/scan-only-mode branches, and the
  filename-echo strip.
- **Non-obvious finding**: `$bUseCmd` defaults to `$extract`, so in
  *extract mode* this scan is a plain command-line invocation
  (`RunWait` + reading a log file) — not GUI automation. Only the
  scan-only-mode path, and the corrupted/buffer-error retry back into
  it, drive PEiD-style GUI automation.
- **Scope — the GUI path stays out of this port.** The
  scan-only-mode branch (`OpenExeInfo`/`ControlGetText`/
  `CloseExeInfo`) and the corrupted-log retry into it are the same
  deferred-GUI-subsystem blocker already found for C044/C069/C106/
  C056's SFX splitter. Manifest row C042 stays `REQUIRED`; this PR
  covers the portable scan-orchestration half only.
- Parity tests: `detection::exeinfo_scan::tests` (10).
- PR [#397](https://github.com/baileyrd/rusty_extract/pull/397).

---

## C044 — PEiD match dispatch table (partial)
**2026-08-21**

- **Added:** `detection::peid_dispatch` — ports `FileScan_Peid`'s full
  `Select` (20 cases), matched top to bottom exactly as the source
  orders it, including its one case-sensitive comparison
  (`StringInStr($sFileType, "PEtite", 1)`, unique among this table's
  otherwise case-insensitive `Case`s).
- **A structural difference from the other three dispatch tables**:
  this `Select` has no `Case Else` — an unrecognized scan result takes
  no action at all, rather than falling through to a registry-mapping
  lookup. `Action::NoMatch` models that directly.
- **Scope — the actual scan stays out of this port.** `FileScan_Peid`
  drives PEiD through real Win32 GUI automation (`Run`/
  `WinWait("PEiD v")`/`ControlGetText`/`WinClose`, plus backing up and
  restoring three registry values around the call) — the same
  deferred-GUI-subsystem blocker already found for C069/C106/C054's
  `$TYPE_MSCF` fallback/C056's SFX splitter. Manifest row C044 stays
  `REQUIRED`; this PR covers the dispatch-table half only, the same
  partial-coverage shape as C056/C077.
- Parity tests: `detection::peid_dispatch::tests` (8).
- PR [#396](https://github.com/baileyrd/rusty_extract/pull/396).

---

## C040 — Unix `file` tool secondary detector
**2026-08-21**

- **Added:** `detection::unixfile_scan` — ports `FileScan_UnixFile`'s
  output cleanup (stripping the tool's own filename echo and CRLF
  sequences) and its post-scan branch: scan-only mode disables
  `$appendext` for text-like results (renaming a possibly misdetected
  text file is deliberately avoided) and returns without dispatching;
  extract mode hands off entirely to `filecompare`, already ported as
  `detection::file_dispatch::classify` (C041).
- **The "run automatically after TrID" half of this capability** is
  the unconditional call at UniExtract.au3:938, inside `FileScan_Trid`
  (C038) — not a decision this module makes, so it isn't re-modeled
  here; the module doc comment points to where it actually lives.
- **Not modeled:** `FetchStdout(...)`, real process I/O — the same
  external-process boundary already documented elsewhere in this
  port.
- Parity tests: `detection::unixfile_scan::tests` (6).
- PR [#395](https://github.com/baileyrd/rusty_extract/pull/395).

---

## C039 — TrID match dispatch table
**2026-08-21**

- **Added:** `detection::trid_dispatch` — ports `tridcompare`'s full
  `Select` (92 `Case` clauses, the largest of the three detector
  dispatch tables), matched top to bottom exactly as the source orders
  it.
- **The one case-sensitive comparison in the whole table:**
  `StringInStr($sFileType, '(.EXE)', 1)` passes AutoIt's explicit
  case-sensitive mode — unique among ~90 otherwise case-insensitive
  `Case`s in this function. Preserved exactly: this one comparison
  skips the shared lowercased matching closure every other needle
  goes through.
- **Two genuine dead-code quirks, preserved rather than fixed:**
  `"null bytes"` appears both in the disk-image group (checked first)
  and again in the "Not packed" group — the second mention never
  fires. Separately, `"Executable"` is a literal substring of `"ELF
  Executable and Linkable format"`, but that's harmless: the "Not
  packed" case containing it is checked well before the final generic
  executable case, so ELF binaries are correctly classified as
  not-packed rather than misrouted to `IsExe()`.
- **Also modeled**, not dropped as display text: the two-step
  "Windows Help File" case (`extract($TYPE_HLP, ..., "", False,
  True)` — `returnFail = True` returns `false` on failure rather than
  terminating, per `extract::completion::resolve_completion`
  (C054/C181) — falling through to `extract($TYPE_CHM, ...)`); the
  `CheckGame(False, False)` explicit-non-default-args case for
  "Broken Age package"; and the conditional
  `CreateRenamedCopy("z")`-then-`CheckTotalObserver` "InstallShield Z
  archive" case.
- **Verification:** all 134 literal needle strings checked present,
  case-insensitively, in the exact cited source range before writing
  tests — the same discipline applied for C041/C043.
- **Not modeled:** the exact `t('TERM_X')`-composed display text, and
  the internals of `CheckAlz`/`checkIE`/`CheckTotalObserver`/
  `CheckGame`/`CheckGarbro`/`check7z`/`IsExe` — each its own separate
  capability. `Case Else` falls through to
  `detection::detector_mapping::resolve_trid` (C051), already covered.
- Parity tests: `detection::trid_dispatch::tests` (16).
- PR [#394](https://github.com/baileyrd/rusty_extract/pull/394).

---

## C041 — Unix `file` tool match dispatch table
**2026-08-21**

- **Added:** `detection::file_dispatch` — ports `filecompare`'s full
  `Select` (~24 named cases) plus its two trailing not-packed/
  not-supported checks, matched top to bottom exactly as the source
  orders it. `classify` and `trailing_termination` are kept as two
  separate functions: unlike `exeinfo_dispatch` (C043), not every
  `Select` outcome here is guaranteed to terminate before the trailing
  checks — `CheckTotalObserver`/`check7z`/`CheckIso` are themselves
  detection cascades that can fail to dispatch and let control fall
  through, exactly as the source's straight-line function body does.
- **Genuine preserved quirk, not a modeling artifact:** `"POSIX tar
  archive"` (source line 1428) is unreachable in practice — `"tar
  archive"` always contains the earlier `"ar archive"` case (line
  1424) as a literal substring, so the more specific, later case never
  actually matches. Kept in the port (not merged away) to document the
  source's own dead case rather than silently dropping it.
- **Verification:** all 66 literal needle strings checked present,
  case-insensitively, in the exact cited source range before writing
  tests, the same discipline applied for C043.
- **Not modeled:** the exact `t('TERM_X')`-composed display text, and
  the internals of `CheckTotalObserver`/`check7z`/`CheckIso` — each
  its own separate capability. `Case Else` falls through to
  `detection::detector_mapping::resolve_file` (C051), already covered.
- Parity tests: `detection::file_dispatch::tests` (15).
- PR [#393](https://github.com/baileyrd/rusty_extract/pull/393).

---

## C043 — Exeinfo PE match dispatch table
**2026-08-21**

- **Added:** `detection::exeinfo_dispatch` — ports `FileScan_ExeInfo`'s
  full `Select` (~45 cases), matched top to bottom exactly as the
  source orders it, including its two explicit ordering comments
  (`InstallAware` must be checked before `InstallShield`; the `upx`
  case must be last before `Case Else`). `$TYPE_ISCRIPT` is
  `"installscript"` and `$TYPE_VSSFX_PATH` is `"vssfxpath"` — both
  verified against the source's own `Const` block rather than guessed.
- **Verification:** every one of the 64 literal needle strings this
  module matches against was checked present, case-insensitively,
  in the exact cited source range (UniExtract.au3:1141-1278) before
  writing a single test — catching transcription errors before they
  could hide behind a passing test suite.
- **Not modeled:** the exact `t('TERM_X')`-composed display text
  passed alongside most `extract(...)` calls (translation/formatting
  only), and the internals of `checkInno`/`checkIE`/`checkNSIS`/
  `CheckTotalObserver`/`unpack`/`BmsExtract` — each is its own
  separate capability or mechanism; `Action` only signals which one
  this dispatch reaches. `Case Else` falls through to
  `detection::detector_mapping::resolve_exeinfo` (C051), already
  covered, not duplicated.
- Parity tests: `detection::exeinfo_dispatch::tests` (13).
- PR [#392](https://github.com/baileyrd/rusty_extract/pull/392).

---

## C037 — Top-level detection cascade order (`StartExtraction`)
**2026-08-21**

- **Added:** `detection::cascade` — ports `StartExtraction()`'s step
  order right after `InitialCheckExt()` (C046), gated on file
  extension and extract-vs-scan-only mode. Every named step
  (`IsExe`/`FileScan_Trid`/`FileScan_ExeInfo`/`FileScan_MediaInfo`/
  `CheckIso`/`CheckGame`/`CheckTotalObserver`/`CheckExt`/`check7z`) is
  its own separately-tracked capability — this is purely the order and
  gating decision between them, matching the source comment's own
  framing ("order itself is behavior-significant").
- **Non-obvious finding:** in extract mode, an `.exe`/`.dll` file is
  delegated entirely to `IsExe()`, which — by the shape of its own
  body — never returns control back to `StartExtraction()` in that
  mode; it always terminates internally, one way or another. So none
  of `StartExtraction()`'s other steps ever run for such a file; only
  a *non*-exe file in extract mode reaches the full remaining cascade
  (`TridScan` → `ExeInfoScan` → `CheckIso` → `CheckGame` →
  `CheckTotalObserver` → `CheckExt` → `check7z` →
  `terminate($STATUS_UNKNOWNEXT)`).
- Parity tests: `detection::cascade::tests` (4), one per
  (exe-extension, extract-mode) combination.
- PR [#391](https://github.com/baileyrd/rusty_extract/pull/391).

---

## C046 — Extension-based pre-check (`InitialCheckExt`)
**2026-08-21**

- **Added:** `detection::initial_ext_check` — ports `InitialCheckExt`'s
  `Switch $fileext`, the pre-scan routing for extensions whose file
  magic is unreliable on its own (split-archive first parts, compound
  tar variants, disk images). Every routing target already has a home
  elsewhere in this port: the blind 7-Zip probe
  (`detection::sevenzip_probe`, C048), `extract::qbms`'s ISO detector
  (C077), and `extract::ctar`/`extract::sevenzip`/`extract::unity`
  (C181/C056/C054) — so this capability is purely the *order and
  grouping* the source's `Switch` decides, not new extraction logic.
- **Preserved quirk:** `{bin, cdi, mdf}` calls `CheckIso()` then the
  blind 7-Zip probe; `{cue, gdi, iso, mds}` calls them in the reverse
  order. Modeled as two distinct `Routing` variants
  (`DiskImageIsoThenCheck7z`/`Check7zThenDiskImageIso`) rather than one
  shared "disk image" outcome, so the order can't be silently lost.
- Parity tests: `detection::initial_ext_check::tests` (7).
- PR [#390](https://github.com/baileyrd/rusty_extract/pull/390).

---

## C077 — QuickBMS + WCX plugin fan-out (4 of 6 sites)
**2026-08-21**

- **Added:** `extract::qbms` — the three probe-then-classify detectors
  that each wrap a different WCX plugin around `quickbms.exe`
  (InstallExplorer, ISO/CD-DVD image, TotalObserver), plus the shared
  `Case $TYPE_QBMS` extraction invocation and its plugin-path
  resolution (`resolve_plugin_path`: a non-empty selector resolves
  against `bindir`, an empty one falls back to the dynamically-written
  `.bms` script path) and InstallExplorer-specific cleanup targets.
- **Not modeled:** `BmsExtract` and `CheckGame`'s GAUP probe (already
  backed off for C055/C180) — both load a game-specific `.bms` script
  via `_SQLite_GetTable`, the same array-indexing semantics already
  found genuinely ambiguous for C055. Guessing at its exact shape risks
  silently-wrong parity rather than an honest gap.
- Parity tests: `extract::qbms::tests` (9), plus
  `extract::dispatch::tests::routes_qbms_to_its_own_module`.
- PR [#389](https://github.com/baileyrd/rusty_extract/pull/389).

---

## C056 — 7-Zip integration (everything but the SFX splitter)
**2026-08-21**

- **Added:** `extract::sevenzip` — `Case $TYPE_7Z`'s main extraction
  invocation (password-conditional, reusing `password_search`'s
  already-`DONE` C160/C161 mechanism rather than duplicating it), the
  `@error`/`@extended` classification into `Status::MissingPart`/
  `Status::Password`, and the full RPM/Debian/gzip-family
  post-extraction branch tree.
- **Operator precision:** this one `Case` block mixes three different
  AutoIt case-sensitivity rules — bare `StringInStr` and
  `StringInStr(..., 0)` (both case-insensitive; `0` is the documented
  default), single-`=` string comparison (also case-insensitive, per
  this script's default `StringCompareMode`), and double-`==` (always
  case-sensitive, unconditionally). Verified each against AutoIt's own
  operator documentation before writing this module, given the
  `StringInStr` mistake just corrected in `extract::ctar`.
- `extract::dispatch::HARDCODED_CASES` gains a `"7z"` entry.
- **Scope — the SFX-splitter branch is genuinely GUI-blocked, manifest
  row stays `REQUIRED`.** When the file type mentions "SFX" but not
  "CAB", the source drives `7ZSplit.exe` via real Win32 window/control
  automation (`WinWait`/`ControlClick`) — the same blocker already
  found for C069/C106/C054's `$TYPE_MSCF`.
- Parity tests: `extract::sevenzip::tests` (17), plus
  `extract::dispatch::tests::routes_7z_to_its_own_module`.
- PR [#388](https://github.com/baileyrd/rusty_extract/pull/388).

---

## Correction — fix `extract::ctar`'s `StringInStr` case-sensitivity claim
**2026-08-21**

- **Fixed:** PR #386 (`extract::ctar`, C181) documented and implemented
  `is_nested_archive`'s marker check as case-*sensitive*, citing the
  source's explicit `StringInStr($return, "Listing archive:", 0)` third
  argument. That's backwards: `0` is AutoIt's `$STR_NOCASESENSE` — the
  documented *default* value — so an explicit `0` behaves identically to
  omitting the argument entirely, i.e. case-*insensitive*, the same as
  every other bare `StringInStr` call already correctly documented
  elsewhere in this port. Verified against AutoIt's own `StringInStr`
  documentation before making this fix, rather than trusting memory a
  second time.
- `is_nested_archive` now lowercases both sides before comparing; its
  test renamed and extended to assert the marker matches regardless of
  case.
- No other capability in this port was affected — this is the first
  place an *explicit* `StringInStr` casesense argument (rather than a
  bare/omitted one) was encountered and gotten wrong.

---

## C181 — Complete: `$TYPE_CTAR`'s same-tool nested-archive loop
**2026-08-21**

- **Added:** `extract::ctar` — `Case $TYPE_CTAR`'s full sequence
  (UniExtract.au3:2477-2497): decompress with 7-Zip, then for every
  newly-created file, probe it with `7z l` and, if 7-Zip recognizes it
  as a listable archive, extract it too and delete the original.
- **Not `extract()` recursion.** Unlike C054's six call sites, this
  loop re-invokes `7z` directly on each newly-discovered file — the
  same tool, not a dispatched type — so `extract::completion` doesn't
  apply. It's its own probe-then-classify shape, matching
  `detection::sevenzip_probe`'s.
- **Preserved quirk:** the old-files check (`Not StringInStr($oldfiles,
  $fname)`) is a raw substring search against `ReturnFiles`'s
  pipe-delimited snapshot string, not an exact-token comparison — a
  newly-extracted file whose name happens to be a substring of an old
  file's name (e.g. old `notes.txt.bak`, new `notes.txt`) is
  incorrectly treated as already existing and skipped. `is_newly_created`
  reproduces this exactly.
- `extract::dispatch::HARDCODED_CASES` gains a `"ctar"` entry.
- **C181 status: `DONE`.** All 3 cited ranges now covered.
- Parity tests: `extract::ctar::tests` (8), plus
  `extract::dispatch::tests::routes_ctar_to_its_own_module`.
- PR [#386](https://github.com/baileyrd/rusty_extract/pull/386).

---

## C054 — Recursive dispatch complete: `$TYPE_UNITYPACKAGE`, all 6 sites covered
**2026-08-21**

- **Added:** `extract::unity`'s module doc comment now documents `Case
  $TYPE_UNITYPACKAGE`'s recursive `extract($TYPE_7Z, -1, "gz", True,
  False)` call (UniExtract.au3:3173), the 6th and final of C054's cited
  sites. `extract::unity` (C121) was already `DONE` for the invocation
  and rename-loop halves of this capability — only the recursive
  dispatch piece was outstanding, matching its own already-honest
  "not modeled here" note.
- Shares `extract::forge`'s/`extract::raiu`'s `return_success=true,
  return_fail=false` shape, including the same `$outdir`-redirect-to-
  `$tempoutdir` dance `extract::forge` uses for its own recursive call
  — the primary extraction lands in the scratch directory, restored
  afterward. A failed recursive extraction terminates the whole
  process right there, same as `forge`/`raiu`.
- **C054 status: `DONE`.** All 6 cited call sites now have their
  recursion piece covered (`extract::actual`, `extract::forge`,
  `extract::raiu`, `extract::zip`, `extract::mscf`, `extract::unity`).
  `$TYPE_MSCF`'s own `RipExeInfo` fallback stays out of scope under
  C069's existing GUI-automation blocker — that's `$TYPE_MSCF`'s own
  base-extractor behavior, not part of the recursive-dispatch mechanism
  C054 itself describes.
- PR [#385](https://github.com/baileyrd/rusty_extract/pull/385).

---

## C054 — Recursive dispatch: `$TYPE_MSCF`
**2026-08-21**

- **Added:** `extract::mscf` — `Case $TYPE_MSCF`'s recursive
  `extract($TYPE_7Z, -1, "", False, True)` call (UniExtract.au3:2816),
  the 5th of C054's 6 cited sites. Shares `extract::zip`'s exact
  `return_success=false, return_fail=true` shape, so a successful
  recursive extraction terminates the process outright and everything
  after it in the `Case` only ever runs on failure — even more starkly
  than `$TYPE_ZIP`, since this `Case` doesn't even bother with an
  explicit `If` around the return value the way `$TYPE_ZIP` does; the
  termination side effect alone guarantees it. Matches the source's own
  comment: "If 7z fails, remove useless files and extract cab files
  from installer."
- Also ports `cab_extract_invocation` (the per-`.cab`-file extraction
  loop's `7z x` call), `RIP_EXEINFO_KEY_SEQUENCE` (the literal keystroke
  string this call site sends `RipExeInfo`), and
  `SUCCESS_CLEANUP_TARGETS`.
- **Scope — genuinely GUI-blocked, manifest row stays `REQUIRED`.**
  `RipExeInfo` drives Exeinfo PE via real Win32 window/control
  automation (`WinWait`/`ControlClick`/`ControlSend`, the same blocker
  already found for C069) — whether `$TYPE_MSCF`'s fallback path even
  finds an MSI to rip can't be determined in this port. Real filesystem
  I/O (`ReturnFiles`, `MoveFiles`, `DirRemove`, `_FileListToArrayRec`,
  `Cleanup`) stays out of scope too.
- `extract::dispatch::HARDCODED_CASES` gains an `"mscf"` entry.
- **C054 status:** 5 of 6 cited call sites now have their recursion
  piece covered. Only `$TYPE_UNITYPACKAGE` remains — no base extractor
  module exists for it yet.
- Parity tests: `extract::mscf::tests` (4), plus
  `extract::dispatch::tests::routes_mscf_to_its_own_module`.
- PR [#384](https://github.com/baileyrd/rusty_extract/pull/384).

---

## C054 — Recursive dispatch: `$TYPE_ZIP`
**2026-08-21**

- **Added:** `extract::zip` — `Case $TYPE_ZIP`'s recursive
  `extract($TYPE_7Z, -1, $additionalParameters, False, True)` call
  (UniExtract.au3:3385), the 4th of C054's 6 cited sites to land, and
  `should_show_extracting_message` (the `$arcdisp > -1` tray-message
  gate).
- **A genuine, non-obvious finding:** this call site's exact
  `(return_success=false, return_fail=true)` arguments mean a
  *successful* recursive 7-Zip extraction terminates the whole process
  outright — it never returns control to `Case $TYPE_ZIP` at all — while
  a *failed* one always returns `false`. So `If Not extract(...) Then`
  is effectively always true whenever it's reached: the Info-ZIP
  fallback isn't conditional in any meaningful sense, it runs whenever
  the recursive extraction fails and the case has already exited the
  process otherwise. Pinned down directly against
  `extract::completion::resolve_completion` rather than re-derived by
  hand.
- **Reused, not duplicated:** the fallback invocation itself
  (`_Run($zip & ' -x "' & $file & '"', ...)`) is already `extract::table`'s
  `unzip` entry, C109 (`DONE`).
- `extract::dispatch::HARDCODED_CASES` gains a `"zip"` entry routing to
  the new module.
- **C054 status:** 4 of 6 cited call sites now covered (`extract::actual`,
  `extract::forge`, `extract::raiu`, `extract::zip`); manifest row stays
  `REQUIRED` — `$TYPE_MSCF`/`$TYPE_UNITYPACKAGE` still need their own
  base extractor modules built first.
- Parity tests: `extract::zip::tests` (3), plus
  `extract::dispatch::tests::routes_zip_to_its_own_module`.
- PR [#383](https://github.com/baileyrd/rusty_extract/pull/383).

---

## C054/C181 — Recursive dispatch: the completion contract
**2026-08-21**

- **Added:** `extract::completion` — the completion contract every
  `extract()` call in the source goes through (UniExtract.au3:3408-3441),
  the piece that makes a recursive `extract($otherType, ...)` call
  return a plain boolean to its caller instead of terminating the whole
  process. `resolve_completion(result, return_success, return_fail)`
  decides `Terminate(Status)` vs. `Return(bool)`, composing directly
  with `result_heuristic::resolve_unknown_result` (C171) for the
  `$RESULT_UNKNOWN` case.
- **Preserved quirks:** `$RESULT_CANCELED` silently takes the exact same
  path as `$RESULT_SUCCESS` (the source's `Case $RESULT_CANCELED` is
  empty); `$RESULT_NOFREESPACE` always terminates regardless of
  `return_success`/`return_fail`, since its `terminate()` call sits
  inside the `Switch $success` block itself, before the return-flag
  gating is even reached.
- **Completed with the new mechanism:** `extract::actual` (`$TYPE_ACTUAL`,
  UniExtract.au3:2362) now ports its post-recursion branch decision
  (`decide_post_recursion_action`) in full; `extract::forge`
  (`$TYPE_FORGE`, :2543) and `extract::raiu` (`$TYPE_RAI`, :2999) now
  document their exact `return_success`/`return_fail` arguments and the
  real consequence of `return_fail = false` — a failed recursive call
  terminates the whole process, so their trailing cleanup/rename steps
  are not unconditional the way the source's linear layout might
  suggest.
- **Scope — genuinely partial, both rows stay REQUIRED.** C054 cites 6
  call sites; 3 (`$TYPE_MSCF`, `$TYPE_UNITYPACKAGE`, `$TYPE_ZIP`) have no
  base extractor module in this port yet, so their recursion piece can't
  be completed until that separate, not-yet-tracked work lands. C181's
  third citation (`$TYPE_CTAR`) turned out to be a different mechanism
  entirely — it loops re-invoking `7z` directly on newly-discovered
  nested archives, never calling `extract()` recursively — so
  `extract::completion` doesn't cover it at all.
- Parity tests: 5 in `extract::completion::tests`, 3 new in
  `extract::actual::tests` (`post_recursion_*`).
- PR [#382](https://github.com/baileyrd/rusty_extract/pull/382).

---

## C155 — Generic post-extraction cleanup utility
**2026-08-21**

- **Added:** `cleanup::split_wildcard_target` — the last remaining
  decision-logic gap in `Cleanup()`'s wildcard-expansion path
  (UniExtract.au3:3669-3673): splits a wildcard target at its *last*
  backslash into `(directory, pattern)`, the pair `_FileListToArray`
  is called with. `None` when there's no backslash with at least one
  character before it, matching the source's own `$iPos > 1` guard —
  under which the expansion, and the append to the working array,
  simply doesn't happen for that target.
- **Status change:** this closes the gap a prior pass on this module
  left documented as "not ported" (real filesystem I/O only remains:
  `_FileListToArray`'s actual directory listing, and the
  `DirRemove`/`FileDelete`/`_DirMove`/`_FileMove` calls) — every
  decision `Cleanup()` makes is now covered, so the manifest row moves
  REQUIRED → DONE rather than staying partial.
- 3 new parity tests in `cleanup::tests`, joining the module's existing
  11.
- PR [#381](https://github.com/baileyrd/rusty_extract/pull/381).

---

## C150 — InstallShield-cab batch crash risk
**2026-08-21**

- **Verified, still present:** `is6comp.exe`'s extraction call
  (`extract::iscab::is6comp_extract_invocation`, UniExtract.au3:2668-2674)
  runs via a blocking `RunWait` with no crash guard or timeout —
  `todo.txt:27`'s documented "might crash, stopping batch processing"
  bug. This port's own `extract::runner::CommandExtractorRunner` blocks
  unconditionally on every invocation it runs (`Command::output()`,
  uncapped) — `Invocation` carries no timeout/watchdog field for any
  call site to opt into, so the quirk isn't fixed, matching the source.
- Parity test: `extract::runner::tests::command_runner_blocks_until_exit_with_no_timeout_escape`
  demonstrates `run()` blocks until the child process actually exits and
  surfaces its real exit code, with no timeout escape hatch.
- PR [#380](https://github.com/baileyrd/rusty_extract/pull/380).

---

## C075 — InstallShield CAB fallback chain
**2026-08-21**

- **Added:** `extract::iscab` — `Case $TYPE_ISCAB`'s full fallback
  chain: try `unshield` (retrying once with `-O` if it asks for
  `unshield_file_save_old()`), and on failure route through C053's
  `ISCAB_CANDIDATES` disambiguation to a choice of `is6comp` (with its
  own listing-count probe and unshield fallback), `is5comp`, or `iscab`
  (list-then-extract, two invocations).
- **Preserved quirk:** choice 1's `is6comp`-listing-failed fallback to
  `unshield` is *not* a repeat of the initial attempt — it drops `-D 2`
  and uses the raw (non-UNIX-style) file path, reproduced as two
  distinct invocation builders rather than one reused across both call
  sites.
- **Scope note:** `HasPlugin` preconditions, `FileDelete`, and the
  final `Cleanup(...)` call on the success path are real filesystem
  I/O, out of scope — the two literal cleanup-target wildcards are
  pinned down as data (`SUCCESS_CLEANUP_TARGETS`) without re-implementing
  `cleanup`'s classification/expansion machinery.
- 14 parity tests in `extract::iscab::tests`, including a hand-rolled
  reproduction of `_StringBetween`/`StringStripWS(mode=8)`/`Number(...)`
  for `is6comp`'s file-count listing parse.
- PR [#379](https://github.com/baileyrd/rusty_extract/pull/379).

---

## C053 — Manual disambiguation
**2026-08-21**

- **Added:** `method_select` — `GUI_MethodSelect`'s pre-dialog branch
  selection (an active `$sMethodSelectOverride` wins outright; otherwise
  silent mode auto-picks choice `1`; otherwise the radio-button dialog),
  plus the five real candidate lists it dispatches (`$TYPE_ISCAB`,
  `$TYPE_ISEXE`, the MSI lessmsi-failure fallback, `$TYPE_MSP`, the
  `$TYPE_WISE` failure fallback) — kept as raw `(radio_label_key,
  method_key)` data rather than resolved/localized strings.
- **Scope note:** the GUI dialog itself stays out of scope, deferred
  under the GUI subsystem (manifest row D001) — this unblocks the
  several other capabilities documented as "user/caller-selectable per
  C053" (e.g. C075, C100).
- Parity tests: `method_select::tests::override_wins_regardless_of_silent_mode`,
  `silent_mode_without_override_auto_picks_first`,
  `interactive_mode_without_override_prompts`,
  `all_zero_override_is_treated_as_unset`, `candidate_lists_match_source`.
- PR [#378](https://github.com/baileyrd/rusty_extract/pull/378).

---

## C172 — Undifferentiated failure messaging
**2026-08-21**

- **Added:** `failure_message` — pins down `terminate()`'s `Case
  $STATUS_FAILED` quirk: `FAILURE_MESSAGE_KEY` is the single,
  unconditional `'EXTRACT_FAILED'` key every failure outcome shows,
  regardless of total vs. partial failure (there's no such distinction
  anywhere upstream of this branch — a documented open TODO,
  `todo.txt:48`); `failure_prompt_should_fire` ports the `Not
  $silentmode` half of `If Not $silentmode And Prompt(...) Then`.
- **Scope note:** `Prompt(...)`'s own `MsgBox` GUI dialog, and the
  save-log branch its return value governs (`ShellExecute(SaveLog(...))`),
  are out of scope — deferred under the GUI subsystem (manifest row
  D001), the same boundary `free_space::FreeSpaceOutcome::PromptInteractive`
  already documents.
- Parity tests: `failure_message::tests::failure_message_key_matches_source_literal`,
  `failure_prompt_fires_when_not_silent`, `failure_prompt_suppressed_in_silent_mode`.
- PR [#377](https://github.com/baileyrd/rusty_extract/pull/377).

**Tooling note:** this and the following entries were verified against a
directly downloaded copy of the raw source
(`raw.githubusercontent.com/.../UniExtract.au3`) rather than the
`WebFetch` tool, which truncates and occasionally reconstructs-rather-
than-quotes past a certain point in this ~8,200-line file — a more
reliable verification path discovered mid-loop, going forward from here.

---

## C099 — ThinApp/Thinstall extractor integration
**2026-08-21**

- **Added:** `extract::thinapp` — the `Extractor.exe` invocation `Case
  $TYPE_THINSTALL` makes on a relocated copy of the input:
  `<program> "<tempoutdir><filename_full>"`, run in `tempoutdir` with the
  window hidden.
- **Scope note:** the surrounding relocate-in/collect-out/cleanup steps
  (`DirCreate`, `_FileMove`, `MoveFiles`, `DirRemove`) are real filesystem
  I/O, out of scope here — the same split `extract::raiu` already uses for
  its own temp-directory unwrap step.
- **Triage correction:** the manifest's own summary calls this
  "GUI-automated"; direct source verification found a plain `_Run` call,
  no GUI scripting involved — worth double-checking a manifest summary
  against real source before assuming its complexity classification (the
  same lesson C098/swfextract surfaced).
- Parity tests: `extract::thinapp::tests::matches_source_invocation`,
  `relocated_file_path_matches_source_shape`.
- PR [#376](https://github.com/baileyrd/rusty_extract/pull/376).

---

## C036 — Third-party detector tool silencing
**2026-08-21**

- **Added:** `detector_silence` — decision logic for PEiD/Exeinfo PE
  registry silencing, ported from `FileScan_Peid`'s and
  `OpenExeInfo`/`CloseExeInfo`'s identical backup/overwrite/restore-or-delete
  pattern over `HKCU\Software\PEiD` (3 values) and `HKCU\Software\ExEi-pe`
  (9 values). `restore_plan` decides `Restore(...)` vs. `Delete` from a
  caller-supplied backup read — real `RegRead`/`RegWrite`/`RegDelete` is out
  of scope (this crate has no Win32 registry FFI yet), matching the existing
  `extract::plugin::resolve_plugin_ini_with` dependency-injection seam.
- **Preserved quirk:** only the *first* backed-up value's read failure
  decides whether the key existed at all; any other value's failed read
  silently restores as `0` (AutoIt's `RegRead` returns `""` on failure,
  coerced to `0` by a subsequent `REG_DWORD` write) — reproduced exactly
  rather than "fixed."
- Parity tests: `detector_silence::tests::silence_value_sets_match_source`,
  `restore_plan_restores_when_key_existed`,
  `restore_plan_deletes_when_key_did_not_exist`,
  `restore_plan_deletes_for_empty_backup`.
- PR [#375](https://github.com/baileyrd/rusty_extract/pull/375).

---

## C152 — Scan-only-mode short-circuit
**2026-08-21**

- **Added:** `entry_gate::scan_only_gate` — a third gate from
  `StartExtraction()`, joining C014/C015: `If Not $extract Then
  FileScan_MediaInfo(); terminate($STATUS_FILEINFO, $filenamefull,
  $fileext); EndIf`. Returns whether the gate fires, not the
  `Status::FileInfo` value itself — that variant also carries
  `silent_mode`/`filetype_identified` (C153/C154, already `DONE`), which
  this gate doesn't have.
- **Scope note:** `FileScan_MediaInfo`'s media-info scan is C045
  (REQUIRED, separate), out of scope here.
- Parity tests: `entry_gate::tests::scan_only_gate_fires_when_not_extracting`,
  `scan_only_gate_does_not_fire_when_extracting`.
- PR [#374](https://github.com/baileyrd/rusty_extract/pull/374).

---

## C098 — swfextract extractor integration
**2026-08-21**

- **Added:** `extract::table`'s `swf` builder + `FORMATS` row —
  `swfextract.exe`'s `-X -D <outdir> <file>`, in `file_dir`, hidden.
  Verified via exact-string search against the live source: `-X -D`
  extracts every content type (sounds, images, streams) in one pass,
  a single invocation rather than the per-target sequence this row's
  manifest summary might otherwise suggest — added directly to the
  existing table (49 rows now).
- Parity test: `extract::table::tests::swf_matches_source_invocation`.
- PR [#373](https://github.com/baileyrd/rusty_extract/pull/373).

---

## C143 — No centralized overwrite policy
**2026-08-21**

- **Added:** `extract::table::tests::no_format_injects_a_global_overwrite_flag`
  — sweeps all 48 already-ported formats in `extract::table`'s `FORMATS`
  table, asserting none of their built arguments include a general
  "overwrite all" flag (`-y`, `/y`, `-o+`, `-aoa`, etc.).
  **Deliberately preserved as a documented gap, not fixed**
  (`todo.txt:53`, UniExtract.au3:2269-3403 — the whole `extract()`
  switch): overwrite behavior is fully delegated to each helper binary's
  own default, matching upstream. This is a regression guard as much as
  a parity test — it sweeps every format in one pass so a future builder
  can't silently start injecting an overwrite flag without this test
  catching it.
- PR [#372](https://github.com/baileyrd/rusty_extract/pull/372).

---

## C161 — ACE excluded from the password-list trial
**2026-08-20**

- **Added:** `password_search::PASSWORD_TRIAL_EXTRACTOR_TYPES` — the
  exact set of extractor types `_FindArchivePassword` (C160) is wired
  into: 7z, DGCA, RAR. ACE is deliberately excluded — its `Case
  $TYPE_ACE` block (UniExtract.au3:2347) carries its own `; TODO:
  _FindArchivePassword` comment, left unimplemented in the source
  itself. **Deliberately preserved as a documented gap, not fixed**: a
  wrong or missing password just fails generically for this one
  extractor, matching upstream.
- Turns what was already true by omission (the set only ever named 7z,
  DGCA, RAR) into an explicit, tested invariant, so adding ACE here
  later would have to be a conscious decision, not an accident.
- Parity tests: `password_search::tests::ace_is_not_in_the_password_trial_set`,
  `password_trial_set_is_exactly_7z_dgca_and_rar`.
- PR [#371](https://github.com/baileyrd/rusty_extract/pull/371).

---

## C090 — PeaZip extractor integration
**2026-08-20**

- **Added:** `extract::table`'s `pea` builder + `FORMATS` row —
  `pea.exe`'s `x -dp"<outdir>" <file>`, in `file_dir`, hidden. Same
  argument shape as `freearc` (C071) — `pea.exe` mirrors the same
  `x`/`-dp"..."` CLI convention. Added directly to the existing table
  (48 rows now) rather than as a new standalone file, per the note
  already left on issue #157 when this repeat-of-the-original-pattern
  risk was first flagged.
- Parity test: `extract::table::tests::pea_matches_source_invocation`.
- PR [#370](https://github.com/baileyrd/rusty_extract/pull/370).

---

## C017 — `language` preference resolution
**2026-08-20**

- **Added:** `prefs::resolve_language` — ports the fallback chain after
  `LoadPref("language", $language, False)`:
  ```autoit
  If Not HasTranslation($language) Then
      $language = _WinAPI_GetLocaleInfo(_WinAPI_GetSystemDefaultUILanguage(), $LOCALE_SENGLANGUAGE)
      If Not HasTranslation($language) Then $language = _GetOSLanguage()
      If Not HasTranslation($language) Then $language = "English"
      SavePref('language', $language)
  EndIf
  ```
  A stored value with an installed translation is kept; otherwise falls
  through two OS-locale candidates in order, then `"English"`.
- **Scope note:** `has_translation`, the OS UI-language candidate, and
  the second OS-locale candidate are all caller-supplied (real
  filesystem/OS calls). Full translation catalogs beyond a default
  English set are out of scope (manifest row D006). Persisting the
  resolved value (`SavePref`) is the caller's job — this function only
  decides what the value should be.
- Parity tests: `prefs::tests::resolve_language_*` (4 tests).
- PR [#369](https://github.com/baileyrd/rusty_extract/pull/369).

---

## C014, C015 — Directory-input and second-instance entry gates
**2026-08-20**

- **Added:** `entry_gate::directory_input_gate` (C014) and
  `entry_gate::second_instance_gate` (C015), ported from the two gates
  right before/inside `StartExtraction()`:
  ```autoit
  $hMutex = _Singleton($name & " " & $sVersion, 1)
  If $hMutex = 0 And $extract Then
      AddToBatch()
      terminate($STATUS_SILENT)
  EndIf
  StartExtraction()

  Func StartExtraction()
      If _IsDirectory($file) Then
          GUI_Batch_AddDirectory($file)
          terminate($STATUS_BATCH)
      EndIf
      ; ...
  EndFunc
  ```
  Both return the resulting `Status` (C016's contract) rather than just a
  bool: a directory input → `Status::Batch`; a second instance that would
  extract → `Status::Silent` — both exit code 0.
- **Scope note:** routing/status decisions only. `_Singleton`'s OS mutex
  and `GUI_Batch_AddDirectory`'s per-file enumeration are caller-supplied
  — the latter's `GUI_` prefix marks it as deferred subsystem work (D001),
  the same convention already applied to `GUI_MethodSelectList` in C006.
- Parity tests: `entry_gate::tests::*` (5 tests).
- PR [#368](https://github.com/baileyrd/rusty_extract/pull/368).

---

## C011 — `/batch` flag detection
**2026-08-20**

- **Added:** `cli::has_batch_flag` — ports `_ArraySearch($cmdline,
  "/batch") > -1` (UniExtract.au3:687-690): case-insensitive presence
  check, following the same pattern as every other flag in `cli.rs`.
  Doesn't false-positive on `/batchclear` — `_ArraySearch` is an exact
  whole-element match, not a substring search.
- **Scope note:** flag detection only. The source's branch also calls
  `AddToBatch()` (real queue-file I/O) then `terminate($STATUS_SILENT)`
  — adding the queued entry is `batch::build_command_line` (C147,
  already `DONE`) and the caller's job.
- **Note on citations:** while tracking down this flag's exact source
  block (verified via exact-string search, not line-range guessing — see
  the retraction below), found that `/batch` sits directly adjacent to
  the `/type` block from C006, which a content-search fetch placed far
  from where the manifest cites it. The citations for this whole late
  region of `ParseCommandLine` (`/type`, `/batch`, `/close`, and by
  extension the already-`DONE` C007-C013 flags in `cli.rs`) may be
  systematically drifted from wherever they actually live in the current
  upstream source — worth its own dedicated citation-audit issue rather
  than folding an uncertain fix into this one. Not acted on further here.
- Parity test: `cli::tests::batch_flag_detected_case_insensitively`.
- PR [#367](https://github.com/baileyrd/rusty_extract/pull/367).

---

## Correction — retract PR #365's "corrected citations" claim
**2026-08-20**

- **Fixed:** PR #365 claimed to have "verified against the live source"
  and corrected C002/C003's AutoIt line citations from `643-649`/`640-642`
  to `635-646`/`635-639`. That verification used `WebFetch`'s reported
  line numbers on `UniExtract.au3` (a large file) as ground truth. A
  follow-up fetch for an unrelated, easily string-matched block (the
  `/type` override, C006) came back reported at line ~1272 — nowhere near
  the manifest's own `652-682` citation for that same block, and nowhere
  near where the earlier fetch had placed nearby content. That's not
  citation drift; it means the tool's line-counting on this file isn't
  reliable ground truth at all, so the earlier "correction" wasn't
  actually verified to the standard it claimed.
- `capability-manifest.md` and `dest_arg.rs`'s doc comment: reverted to
  the original `643-649`/`640-642` citations. The *behavior* ported in
  #365 is unaffected — it was implemented from the manifest's textual
  description, which this doesn't call into question, only the specific
  replacement line numbers.
- **Going forward:** citations in this repo trust the manifest's/issue's
  own pre-given line numbers rather than attempting to re-derive or
  "correct" them via `WebFetch`, which isn't reliable for pinpoint line
  numbers on a large file — only for confirming behavior via exact-string
  search, where a match is either found verbatim or not found at all.
- Bundled into the next capability's PR rather than a standalone one —
  C006, PR [#366](https://github.com/baileyrd/rusty_extract/pull/366).

---

## C006 — `/type[=value]` override routing
**2026-08-20**

- **Added:** `type_override::parse_type_override` — ports
  `ParseCommandLine()`'s `$cmdline[3]`-driven `/type` block
  (UniExtract.au3:652-682,420): no third argument, or one that doesn't
  start with `/type` → `None`; bare `/type` (nothing after `=`) →
  `PromptForType` (routes to the GUI candidate list, deferred subsystem
  D001); a recognized type name, or an unrecognized one with no trailing
  digits → `ArcType(value)` unchanged; an unrecognized value with a
  trailing digit run → `ArcTypeWithMethodSelect { arctype, method_select }`,
  the digits peeled off as a method-select index for C053's
  disambiguation.
- **Deliberately preserved quirk:** the peeled `arctype` prefix is never
  re-validated against the known-types list — `/type=kgb2` peels to
  `arctype: "kgb"` unconditionally, even though the source only checked
  whether the *whole* `"kgb2"` string was recognized, not the remainder.
  Not "fixed" into a re-validating version.
- **Scope note:** routing/parsing only. Building the candidate type-name
  list is real filesystem I/O (`_FileListToArray($defdir, "*.ini", ...)`),
  so it's caller-supplied (`known_types: &[&str]`), matching the seam
  `plugin::resolve_plugin_ini_with` uses for its own existence check. The
  GUI candidate-list prompt itself is out of scope (D001).
- Parity tests: `type_override::tests::*` (6 tests).
- PR [#366](https://github.com/baileyrd/rusty_extract/pull/366).

---

## C002, C003 — Destination-argument routing and scan-only mode
**2026-08-20**

- **Added:** `dest_arg::parse_destination_argument` — ports
  `ParseCommandLine()`'s `$iArgs > 1` block (UniExtract.au3:635-646): no
  second positional argument at all → `Absent`; `/scan` (case-insensitive)
  → `ScanOnly { extract: false, create_log: false }` (C003); `/sub`/`/last`
  pass through unresolved for `outdir::resolve_output_directory`
  (C004/C005) to handle; anything else is pre-resolved via the same
  `_PathFull` logic the file argument uses (C001,
  `file_arg::resolve_file_argument_path`) before that function ever sees
  it.
- ~~**Corrected citations:** ... verified both capabilities' AutoIt
  source-line citations against the live source ... Manifest rows updated
  to the verified `635-646`/`635-639` ranges.~~ **Retracted** — see the
  entry below. That "verification" trusted `WebFetch`'s line numbers on a
  large file, which turned out not to be trustworthy; the original
  `643-649`/`640-642` citations were restored.
- **Scope note:** C003 here covers only the mode-routing decision and its
  two flags — the "detect and report file type" half is C037-046
  (detection engine, not yet wired into this phase's `main.rs`) plus
  C153/C154 (scan-only output, already `DONE`).
- Parity tests: `dest_arg::tests::*` (4 tests).
- PR [#365](https://github.com/baileyrd/rusty_extract/pull/365).

---

## C001 — Positional file argument resolution
**2026-08-20**

- **Added:** `file_arg::resolve_file_argument_path`,
  `file_arg::validate_file_argument` — port the file-argument half of
  `ParseCommandLine()` (UniExtract.au3:625-628): resolve `$cmdline[1]`
  to a full path (drive-absolute/UNC pass through, else joined onto
  `cwd`), then map a missing file to `Status::InvalidFile` (exit code
  5).
- **Scope note:** the same documented `_PathFull` segment-normalization
  gap already noted for `outdir::resolve_output_directory` (C139) and
  `prefs::resolve_batchqueue_path`/`resolve_filescanlogfile_path`
  (C018/C019) — not modeled, `_PathFull` isn't defined in this port's
  source checkout. Not yet wired into `main.rs`; full CLI argument
  wiring is deferred to C006's `/type` override work, per `main.rs`'s
  own scope note.
- Parity tests: `file_arg::tests::*` (2 tests).
- PR [#364](https://github.com/baileyrd/rusty_extract/pull/364).

---

## Refactor — Fold `freearc`/`uharc` into the extractor table
**2026-08-19**

- **Changed:** `extract::freearc` (C071) and `extract::uharc` (C101) were
  missed by the collapse below — `freearc` matched the trivial pattern
  exactly and was dropped transcribing that batch's file list by hand;
  `uharc`'s 3-binary fallback chain (`UNUHARC06.EXE` → `UHARC04.EXE` →
  `UHARC02.EXE`) has two functions that were pure delegation and a third
  that only swaps the caller-supplied strings for 8.3 short paths, not the
  code shape. Both fold into `extract::table`, which grows from 43 to 47
  rows. No behavior change — every original assertion is reproduced.
- Parity tests: `extract::table::tests::freearc_matches_source_invocation`,
  `uharc_matches_source_invocation`, `uharc04_matches_source_invocation`,
  `uharc02_matches_source_invocation`.
- PR [#362](https://github.com/baileyrd/rusty_extract/pull/362).

---

## Refactor — Collapse 18 `def/*.ini`-only wrapper modules
**2026-08-19**

- **Changed:** `extract::alz`, `extract::arc`, `extract::adf`,
  `extract::bitrock`, `extract::bsa`, `extract::godot`, `extract::lbr`,
  `extract::lit`, `extract::mo`, `extract::pex`, `extract::qm`,
  `extract::rpgmvp`, `extract::sgb`, `extract::sim`, `extract::sit`,
  `extract::spoon`, `extract::utage`, `extract::uu` (capabilities C059,
  C060, C122-C137) each existed solely to `include_str!` one bundled
  `def/*.ini` file and assert it still parses and substitutes into the
  expected command line — confirmed none of them or their `BUNDLED_INI`
  consts had any caller outside their own file; the real runtime dispatch
  path (`extract::plugin::resolve_plugin_ini`) reads `def/*.ini` straight
  off disk by name at runtime. Collapsed into one table-driven regression
  test, `extract::plugin_defs_test`.
- Parity test:
  `extract::plugin_defs_test::bundled_plugin_only_inis_produce_source_matching_command_lines`
  (18 cases, one per format).
- PR [#361](https://github.com/baileyrd/rusty_extract/pull/361).

---

## Refactor — Collapse 43 single-invocation extractor modules
**2026-08-19**

- **Changed:** 43 of `src/extract/`'s formats (`extract::ace`,
  `extract::kgb`, `extract::rar`, `extract::unzip`, etc. — capabilities
  C057, C058, C062, C063, C065, C067, C068, C070, C072, C076, C078,
  C079-C088, C092-C097, C100, C102, C103, C107-C113, C115-C118, C120,
  C146) each followed one exact shape: a single
  `pub fn invocation(...) -> Invocation` shelling out to one helper binary
  with a fixed, non-branching argument pattern, plus one parity test.
  Collapsed into one data-driven module, `extract::table` — a shared
  `Ctx` input struct, one small builder fn per format, and a `FORMATS`
  table tying a format name to its builder and `UniExtract.au3`
  provenance. No behavior change: same `Invocation` output for the same
  inputs.
- `capability-manifest.md`: updated the **Evidence** test-path citation
  for every affected capability across all three PRs in this trilogy (63
  rows total).
- Parity tests: `extract::table::tests::*` (one per format, plus a
  table-shape sanity test).
- PR [#360](https://github.com/baileyrd/rusty_extract/pull/360).

---

## C074 — innounp/innoextract primary/fallback pair
**2026-08-18**

- **Added:** `extract::inno::unnp_invocation`,
  `extract::inno::innoextract_invocation`,
  `extract::inno::should_use_innoextract_fallback`,
  `extract::inno::rename_first_version_target` — port `Case
  $TYPE_INNO`'s two `_Run` invocations (UniExtract.au3:2616, 2649),
  the fallback gate deciding whether innoextract runs after innounp
  (`$additionalParameters Or $success == $RESULT_FAILED`), and the
  multi-version file rename target (`,1`/`,2`/`,3`-suffixed duplicate
  files Inno Setup can produce, `StringReplace(..., ",1", "", -1)`
  replacing every occurrence).
- **Scope note:** invocations and the fallback/rename decisions only.
  The multi-version file discovery (`_FileListToArrayRec`), cleanup
  lists, and `MoveFiles` output-restructuring calls are real
  filesystem I/O left to the caller.
- Parity tests: `extract::inno::tests::*` (6 tests).

---

## C091 — RAIU extractor integration
**2026-08-18**

- **Added:** `extract::raiu::invocation`,
  `extract::raiu::intermediate_file_path` — port `Case $TYPE_RAI`'s
  shell-unwrap invocation (UniExtract.au3:2994-2999): `RAIU.exe
  "<file>" "<tmp>"`, run in `filedir`, plus the intermediate
  unpacked-file path it's built for
  (`<tempoutdir><filename>_<term>.exe`, with the localized
  `t('TERM_UNPACKED')` term injected by the caller, same convention
  as C138/C073).
- **Scope note:** invocation only. The recursive re-dispatch into
  `extract($TYPE_INNO, ...)` on the unwrapped file (C181) and the
  `Cleanup`/`DirRemove` temp-file teardown are out of scope.
- Parity tests: `extract::raiu::tests::*` (2 tests).

---

## C076 — IsXunpack extractor integration
**2026-08-18**

- **Added:** `extract::isxunpack::invocation` — ports `Case
  $TYPE_ISEXE`'s isxunpack candidate invocation
  (UniExtract.au3:2711): `IsXunpack.exe "<outdir>\<filenamefull>"`,
  run in `outdir` with the window shown. This call site uses the raw
  AutoIt `Run()` built-in directly, not the crate's usual `_Run`
  wrapper — `Run()`'s own default `$show_flag` is `@SW_SHOWNORMAL`,
  mapped to `WindowMode::Show`, distinct from `_Run`'s minimized
  default used everywhere else this omits `$show_flag`.
- **Scope note:** invocation only. Reached only through
  `$TYPE_ISEXE`'s GUI candidate list (C053, deferred GUI, D001). The
  pre-move of the input file into `outdir`, the
  `WinWait`/`WinActivate`/`Send("{ENTER}")` keypress automation, and
  the final move back to `filedir` are out of scope.
- Parity tests: `extract::isxunpack::tests::matches_source_invocation`.

---

## C073 — helpdeco extractor integration, RTF reconstruction pass
**2026-08-18**

- **Added:** `extract::helpdeco::extract_invocation`,
  `extract::helpdeco::reconstruct_invocation`,
  `extract::helpdeco::should_reconstruct_rtf`,
  `extract::helpdeco::reconstructed_rtf_filename` — port `Case
  $TYPE_HLP`'s two `_Run` invocations (UniExtract.au3:2606-2610): a
  primary extraction pass, the `_DirGetSize($outdir, $initdirsize + 1) >
  $initdirsize` gate deciding whether the primary pass produced any
  output, and a conditional RTF reconstruction pass whose output
  filename embeds a translated term (injected by the caller, same
  convention as `outdir::default_output_subfolder`, C138).
- **Scope note:** invocations and the size-growth/filename decisions
  only. The `$tempoutdir` creation/removal and the `_FileMove` of the
  reconstructed RTF into `outdir` are real filesystem I/O, left to the
  caller.
- Parity tests: `extract::helpdeco::tests::*` (5 tests).

---

## C105 — Visionaire Engine v3 two-pass extraction
**2026-08-18**

- **Added:** `extract::visionaire3::generate_names_invocation`,
  `extract::visionaire3::extract_invocation` — port `Case
  $TYPE_VISIONAIRE3`'s two `_Run` invocations
  (UniExtract.au3:3310,3317,3321): a first pass generating
  `<outdir>\names.txt` from the archive's main `.vis` data file, and a
  second extraction pass that includes `/names=` when the first pass
  produced a non-empty `names.txt`, falling back to a bare `/force`
  otherwise.
- **Scope note:** invocation only. Locating the main `.vis` file
  (searches up to three parent directories, GUI candidate list when
  ambiguous — C053, deferred GUI, D001) and the `names.txt`
  existence/size checks driving which pass runs are real filesystem
  concerns left to the caller.
- Parity tests: `extract::visionaire3::tests::*` (3 tests).

---

## C100 — ttarchext extractor integration
**2026-08-18**

- **Added:** `extract::ttarch::invocation` — ports `Case
  $TYPE_TTARCH`'s game-selected extraction step
  (UniExtract.au3:3147): `ttarchext.exe -m <game_index> "<file>"
  "<outdir>"`, run in `outdir` with the window hidden.
- **Scope note:** invocation only. The preceding game-listing
  `FetchStdout` call and its GUI candidate list
  (`GUI_MethodSelectList`, C053, deferred GUI, D001) that resolve
  `game_index` are out of scope — composite, conditional dispatch, not
  registered in `extract::dispatch::HARDCODED_CASES`.
- Parity tests: `extract::ttarch::tests::matches_source_invocation`.

---

## C156 — Per-run temp output directory always removed
**2026-08-18**

- **Added:** `outdir::should_remove_temp_outdir` — ports the temp output
  directory's cleanup check at the top of `extract()`'s "success
  evaluation" section (UniExtract.au3:3412): runs before the `$success`
  `Switch` that decides success/failure/cancellation, so removal is
  never conditioned on the run's outcome — a still-present temp
  directory is always removed. Sits alongside C157's
  `should_remove_empty_created_outdir`, governing the separate
  `$tempoutdir` staging directory rather than the final `$outdir`
  destination.
- Parity tests: `outdir::tests::should_remove_temp_outdir_when_present`,
  `should_remove_temp_outdir_when_absent`.

---

## C159 — Unicode-relocation reversion at end of run
**2026-08-18**

- **Added:** `unicode_relocation::decide_unicode_reversion` — ports the
  end-of-run half of the `$iUnicodeMode` state machine: `terminate()`'s
  unconditional reversal (UniExtract.au3:4101-4114), run at the top of
  every `terminate()` call — success, failure, or anything else — never
  gated on the run's outcome, only on whether a relocation happened at
  all. Given `UnicodeMode::None`/`Move`/`Copy`, decides whether to move
  the working copy back, recycle it, or do nothing, and whether the
  output directory needs moving back too.
- **Scope note:** reversion decision only. The relocation itself —
  `MoveInputFileIfNecessary()` (UniExtract.au3:2218-2266), which decides
  *whether* to relocate and sets the mode this function consumes — is
  C175/C176, not yet ported.
- Parity tests: `unicode_relocation::tests::*` (3 tests, one per mode).

---

## C148 — Batch-item-per-process execution model
**2026-08-18**

- **Added:** `batch_runner::pop_and_relaunch_next_batch_item`,
  `BatchProcessLauncher`/`RealBatchProcessLauncher`/
  `FakeBatchProcessLauncher` — ports `BatchQueuePop()`'s "spawn the next
  queued item" branch (UniExtract.au3:4455-4460): pops the queue (via the
  already-shipped `batch::pop_batch_queue`) and spawns the current
  executable with the popped item's own arguments, a fresh, non-waited
  process (`Run()`, not `RunWait()`) — never a loop inside the running
  process. The chain is driven entirely by each spawned process's own
  exit reaching the C173 continuation check.
- **Added:** `batch_runner::split_batch_command_line` — reverses
  `batch::build_command_line`'s (C147) known output shape
  (`"<file>" [/sub|"<outdir>"|/scan] [/silent]`) back into argv, so the
  relaunch goes through `std::process::Command::args` like every other
  spawn in this crate, rather than needing a Windows-specific
  raw-command-line API.
- **Scope note:** closes the process-spawning mechanism itself. Still
  separate: the batch-queue *file* read/write I/O, the `/batch` CLI flag
  (C011), single-running-instance queuing (C015), and wiring this into
  `main.rs`'s composition root.
- Parity tests: `batch_runner::tests::*` (8 tests covering the argv
  tokenizer's three token shapes, the pop-and-relaunch happy path, the
  empty-queue no-op, and that a queue item is popped regardless of
  whether the spawn itself succeeds).

---

## C160 — Automated password-list trial
**2026-08-18**

- **Added:** `password_search::probe_shows_protected`,
  `password_search::find_password` — port `_FindArchivePassword`'s
  decision logic (UniExtract.au3:4847-4877): whether a probe command's
  captured output shows an archive is password-protected, and which
  password (if any) from a list satisfies a per-password test-command
  check. Used by 7-Zip (C056), DGCA, and RAR (C092) extraction
  (UniExtract.au3:2290, 2501, 3004).
- **Added:** `password_search::nth_line_from_end` — generalizes the
  `$iLine < 0` branch of `_StringGetLine` (UniExtract.au3:4577-4583),
  previously ported only for `$iLine = -1` as `log_eval`'s private
  `tail_for_password_prompt_search`, to arbitrary negative values since
  `_FindArchivePassword`'s own default `$iLine` is `-3`.
- **Scope note:** decision policy only, no process spawning — the source
  runs two shell commands per archive type via `FetchStdout`; this
  module doesn't run anything (that's `extract::runner`'s job).
  `probe_shows_protected` and `find_password` take already-captured
  output / a caller-supplied closure standing in for "run the test
  command", the same dependency-injection split already used by
  `extract::plugin::resolve_plugin_ini`/`resolve_plugin_ini_with`.
  Reading the password-list file (with its `@ScriptDir\passwords.txt`
  fallback) is left to the caller.
- Parity tests: `password_search::tests::*` (12 tests covering the
  `$iLine` line-selection fallback quirk, case-insensitive matching, and
  the password-trial loop's first-match/exhausted/empty-list outcomes).

---

## C087 — msiexec administrative-install fallback
**2026-08-18**

- **Added:** `extract::msiexec::invocation` — ports `$TYPE_MSI`'s
  "Administrative install" fallback candidate
  (UniExtract.au3:2882-2883): `msiexec.exe /a "<file>" /qb
  TARGETDIR="<outdir>"`, run in `filedir` with the window shown.
- **Scope note:** the source wraps the command string in
  `Warn_Execute(...)` — a gate on the `warnexecute` preference that
  either passes the command through unchanged or shows a confirmation
  dialog (deferred GUI, D001) and terminates silently. The command
  string itself is unaffected either way, so it's a separate concern
  from building this invocation. Reached only through `$TYPE_MSI`'s GUI
  candidate-list fallback (see C084's scope note for the full chain) —
  composite, conditional dispatch, not registered in
  `extract::dispatch::HARDCODED_CASES`.
- Parity tests: `extract::msiexec::tests::matches_source_invocation`.

---

## C086 — MsiX extractor integration
**2026-08-18**

- **Added:** `extract::msix::invocation` — ports the single command
  shape shared by three dispatch cases: `$TYPE_MSI`'s "MsiX" fallback
  candidate (UniExtract.au3:2862-2864), `$TYPE_MSM` merge modules
  (2887-2889), and `$TYPE_MSP`'s "MsiX" fallback candidate (2907-2908).
  All three build `<program> "<file>" /out "<outdir>" [/ext]`, run in
  `filedir` with the window minimized (none of the three `_Run` calls
  pass an explicit show-flag, so its `@SW_MINIMIZE` default applies).
  `append_ext` should be the resolved `appendext` preference (C022) for
  the `$TYPE_MSI`/`$TYPE_MSM` cases; `$TYPE_MSP`'s case is a literal,
  unconditional `/ext`.
- **Behavioral finding, flagged not asserted:** every other inline
  ternary this source embeds inside a `&` concatenation chain is
  parenthesized (UniExtract.au3:2291, 2502, 3005, 3599, 7881) — a
  consistent six-site idiom. The `$TYPE_MSM` line (2889) is the *only*
  one missing those parens: `... & '" ' & $appendext? '/ext': ''`.
  Depending on AutoIt's actual `?:` precedence relative to `&` (not
  conclusively verified here), `$appendext` may never actually gate
  `/ext` for this one case the way it does everywhere else. Documented
  as an open question for whoever wires up `$TYPE_MSM`'s real dispatch,
  not settled as fact.
- Parity tests: `extract::msix::tests::matches_source_invocation_without_ext`,
  `matches_source_invocation_with_ext`.

---

## C085 — jsMSIx extractor integration
**2026-08-18**

- **Added:** `extract::jsmsix::invocation` — ports `$TYPE_MSI`'s "jsMSI
  Unpacker" fallback candidate (UniExtract.au3:2858): `<program>
  "<file>|<outdir>"`, run in `filedir` with the window hidden.
- **Behavioral finding — the source's `"<file>"|"<outdir>"` collapses
  to one argument:** the literal command-line string has no whitespace
  anywhere between the two quoted segments and the `|` between them.
  Standard Windows command-line tokenization only splits on whitespace,
  so after dequoting this is a *single* argument, `<file>|<outdir>` —
  jsMSIx's own file/output-path delimiter convention, not a shell pipe.
- **Scope note:** reached only through `$TYPE_MSI`'s GUI candidate-list
  fallback (see C084's own scope note for the full chain) — composite,
  conditional dispatch, not registered in
  `extract::dispatch::HARDCODED_CASES`. Also out of scope: reading
  `<outdir>\MSI Unpack.log` and the follow-up `Cleanup("*.cab")` call,
  both real filesystem I/O.
- Parity tests: `extract::jsmsix::tests::matches_source_invocation`.

---

## C084 — lessmsi extractor integration
**2026-08-18**

- **Added:** `extract::lessmsi::invocation` — ports `$TYPE_MSI`'s
  primary extraction attempt (UniExtract.au3:2843-2845): `<program> x
  "<file>" "<outdir>\"`, run in `outdir` with the window hidden.
- **Scope note:** `$TYPE_MSI`'s full source behavior is a fallback
  chain — lessmsi first, then (only on failure or a missing .NET
  runtime) a GUI candidate list among jsMSI Unpacker (C085), MsiX
  (C086), an MSI Total Commander plugin path, and an administrative
  `msiexec.exe` install (C087). Like C075's InstallShield chain, that
  makes `msi` a composite/conditional dispatch case, not registered in
  `extract::dispatch::HARDCODED_CASES`. Also out of scope: the
  post-extraction `SourceDir`-flattening `MoveFiles` call and the
  `DirGetSize($outdir) == $initdirsize` success/failure check right
  after it — both real filesystem I/O.
- Parity tests: `extract::lessmsi::tests::matches_source_invocation`.

---

## C092 — UnRAR extractor integration
**2026-08-18**

- **Added:** `extract::rar::invocation` — ports the `$TYPE_RAR` dispatch
  case's extraction call (UniExtract.au3:3005): `<program> x -kb
  [-p<password>] "<file>"`, run in `outdir` with the window shown.
- **Scope note:** resolving `password` is `_FindArchivePassword()`'s job
  (C160's automated password-list trial, not yet ported) — this function
  takes an already-resolved `Option<&str>`. Interpreting the run's
  result (`@error = 3` → missing part, `@extended` → wrong password) is
  real process-execution outcome handling, out of scope for an
  invocation builder, matching every other extractor module in this
  crate. Not registered in `extract::dispatch::HARDCODED_CASES` for the
  same reason `rpa` isn't — its upstream password resolution doesn't
  exist yet.
- Parity tests: `extract::rar::tests::matches_source_invocation_without_password`,
  `matches_source_invocation_with_password`.

---

## C155 (partial) — Generic post-extraction cleanup utility
**2026-08-18**

- **Added:** `cleanup::resolve_target_path`, `classify_target`,
  `should_expand_wildcard`, `decide_cleanup_action` — port the pure
  decision core of `Cleanup()` (UniExtract.au3:3645-3703): mode gating
  (`$OPTION_KEEP` disables entirely), the delete-vs-move/folder-vs-file
  action selection, `$outdir`-prefixing path resolution, and wildcard
  target classification.
- **Behavioral finding — `$bIsFolderWildcard` is a silent no-op:** a
  target ending `\*` (meant to mean "everything inside this folder") is
  computed and excluded from the wildcard-expansion trigger, but then
  never read again — it isn't a real, existing directory either (a
  literal `...\*` path fails `_IsDirectory`), so it falls straight
  through to the *file* delete/move calls, which silently do nothing
  against a path that can't exist. The source logs "Cleanup:
  Deleting/Moving ..." for it regardless. Reproduced as-is by letting
  `decide_cleanup_action` take `is_folder` as a plain caller fact — a
  `FolderWildcard` target's real `is_folder = false` naturally lands on
  the same no-op outcome.
- **Behavioral finding — `$OPTION_ASK` silently means "move":** the
  source's action selector is a plain `If $iMode = $OPTION_DELETE Then
  ... Else ...`, so *any* non-Keep, non-Delete mode — in practice just
  `$OPTION_ASK` alongside the intended `$OPTION_MOVE` — takes the move
  branch, with no prompt ever shown from inside `Cleanup()` itself.
- **Scope note — partial, manifest row stays REQUIRED:** actually
  expanding a wildcard target into the files it matches
  (`_FileListToArray`) and the real `DirRemove`/`FileDelete`/
  `_DirMove`/`_FileMove` calls are real filesystem I/O, the caller's
  job.
- Parity tests: `cleanup::tests::resolve_target_path_leaves_outdir_prefixed_path_unchanged`,
  `resolve_target_path_prefixes_relative_name`,
  `resolve_target_path_containment_check_is_case_insensitive`,
  `classify_target_distinguishes_all_three_shapes`,
  `should_expand_wildcard_only_for_wildcard_kind`,
  `keep_mode_disables_cleanup`, `delete_mode_selects_folder_or_file_delete`,
  `move_and_ask_modes_both_select_move_action`.

---

## C164 — Debug-line accumulation
**2026-08-18**

- **Added:** `run_log::build_debug_line` — ports `Cout()`'s debug-line
  format (UniExtract.au3:5352-5357): `<datetime>:<msec>\t<msg>\r\n`.
- **Scope note:** the source appends every formatted line onto a
  growing `$sFullLog` string for the whole run's duration ("buffered in
  memory for the full run") — that accumulation is the caller's own
  trivial responsibility (one `push_str` per call), not modeled as its
  own function here. Reading the current date/time/millisecond
  (`GetDateTime()`, `@MSEC`) and `ConsoleWrite`ing the line when not
  running as a compiled executable are both real I/O, out of scope.
- Parity tests: `run_log::tests::build_debug_line_matches_source_format`,
  `build_debug_line_with_empty_message`.

---

## C167 — Output-log evaluation and warning extraction
**2026-08-18**

- **Added:** `log_eval::evaluate_log` — ports the whole of `EvaluateLog()`'s
  classification `ElseIf` chain (UniExtract.au3:4778-4825) as a single
  ordered decision (`LogEvalOutcome`), applying every branch in the
  source's exact priority order: password failure (C162) → cancellation
  → no free space → missing part → generic success → generic failure →
  overwrite-as-success (C144) → unclassified. Five new standalone branch
  predicates (`is_canceled_message`, `is_no_free_space_message`,
  `is_missing_part_message`, `is_generic_success_message`,
  `is_generic_failure_message`) back it, each independently documented
  and testable like the two that already shipped.
- **Added:** `log_eval::parse_warnings` — ports `ParseWarnings()`
  (UniExtract.au3:4832-4845): three tool-specific warning-block
  extractions (7-Zip's `WARNINGS:` block, UnRAR's checksum-error line, a
  generic `Open WARNING: ` line), each appended only when found.
- **Behavioral finding — mixed case sensitivity within one branch:** the
  generic-failure branch is the one place in this whole chain that isn't
  uniformly case-insensitive. Five substrings (`"err code("`,
  `"stacktrace"`, `"Write error: "`, `"ERROR: Wrong tag in package"`,
  `"unzip:  cannot find"`) pass AutoIt's explicit case-sensitive mode;
  the other nine default to case-insensitive like every other branch.
- **Behavioral finding — one nested `And` inside an otherwise all-`Or`
  chain:** `"Cannot create"` and `"No files to extract"` (both
  case-sensitive) must *both* appear for that pair to count toward a
  generic-failure match.
- **Scope note:** `_StringExtractAfter`/`_StringInStrGetLine` (the two
  project-local helpers `ParseWarnings` depends on) are reproduced via
  new private `extract_after`/`in_str_get_line` functions. Both bail out
  (`None`) rather than risk a misaligned slice on text whose
  case-folding would change its byte length — plain ASCII (what these
  helper-binary logs are in practice) is unaffected.
- Parity tests: `log_eval::tests::recognizes_all_three_cancellation_substrings`,
  `recognizes_both_no_free_space_substrings`,
  `recognizes_all_three_missing_part_substrings`,
  `recognizes_all_thirteen_generic_success_substrings`,
  `generic_failure_case_sensitive_substrings_require_exact_case`,
  `generic_failure_and_combo_requires_both_substrings`,
  `generic_failure_case_insensitive_substrings_match_any_case`,
  `evaluate_log_password_failure_takes_priority_over_success_text`,
  `evaluate_log_classifies_each_outcome`,
  `parse_warnings_extracts_7zip_warnings_block`,
  `parse_warnings_extracts_unrar_checksum_error_line`,
  `parse_warnings_extracts_open_warning_line`,
  `parse_warnings_collects_multiple_and_none`.

---

## C162 — Generic password-failure detection via output-text matching
**2026-08-18**

- **Added:** `log_eval::is_password_failure` — ports the invalid-password
  branch of `EvaluateLog()` (UniExtract.au3:4782-4787), the first and
  highest-priority arm of its classification chain: five substrings
  matched anywhere in the log, plus a sixth ("Enter password") matched
  only against the log's own custom-defined "last line" helper.
- **Behavioral finding — `_StringGetLine($sLog, -1)`'s off-by-one
  fallback:** that helper (a project-local function, not an AutoIt
  built-in) searches for the *second*-to-last `@CRLF`, not the last one.
  When a log has two or more line breaks, this correctly isolates the
  true last line; but when it has zero or exactly one line break,
  `StringInStr` can't find a second occurrence, returns 0, and the
  fallback (`StringTrimLeft($sString, 0)`) returns the **entire,
  unmodified log** — not just its one existing line. A two-line log
  therefore has "Enter password" searched across *both* lines, not just
  the second one. Reproduced exactly via `tail_for_password_prompt_search`.
- Parity tests: `log_eval::tests::recognizes_all_five_whole_log_password_substrings`,
  `recognizes_enter_password_on_true_last_line`,
  `does_not_recognize_enter_password_on_earlier_line_of_long_log`,
  `two_line_log_searches_whole_text_not_just_last_line`,
  `single_line_log_searches_whole_text`,
  `password_failure_matches_case_insensitively`,
  `does_not_match_unrelated_password_log_text`.

---

## C165 — Per-run log file naming/location/encoding
**2026-08-18**

- **Added:** `run_log::build_log_file_name` — ports `SaveLog()`'s log
  file name construction (UniExtract.au3:4765-4768): `<logdir>`
  `YYYY-MM-DD_HH-MM-SS_` `[STATUS_UPPER]` `[_<filename>.<ext>]` `.log`.
  The status marker is omitted entirely on a successful run.
- **Behavioral finding — two unconditional underscores, not
  separators:** neither the `"_"` right after the timestamp nor the
  `"_"` right before the file segment only appears when needed to join
  two non-empty pieces — both are always emitted. A successful run with
  a file therefore gets a doubled `"__"` between the timestamp and the
  file name (no status marker consumed the first one); a successful run
  with no file ends with that first `"_"` immediately before `.log` —
  e.g. `...12-00-00_.log`. Neither is a typo in this port.
- **Scope note:** reading the current date/time (`@YEAR`/`@MON`/etc.)
  is real I/O and stays the caller's job (matching this crate's existing
  `datetime` convention from C169's `build_error_log_line`), as is
  `$logdir`'s own trivial resolution (`$settingsdir & "\log\"`) and the
  UTF-16 file write itself (`FileOpen($FO_UNICODE + ...)`)  — none of
  those involve any further decision logic beyond what this function
  already formats.
- Parity tests: `run_log::tests::build_log_file_name_failed_run_includes_status_and_file`,
  `build_log_file_name_success_run_omits_status_marker`,
  `build_log_file_name_success_no_file_reproduces_trailing_underscore_quirk`,
  `build_log_file_name_failed_no_file_includes_status_only`.

---

## C179 (partial) — Free-space check
**2026-08-18**

- **Added:** `free_space::measure_free_space`, `has_enough_free_space`,
  `decide_free_space_outcome` — port the pure decision core of
  `HasFreeSpace()` (UniExtract.au3:3782-3808): whether a measured drive
  has enough free space for the file being extracted, and whether that
  should terminate the run outright (silent mode) or hand off to an
  interactive prompt.
- **Behavioral finding — rounding order:** the megabyte conversion
  rounds to 2 decimals *before* the modifier is applied
  (`Round(FileGetSize($file) / 1048576, 2) * $fModifier`), not after —
  reproduced exactly rather than rounding the final product.
- **Behavioral finding — exit-status quirk:** the silent-mode
  termination call is `terminate($STATUS_FAILED, $filenamefull,
  $STATUS_NOFREESPACE, $sMsg)` — the actual exit status passed is
  `$STATUS_FAILED`, not `$STATUS_NOFREESPACE`; `$STATUS_NOFREESPACE` is
  stuffed into the `$arctype` parameter slot purely for message display.
  This is a distinct code path from the post-extraction `Case
  $RESULT_NOFREESPACE: terminate($STATUS_NOFREESPACE)` branch
  (`result_heuristic`'s neighboring capability), which *does* use the
  real `$STATUS_NOFREESPACE` exit status — documented explicitly so the
  two aren't conflated.
- **Scope note — partial, manifest row stays REQUIRED:** the interactive
  abort/retry/ignore `MsgBox` prompt is GUI (deferred, manifest row
  D001) and isn't implemented; neither is the source's preliminary
  walk-up-to-an-existing-directory-ancestor step, which is real
  filesystem I/O entangled with path manipulation. Only the free-space
  arithmetic and the silent-mode termination decision are covered.
- Parity tests: `free_space::tests::measure_free_space_rounds_before_applying_modifier`,
  `has_enough_free_space_boundary_is_inclusive`,
  `disabled_check_always_continues`, `enough_space_always_continues`,
  `not_enough_space_silent_mode_terminates`,
  `not_enough_space_interactive_prompts`.

---

## C154 — Scan-only silent-mode file output
**2026-08-18**

- **Added:** `filetype_report::build_scan_log_entry` — ports the
  `$STATUS_FILEINFO`/silent-mode branch of `terminate()`
  (UniExtract.au3:4139-4142): the block appended to the file-scan log
  when a scan-only run finishes in silent mode — file path, blank line,
  filetype text, a 60-dash separator line, each `\r\n`-terminated.
  `filetype` is exactly `format_filetype_results`'s `with_header = false`
  output (C153).
- **Scope note:** opening `$fileScanLogFile` in append mode
  (`$FO_CREATEPATH + $FO_APPEND`) is real filesystem I/O and stays the
  caller's job — that append-per-call is also how results accumulate
  across a batch run; this function only builds the one block for a
  single item.
- Parity tests: `filetype_report::tests::build_scan_log_entry_matches_source_format`,
  `build_scan_log_entry_separator_is_sixty_dashes`.

---

## C153 — Scan-only full-detail output
**2026-08-18**

- **Added:** `filetype_report::format_filetype_results` — ports
  `_FiletypeGet($bHeader = True, $iWidth = 50)`
  (UniExtract.au3:5292-5313): concatenates every scanner's
  `($sScanner, $sType)` result (TrID, Unix `file`, Exeinfo PE, PEiD,
  MediaInfo) into one report string, entries separated by a blank line.
- **Behavioral finding — two distinct output shapes, same function:**
  `with_header = false` (used to build the plain `$sFileType` value both
  `terminate()` and C154's silent-mode scan log consume) yields only
  concatenated type text, no scanner names anywhere; `with_header = true`
  (used for on-screen display) centers each scanner name between dashes
  sized to `Floor((width - len(" name ")) / 2)` per side.
- **Behavioral finding — `Floor`, not round, and no negative-padding
  guard needed:** an odd remainder always shortens the header by one
  character rather than rounding up; a scanner name longer than `width`
  drives the computed padding negative, which reproduces as no dashes at
  all (matching the standard `_StringRepeat` UDF's own non-positive-count
  guard) rather than a truncated name or a crash. `width <= 0` skips
  padding entirely and uses the bare scanner name.
- Parity tests: `filetype_report::tests::no_header_joins_type_text_only`,
  `no_header_single_entry_has_no_leading_separator`,
  `empty_entries_produce_empty_string`,
  `header_centers_scanner_name_with_floor_division`,
  `header_floor_rounding_on_odd_remainder`,
  `header_name_longer_than_width_has_no_padding`,
  `zero_width_uses_bare_scanner_name`,
  `multiple_header_entries_are_separated`.

---

## C171 — Generic success/failure fallback heuristic
**2026-08-18**

- **Added:** `result_heuristic::resolve_unknown_result` — ports the
  `$RESULT_UNKNOWN` arm of the success-evaluation `Switch $success` in
  `extract()` (UniExtract.au3:3415-3430). When an extractor case never
  explicitly sets `$success`, the fallback compares the output
  directory's size and modification time against a before-extraction
  snapshot: no size growth (when a real size measurement was taken) or
  an unchanged mtime is enough to reclassify the run as failed.
- **Behavioral finding — cheap-measurement sentinel:** `initdirsize ==
  -1` reproduces `_DirGetSize`'s own `$return` default, returned instead
  of an actual (expensive) `DirGetSize` call when the output directory is
  a drive root with more than 4 GB already in use. The source's own
  `$initdirsize > -1` guard exists specifically to skip the
  size-comparison half of the heuristic in that case, leaving only the
  mtime comparison.
- **Behavioral finding — ace/exe carve-out:** `$arctype = "ace" And
  $fileext = "exe"` is a genuinely different code path, not just "not
  failed": the source `Return False`s out of the entire `extract()`
  function right there, bypassing the normal `terminate()`/success flow
  this heuristic otherwise feeds into. Modeled as its own
  `AceExeEarlyAbort` outcome rather than folding it into `TreatAsSuccess`.
- **Scope note:** capturing the before/after directory size and mtime
  snapshots is real filesystem I/O (`_DirGetSize`, `FileGetTime`) and
  stays the caller's job — this ports the pure decision over
  already-known values only.
- Parity tests: `result_heuristic::tests::no_size_growth_resolves_to_failed`,
  `unchanged_mtime_alone_resolves_to_failed`,
  `growth_and_mtime_change_treated_as_success`,
  `negative_one_initdirsize_skips_size_check`,
  `ace_exe_case_early_aborts_instead_of_failing`,
  `ace_exe_case_with_evidence_of_output_treats_as_success`.

---

## C173 — Batch continues past an ordinary per-item failure
**2026-08-18**

- **Added:** `batch::should_continue_batch` — ports the batch-continuation
  gate inside `terminate()` (UniExtract.au3:4235-4237): `If $batchEnabled
  = 1 And $status <> $STATUS_SILENT Then BatchQueuePop()`. A `Failed`
  status (or any other ordinary terminal status) still satisfies `status
  != Silent`, so a normal, clean-exit per-item failure does **not** stop
  the chain — only `$STATUS_SILENT` (used when the GUI itself has been
  closed/aborted) does.
- **Scope note:** this is the condition `pop_batch_queue`'s own doc
  comment (C148) already described in prose — the next process's own
  `terminate()` call reaching this check before popping again. This
  function is that check itself, now ported and tested directly. Not
  modeled: whether an extraction *hangs or crashes* instead of exiting
  cleanly, which is a process-liveness concern for this port's
  not-yet-built runtime orchestration, not something a status-comparison
  function can observe.
- Parity tests: `batch::tests::should_continue_batch_continues_on_ordinary_statuses`,
  `should_continue_batch_stops_on_silent_status`,
  `should_continue_batch_stops_when_batch_disabled`.

---

## C169, C170 — Run-log write policy
**2026-08-18**

- **Added:** `run_log::should_append_error_log`/`build_error_log_line`
  (C169) — port the batch-mode error-log append inside `terminate()`
  (UniExtract.au3:4216-4218): a line is appended only when exiting
  non-zero, in silent mode, on an extraction run, formatted as `<datetime>
  <name> (<STATUS-UPPERCASE>) - <arctype>\r\n` (`filenamefull` preferred
  over `fname` when non-empty).
- **Added:** `run_log::should_save_log` (C170) — ports the guard deciding
  whether `SaveLog()` actually writes a log file for this run
  (UniExtract.au3:4231-4233): enabled, not already saved, and the status
  isn't one of five suppressed terminal statuses (`Silent`, `Syntax`,
  `FileInfo`, `NotPacked`, `Batch`) — unless it's `FileInfo` in silent
  mode, which bypasses both gates.
- **Behavioral finding — operator precedence, easy to misread:** the
  source's guard is `$bOptCreateLog And Not $bLogSaved And Not (...) Or
  ($status = $STATUS_FILEINFO And $silentmode)`. AutoIt's `And` binds
  tighter than `Or` (as in most languages), so this parses as `(A And B
  And C) Or D`, not `A And B And (C Or D)` — the silent-mode `FileInfo`
  branch is a genuine unconditional bypass of the enabled/already-saved
  gates, not just an exception folded into the suppressed-status check.
  Asserted directly by its own test.
- Parity tests: `run_log::tests::should_append_error_log_requires_all_three_conditions`,
  `build_error_log_line_prefers_filenamefull_when_present`,
  `build_error_log_line_falls_back_to_fname_when_filenamefull_empty`,
  `should_save_log_writes_for_ordinary_enabled_case`,
  `should_save_log_suppresses_five_terminal_statuses`,
  `should_save_log_fileinfo_in_silent_mode_bypasses_other_gates`,
  `should_save_log_respects_enabled_and_already_saved_gates`.

## C104 (partial) — ffmpeg audio conversion + video-convert + stream probe
**2026-08-18**

- **Added:** `extract::ffmpeg::audio_invocation` — `Case $TYPE_AUDIO`
  (UniExtract.au3:2414-2416): `<program> -i "<file>" "<stem>.wav"`, run
  in `outdir` with the window hidden.
- **Added:** `extract::ffmpeg::video_convert_invocation` — `Case
  $TYPE_VIDEO_CONVERT` (UniExtract.au3:3288): `<program> -i "<file>"
  "<stem>.mp4"`, run in `outdir` with the window hidden.
- **Added:** `extract::ffmpeg::probe_invocation` — `Case $TYPE_VIDEO`'s
  stream-info probe (UniExtract.au3:3220-3221): `<program> -i "<file>"`,
  the same probe-then-classify shape as `detection::sevenzip_probe`/
  `detection::alz_probe`/`detection::arj_probe`.
- **Registered** in `extract::dispatch::HARDCODED_CASES`: `"audio"` and
  `"videoconv"` → `extract::ffmpeg` (both are complete, single-invocation
  cases). `"video"` is deliberately **not** registered — see below.
- **Scope note — capability C104 stays `REQUIRED`, not closed by this
  PR.** `$TYPE_VIDEO`'s actual per-stream extraction — parsing ffmpeg's
  raw `-i` stdout via a regex-based pattern match
  (UniExtract.au3:3235-3236) to discover each stream's codec/type/index,
  then dynamically building one extraction command per stream via
  `_MakeFFmpegCommand` (UniExtract.au3:5118-5126) — is a real, substantial
  piece of this capability that isn't ported yet, documented as a gap
  rather than silently dropped. C104's own manifest description covers
  all three call sites, so the row can't be marked `DONE` until that
  parsing lands too — the same precedent C140 set (ported across two PRs
  before being marked `DONE`).
- Parity tests: `extract::ffmpeg::tests::audio_invocation_matches_source`,
  `video_convert_invocation_matches_source`,
  `probe_invocation_matches_source`.

## C115 — Advanced Installer self-extraction
**2026-08-18**

- **Added:** `extract::ai::invocation` — builds the self-extracting
  Advanced Installer command UniExtract2's `Case $TYPE_AI`
  (UniExtract.au3:2385-2390) makes: `<file> /extract:<outdir>`, run in
  `outdir` with a normally shown window.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"ai"` →
  `extract::ai`).
- **Behavioral finding:** run via `ShellExecute`, not
  `_Run`/`Run`/`RunWait` — needed (per the source's own comment on this
  exact case) so the OS can raise a UAC elevation prompt, the same
  reasoning C116-C118 (`extract::ei`/`extract::fead`/`extract::superdat`)
  already document. Its `$iShowFlag` parameter defaults to
  `@SW_SHOWNORMAL` when omitted here, mapped to `WindowMode::Show`.
- **Scope note:** the preceding `Warn_Execute(...)` confirmation gate
  (`warnexecute` preference, C023, deferred GUI subsystem D001) and the
  trailing `ProcessWait`/`ProcessWaitClose` calls that wait for the
  self-extractor to finish are not modeled — separate runtime behavior.
- Parity test: `extract::ai::tests::matches_source_invocation`.

## C114 — Actual Installer inner-blob handling
**2026-08-18**

- **Added:** `extract::actual::meta_invocation` — the metadata-extraction
  invocation UniExtract2's `Case $TYPE_ACTUAL` (UniExtract.au3:2355)
  makes: `unzip.exe "<file>"`, run in `tempoutdir` with the window
  minimized. This pulls out `aisetup.ini`, the rename manifest the rest
  of this capability consumes — the actual installer payload is a
  second, recursive `extract($TYPE_7Z, ...)` dispatch (composite/
  recursive dispatch, C054, not yet ported), not modeled here.
- **Added:** `extract::actual::sanitize_destination_filename`/
  `resolve_rename` — port the per-entry rename the source applies to
  each raw name read from `aisetup.ini`'s `[Files]` section
  (UniExtract.au3:2367-2380): `<`/`>` become `[`/`]`, then a
  `?`-truncation step.
- **Behavioral finding — a real, preserved source bug.** The `?`-
  truncation guard is `Local $iPos = StringInStr($sDestination, "?") ...
  If $iPos > -1 Then $sDestination = StringLeft($sDestination, $iPos -
  1)`. `StringInStr` returns `0` (not `-1`) when the substring isn't
  found, so `$iPos > -1` is true unconditionally — the author evidently
  meant `<> 0` or `> 0`. When a `?` genuinely is present, this correctly
  truncates everything before it; when it *isn't*, `$iPos` is `0`, so
  `StringLeft($s, -1)` runs instead — AutoIt's negative-count form,
  meaning "all but the last N characters" — which silently drops the
  last character of every renamed file that has no `?` in its name. A
  genuine bug in the source, preserved here exactly rather than "fixed"
  into the evidently intended `?`-only truncation.
- **No `extract::dispatch::HARDCODED_CASES` entry:** `$TYPE_ACTUAL`'s
  real dispatch is the recursive 7-Zip call plus the rename loop, not
  either function this PR adds — the same reasoning `extract::forge`
  (C119), `extract::mscf` (C120), and `extract::unity` (C121) already
  use for the same kind of exclusion.
- Parity tests: `extract::actual::tests::meta_invocation_matches_source`,
  `sanitize_replaces_angle_brackets`, `sanitize_truncates_at_question_mark`,
  `sanitize_drops_last_character_when_no_question_mark_present`,
  `resolve_rename_builds_source_and_destination_paths`.

## C113 — arc_conv integration
**2026-08-18**

- **Added:** `extract::arc_conv::invocation` — builds the `arc_conv.exe`
  (UniExtract-authored) KiriKiri/ERISA/YU-RIS engine archive conversion
  command, matching UniExtract.au3:2394's `Case $TYPE_ARC_CONV`:
  `<program> "<file>"`, run in `outdir` with the window hidden.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"arc_conv"` →
  `extract::arc_conv`).
- **Scope note:** the preceding `HasPlugin($arc_conv, ...)` precondition
  check; the `WinWait`/window-text-polling loop that drives a tray-status
  display off arc_conv's own window title (deferred GUI subsystem,
  manifest row D001, matching this row's own "GUI-automated" description);
  and the trailing `MoveFiles($file & "~", $outdir, ...)` relocation are
  not modeled — separate runtime behavior.
- Parity test: `extract::arc_conv::tests::matches_source_invocation`.

## C121 — Unity `.unitypackage` decoder (7z.exe wrapper + path-remapping)
**2026-08-18**

- **Added:** `extract::unity::inner_tar_invocation` — the conditional
  inner-tar extraction UniExtract2's `Case $TYPE_UNITYPACKAGE`
  (UniExtract.au3:3177) makes for newer-format packages (those containing
  an `archtemp.tar`): `<program> x "<tar_file>"`, run in `tempoutdir` with
  the window minimized (`_Run`'s own default for the omitted `$show_flag`
  argument).
- **Added:** `extract::unity::resolve_asset_destination` — ports the
  per-asset destination computation (UniExtract.au3:3196),
  `_PathFull($sName, $outdir)`, approximated via `prefs::resolve_relative_path`
  (now `pub(crate)`, shared with C018/C019) — the same already-documented
  `_PathFull` gap.
- **Added:** `extract::unity::is_destination_within_outdir` — ports the
  safety check the rename loop makes before moving an asset into place
  (UniExtract.au3:3197): `StringInStr($sDestination, $outdir)`,
  case-insensitive per the same bare-`StringInStr` default already
  documented for C007-C013, C144, C145, C147, C061.
- **Behavioral finding — a preserved weak-check quirk, not hardened:**
  this is a substring containment check, not a proper path-prefix
  validation — `outdir` merely needs to appear *somewhere* in
  `destination`, which a crafted relative pathname could satisfy without
  the resolved path actually staying under `outdir`. This is exactly the
  class of directory-escape risk `ExtractionTransaction` (ADR-0119)
  exists to close properly once built; this function reproduces the
  source's weaker check as written, not a fixed version of it.
- **Not modeled:** the recursive `extract($TYPE_7Z, -1, "gz", True,
  False)` dispatch that runs first (composite/recursive dispatch, C054,
  not yet ported); the `FileExists`/`FileDelete` staging around the
  inner-tar call; the rest of the per-asset rename/restructure loop
  (reading `pathname` files, moving `asset`/`asset.meta`/`preview.png`
  into place) — all separate runtime behavior.
- **No `extract::dispatch::HARDCODED_CASES` entry:** `$TYPE_UNITYPACKAGE`'s
  real dispatch is the recursive 7-Zip call plus the rename loop, not
  either function this PR adds — the same reasoning `extract::forge`
  (C119) and `extract::mscf` (C120) already use for the same kind of
  exclusion.
- Parity tests: `extract::unity::tests::inner_tar_invocation_matches_source`,
  `resolve_asset_destination_resolves_relative_pathname`,
  `is_destination_within_outdir_accepts_contained_path`,
  `is_destination_within_outdir_is_case_insensitive`,
  `is_destination_within_outdir_rejects_unrelated_path`.

## C120 — MSCF Cab installer (7z.exe wrapper)
**2026-08-18**

- **Added:** `extract::mscf::cab_extract_invocation` — the per-file
  7-Zip extraction invocation UniExtract2's `Case $TYPE_MSCF`
  (UniExtract.au3:2827) makes for each `.cab` file `RipExeInfo`'s GUI
  automation extracted from the MSCF installer: `<program> x
  "<cab_file>"`, run in `tempoutdir` with the window hidden.
- **Scope note:** matching this row's own "Exeinfo-PE GUI rip"
  description, `RipExeInfo`'s scripted-keystroke GUI automation that
  extracts the `.cab` files in the first place is out of scope (deferred
  GUI subsystem, manifest row D001). Also not modeled: the recursive
  `extract($TYPE_7Z, ...)` dispatch that runs first (composite/recursive
  dispatch, C054, not yet ported); the surrounding `MoveFiles`/
  `DirRemove`/`Cleanup` staging; and the recursive `.cab`-file listing
  that decides which files this invocation runs against.
- **No `extract::dispatch::HARDCODED_CASES` entry:** `$TYPE_MSCF`'s real
  dispatch is the recursive 7-Zip call plus GUI automation, not this
  single per-file invocation — the same reasoning `extract::forge`
  (C119) already uses for the same kind of exclusion.
- Parity test: `extract::mscf::tests::cab_extract_invocation_matches_source`.

## C119 — InstallForge (7z.exe wrapper + base64 path-rename logic)
**2026-08-18**

- **Added:** `extract::forge::inner_archive_invocation` — the conditional
  inner-archive unpack UniExtract2's `Case $TYPE_FORGE`
  (UniExtract.au3:2546) makes when the primary 7-Zip extraction produced a
  gzip-compressed inner archive: `<program> x "<tmp>"`, run in
  `tempoutdir` with the window hidden.
- **Added:** `extract::forge::decide_rename`/`RenameDecision` — ports
  `RenameBase64PathNames`'s per-entry decision (UniExtract.au3:4006-4018)
  as a pure function of a directory entry's name: the `"empty.empty"`
  placeholder is a delete; anything else is a rename to its base64-decoded
  name, or a skip if it doesn't decode. The actual directory listing,
  deletion, and rename/recurse calls are the caller's job, the same
  "decision vs. I/O" boundary `outdir::decide_outdir_outcome` already
  draws.
- **Added:** `extract::forge::base64_decode_utf16le` — ports AutoIt's
  `_Base64Decode($sInput)` (UniExtract.au3:4728-4750) exactly as
  `RenameBase64PathNames` calls it: no explicit `$eEncoding` argument, so
  it takes the function's own default, `$SB_UTF16LE` — decode as standard
  base64 (`Crypt32.dll`'s `CryptStringToBinary`/`CRYPT_STRING_BASE64`),
  then interpret the decoded bytes as UTF-16LE text. Backed by a
  hand-rolled base64 decoder (no `base64` crate — this migration's
  no-new-dependency policy makes hand-rolling the smaller cost here, the
  same reasoning already used for `batch`'s multipart-archive pattern
  matching, C147), verified against the RFC 4648 `"Man"`→`"TWFu"` test
  vector at the raw-byte level and a UTF-16LE round-trip vector.
- **Scope note:** an empty input decodes to `""`, matching
  `_Base64Decode`'s own early-exit; an odd decoded byte count (can't form
  whole UTF-16 code units) is treated as a decode failure — no direct
  source equivalent, since `BinaryToString` itself doesn't error on this,
  but there's no other sensible outcome for a mis-sized buffer.
- **Not modeled:** the recursive `extract($TYPE_7Z, -1, "", True, False)`
  dispatch that runs first (composite/recursive dispatch, capability
  C054, not yet ported); the `FileExists`/`_FileDelete` staging around the
  inner-archive invocation; `MoveFiles`' final relocation into `outdir` —
  all separate runtime behavior.
- **No `extract::dispatch::HARDCODED_CASES` entry:** `$TYPE_FORGE`'s real
  dispatch is the recursive 7-Zip call, not either function this PR adds
  — the same reasoning `extract::pdf` and `extract::unzip` already use for
  the same kind of exclusion.
- Parity tests: `extract::forge::tests::inner_archive_invocation_matches_source`,
  `base64_decode_bytes_matches_rfc4648_vector`,
  `base64_decode_utf16le_decodes_utf16le_text`,
  `base64_decode_utf16le_empty_input_returns_empty_string`,
  `base64_decode_utf16le_rejects_invalid_input`,
  `decide_rename_recognizes_empty_marker`,
  `decide_rename_decodes_valid_base64_name`,
  `decide_rename_skips_undecodable_name`.

## C066 — ci-extractor integration
**2026-08-18**

- **Added:** `extract::ci::control_file_content` — the scripted-answer
  control-file content UniExtract2's `Case $TYPE_CI`
  (UniExtract.au3:2461-2463) writes before invoking `ci-extractor.exe`:
  `1\n<file>\n<outdir>\n3\n1`, matching `"1" & @LF & $file & @LF &
  $outdir & @LF & "3" & @LF & "1"` exactly (AutoIt's `@LF` is a bare line
  feed).
- **Added:** `extract::ci::invocation` — builds the `ci-extractor.exe`
  command (UniExtract.au3:2465): `<program> <tempfile>`, run in `outdir`
  with the window shown normally.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"ci"` →
  `extract::ci`).
- **Scope note:** the preceding `HasPlugin($ci)` precondition check; the
  `WinWait`/`ControlClick` GUI automation that clicks "Finish" on
  `ci-extractor.exe`'s wizard, `ProcessClose`, the temp-file cleanup, and
  the trailing `terminate($STATUS_SILENT)` call are not modeled — GUI
  automation is out of scope (deferred GUI subsystem, manifest row D001),
  matching this row's own "GUI-automated" manifest description; the rest
  is separate runtime behavior.
- Parity tests: `extract::ci::tests::control_file_content_matches_source`,
  `extract::ci::tests::invocation_matches_source`.

## C118 — SuperDAT Updater self-extraction
**2026-08-18**

- **Added:** `extract::superdat::invocation` — builds the self-extracting
  SuperDAT Updater command UniExtract2's `Case $TYPE_SUPERDAT`
  (UniExtract.au3:3038-3043) makes: `<file> /LOGFILE
  "<outdir>\SuperDAT.log" /e "<outdir>"`, run in `outdir` with a normally
  shown window.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"superdat"` →
  `extract::superdat`).
- **Behavioral finding:** run via `ShellExecuteWait`, not
  `_Run`/`Run`/`RunWait` — needed (per the sibling `$TYPE_AI` case's own
  comment) so the OS can raise a UAC elevation prompt, the same reasoning
  as C116 (`extract::ei`) and C117 (`extract::fead`). Its `$iShowFlag`
  parameter defaults to `@SW_SHOWNORMAL` when omitted here, mapped to
  `WindowMode::Show`.
- **Scope note:** the preceding `Warn_Execute(...)` confirmation gate
  (`warnexecute` preference, C023, deferred GUI subsystem D001) and the
  trailing `_FileRead($sPath, True)` call that reads the log file back in
  are not modeled — separate runtime behavior, not part of building this
  invocation.
- Parity test: `extract::superdat::tests::matches_source_invocation`.

## C117 — Netopsystems FEAD self-extraction
**2026-08-18**

- **Added:** `extract::fead::invocation` — builds the self-extracting FEAD
  command UniExtract2's `Case $TYPE_FEAD` (UniExtract.au3:2530-2536)
  makes: `<file> /s -nos_ne -nos_o<tempoutdir>\`, run in `filedir` with a
  normally shown window.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"fead"` →
  `extract::fead`).
- **Behavioral finding:** `-nos_o<tempoutdir>\` is a single
  concatenated-flag argument token, including a trailing literal
  backslash the source's own string concatenation adds — preserved
  exactly, matching the pattern already established in
  `extract::bcm`/`extract::lzop`/`extract::unreal`.
- **Behavioral finding:** run via `ShellExecuteWait`, not
  `_Run`/`Run`/`RunWait` — needed (per the sibling `$TYPE_AI` case's own
  comment) so the OS can raise a UAC elevation prompt. Its `$iShowFlag`
  parameter defaults to `@SW_SHOWNORMAL` when omitted here, mapped to
  `WindowMode::Show`, the same as every other unspecified-show-flag call
  in this crate.
- **Scope note:** the preceding `Warn_Execute(...)` confirmation gate
  (`warnexecute` preference, C023, deferred GUI subsystem D001) and the
  trailing `FileSetAttrib`/`MoveFiles`/`DirRemove` calls that move
  `tempoutdir`'s contents into `outdir` and clean up afterward are not
  modeled — separate runtime behavior, not part of building this
  invocation.
- Parity test: `extract::fead::tests::matches_source_invocation`.

## C116 — Excelsior Installer self-extraction
**2026-08-18**

- **Added:** `extract::ei::invocation` — builds the self-extracting
  Excelsior Installer command UniExtract2's `Case $TYPE_EI`
  (UniExtract.au3:2514-2516) makes: `<file> /batch /no-reg
  /no-postinstall /dest "<outdir>"`, run in `outdir` with a normally
  shown window.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"ei"` →
  `extract::ei`).
- **Behavioral finding:** the source runs this via `ShellExecuteWait`, not
  `_Run`/`Run`/`RunWait` — needed (per the sibling `$TYPE_AI` case's own
  comment) so the OS can raise a UAC elevation prompt, which plain `Run`
  can't trigger. `ShellExecuteWait`'s `($sFilePath, $sParameters,
  $sWorkingDir)` shape maps directly onto this crate's `Invocation`
  (program, args, working dir); its `$iShowFlag` parameter defaults to
  `@SW_SHOWNORMAL` when omitted, the same as every other
  unspecified-show-flag call in this crate, mapped to `WindowMode::Show`.
- **Scope note:** the preceding `Warn_Execute(...)` confirmation gate
  (`warnexecute` preference, C023, deferred GUI subsystem D001) is not
  modeled — this function reproduces only the command it passes through,
  the same as `extract::expand::cab_self_extract_invocation`.
- Parity test: `extract::ei::tests::matches_source_invocation`.

## C064 — Windows `expand.exe` (CAB/MSU)
**2026-08-18**

- **Added:** `extract::expand::invocation` — the shared `expand.exe`
  invocation both call sites use: the non-self-extracting `Case $TYPE_CAB`
  branch (UniExtract.au3:2438) and `Case $TYPE_MSU`'s two `expand.exe`
  calls (UniExtract.au3:2916,2927): `<program> -F:* "<file>" "<destdir>"`,
  run in `filedir` with the window hidden.
- **Added:** `extract::expand::cab_self_extract_invocation` — the "Type 1"
  self-extracting-CAB branch (UniExtract.au3:2432-2433): the archive is
  itself a self-extracting executable, run directly as `<file> /q
  /x:<outdir>`, with a normally shown window (`RunWait`'s own default,
  same mapping `extract::nbh` already uses).
- **Scope note — shell wrapping not modeled as a literal string:** the
  shared `expand.exe` call builds its command as a literal `cmd.exe /d /c
  ` prefix concatenated directly onto the string (bypassing `_Run`'s own
  bindir-prefixing, since `$expand` is already a fully resolved,
  pre-quoted `@SystemDir` path) — functionally still `<expand> -F:*
  "<file>" "<destdir>"`, so this port's `Invocation` targets the exe
  directly, the same as every other module in this crate.
- **Not modeled:** the CAB call site's preceding `check7z($arcdisp)` probe
  and `HasPlugin($expand)` precondition check; `Warn_Execute`'s
  "continue?" confirmation gate around the self-extracting-CAB path
  (deferred GUI subsystem, manifest row D001, `warnexecute` preference
  C023); and the MSU call site's surrounding orchestration (temp-directory
  staging, extracting a nested `.cab` found inside the first expansion,
  sorting the second expansion's output into `x86`/`x64`/`WOW64`/`MSIL`
  subfolders by filename prefix) — all separate runtime behavior, not part
  of building either `expand.exe` call.
- **No `extract::dispatch::HARDCODED_CASES` entry:** both `$TYPE_CAB` and
  `$TYPE_MSU` dispatch to more than one invocation depending on runtime
  conditions the flat dispatch table doesn't model — the same reasoning
  `extract::pdf` (4 invocations) and `extract::unzip` (composite
  `$TYPE_ZIP` dispatch) already use for the same kind of exclusion.
- Parity tests: `extract::expand::tests::matches_source_invocation`,
  `extract::expand::tests::cab_self_extract_matches_source_invocation`.

## C061 — ARJ SFX verification
**2026-08-18**

- **Added:** `detection::arj_probe::probe_invocation`/`is_arj_sfx` — ports
  `checkArj` (UniExtract.au3:1958-1972) exactly, the same shape as
  `detection::alz_probe`'s `CheckAlz` (C059): builds the `arj l "<file>"`
  listing probe, then reimplements the source's exact recognition
  predicate (`Archive created:` present in the captured output) as a pure
  function of that output.
- **Behavioral finding:** matches case-insensitively — the source's
  `StringInStr($return, "Archive created:", 0)` passes an explicit `0`
  case-sensitivity argument, the same AutoIt default already documented
  for every other bare/explicit-`0` `StringInStr` call this port has
  encountered (C007-C013, C144, C145, C147).
- **Scope note:** the recursive `extract($TYPE_7Z, ...)` dispatch call the
  source makes when this returns `true` (UniExtract.au3:1966) is
  composite/recursive dispatch (capability C054, not yet ported), not this
  probe's job — the same "probe vs. dispatch" boundary `detection::alz_probe`
  already draws for its own recursive `extract($TYPE_ALZ, -1)` call.
- Parity tests: `detection::arj_probe::tests::probe_invocation_matches_source`,
  `recognizes_a_valid_arj_listing`, `matches_case_insensitively`,
  `rejects_output_missing_the_created_header`.

## C063 — bootimg extractor integration
**2026-08-18**

- **Added:** `extract::bootimg::invocation` — builds the `bootimg.exe`
  Android boot-image unpack command, matching UniExtract.au3:2421-2429's
  `Case $TYPE_BOOTIMG`: `<program> --unpack-bootimg`, run in `outdir` with
  the window minimized.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"bootimg"` →
  `extract::bootimg`).
- **Behavioral finding:** `bootimg.exe` takes no file argument at all —
  `--unpack-bootimg` implicitly operates on a file named exactly
  `boot.img` in its own working directory. That's why the source stages
  the input before running this: it copies `bootimg.exe` itself into
  `outdir` and renames the archive to `outdir\boot.img` first, then
  renames it back and deletes the copied exe afterward.
- **Scope note:** that staging (`FileCopy`, two `_FileMove`s,
  `FileDelete`) and the preceding `HasPlugin($bootimg)` precondition check
  are separate runtime behavior, not part of building this invocation —
  `program` must already point at the exe as copied into `outdir` by the
  time this invocation runs, the same "invocation vs. staging" boundary
  every module in this crate already draws.
- **Scope note — shell wrapping not modeled as a literal string:** the
  source builds this via `cmd.exe /d /c "<program> --unpack-bootimg"` (the
  whole program+argument pair in one quoted token, the classic idiom for
  running a space-containing path through `cmd.exe`) — functionally still
  `<program> --unpack-bootimg`, so this port's `Invocation` targets the exe
  directly, the same as every other module in this crate.
- Parity test: `extract::bootimg::tests::matches_source_invocation`.

## C088 — NBHextract extractor integration
**2026-08-18**

- **Added:** `extract::nbh::invocation` — builds the NBHextract
  (`NBHextract.exe`) HTC NBH ROM image extraction command, matching
  UniExtract.au3:2952-2953's `Case $TYPE_NBH`: `<program> "<file>"`, run in
  `outdir` with the window shown normally.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"nbh"` →
  `extract::nbh`).
- **Behavioral finding:** the source calls AutoIt's native `RunWait`
  directly (not this script's own `_Run` wrapper) with no explicit
  `show_flag` argument, so it takes `RunWait`'s own default,
  `@SW_SHOWNORMAL` — mapped to `WindowMode::Show`, the same mapping this
  crate already uses for an explicit `True` show-flag literal.
- **Scope note — shell wrapping not modeled:** `_MakeCommand($nbh, True)`
  routes through the same generic `cmd.exe /d /c` shell-wrapping as
  `extract::sqlite`'s call site, with no effect on the arguments
  `NBHextract.exe` itself receives — this port's `Invocation` targets
  `NBHextract.exe` directly, consistent with every other module in this
  crate.
- Parity test: `extract::nbh::tests::matches_source_invocation`.

## C101 — UHARC 3-version fallback chain
**2026-08-18**

- **Added:** `extract::uharc::uharc_invocation`, `extract::uharc::uharc04_invocation`,
  `extract::uharc::uharc02_invocation` — build the three `.uha`-archive
  extraction attempts UniExtract2's `Case $TYPE_UHA` makes in sequence
  (UniExtract.au3:3154-3159): a first attempt with `UNUHARC06.EXE`, a
  fallback to the older `UHARC04.EXE` on failure, and a last fallback to
  `UHARC02.EXE` using 8.3 short-form paths. All three run with the window
  minimized (`_Run`'s own default for the omitted `$show_flag` argument).
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"uha"` →
  `extract::uharc`).
- **Behavioral finding — the third attempt is unquoted:** unlike the first
  two (which quote both the `-t<outdir>` value and the file path), the
  third attempt's arguments are bare, unquoted 8.3 short-form paths — a
  real source quirk (short names never contain spaces, so the source skips
  the quoting it uses everywhere else), preserved as written.
- **Scope note — not modeled:**
  - Which attempt actually runs is decided by the caller: the source only
    tries `UHARC04.EXE`/`UHARC02.EXE` if the previous attempt's
    `_DirGetSize` check shows the output directory didn't grow — that
    success/failure evaluation and the resulting fallback decision are
    runtime orchestration, not part of building any one attempt's
    invocation, the same "invocation vs. orchestration" boundary every
    module in this crate already draws.
  - The third attempt's short-form paths (`FileGetShortName($outdir)`/
    `FileGetShortName($file)`) are caller-supplied parameters rather than
    computed here — that Windows 8.3-short-name API is a real OS call this
    pure function can't perform itself, the same "caller supplies an
    OS-dependent fact" pattern `outdir::decide_outdir_outcome` already
    uses for filesystem checks.
- Parity tests: `extract::uharc::tests::uharc_matches_source_invocation`,
  `extract::uharc::tests::uharc04_matches_source_invocation`,
  `extract::uharc::tests::uharc02_matches_source_invocation_and_is_unquoted`.

## C097 — SQLite database dump extractor integration
**2026-08-18**

- **Added:** `extract::sqlite::invocation` — builds the `sqlite3.exe`
  database-dump command, matching UniExtract.au3:3032-3033's `Case
  $TYPE_SQLITE`: `<program> "<file>" .dump`, run in `filedir` (not
  `outdir`) with the window hidden.
- **Registered** in `extract::dispatch::HARDCODED_CASES` (`"sqlite"` →
  `extract::sqlite`).
- **Scope note — shell wrapping not modeled:** the source calls this
  through `FetchStdout`, which routes through `_MakeCommand`'s generic
  `cmd.exe /d /c` shell-wrapping. That wrapping has no effect on the
  arguments `sqlite3.exe` itself receives for this call site (no
  redirection/piping happens here, unlike `FetchStdout`'s tee-log caller)
  — this port's `Invocation` model targets `sqlite3.exe` directly, the
  same as every other module in this crate, rather than reproducing the
  shell wrapper.
- **Behavioral finding — a source typo with no observable effect:** the
  source's string literal (`' "' & $file & '" .dump"'`) has a stray,
  unbalanced trailing double quote after `.dump`. Windows' standard
  command-line argument parsing toggles "inside quotes" on each unescaped
  `"`; an unmatched trailing one with nothing after it contributes no
  literal character to the parsed token, so the actual argument
  `sqlite3.exe` receives is exactly `.dump` — the well-known `sqlite3
  <db> ".dump"` CLI usage this case is clearly invoking. This function's
  `args` reflect that effective argument, not the source's literal (and
  inert) stray quote.
- **Scope note:** capturing `sqlite3.exe`'s stdout and writing it to
  `<outdir>\<filename>.sql` (UniExtract.au3:3034-3037) is separate runtime
  behavior — committing captured output to the destination — not part of
  building this invocation, matching every other module in this crate.
- Parity test: `extract::sqlite::tests::matches_source_invocation`.

## Composition root: minimal end-to-end wiring for `rgss`/`ace`
**2026-08-18**

- **Context:** this repo had ~150 pure ported functions but nothing driving
  them — `src/main.rs` was a one-line stub, and none of `ARCHITECTURE.md`'s
  three ports (`TypeDetector`, `ExtractorRunner`, `ExtractionTransaction`)
  had an adapter. This surfaced while investigating C148 (batch-item
  process spawning): there was no process-spawning runtime anywhere for
  that capability to hook into. Rather than patch around the gap
  capability-by-capability, this PR builds the first real, working slice —
  a `main.rs` that actually extracts a file end-to-end for two extractors —
  proving the wiring pattern before it's extended further. The
  ports-and-adapters design itself is unchanged (it implements ADR-0119/
  ADR-0120 from `rusty_foundation_akb`, already a pre-accepted governing
  standard); this PR only implements the pieces that were missing.
- **Added:** `extract::runner::ExtractorRunner` — the `ExtractorRunner`
  port: `CommandExtractorRunner` is the real adapter (`std::process::Command`,
  full-capture `.output()`, not streaming — streaming was only ever in
  service of force-showing a window on a detected prompt, itself
  out-of-scope GUI, manifest row D001), `FakeExtractorRunner` is a
  hand-rolled test double (records calls, returns a canned `RunOutcome`),
  following the same real-I/O/testable-core split `extract::plugin`'s
  `resolve_plugin_ini`/`resolve_plugin_ini_with` already established. No
  new dependency — `Cargo.toml` still has zero.
- **Added:** `main.rs` — a real composition root. Takes `<extractor-type>
  <program> <file> [outdir]` positionally, resolves the output directory
  via the already-shipped `outdir::resolve_output_directory`/
  `outdir::default_output_subfolder`/`outdir::decide_outdir_outcome`
  (C004, C138, C139, C140, C142), dispatches via the already-shipped
  `extract::dispatch::dispatch` but only actually wires the two extractors
  whose invocation builders need no unresolved dependency (`rgss`, `ace`;
  `rpa` needs an `@ScriptDir`-equivalent that doesn't exist yet), sandwiches
  the run in the already-shipped C140 trailing-backslash strip/reappend
  cycle, classifies the captured output with `log_eval::is_overwrite_success_message`,
  and exits via the already-shipped `status::exit_code` contract (C016).
- **Scope note — deliberately deferred, not dropped:** the `/type` override
  (C006) and the `cli` flags (C007-C013) — this binary only takes
  positional arguments; the detection cascade (C037-046) picking
  `extractor_type` automatically — this phase requires it as an explicit
  argument; `def/*.ini` plugin-engine dispatch (`DispatchTarget::Plugin`);
  batch-queue execution/process chaining (C011, C015, the remainder of
  C148); `ExtractionTransaction`/ADR-0119 staged-commit hardening
  (extractors here still write straight to `outdir`, matching today's
  documented behavior); `/last` (no output-directory history wired up yet
  — a clear error is returned instead of silently misresolving).
- Parity/wiring tests: `extract::runner::tests::command_runner_captures_a_real_process_output`,
  `extract::runner::tests::command_runner_reports_launch_failure_without_panicking`,
  `extract::runner::tests::fake_runner_records_calls_and_returns_canned_outcome`,
  `tests::wires_rgss_through_dispatch_to_the_runner`,
  `tests::wires_ace_through_dispatch_to_the_runner`,
  `tests::rejects_extractor_types_not_wired_up_yet`,
  `tests::split_file_path_separates_dir_stem_and_extension` (in `main.rs`).

## C148 — Batch-item-per-process execution model
**2026-08-18**

- **Added:** `batch::pop_batch_queue` — ports the queue-array mechanics
  of `BatchQueuePop()` (UniExtract.au3:4444-4462): removes and returns
  the first queued command line, leaving the rest as the persisted
  queue, or `None` when the queue is already empty.
- **Scope note:** this is only the FIFO half of C148's own description.
  The other half — spawning each queued item as a brand-new process
  (`Run(@ScriptFullPath & " " & $element)`) rather than looping
  in-process, with the chain advancing only when *that* new process's
  own `terminate()` call reaches its `$batchEnabled` check
  (UniExtract.au3:4235) and pops again — is this port's own runtime
  concern (process spawning/orchestration, not yet built), not portable
  pure logic, so it isn't reproduced here.
- Parity tests: `batch::tests::pop_batch_queue_returns_first_and_rest`,
  `batch::tests::pop_batch_queue_last_item_leaves_empty_queue`,
  `batch::tests::pop_batch_queue_empty_queue_returns_none`.

## C147 — Batch queue file format and duplicate handling
**2026-08-18**

- **Added:** `batch::build_command_line` — ports `GetCmd()`
  (UniExtract.au3:4370-4386): the re-invocable, always-double-quoted
  command line UniExtract2 appends to the batch queue file for the
  current run's file.
- **Added:** `batch::should_add_to_batch` — ports `AddToBatch()`'s
  add-vs-skip decision (UniExtract.au3:4398-4404): an exact-duplicate
  command line already in the queue defers to a (caller-supplied,
  standing in for the out-of-scope GUI prompt) confirmation; otherwise, a
  multipart-archive match against the existing queue content silently
  suppresses the add with no prompt at all.
- **Added:** `batch::is_multipart_archive_already_queued` — ports
  `IsMultipartArchive()`/`__TestMultipart()` (UniExtract.au3:4354-4367):
  three fixed multipart-volume naming patterns
  (`.part<digits>.rar`, `.7z<any char><3 digits>`, `.r<2 digits>`/`.r`+`ar`),
  hand-parsed rather than via a general regex engine — adding a `regex`
  dependency would be its own stop-and-ask under this migration's
  dependency policy, and each pattern is a small, fixed structural shape
  well suited to direct parsing.
- **Behavioral finding — a real quirk in the third pattern:** `.r` followed
  by the literal `ar` is one of the pattern's two alternatives, which
  means a plain solo `.rar` file (no volume digits at all) also matches —
  `.r` + `ar` decomposes any `....rar` ending exactly. A single-volume
  archive is therefore treated the same as a genuine multipart one for
  batch-queue collision purposes, a real behavior in the source, not
  something this port introduces.
- **Scope note:** all three `StringInStr` calls this capability relies on
  (`AddToBatch`'s exact-duplicate check and `__TestMultipart`'s
  queue-content check) have no case-sensitivity argument, so — like every
  other bare `StringInStr` this port has encountered (C007-C013, C144,
  C145) — they default to case-insensitive.
- Parity tests: `batch::tests::build_command_line_matches_source_shapes`,
  `batch::tests::detects_queued_part_rar_volume`,
  `batch::tests::detects_queued_7z_volume`,
  `batch::tests::solo_rar_matches_via_ar_alternative_quirk`,
  `batch::tests::should_add_to_batch_matches_source_branches`.

## C145 — Overwrite/password/no-space/new-filename prompt live-detection
**2026-08-18**

- **Added:** `log_eval::needs_manual_input` — ports the live
  user-input-needed detection inside the subprocess-output-streaming
  loop (UniExtract.au3:4930-4933): each new chunk of a helper binary's
  live output is scanned for any of eight substrings signaling a blocked
  modal prompt (overwrite confirmation, password request, low disk
  space, a request for a new filename, a request to insert removable
  media, or a bare `[R]etry` option).
- **Scope note:** a match doesn't answer the prompt — the source has no
  auto-answer logic at all — it only force-shows the extractor's window
  so a human can respond manually. That windowing (and the surrounding
  tray-status/GUI side effects) is out of scope, deferred GUI subsystem;
  this function reproduces only the substring predicate driving it.
- **Behavioral finding:** matches case-insensitively, same as `EvaluateLog`'s
  other checks and `cli`'s flag detection (C007-C013) — AutoIt's
  `StringInStr` defaults its case-sensitivity parameter to `0` (not case
  sensitive) when omitted, as it is for every one of these eight calls.
- Parity tests: `log_eval::tests::recognizes_all_eight_prompt_substrings`,
  `log_eval::tests::matches_case_insensitively`,
  `log_eval::tests::does_not_match_ordinary_progress_output`.

## C144 — Overwrite message treated as extraction success
**2026-08-18**

- **Added:** `log_eval::is_overwrite_success_message` — ports the
  "already exists."/"Overwrite" branch of `EvaluateLog()`
  (UniExtract.au3:4819-4823): a log mentioning either substring is
  treated as success, not failure — the source's own stated reasoning is
  that an overwritten file leaves the output folder's total size roughly
  unchanged, so the separate "did the folder size change" check that
  would otherwise flag this as a failure gets skipped for exactly that
  reason.
- **Scope note:** `EvaluateLog()` is a much longer `ElseIf` chain —
  invalid password, user cancellation, low disk space, missing archive
  part, and several generic success/failure phrasings all take priority
  over this branch in the source (UniExtract.au3:4778-4818). Each is its
  own capability (or not yet ported); this predicate reproduces only the
  overwrite branch itself, and a caller must replicate the source's
  ordering — ruling those out first — to match behavior exactly.
- Parity tests: `log_eval::tests::recognizes_both_overwrite_substrings`,
  `log_eval::tests::does_not_match_unrelated_log_text`.

## C138 — Output-subfolder default resolution
**2026-08-18**

- **Added:** `outdir::default_output_subfolder` — ports `$initoutdir`'s
  computation inside `FilenameParse()` (UniExtract.au3:500-518), the
  default `/sub` destination (C004): with an extension, resolves to a
  same-name subfolder (`filedir\<stem>`) unless the stem itself still
  has an embedded dot (a multi-extension name, e.g. `"archive.tar"` from
  `"archive.tar.gz"`) *and* a plain file already exists at that exact
  path, in which case it falls back to an underscore-replaced name
  (`archive_tar`); without an extension, appends the (caller-supplied,
  localization out of scope) `_unpacked`-style suffix.
- **Scope note:** the collision check is narrowly scoped to
  multi-extension stems — a single-extension name never triggers it,
  matching the source's own `StringInStr($filename, ".")` guard exactly
  rather than generalizing to "retry on any collision."
- Parity tests: `outdir::tests::default_output_subfolder_single_extension`,
  `outdir::tests::default_output_subfolder_multi_extension_no_collision`,
  `outdir::tests::default_output_subfolder_multi_extension_collision_falls_back`,
  `outdir::tests::default_output_subfolder_no_extension_gets_suffix`.

## C157 — Empty created-output-directory cleanup on failure
**2026-08-18**

- **Added:** `outdir::should_remove_empty_created_outdir` — ports the
  cleanup check inside `terminate()` (UniExtract.au3:4224): a directory
  *this run* created (C142's `OutdirOutcome::Created`, not one that
  already existed) gets removed if the run didn't succeed and the
  directory is still empty. A non-empty failed output directory is left
  in place; a pre-existing output directory is never removed regardless
  of outcome.
- Parity tests: `outdir::tests::empty_created_outdir_removed_on_failure`,
  `outdir::tests::nonempty_created_outdir_not_removed_on_failure`,
  `outdir::tests::preexisting_outdir_never_removed`,
  `outdir::tests::successful_run_never_removes_outdir`.

## C142 — Output-directory creation and validation
**2026-08-18**

- **Added:** `outdir::OutdirOutcome`/`outdir::decide_outdir_outcome` —
  ports `CreateOutdir()`'s decision tree (UniExtract.au3:3968-3978) as a
  pure function of already-known filesystem facts: an existing, writable
  directory needs no action; a missing one is created (tracked as
  `Created`, standing in for the source's `$createdir = True`, consumed
  by C157's later cleanup-on-failure logic, not ported here); anything
  else — exists but isn't a directory, exists but isn't writable, or
  creation failed — is one of three distinct fatal outcomes, all mapping
  to `terminate($STATUS_INVALIDDIR, ...)` (exit 5, per
  `status::exit_code` / C016).
- **Scope note:** the actual `FileExists`/`_IsDirectory`/`CanAccess`/
  `DirCreate` filesystem calls are the caller's job — this function only
  reproduces the branching once those facts are known, consistent with
  this port's pattern for source functions that mix real I/O with a pure
  decision (e.g. `prefs::password_list_path`, C035).
- Parity tests: `outdir::tests::existing_writable_directory_is_already_valid`,
  `outdir::tests::missing_directory_created_successfully`,
  `outdir::tests::invalid_directory_cases_are_all_fatal`.

## C141 — Drive-root output directory behavior
**2026-08-18**

- **Added:** a dedicated parity test proving `strip_trailing_backslash_for_extraction`
  (C140) reproduces `todo.txt`'s documented "Extracting to C:/ creates
  file in @ScriptDir" bug: stripping the trailing backslash from a
  drive-root outdir (`C:\`) produces the ambiguous drive-relative
  reference `C:`, not the drive's root, because the function has no
  drive-root special case. This is a real Windows ambiguity — a process
  given `C:` as its working directory resolves relative paths against
  whatever that drive's own current directory happens to be, not `C:\`.
  Preserved rather than special-cased away, matching C141's "known
  quirk, verify still present" framing.
- **Scope note:** no new production code — the transformation was already
  correctly reproduced by C140's existing
  `outdir::strip_trailing_backslash_for_extraction`; this closes C141 by
  making that specific consequence explicit and asserted.
- Parity test: `outdir::tests::strip_trailing_backslash_reproduces_drive_root_ambiguity`.

## C140 (continued) — `extract()`'s trailing-backslash strip/reappend cycle
**2026-08-18**

- **Added:** `outdir::strip_trailing_backslash_for_extraction` and
  `outdir::reappend_trailing_backslash_after_extraction` — complete C140
  by porting the second half of its documented quirk: `extract()`
  (UniExtract.au3:2278) strips `ValidateOutputDirectory`'s trailing
  backslash immediately on entry, and only re-appends it
  (UniExtract.au3:3413) once, right before returning. Every extraction
  routine in between therefore sees an outdir with *no* trailing slash —
  a real inconsistency `todo.txt:35` documents in the source itself,
  preserved here rather than normalized away. The first half (that
  `ValidateOutputDirectory` always *adds* the trailing backslash) shipped
  in the previous PR as part of `outdir::resolve_output_directory`.
- Parity tests: `outdir::tests::strip_trailing_backslash_matches_extract_start`,
  `outdir::tests::reappend_trailing_backslash_matches_extract_end`.

## C004, C005, C139, C140 — Output-directory token and path resolution
**2026-08-18**

- **Added:** `outdir::resolve_output_directory` — ports
  `ValidateOutputDirectory()` (UniExtract.au3:526-544): `/sub` (C004)
  resolves to a caller-supplied `initoutdir`; `/last` (C005) resolves to
  a caller-supplied, already-resolved `last_outdir`; a drive-absolute
  (`X:...`) or UNC (`\\...`) path passes through unchanged; a single
  leading backslash inherits the input file's drive letter rather than
  being treated as relative (C139); anything else resolves against
  `filedir` by concatenation (C139); and a trailing `/` is stripped, then
  a trailing `\` is unconditionally appended regardless of which branch
  produced the path (C140).
- **Added:** `outdir::get_last_outdir` (C005) — ports `GetLastOutdir()`
  (UniExtract.au3:872-878): the most recently used output directory is
  the `"Directory History"` ini section's slot `"0"` (the newest slot in
  `prefs::push_history`'s convention, C021). A missing history maps to
  `None`, standing in for the source's failure path — a `MsgBox` (out of
  scope, deferred GUI subsystem) followed by `terminate($STATUS_SILENT)`
  (exit 0, C016) — which never returns a directory at all.
- **Scope note — `_PathFull`'s segment normalization not modeled:** the
  relative-path branch mirrors the source's exact string concatenation
  (`$filedir & '\' & $outdir`) but doesn't reproduce whatever `.`/`..`
  collapsing AutoIt's single-argument `_PathFull(path)` performs
  internally — that UDF isn't defined anywhere in this port's source
  checkout, the same gap already noted for the two-argument `_PathFull`
  behind C018/C019's path-override preferences.
- Parity tests: `outdir::tests::get_last_outdir_matches_source`,
  `outdir::tests::sub_token_resolves_to_initoutdir`,
  `outdir::tests::last_token_resolves_to_last_outdir`,
  `outdir::tests::drive_absolute_and_unc_paths_pass_through`,
  `outdir::tests::single_leading_backslash_inherits_drive_letter`,
  `outdir::tests::relative_path_resolves_against_filedir`,
  `outdir::tests::trailing_slash_normalized_to_backslash`.

## C007, C008, C009, C010, C012, C013 — Command-line flag detection
**2026-08-18**

- **Added:** `cli::has_silent_flag` (C007), `cli::has_nolog_flag` (C008),
  `cli::has_nostats_flag` (C009), `cli::is_help_flag` (C010),
  `cli::is_batchclear_flag` (C012), `cli::has_close_flag` (C013) — port
  the flag-presence checks `ParseCommandLine` (UniExtract.au3:589-694)
  makes directly against the raw argv array, before any
  positional-argument-dependent parsing happens.
- **Behavioral finding — every check is case-insensitive:** `_ArraySearch`
  (used for C007/C008/C009/C013) defaults its `$iCase` parameter to `0`
  (not case sensitive), and a plain `=` comparison (used for C010/C012's
  `$cmdline[1] = "..."` checks) is itself case-insensitive by default in
  AutoIt — the script never calls `Opt("StringCompareMode", 1)` to change
  that. `/SILENT`, `/Silent`, and `/silent` are all the same flag to
  UniExtract2. This is a real, easy-to-miss AutoIt-ism, not something
  this port invented, and every function here preserves it via
  `eq_ignore_ascii_case` rather than "fixing" it into a conventional
  case-sensitive CLI. Each function's parity test asserts an uppercased
  spelling still matches.
- **Scope note:** C009's own manifest description is "accepted without
  error" — the actual stats-send suppression this flag drives is a
  separate, deferred capability (manifest row D004); this PR only ports
  the flag's detectability.
- Parity tests: `cli::tests::silent_flag_detected_case_insensitively`,
  `cli::tests::nolog_flag_detected_case_insensitively`,
  `cli::tests::nostats_flag_detected_case_insensitively`,
  `cli::tests::help_flag_matches_all_six_spellings_case_insensitively`,
  `cli::tests::batchclear_flag_matches_case_insensitively`,
  `cli::tests::close_flag_detected_case_insensitively`.

## C021 — `history` preference
**2026-08-17**

- **Added:** `prefs::push_history` — ports `WriteHist`'s move-to-front /
  dedupe / cap-at-10 semantics (UniExtract.au3:857-869), expressed as the
  resulting ordered list a subsequent `ReadHist`
  (UniExtract.au3:844-854) would observe.
- **Scope note — preserved quirks:** `WriteHist` writes the new item to
  ini key `"0"` unconditionally, then re-writes each of the old history's
  first 9 entries to its own original key — except any entry equal to the
  new item, which gets deleted instead of rewritten, leaving a *hole* in
  the ini rather than shifting later entries down to fill it. `ReadHist`
  skips empty slots when reconstructing the list, so that hole is
  invisible to every consumer that only ever reads history through
  `ReadHist` (every consumer in the source) — this function models that
  externally observable list, not `WriteHist`'s raw ini key layout.
  Separately, the 9-entry scan over the old list is positional, not
  count-of-survivors: a duplicate found among the first 9 old entries
  shrinks the resulting list below 10 rather than reaching into a 10th
  old entry to backfill the freed slot. Both preserved as written, not
  "fixed" into a cleaner LRU.
- Parity tests: `prefs::tests::push_history_prepends_new_item`,
  `prefs::tests::push_history_deduplicates_and_moves_to_front`,
  `prefs::tests::push_history_ten_entry_cap_does_not_backfill_a_deduped_slot`,
  `prefs::tests::push_history_from_empty_history`.

## C034 — `BatchRecurse` preference
**2026-08-17**

- **Added:** `prefs::BATCHRECURSE_DEFAULT` — the `BatchRecurse` preference
  defaults to `true`. Unlike every other preference this module ports,
  it's read directly via `IniRead` with its own default argument
  (UniExtract.au3:6611: `Local Static $bRecurse = Number(IniRead($prefs,
  "UniExtract Preferences", "BatchRecurse", 1))`), not through the
  generic `LoadPref` helper — no `SavePref` write-back on a missing key,
  and the read happens only once per process (`Local Static`).
  Observably, `IniRead`'s own default argument resolves a
  missing/unreadable key the same way `resolve_bool_pref`'s
  `default_when_missing` parameter does, so this preference reuses that
  function rather than duplicating its shape.
- Parity test: `prefs::tests::batchrecurse_preference_default_matches_source`.

## C018, C019 — `batchqueue` and `filescanlogfile` path-override preferences
**2026-08-17**

- **Added:** `prefs::resolve_batchqueue_path` (C018) and
  `prefs::resolve_filescanlogfile_path` (C019) — port each preference's
  default-and-override resolution (UniExtract.au3:721-722,725,729-732):
  `LoadPref` in string mode (a present ini value used verbatim, a
  missing/unreadable key leaving the `Global` default untouched), then a
  relative override resolved against `$settingsdir` via `_PathFull`.
- **Scope note — preserved asymmetry:** the two post-`LoadPref` gates
  differ in the source: `batchqueue` checks the *value's* truthiness
  (`If $batchQueue Then ...`), so path resolution runs even on the
  default (a no-op, since it's already absolute) and is skipped only if
  the ini explicitly sets an empty `batchqueue=`; `filescanlogfile` checks
  `LoadPref`'s `@error` flag instead (`If Not @error Then ...`), so
  resolution runs only on a value actually read from the ini, never on
  the default. Both defaults are already absolute paths, so the two gates
  produce identical practical output — but the *procedure* differs, and
  this port models each one as written rather than collapsing them to a
  single shared code path.
- **Scope note — `_PathFull`:** this UDF isn't defined anywhere in this
  port's source checkout (an external/bundled AutoIt include not carried
  into this repo), so `resolve_relative_path` approximates its
  well-established standard meaning — an absolute path (drive letter or
  UNC share) is returned unchanged, a relative one joins against the base
  directory.
- **Not yet in scope:** C017 (`language` preference) shares `LoadPref`'s
  string-mode/default-on-missing-key mechanics with C018/C019, but its
  own fallback chain — auto-detecting from OS locale via
  `_WinAPI_GetLocaleInfo`/`_GetOSLanguage` when the stored value doesn't
  match a known translation catalog (UniExtract.au3:780-786) — is a
  genuinely different, OS-integration-shaped capability, not addressed by
  this PR.
- Parity tests: `prefs::tests::batchqueue_path_matches_source_default_and_override`,
  `prefs::tests::filescanlogfile_path_matches_source_default_and_override`.

## C020, C022, C023, C025, C027, C028, C029, C030, C031, C032 — Simple boolean preferences
**2026-08-17**

- **Added:** `prefs::resolve_bool_pref` — ports `LoadPref`'s int-preference
  path (UniExtract.au3:825-841) as applied to a 0/1-valued preference read
  as a boolean. AutoIt treats any nonzero integer as truthy, so
  `LoadPref`'s `_Max(Int($return), $iMin)` clamp never changes the boolean
  outcome for any of these ten preferences — the function only needs to
  model the missing-key fallback (`LoadPref`'s error path never assigns
  its `ByRef` output, so the preference keeps its `Global` declaration's
  default).
- **Added:** one `pub const ..._DEFAULT: bool` per preference, each
  documented against its own `Global` declaration line: `BATCHENABLED_DEFAULT`
  (C020, false), `APPENDEXT_DEFAULT` (C022, false), `WARNEXECUTE_DEFAULT`
  (C023, true), `FREESPACECHECK_DEFAULT` (C025, true),
  `KEEPOUTPUTDIR_DEFAULT` (C027, false), `LOG_DEFAULT` (C028, false),
  `EXTRACT_DEFAULT` (C029, true), `UNICODECHECK_DEFAULT` (C030, true),
  `EXTRACTVIDEOTRACK_DEFAULT` (C031, true), `SILENTMODE_DEFAULT` (C032,
  false).
- **Scope note:** these ten preferences are functionally identical once
  `LoadPref`'s generic mechanics are captured — none has derivation logic
  beyond "persisted flag, defaults to X" — so they're closed together in
  one batch, the same reasoning as the earlier `def/*.ini`-only extractor
  batches.
- Parity tests: `prefs::tests::resolve_bool_pref_prefers_raw_value_over_default`,
  `prefs::tests::batchenabled_preference_default_matches_source`,
  `prefs::tests::appendext_preference_default_matches_source`,
  `prefs::tests::warnexecute_preference_default_matches_source`,
  `prefs::tests::freespacecheck_preference_default_matches_source`,
  `prefs::tests::keepoutputdir_preference_default_matches_source`,
  `prefs::tests::log_preference_default_matches_source`,
  `prefs::tests::extract_preference_default_matches_source`,
  `prefs::tests::unicodecheck_preference_default_matches_source`,
  `prefs::tests::extractvideotrack_preference_default_matches_source`,
  `prefs::tests::silentmode_preference_default_matches_source`.

## C035 — Password list file path resolution
**2026-08-17**

- **Added:** `prefs::password_list_path` — ports the path fallback
  `_FindArchivePassword` performs (UniExtract.au3:726,4855-4860): the
  default `$settingsdir\passwords.txt` is used unless reading it fails
  (`FileReadToArray` setting `@error`), in which case it falls back to
  `@ScriptDir\passwords.txt`.
- **Scope note:** `settingsdir_password_file_readable` stands in for the
  outcome of that read attempt — the actual file I/O, and everything
  `_FindArchivePassword` does with the passwords it reads (probing archive
  encryption, trying each password in turn), is capability C160
  (Automated password-list trial), not yet ported. This capability is
  scoped to path selection alone, matching C035's own manifest
  description.
- Parity test: `prefs::tests::password_list_path_matches_source_fallback`.

## C033 — `cleanup` preference
**2026-08-17**

- **Added:** `prefs::parse_cleanup_option` — parses the `cleanup`
  preference's raw ini integer using the same `$OPTION_*` numbering as
  `deletesourcefile` (C024), reusing `prefs::DeleteSourceFileOption`, but
  with a different missing/unreadable/out-of-range fallback: `Move`
  (`Global $iCleanup = $OPTION_MOVE`, UniExtract.au3:162), not `Keep`.
- **Scope note:** in practice `cleanup` only ever gets *written* as
  `Delete` or `Move` — its one GUI control is a checkbox
  (`$iCleanup = _IsChecked(...) ? $OPTION_DELETE : $OPTION_MOVE`,
  UniExtract.au3:6525) — but `Keep`/`Ask` are still representable in the
  parsed result because `LoadPref` never validates the stored integer
  against that. What `Cleanup()` (UniExtract.au3:3645ff, not yet ported)
  actually does with each value is its own capability, out of scope here.
- Parity test: `prefs::tests::cleanup_option_parses_autoit_enum_numbering_with_move_fallback`.

## C024, C158 — `deletesourcefile` preference and its deletion policy
**2026-08-17**

- **Added:** `prefs::DeleteSourceFileOption`/`prefs::parse_delete_source_file_option`
  (C024) — mirrors UniExtract2's shared `$OPTION_*` enum
  (UniExtract.au3:97) and parses the `deletesourcefile` preference's raw
  ini integer using its AutoIt numbering, falling back to `Keep` for a
  missing/unreadable/out-of-range value, matching `LoadPref`'s error path
  leaving `$eOptDeleteSourceFile` at its `Global` declaration's value.
- **Added:** `prefs::should_delete_source_file` (C158) — ports the
  deletion condition inside `terminate()`'s `$STATUS_SUCCESS` case
  (UniExtract.au3:4204) exactly: `Delete` always deletes; `Ask` deletes
  only outside silent mode and only if confirmed; `Keep`/`Move` never
  delete.
- **Scope note:** `Move` is representable in `DeleteSourceFileOption`
  because `LoadPref` stores whatever integer was in the ini without
  validating it against `deletesourcefile`'s own GUI (which only offers
  Keep/Ask/Delete, UniExtract.au3:6393-6395) — the source's own decision
  condition treats a stray `Move` value exactly like `Keep`, and this port
  does too. The GUI confirmation prompt itself (`Prompt(...)`) is out of
  scope under the deferred GUI subsystem (manifest row D001);
  `should_delete_source_file` takes its result as a plain `bool`.
- Parity tests: `prefs::tests::delete_source_file_option_parses_autoit_enum_numbering`,
  `prefs::tests::should_delete_source_file_matches_source_condition`.

## C026 — `Timeout` preference
**2026-08-17**

- **Added:** `prefs::resolve_timeout_ms` — ports `LoadPref($STATUS_TIMEOUT,
  $Timeout)` plus the two lines immediately after it
  (UniExtract.au3:744-746): a stored preference value in seconds converts
  to milliseconds, and anything under 10 seconds after that conversion
  resets fully to the 60-second default (not clamped up to 10s).
- **Scope note — preserved quirk:** `LoadPref`'s error branch (missing or
  unreadable `timeout` key) never assigns its `ByRef` output parameter, so
  `$Timeout` keeps its pre-call value: the `Global $Timeout = 60000`
  declaration (UniExtract.au3:151), which the comment there marks as
  *milliseconds*. The unconditional `$Timeout *= 1000` that follows
  `LoadPref` still runs, misinterpreting that leftover millisecond value
  as seconds — so a genuinely first-run process with no `timeout` key ends
  up with a 60,000,000ms (~16.7 hour) extraction timeout instead of the
  intended 60 seconds. This is a real unit-mismatch bug in the source, not
  something this port introduced, and it's preserved rather than fixed
  under the migration's parity contract; `resolve_timeout_ms(None)`
  reproduces it exactly and is asserted by its own test.
- Parity tests: `prefs::tests::stored_seconds_convert_to_milliseconds`,
  `prefs::tests::values_under_ten_seconds_reset_to_the_sixty_second_default`,
  `prefs::tests::exactly_ten_seconds_is_not_reset`,
  `prefs::tests::missing_preference_key_reproduces_the_sixty_million_millisecond_quirk`.

## C016 — Process exit code contract
**2026-08-17**

- **Added:** `status::exit_code` — ports the `Switch $status` block inside
  `terminate()` (UniExtract.au3:4132-4213) that decides the process's
  numeric exit code: a pure function from `status::Status` (mirroring the
  source's `$STATUS_*` constants) to the same `$exitcode` values the
  source assigns. `Status::FileInfo` carries the two booleans
  (`$silentmode`, whether `$aFiletype` came back non-empty) its exit code
  actually depends on; every other variant's code is fixed.
- **Scope note:** `terminate()` itself does much more than compute an exit
  code — GUI prompts, per-run logging, local statistics, update checks,
  batch-queue continuation, unicode-filename cleanup. Those are each their
  own capability (or, for the GUI prompts, out of scope under the deferred
  GUI subsystem, manifest row D001); this capability is scoped to the exit
  code mapping alone, matching the manifest row's own description.
- Parity tests: `status::tests::fixed_exit_codes_match_source`,
  `status::tests::fileinfo_exit_code_depends_on_silent_mode_and_filetype_identification`.

## C126, C127, C128, C129, C130, C131, C132, C133, C134, C135, C136, C137 — `def/*.ini`-only extractor integrations
**2026-08-17**

- **Added:** `extract::lbr` (C126, lbrate), `extract::lit` (C127, ConvertLIT),
  `extract::mo` (C128, GNU gettext `msgunfmt`), `extract::pex` (C129,
  Champollion), `extract::qm` (C130, Qt Linguist `lconvert`), `extract::rpgmvp`
  (C131, rmvdec), `extract::sgb` (C132, sgbdec), `extract::sim` (C133,
  simdec), `extract::sit` (C134, unar / TheUnarchiver), `extract::spoon`
  (C135, spoondec), `extract::utage` (C136, utagedec), `extract::uu` (C137,
  UUDeview) — twelve more extractor types with no hardcoded `Case
  $TYPE_...` in the source, dispatched entirely through the `def/*.ini`
  plugin path (extension routing → C047, file resolution → C050, schema
  parsing → C052, placeholder substitution → C182 — all already ported).
  Each module bundles its `def/*.ini` file verbatim (`include_str!`) and
  has no new production logic of its own: the capability is proven by
  composing those primitives against the bundled file and checking the
  resulting command line matches what `pluginExtract`
  (UniExtract.au3:3468-3520) would run.
- **Scope note:** every parity test here verifies the *substituted
  command-line string* (`$sBinary & " " & $sParameters`), not a tokenized
  `Invocation.args` the way every hardcoded `extract::*` module builds —
  same known gap as the C060/C122-125 batch (a quote-aware command-line
  tokenizer for the plugin path hasn't been built yet).
- **`extract::mo`/`extract::qm` scope note:** `def/mo.ini` and `def/qm.ini`
  both set `parameters=%file% -o %outdir%\%filename%.<ext>`; the literal
  backslash and extension after `%outdir%` are not part of the `%outdir%`
  placeholder, so the substituted output is the quoted outdir immediately
  followed by the literal `\<filename>.<ext>` with no space — preserved
  exactly, not normalized into a separate quoted path segment.
- **`extract::lit` scope note:** `def/lit.ini` reproduces the
  `todo.txt:29` quirk of passing the output directory as a second raw
  positional argument with no special quoting/escaping beyond the
  placeholder substitution's own quoting — a known upstream rough edge,
  preserved rather than fixed.
- **`extract::uu` scope note:** `def/uu.ini` is the first bundled plugin in
  this port with an explicit `workingdir=%filedir%` (most omit
  `workingdir` entirely, falling back to `outdir`); its parity test
  resolves that working directory with the same single
  `replace_placeholders` call used for the command line, since `%filedir%`
  is one of the five named substitutions.
- Parity tests: `extract::lbr::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::lit::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::mo::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::pex::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::qm::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::rpgmvp::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::sgb::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::sim::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::sit::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::spoon::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::utage::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::uu::tests::bundled_ini_produces_source_matching_command_line_and_workingdir`.

## C059 — unalz (ALZip) probe + extractor integration
**2026-08-17**

- **Added:** `detection::alz_probe::probe_invocation`/`is_alz_archive` —
  ports `CheckAlz` (UniExtract.au3:1945-1956) exactly, the same shape as
  `detection::sevenzip_probe`'s `check7z` (C048): builds the `unalz -l
  "<file>"` listing probe, then reimplements the source's exact recognition
  predicate (`Listing archive:` present, `corrupted file`/`file open error`
  both absent) as a pure function of the captured output. The recursive
  `extract($TYPE_ALZ, -1)` dispatch call the source makes when this returns
  `true` is the extractor dispatcher's job (C049, already done — `$TYPE_ALZ`
  has no hardcoded case, so it falls through to the plugin path), not this
  probe's.
- **Added:** `extract::alz` — the `def/*.ini`-plugin half, following the
  exact `extract::arc`/`extract::bitrock` pattern (bundled `def/alz.ini`,
  parity test verifying the substituted command-line string).
- Parity tests: `detection::alz_probe::tests::probe_invocation_matches_source`,
  `recognizes_a_valid_alz_listing`, `rejects_output_missing_the_listing_header`,
  `rejects_a_listing_reporting_a_corrupted_file`,
  `rejects_a_listing_reporting_a_file_open_error`,
  `extract::alz::tests::bundled_ini_produces_source_matching_command_line`.

## C060, C122, C123, C124, C125 — `def/*.ini`-only extractor integrations
**2026-08-17**

- **Added:** `extract::arc` (C060, ARC), `extract::adf` (C122, unadf),
  `extract::bitrock` (C123, bitrock-unpacker), `extract::bsa` (C124, BSA
  Browser), `extract::godot` (C125, godotdec) — five extractor types with
  no hardcoded `Case $TYPE_...` in the source at all; each is dispatched
  entirely through the `def/*.ini` plugin path (extension routing → C047,
  file resolution → C050, schema parsing → C052, placeholder substitution →
  C182 — all already ported). Each module bundles its `def/*.ini` file
  verbatim (`include_str!`, matching `detection::registry`'s precedent) and
  has no new production logic of its own: the capability is proven by
  composing those primitives against the bundled file and checking the
  resulting command line matches what `pluginExtract` (UniExtract.au3:3468-3520)
  would run.
- **Scope note:** every parity test here verifies the *substituted
  command-line string* (`$sBinary & " " & $sParameters`), not a tokenized
  `Invocation.args` the way every hardcoded `extract::*` module builds.
  Turning that string into pre-split argument tokens needs a quote-aware
  command-line tokenizer this port hasn't built for the plugin path yet —
  a real gap, not something any of these five rows' own one-line manifest
  descriptions ask them to solve.
- **`extract::adf` scope note:** `def/adf.ini` sets
  `workingdir=%tempoutdir%`. `%tempoutdir%` isn't one of
  `extract::placeholder::replace_placeholders`'s five named substitutions —
  the source resolves it via a separate, direct `StringReplace` in
  `pluginExtract` (UniExtract.au3:3506) before running the result through
  `ReplacePlaceholders` (line 3507); this capability's test does the same
  two-step substitution to match.
- Parity tests: `extract::arc::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::adf::tests::bundled_ini_produces_source_matching_command_line_and_workingdir`,
  `extract::bitrock::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::bsa::tests::bundled_ini_produces_source_matching_command_line`,
  `extract::godot::tests::bundled_ini_produces_source_matching_command_line`.

## C182 — Plugin extension point placeholder substitution
**2026-08-17**

- **Added:** `extract::placeholder::replace_placeholders` — ports
  `ReplacePlaceholders` (UniExtract.au3:3523-3541) exactly: substitutes
  `%filename%`/`%fileext%`/`%filedir%` verbatim and `%file%`/`%outdir%`
  optionally quoted, then resolves every remaining `%...%` pair via a
  caller-supplied `translate` closure, skipping any pair whose contents
  contain a space (matching the source's own literal-percent-sign
  tolerance).
- **Scope note:** the source's fallback path resolves unknown placeholders
  via `t($sPlaceholder)`, reading language-catalog `.ini` files — that
  translation-catalog subsystem is a separate, deferred capability (out of
  scope for this migration). `translate` is a closure instead of a built-in
  lookup for exactly that reason, the same "caller supplies the resolved
  value" approach `extract::pdf::to_png_invocation`'s `term_page` parameter
  already uses. `t`'s own missing-translation fallback returns the
  placeholder key unchanged (UniExtract.au3:559-586), so an identity
  `translate` closure is the source-matching default when no catalog is
  wired up.
- Parity tests: `no_percent_sign_returns_unchanged`,
  `substitutes_filename_fileext_filedir_verbatim`,
  `quotes_file_and_outdir_when_quote_values_is_true`,
  `leaves_file_and_outdir_unquoted_when_quote_values_is_false`,
  `resolves_remaining_placeholders_via_translate_closure`,
  `leaves_percent_pairs_containing_a_space_untouched`,
  `identity_translate_matches_source_fallback_for_missing_translations`.

## C052 — `def/*.ini` plugin definition schema
**2026-08-17**

- **Added:** `extract::plugin_config::PluginConfig` — parses a `def/*.ini`
  file's `[Plugin]` section, porting `pluginExtract`'s field reads
  (UniExtract.au3:3480-3501,3515): `display`, `executable`, `parameters`,
  `workingdir`, `runInTempOutdir`, `hide`, `useCmd`, `log`, `patternSearch`,
  `initialShow`, `requireNetFramework`, `cleanup`, each with the exact
  default/type coercion its `_ArrayGet(...)` call uses. `window_mode()`
  derives the `hide` → `WindowMode::Hidden`/`Minimized` mapping
  (`@SW_HIDE`/`@SW_MINIMIZE` — plugin-driven extraction never reaches
  `@SW_SHOW`).
- **Verified against every bundled file:** grepped all 18 non-`registry.ini`
  files under `def/` for their actual `[Plugin]` keys — confirmed the set
  above is exhaustive; none set `cleanup` today, but the schema still
  supports it since the source still reads it.
- **Scope note:** this parses the schema only — building a real
  `Invocation` from a `PluginConfig` needs its `parameters`/`workingdir`
  strings substituted first (`%file%`/`%outdir%`/etc., `ReplacePlaceholders`
  at UniExtract.au3:3523-3541 — capability C182, not yet ported) plus the
  resolved executable path (`extract::plugin::resolve_plugin_ini`, C050).
  That wiring is left for once both exist, not part of this row.
- Parity tests: `parses_every_key_from_a_bundled_plugin_file`,
  `missing_keys_use_array_get_defaults`,
  `missing_plugin_section_parses_as_all_defaults`,
  `empty_workingdir_value_is_treated_as_absent`,
  `nonempty_workingdir_is_preserved_unsubstituted`,
  `require_net_framework_zero_is_none`, `cleanup_splits_on_pipe`.

## C146 — DAA→ISO conversion (no pre-existing-output-file check, preserved)
**2026-08-17**

- **Added:** `extract::daa::invocation` — builds the DAA→ISO conversion
  (`daa2iso.exe`) command, matching UniExtract.au3:2505-2508's `Case
  $TYPE_DAA`: `<program> "<file>" "<outdir>\<filename>.iso"`, run in
  `outdir` with the window minimized (`_Run`'s own default for the omitted
  `$show_flag` argument).
- **Quirk preserved, deliberately:** the source builds the target `.iso`
  path and passes it straight to `_Run` with no check for whether that
  file already exists — a pre-existing `<filename>.iso` in `outdir` is
  silently overwritten (or `daa2iso.exe` does whatever it does when its
  target already exists). This matches the source's own documented bug
  (`todo.txt:52`, "Converting to iso failes when iso file already
  exists"). This capability exists specifically to preserve that quirk,
  not to fix it — no existence check was added.
- **Scope note:** `_CreateTrayMessageBox(...)` (UniExtract.au3:2506), the
  "Extracting... DAA disk image (stage 1)" progress notification, is
  separate, out-of-scope GUI-subsystem behavior (manifest row D001), not
  part of this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"daa"` →
  `extract::daa`).
- Parity test: `matches_source_invocation`.

## C089 — Xpdf tools extractor integration
**2026-08-17**

- **Added:** `extract::pdf::detach_invocation`, `extract::pdf::to_html_invocation`,
  `extract::pdf::to_png_invocation`, `extract::pdf::to_text_invocation` — build
  the 4 Xpdf tool commands (`pdfdetach.exe`, `pdftohtml.exe`, `pdftopng.exe`,
  `pdftotext.exe`) matching UniExtract.au3:2967-2970's `Case $TYPE_PDF`: 4
  independent, sequential `_Run` calls, all run in `outdir` with the window
  hidden.
- **Scope note:** the third call's `t('TERM_PAGE')` (UniExtract.au3:2969)
  resolves a localized UI string ("Page") via the deferred translation-catalog
  subsystem, out of scope for this migration (see `capability-manifest.md`'s
  OUT-OF-SCOPE rows). `to_png_invocation` takes the resolved value as an
  explicit `term_page: &str` parameter instead, the same way it already takes
  `filename`/`outdir` as parameters rather than resolving them internally —
  keeping the invocation-builder translation-agnostic.
- **No `extract::dispatch::HARDCODED_CASES` entry:** that table maps one
  `$arctype` key to one Rust module/invocation shape, and `$TYPE_PDF`'s
  4-invocation case doesn't fit that model without `HARDCODED_CASES` itself
  gaining multi-invocation support — the same reasoning `extract::xor` and
  `extract::unzip` use for the same kind of exclusion (see their module doc
  comments).
- Parity tests: `detach_matches_source_invocation`,
  `to_html_matches_source_invocation`, `to_png_matches_source_invocation`,
  `to_text_matches_source_invocation`.

## C082 — unlzx extractor integration
**2026-08-17**

- **Added:** `extract::lzx::invocation` — builds the unlzx (`unlzx.exe`)
  `.lzx` extraction command, matching UniExtract.au3:2789-2790's `Case
  $TYPE_LZX`: `<program> -x "<file>"`, run in `outdir` with the window
  minimized (`_Run`'s own default for the omitted `$show_flag` argument).
- Registered in `extract::dispatch::HARDCODED_CASES` (`"lzx"` →
  `extract::lzx`).
- Parity test: `matches_source_invocation`.

## C083 — demoleition / MoleBox extractor integration
**2026-08-17**

- **Added:** `extract::mole::invocation` — builds the demoleition
  (`demoleition.exe`) MoleBox-packaged-executable extraction command,
  matching UniExtract.au3:2792-2811's `Case $TYPE_MOLE`: `<program> /nogui
  "<file>"`, run in `$outdir` with the window hidden.
- **Scope note:** the source calls `_RunInTempOutdir`, passing `$tempoutdir`
  as the staging argument but `$outdir` (not `$tempoutdir`) as the explicit
  working-directory argument — unlike `extract::lzip`/`extract::isz`, whose
  `_RunInTempOutdir` calls use `$tempoutdir` as both. The working directory
  for this invocation is therefore `outdir`; the temp-dir-then-move
  orchestration `_RunInTempOutdir` layers on top is a separate,
  already-tracked runtime-behavior capability, not part of this row. Same
  quirk, same reasoning as `extract::wolf`'s precedent (see its module doc
  comment).
- **Scope note:** the file-move logic that follows `_RunInTempOutdir`
  (renaming `<filename>_unpacked.exe` and relocating the `_extracted`
  directory into place), reading and deleting the `!unpacker.log` file, and
  evaluating that log's contents to determine `$success` are all separate
  runtime behavior, out of scope for this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"mole"` →
  `extract::mole`).
- Parity test: `matches_source_invocation`.

## C058 — AspackDie invocation (packed-executable unpack)
**2026-08-17**

- **Added:** `extract::aspack::invocation` — builds the `AspackDie.exe`
  unpack command, matching UniExtract.au3:3624-3625's call from inside
  `Case $PACKER_ASPACK`: `<program> "<file>" "<dest_path>" /NO_PROMPT`,
  run in `$filedir` with the window minimized (`_Run`'s own default for
  the omitted `$show_flag` argument).
- **Scope note:** `Case $PACKER_ASPACK` belongs to a separate `Switch
  $packer` (a post-extraction "unpack a packed executable" routine keyed
  on `$PACKER_UPX`/`$PACKER_ASPACK`), not the main `extract($arctype,
  ...)` dispatch this repo's `extract::dispatch::HARDCODED_CASES`
  represents, so it's intentionally absent from that table — the same
  reason `extract::upx` (the sibling `$PACKER_UPX` case) is absent from
  it.
- Parity test: `matches_source_invocation`.

## C112 — upx invocation (packed-executable unpack)
**2026-08-17**

- **Added:** `extract::upx::invocation` — builds the `upx.exe` unpack
  command, matching UniExtract.au3:3617-3623's call from inside `Case
  $PACKER_UPX`: `<program> -d -k "<file>"`, run in `$filedir` with the
  window minimized (`_Run`'s own default for the omitted `$show_flag`
  argument).
- **Scope note:** `Case $PACKER_UPX` belongs to a separate `Switch
  $packer` (a post-extraction "unpack a packed executable" routine keyed
  on `$PACKER_UPX`/`$PACKER_ASPACK`), not the main `extract($arctype,
  ...)` dispatch this repo's `extract::dispatch::HARDCODED_CASES`
  represents, so it's intentionally absent from that table — the same
  reason `extract::xor` is absent from it.
- **Scope note:** the source's `StringTrimRight`/`FileExists`/
  `_FileMove` logic that follows `_Run`, renaming UPX's decompressed
  output file into place over the original, is separate runtime
  behavior, not part of this row.
- Parity test: `matches_source_invocation`.

## C109 — Info-ZIP UnZip fallback invocation
**2026-08-17**

- **Added:** `extract::unzip::invocation` — builds the Info-ZIP UnZip
  (`unzip.exe`) fallback command, matching UniExtract.au3:3384-3388's
  innermost `_Run` call from inside `Case $TYPE_ZIP`: `<program> -x
  "<file>"`, run in `$outdir` with the window minimized.
- **Scope note:** the source's `Case $TYPE_ZIP` first recursively calls
  `extract($TYPE_7Z, ...)` and only reaches this Info-ZIP UnZip fallback
  `_Run` if that 7-Zip attempt fails (`If Not extract($TYPE_7Z, ...) Then
  ... EndIf`). That conditional-recursive-dispatch mechanism — try 7-Zip
  first, fall back to Info-ZIP UnZip on failure — is separate,
  already-tracked composite/recursive-dispatch capability territory, not
  part of this row. `_CreateTrayMessageBox(...)` inside the same block is
  also out of scope, belonging to the deferred GUI subsystem (manifest row
  D001).
- **No `extract::dispatch::HARDCODED_CASES` entry:** because the real
  `$TYPE_ZIP` dispatch behavior is composite (try 7z, conditionally fall
  back to unzip), a flat `"zip" -> extract::unzip` entry would misrepresent
  the source's actual dispatch logic — the same reasoning `extract::xor`
  uses for the same kind of exclusion (see its module doc comment).
  Registering `zip` accurately requires the composite/recursive dispatch
  capability to exist first.
- Parity test: `matches_source_invocation`.

## C110 — unzoo extractor integration
**2026-08-17**

- **Added:** `extract::zoo::invocation` — builds the unzoo (`unzoo.exe`)
  `.zoo` Zoo-archive extraction command, matching UniExtract.au3:3390-3394's
  `Case $TYPE_ZOO`: `<program> -x <filename_full>`, run in `tempoutdir`
  with the window hidden.
- **Scope note:** unlike most other extractor invocations in this repo, the
  source does not wrap `$filenamefull` in quotes here (`' -x ' &
  $filenamefull` is a bare, unquoted concatenation) — preserved as a
  deliberate source quirk, not normalized to the quoted style used
  elsewhere (e.g. `extract::kgb`).
- **Scope note:** the surrounding `_FileMove($file, $tempoutdir, 8)` /
  `_FileMove($tempoutdir & $filenamefull, $file)` / `MoveFiles($tempoutdir,
  $outdir, False, "", True)` calls — staging the file into a temp directory
  before running and relocating results to `outdir` afterward — are
  separate runtime behavior, already tracked as their own capabilities, not
  part of this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"zoo"` →
  `extract::zoo`).
- Parity test: `matches_source_invocation`.

## C111 — zpaq extractor integration
**2026-08-17**

- **Added:** `extract::zpaq::invocation` — builds the zpaq (`zpaq.exe`)
  `.zpaq` archive extraction command, matching UniExtract.au3:3396-3399's
  `Case $TYPE_ZPAQ`: `<program> x "<file>" -to "<outdir>"`, run in
  `outdir` with the window shown.
- **Scope note:** the source's comment on this case explains it's
  hardcoded rather than `.ini`-driven because zpaq needs a different
  executable on Windows XP — that's context for why this is a Rust
  module rather than a `def/*.ini` plugin row, not behavior this port
  needs to replicate.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"zpaq"` →
  `extract::zpaq`).
- Parity test: `matches_source_invocation`.

## C108 — WolfDec extractor integration
**2026-08-17**

- **Added:** `extract::wolf::invocation` — builds the WolfDec
  (`WolfDec.exe`) Wolf RPG Editor game-archive extraction command, matching
  UniExtract.au3:3377-3382's `Case $TYPE_WOLF`: `<program> "<file>"`, run in
  `$outdir` with the window minimized.
- **Scope note:** the source calls `_RunInTempOutdir`, passing `$tempoutdir`
  as the staging argument but `$outdir` (not `$tempoutdir`) as the explicit
  working-directory argument — unlike `extract::lzip`/`extract::isz`, whose
  `_RunInTempOutdir` calls use `$tempoutdir` as both. The working directory
  for this invocation is therefore `outdir`; the temp-dir-then-move
  orchestration `_RunInTempOutdir` layers on top is a separate,
  already-tracked runtime-behavior capability, not part of this row.
- **Scope note:** `HasPlugin($wolf)`, the preceding
  `_CreateTrayMessageBox(...)` UI notification (deferred GUI subsystem,
  manifest row D001), the `_Sleep(1000, "CLEANING_UP")` pause, and the
  trailing `MoveFiles(...)` call are all separate runtime behavior, out of
  scope for this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"wolf"` →
  `extract::wolf`).
- Parity test: `matches_source_invocation`.

## C103 — umodel extractor integration
**2026-08-17**

- **Added:** `extract::unreal::invocation` — builds the umodel
  (`umodel.exe`/`unreal.exe`) Unreal Engine package extraction command,
  matching UniExtract.au3:3211-3214's `Case $TYPE_UNREAL`: `<program>
  -export -all -sounds -3rdparty -path="<file_dir>" -out="<outdir>" *`, run
  in `outdir` with the window minimized. `-path="..."` and `-out="..."` are
  each a single concatenated-flag argument token (flag directly joined to a
  quoted value, no space) — the same pattern already established in
  `extract::bcm`/`extract::lzop`.
- **Scope note:** the source's `HasPlugin($unreal)` call immediately
  preceding `_Run` is a precondition check — separate runtime behavior, not
  part of building this invocation, and out of scope for this row.
- **Scope note:** matching the source's own comment on this `Case`, umodel
  extracts files from all packages in the folder rather than only the
  selected one — a documented quirk of the source's behavior, preserved
  here verbatim rather than "fixed".
- Registered in `extract::dispatch::HARDCODED_CASES` (`"unreal"` →
  `extract::unreal`).
- Parity test: `matches_source_invocation`.

## C107 — dark / WiX Toolset extractor integration
**2026-08-17**

- **Added:** `extract::wix::invocation` — builds the dark (`dark.exe`)
  WiX Toolset MSI-based installer extraction command, matching
  UniExtract.au3:3373-3375's `Case $TYPE_WIX`: `<program> -x "<outdir>"
  "<file>"`, run in `outdir` with the window minimized.
- **Scope note:** the source's `HasNetFramework(4)` call immediately
  preceding `_Run` is a precondition check for the .NET Framework version
  `dark.exe` requires — separate runtime behavior, not part of building
  this invocation, and tracked as its own capability, not this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"wix"` →
  `extract::wix`).
- Parity test: `matches_source_invocation`.

## C102 — uif2iso extractor integration
**2026-08-17**

- **Added:** `extract::uif::invocation` — builds the uif2iso (`uif2iso.exe`)
  MagicISO `.uif`-to-ISO stage-1 conversion command, matching
  UniExtract.au3:3161-3163's `Case $TYPE_UIF`: `<program> "<file>"
  "<outdir>\<filename>"`, run in `$filedir` with the window shown normally
  (the source's explicit `True` third argument maps to `WindowMode::Show`,
  same precedent as `extract::rpa`).
- **Scope note:** the source's `_CreateTrayMessageBox(...)` call immediately
  preceding `_Run` is a UI progress notification belonging to the deferred
  GUI subsystem (manifest row D001) — out of scope for this row, not
  represented here.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"uif"` →
  `extract::uif`).
- Parity test: `matches_source_invocation`.

## C072 — FSB extractor integration
**2026-08-17**

- **Added:** `extract::fsb::invocation` — builds the FSB extractor
  (`fsbext.exe`) `.fsb` (FMOD Sample Bank) extraction command, matching
  UniExtract.au3:2559-2562's `Case $TYPE_FSB`: `<program> -o -1 -A -d
  "<outdir>" "<file>"`, run in `$filedir` with the window minimized.
- **Scope note:** the source's `Case $TYPE_FSB` also calls
  `Cleanup("*.ogg")` after the `_Run`, deleting the raw `.ogg` dumps that
  cannot be played — that post-extraction glob-delete is separate runtime
  behavior, not part of building this invocation, and is tracked as its own
  capability, not this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"fsb"` →
  `extract::fsb`).
- Parity test: `matches_source_invocation`.

## C078 — unisz extractor integration
**2026-08-17**

- **Added:** `extract::isz::invocation` — builds the ISZ compressed-ISO
  extraction command, matching UniExtract.au3:2775-2778's `Case
  $TYPE_ISZ`.
- **Scope note:** the source calls `_RunInTempOutdir` rather than plain
  `_Run` — a variant that stages output in a temp directory before moving
  it into place — but the resulting `Invocation` shape (program, args,
  working directory, window) is identical to every other `_Run`-based
  extractor here; the temp-dir-then-move orchestration is a separate,
  already-tracked runtime-behavior capability, not part of this row.
- **Scope note:** the preceding `_CreateTrayMessageBox(...)` call is a UI
  notification — out of scope, deferred GUI subsystem work tracked under
  manifest row D001, not part of this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"isz"` →
  `extract::isz`).
- Parity test: `matches_source_invocation`.

## C067 — cicdec extractor integration
**2026-08-17**

- **Added:** `extract::cic::invocation` — builds the cicdec (`cicdec.exe`)
  Clickteam Install Creator extraction command, matching
  UniExtract.au3:2472-2475's `Case $TYPE_CIC`: `<program> -db "<file>"
  "<outdir>"`, run in the input file's own directory (`$filedir`) with the
  window hidden.
- **Scope note:** the source surrounds this `_Run` call with
  `HasNetFramework(4.5)` (a precondition check) and `Cleanup("Block
  0x*.bin")` (a post-extraction glob delete) — both are separate,
  already-tracked runtime-behavior capabilities, not part of this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"cic"` →
  `extract::cic`).
- Parity test: `matches_source_invocation`.

## C070 — xor invocation (Ghost Installer overlay decode)
**2026-08-17**

- **Added:** `extract::xor::invocation` — builds the `xor.exe` byte-XOR
  decode command, matching UniExtract.au3:2598's call from inside `Case
  $TYPE_GHOST`: `<program> "<overlay_file>" "<outdir>\<filename>.cab"
  0x8D`.
- **Scope note:** unlike the other extractor-integration capabilities in
  this repo, this isn't a top-level `$arctype` dispatch case — there is no
  `$TYPE_XOR` constant in the source. It's an internal helper call the
  Ghost Installer case makes itself after unpacking an overlay blob, so
  it's intentionally absent from `extract::dispatch::HARDCODED_CASES`, the
  same way `extract::plugin` is absent. The source's `_Run` call omits all
  three optional arguments, so `_Run`'s own defaults apply: working
  directory is `$outdir`, window is `@SW_MINIMIZE`.
- Parity test: `matches_source_invocation`.

## C057 — acefile extractor integration
**2026-08-17**

- **Added:** `extract::ace::invocation` — builds the ACE archive
  extraction command, matching UniExtract.au3:2346-2349's `Case
  $TYPE_ACE`.
- **Scope note:** the source's `If $success == $RESULT_FAILED Then
  check7z($arcdisp)` — falling back to 7-Zip when `acefile.exe` fails —
  is separate runtime behavior, not part of this row; this capability
  only builds the `acefile.exe` invocation itself.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"ace"` →
  `extract::ace`).
- Parity test: `matches_source_invocation`.

## C068 — GARbro extractor integration
**2026-08-17**

- **Added:** `extract::garbro::invocation` — builds the GARbro
  (`GARbro.Console.exe`) extraction command, matching UniExtract.au3:2565-
  2566's `Case $TYPE_GARBRO`: `<program> x -ocu -if png -o "<outdir>"
  "<file>"`, run in `outdir`, window minimized.
- **Scope note:** the manifest row also cites UniExtract.au3:2049, which is
  GARbro's own probe/detection step in the type-detection cascade — a
  separate, already-tracked capability. This row covers only the extraction
  invocation at line 2565.
- **Behavior note:** `-if png` forces PNG as the output format for any
  image-format conversion GARbro performs during extraction, preserved
  verbatim from the source rather than simplified.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"garbro"` →
  `extract::garbro`).
- Parity test: `matches_source_invocation`.

## C081 — lzop extractor integration
**2026-08-17**

- **Added:** `extract::lzop::invocation` — builds the LZO compressed-file
  extraction command, matching UniExtract.au3:2786-2787's `Case $TYPE_LZO`.
- **Behavior note:** the source's `_Run` call omits the third positional
  `$show_flag` argument, so `_Run`'s own default (`@SW_MINIMIZE`) applies —
  no window flag appears literally in this `Case`, but the omission itself
  is what selects `Minimized`, not a guess.
- **Behavior note:** the `-p"<outdir>"` argument is preserved as the
  source's single quoted-and-concatenated token (`-p` directly abutting the
  quoted outdir, no space), matching how several other extractors build
  this same argument shape.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"lzo"` →
  `extract::lzop`).
- Parity test: `matches_source_invocation`.

## C080 — lzip extractor integration
**2026-08-17**

- **Added:** `extract::lzip::invocation` — builds the `.lz` LZIP
  decompression command, matching UniExtract.au3:2783-2784's `Case
  $TYPE_LZ`.
- **Scope note:** the source calls `_RunInTempOutdir` rather than plain
  `_Run` — a variant that stages output in a temp directory before moving
  it into place — but the resulting `Invocation` shape (program, args,
  working directory, window) is identical to every other `_Run`-based
  extractor here; the temp-dir-then-move orchestration is a separate,
  already-tracked runtime-behavior capability, not part of this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"lz"` →
  `extract::lzip`).
- Parity test: `matches_source_invocation`.

## C079 — KGB Archiver extractor integration
**2026-08-17**

- **Added:** `extract::kgb::invocation` — builds the KGB Archiver
  (`kgb2_console.exe`) `.kgb`/`.kge` extraction command, matching
  UniExtract.au3:2780-2781's `Case $TYPE_KGB`: `<program> "<file>"`, run in
  `outdir` with the window minimized.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"kgb"` →
  `extract::kgb`).
- Parity test: `matches_source_invocation`.

## C071 — FreeArc extractor integration
**2026-08-17**

- **Added:** `extract::freearc::invocation` — builds the FreeArc `.arc`
  extraction command, matching UniExtract.au3:2556-2557's `Case
  $TYPE_FREEARC`.
- **Argument-construction note:** the source concatenates `-dp` directly
  onto the quoted outdir with no space (`-dp"' & $outdir & '"'`), so the
  resulting command-line token is a single argument `-dp"<outdir>"` — the
  embedded quote characters are literally part of the argument value, not
  two separate args (`-dp` and `"<outdir>"`) and not an unquoted
  `-dp<outdir>`. Preserved exactly as-is.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"freearc"` →
  `extract::freearc`).
- Parity test: `matches_source_invocation`.

## C065 — chdman extractor integration
**2026-08-17**

- **Added:** `extract::chdman::invocation` — builds the MAME CHD compressed
  hard disk image extraction command, matching UniExtract.au3:2441-2442's
  `Case $TYPE_CHD`: `<chdman.exe> extracthd -i "<file>" -o
  "<outdir>\<filename_stem>.img"`.
- **Scope note:** unlike most other extractor cases (including C095's
  `sfark`), this one runs in `outdir`, not the input file's own directory
  (`$filedir`) — that's a faithful match to the source's `_Run(..., $outdir)`
  call, not an inconsistency introduced by this port. The source's `_Run`
  call also omits the show-flag argument, so `_Run`'s own default of
  `@SW_MINIMIZE` applies.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"chd"` →
  `extract::chdman`).
- Parity test: `matches_source_invocation`.

## C062 — BCM extractor integration
**2026-08-17**

- **Added:** `extract::bcm::invocation` — builds the BCM-compressed-file
  extraction command, matching UniExtract.au3:2418-2419's
  `Case $TYPE_BCM`.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"bcm"` →
  `extract::bcm`).
- Parity test: `matches_source_invocation`.

## C051 — Detector-to-plugin mapping (`[Trid]`/`[File]`/`[Exeinfo]`)
**2026-08-17**

- **Added:** `detection::detector_mapping::DetectorMapping`, porting
  `UserDefCompare` (UniExtract.au3:1804-1819): resolves a detector's raw
  output text (TrID, Unix `file`, or Exeinfo PE) to a plugin `.ini` stem via
  substring search against `def/registry.ini`'s `[Trid]`/`[File]`/
  `[Exeinfo]` sections, first match in file order.
- **Behavior note:** the source's loop has no early exit after a match — it
  keeps scanning every remaining row even after dispatching — but
  `extract()` always ends by exiting the process, so no later iteration is
  ever externally observable. This returns only the first match, which is
  the faithful port of that behavior, not a simplification of it.
- Parity tests exercise real bundled-registry rows for all three sections,
  a missing-section case (empty, not an error — matches the source
  tolerating a load failure), and a synthetic-data case proving file order
  (not just presence) determines the winner when two different keys could
  both match.

## C096 — extsis extractor integration
**2026-08-17**

- **Added:** `extract::extsis::invocation` — builds the Symbian OS
  `.sis`/`.sisx` extraction command, matching UniExtract.au3:3026's
  `Case $TYPE_SIS`.
- **Scope note:** the source precedes this with a QuickBMS test-extract
  (`PDunSIS.wcx`, C077) and follows it with a move-from-tempoutdir step
  plus `bindir`/`MyDocuments` cleanup — those are separate capabilities
  (the QuickBMS probe; generic post-extraction cleanup, C155), not part of
  this row.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"sis"` →
  `extract::extsis`).
- Parity test: `matches_source_invocation`.

## C050 — Case-Else plugin-ini resolution
**2026-08-17**

- **Added:** `extract::plugin::resolve_plugin_ini`, porting the
  file-resolution half of `pluginExtract` (UniExtract.au3:3471-3475): for
  an extractor-type key with no hardcoded case, check a user-override
  directory first, then the bundled directory; report which file (if
  either) matched.
- **Scope note:** this answers "which `.ini` file, if any" only — consuming
  the resolved file's `[Plugin]` section into an invocation (C052) and its
  `%placeholder%` substitution (C182) are separate, not-yet-ported
  capabilities.
- `resolve_plugin_ini_with` takes the existence check as a parameter so the
  resolution order is unit-testable without real file I/O;
  `resolve_plugin_ini` wraps it with `Path::exists`. Matches the source's
  own asymmetric error reporting: a fully-missing file's error names only
  the bundled-directory path, since that's the path the source's own
  `$sPluginFile` variable holds by the time `terminate($STATUS_MISSINGDEF)`
  fires.
- Parity tests: `prefers_user_override_over_bundled`,
  `falls_back_to_bundled_when_user_override_absent`,
  `missing_reports_only_the_bundled_path_matching_source_error`.

## C095 — sfarkxtc extractor integration
**2026-08-17**

- **Added:** `extract::sfark::invocation` — builds the sfArk-compressed
  SoundFont extraction command, matching UniExtract.au3:3019-3020's
  `Case $TYPE_SFARK`. Notable divergence from the pattern so far: the
  source names the output file explicitly (`<outdir>\<filename>.sf2`)
  rather than letting the tool pick, and runs in `$filedir` (the input
  file's own directory) rather than `outdir`.
- Registered in `extract::dispatch::HARDCODED_CASES` (`"sfark"` →
  `extract::sfark`), the third extractor this port has wired up.
- Parity test: `matches_source_invocation`.

## C049 — Central extractor dispatcher
**2026-08-17**

- **Added:** `extract::dispatch`, porting the routing decision from
  `extract($arctype, ...)` (UniExtract.au3:2269-3441, `Switch $arctype`):
  given an extractor-type key, route to its hardcoded Rust module or fall
  through to the `def/*.ini` plugin path (`Case Else`, C050).
- **Scope note, stated plainly:** this ports the *dispatch mechanism*, not
  every one of the source's ~70 `Case`s — each extractor case takes
  different explicit inputs (compare `rgss::invocation`'s
  `(program, file, outdir)` against `rpa::invocation`'s extra `script_dir`
  parameter), so there's no single uniform call signature to wire up yet.
  `HARDCODED_CASES` lists only the two extractors already ported (C093,
  C094); a type key the source hardcodes but this port hasn't reached
  correctly falls through to `Plugin` today — that's this capability's
  honest current coverage, not a claim every source `Case` exists. Every
  future extractor-integration PR adds its one line to `HARDCODED_CASES`.
- Parity tests: `routes_ported_extractors_to_their_module`,
  `falls_through_to_plugin_for_unrecognized_or_not_yet_ported_types`,
  `dispatch_is_case_sensitive_matching_the_source`.

## CI: push trigger follows the default branch rename to `main`
**2026-08-17**

- **Fixed:** `.github/workflows/ci-rust.yml`'s `push.branches` still named
  `claude/uniextract2-rust-migration-h8nbgt`, which no longer exists on the
  remote after the default branch was renamed to `main`. PR-triggered CI was
  unaffected; a direct push to `main` would not have triggered CI without
  this.

## Unix-philosophy audit follow-up: F1 (ini skipped-line reporting), F2 (WindowMode coverage verified)
**2026-08-17**

- **Fixed (F1):** `IniFile::parse` no longer silently drops a line matching
  neither `[Section]` nor `key=value` — it now returns
  `(IniFile, Vec<SkippedLine>)`, with each skipped line's 1-indexed line
  number and raw content. `ExtensionRegistry::parse` discards the list today
  (its only input, the bundled `def/registry.ini`, has nothing to skip), but
  the primitive now exists for the user-editable `def/*.ini` plugin-loading
  capabilities (C050-C052) that will need it — a malformed line in a
  hand-edited plugin file will be reportable instead of silently vanishing.
- **Verified (F2):** grepped every `@SW_*` occurrence in the source
  (`UniExtract.au3`) to check `WindowMode`'s 3 variants (`Hidden`,
  `Minimized`, `Show`) against the full value set. Confirmed exhaustive for
  every extractor invocation — the two `@SW_*` constants not covered
  (`@SW_SHOWNORMAL`, `@SW_SHOWNOACTIVATE`) appear only in `GUISetState(...)`
  calls governing the main window, out of scope per the deferred GUI
  subsystem (manifest row D001). No code change; the finding is closed with
  the verification recorded in `WindowMode`'s doc comment so it isn't
  re-investigated later.
- Both from the `/unix-philosophy` audit run earlier this session against
  the 4 capabilities merged so far (C047, C093, C048, C094) — no High
  findings; these were the audit's Medium and Low items.
- New tests: `ini::tests::reports_malformed_line_inside_a_section`,
  `ini::tests::key_value_line_before_any_section_header_is_reported_not_silently_dropped`.

## C094 — unrpa extractor integration
**2026-08-17**

- **Added:** `extract::rpa::invocation` — builds the Ren'Py `.rpa` archive
  extraction command, matching UniExtract.au3:3016-3017's `Case $TYPE_RPA`.
  Notable: unlike most extractor cases, the source runs this with a working
  directory of `@ScriptDir` (the program's own install directory), not
  `outdir`, and with the window shown normally rather than hidden.
- Parity test: `matches_source_invocation`.

## C048 — Blind 7-Zip probe fallback
**2026-08-17**

- **Added:** `detection::sevenzip_probe`, porting `check7z`
  (UniExtract.au3:1917-1942) — the final catch-all detector, tried after
  every other detector fails: attempt a `7z l` listing and see if 7-Zip
  itself recognizes the file. `probe_invocation` builds the listing command;
  `route` reimplements the branch UniExtract2 takes on the result (disk
  image / custom display / `.exe`-with-InstallShield / generic archive /
  not an archive), purely from the captured output text — actually calling
  `extract()`/`extractDiskImage()` on that outcome is the extractor
  dispatcher's job (C049), not this probe's.
- Parity tests cover each branch of `check7z`'s exact predicate (the
  `Listing archive:`-present-but-`Errors:`+`Can not open the file as `
  case in particular, since that's a real trap in the source's logic —
  7-Zip prints a listing header even for some files it then says it
  couldn't open).

## C093 — RGSS Decryptor extractor integration
**2026-08-17**

- **Added:** `src/extract` module with a shared `Invocation`/`WindowMode`
  representation of a UniExtract2 `_Run(...)` call (program, args, working
  dir, window visibility), and `extract::rgss::invocation` — the concrete
  invocation for RPG Maker RGSS(2/3)A archives, matching
  UniExtract.au3:3009-3011's `Case $TYPE_RGSS`.
- **Known limitation, stated plainly:** CI (`windows-latest`) doesn't have
  `RgssDecrypter.exe` installed, so the parity test verifies the
  constructed command line (program, args, cwd, window mode) matches the
  source's `_Run` call — not an actual successful extraction. Same caveat
  applies to every future extractor-integration capability (C056-C137);
  noted once here rather than repeated per PR.
- Parity test: `matches_source_invocation`.

## C047 — Extension-based fallback dispatch
**2026-08-17**

- **Added:** `ExtensionRegistry` (`src/detection/registry.rs`), parsing
  `def/registry.ini`'s `[Extensions]` section and resolving a file extension
  to the extractor-type stem UniExtract2's `CheckExt` (UniExtract.au3:2174-2190)
  would select — the last-resort fallback when every signature-based
  detector fails to identify a file.
- **Added:** `def/registry.ini`, ported verbatim from the source repo (also
  carries the `[Trid]`/`[File]`/`[Exeinfo]` sections later detection-cascade
  capabilities will need — same file, one port).
- **Added:** a small hand-rolled `IniFile` parser (`src/ini.rs`) for the
  `def/*.ini` format family, rather than a new crate dependency.
- Parity test: `resolves_every_extension_in_the_bundled_registry` — every
  mapping in the bundled `[Extensions]` section resolves to the same stem
  the source's `CheckExt` would produce.

## CI: target windows-latest instead of ubuntu-latest
**2026-08-17**

- **Fixed:** `.github/workflows/ci-rust.yml` now runs on `windows-latest` and
  triggers on pushes to this repo's actual default branch
  (`claude/uniextract2-rust-migration-h8nbgt`, not `main`). repo-config's
  generic Rust CI template defaults to `ubuntu-latest`/`main`, which is wrong
  here: `rusty_extract` is a Windows-only parity port (ARCHITECTURE.md) that
  shells out to Windows helper binaries and, starting with capability C036,
  calls Win32 APIs (registry) directly — none of that builds or runs on Linux.
- **Known limitation, stated plainly:** even on `windows-latest`, CI cannot
  exercise the real external helper binaries (7-Zip, innoextract, etc.) —
  they aren't installed on the runner and downloading 50+ proprietary/GPL
  tools into CI is out of scope for this fix. Parity tests for extractor-
  integration capabilities (C056-C137) verify the constructed command line
  (binary path, arguments, placeholder substitution) against the source's
  behavior, not an actual successful extraction — flagged per-capability as
  it comes up, not asserted as full parity.

## Capability manifest — full step-1 inventory of UniExtract2's core-engine surface
**2026-08-17**

- **Added:** `capability-manifest.md` — 194 rows: 182 REQUIRED capabilities
  (CLI interface, core-engine preferences, the detection cascade/dispatcher,
  78 distinct external-extractor integrations, and runtime extraction
  behavior including several documented-and-verified-still-present quirks
  from the source's own `todo.txt`) plus 12 OUT-OF-SCOPE rows for the
  subsystems the user deferred to a later migration phase (GUI,
  context-menu/registry, auto-updater, feedback/telemetry, uninstall,
  translation catalogs), each with a user-attributed reason per the
  migration's boundary contract.
- **Known limitation, stated plainly:** the RustyMill sibling check (does an
  existing `Rusty-Mill/*`/`baileyrd/rusty_*` repo already implement a given
  capability) is directory-description-level judgment only for every row —
  this session could not attach `Rusty-Mill`-owned repos alongside its
  existing `baileyrd`-owned sources to run a real grep-level scan. Flagged
  in the manifest's own header; worth a follow-up scan in a session that can
  attach those repos before treating "none found" as final.

## Repo bootstrap — governance scaffold, Cargo skeleton, migration architecture
**2026-08-17**

- **Added:** standard governance file set (README, ARCHITECTURE, CONTRIBUTING,
  CODE_OF_CONDUCT, SECURITY, CHANGELOG, RELEASE_NOTES, ADR seed, PR/issue
  templates, `.gitattributes`, Rust CI workflow) via `repo-config`.
- **Added:** minimal Cargo skeleton (`src/lib.rs` + `src/main.rs`, single
  crate) so CI has something to build/test/lint against.
- **Added:** `ARCHITECTURE.md` cites `Rusty-Mill/rusty_foundation_akb`
  ADR-0119 (extraction is a validated filesystem transaction) and ADR-0120
  (content identification is evidence, not intrinsic truth) as the governing
  design for this repo's detection/extraction core, rather than inventing an
  architecture from scratch.
- **Known limitation, stated plainly:** this is scaffolding only — no
  extraction logic has been ported yet. Migration scope for this phase is the
  core file-type-detection + extraction-orchestration engine as a Rust
  CLI/library, ported from
  [UniExtract2](https://github.com/mzelivsky-spec/UniExtract2) (AutoIt),
  Windows-only parity, shelling out to the same external helper binaries as
  the source. The GUI, Windows context-menu integration, auto-updater, and
  feedback/telemetry system are explicitly deferred to a later migration
  phase (user decision, 2026-08-17) — see `capability-manifest.md`.
