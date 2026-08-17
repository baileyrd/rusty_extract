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
