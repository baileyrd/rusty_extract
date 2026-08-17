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
