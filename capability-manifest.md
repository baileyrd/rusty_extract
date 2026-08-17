# Capability manifest — UniExtract2 → rusty_extract

Source: https://github.com/mzelivsky-spec/UniExtract2 (AutoIt, commit at time of
inventory: see `docs/adr/` bootstrap PR). Target: this repo, staged migration,
phase 1 = core file-type detection + extraction-orchestration engine as a Rust
CLI/library. Windows-only parity target. Shells out to the same external helper
binaries as the source (user decision, 2026-08-17 — see `ARCHITECTURE.md`).

This is the boundary contract in concrete form (see the `rust-migration` skill's
`SKILL.md`): every row defaults to **REQUIRED** on creation. A row only becomes
**OUT-OF-SCOPE** with a written, user-attributed reason. `DONE` requires a merged
PR and a named parity test in **Evidence** — "compiles" is not evidence.

**RustyMill sibling check**: cross-checked capability names/purposes against
`references/platform-directory.md`'s one-line descriptions for every
`Rusty-Mill/*` and `baileyrd/rusty_*` repo. None describe archive/installer
extraction, file-type signature detection, or external-process-orchestration-
for-unpacking — the closest adjacent repos (`rustils` = OS abstraction,
`rusty_win32` = Win32 API bindings) could become *infrastructure* dependencies
later (process launch, registry access) but are not capability matches for any
row below. **Caveat**: this session could not attach `Rusty-Mill`-owned repos
for a grep-level check (cross-owner restriction — this session's other sources
are `baileyrd`-owned) — the check above is directory-description-level
judgment, not a verified `scan_platform_repos.sh` grep. Re-run a real scan in
a session that can attach `Rusty-Mill/*` before assuming "none found" is final,
per the skill's own stated limitation on this step. Every row below is marked
`none found (directory check only)` accordingly.

## Row groups

This file is **one continuous table** (`scripts/check_manifest_coverage.sh`
parses a single header/separator followed by every data row — see the
skill's `references/capability-manifest-format.md`). Row groups, by ID
prefix, in table order:

- **D001–D012 — Deferred subsystems** (staged migration, user decision
  2026-08-17). This phase covers detection + extraction orchestration only.
  The GUI, Windows context-menu/registry integration, auto-updater,
  feedback/telemetry, uninstall routine, and full translation catalogs are
  deferred to a later phase — **tracked, not dropped**. Per the skill's
  step-0 allowance for partial migrations, these subsystems are not
  inventoried capability-by-capability in this pass (that's the later
  phase's own step-1 job); they're recorded here as coarse rows so the
  manifest stays honest about what this run's coverage check does and
  doesn't cover.
- **C001–C016 — CLI interface** (phase 1: core engine)
- **C017–C036 — Configuration** (phase 1: core-engine-relevant preferences)
- **C037–C055 — Detection engine**
- **C056–C137 — Extractor integrations**, one row per distinct external
  helper binary/tool integration UniExtract2 shells out to. Each ports the
  same external binary per the migration's own scope decision
  (2026-08-17) — the Rust work is the orchestration/invocation code, not a
  reimplementation of the tool itself.
- **C138–C182 — Runtime extraction behavior**

| ID | Capability | Category | Source | Existing RustyMill impl | Status | Reason (if OUT-OF-SCOPE) | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| D001 | Main GUI: window, dialogs, tray icon, status popup, preferences dialog, plugin manager, batch queue UI, first-start wizard | interface | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17 (rust-migration step 0 scope question): staged migration, phase 1 = core detection/extraction engine only. GUI is a later migration phase, to be inventoried in detail when that phase starts. | |
| D002 | Windows context-menu / shell registration (`HKCU\Software\UniExtract`) and file-association management | behavior | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred to a later migration phase, same as D001. | |
| D003 | Auto-updater: `CheckUpdate()`, nightly/beta channel switch, update-interval scheduling | behavior | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred to a later migration phase, same as D001. | |
| D004 | Feedback/telemetry system: feedback prompt + submission, usage-statistics send, per-install GUID | behavior | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred to a later migration phase, same as D001. | |
| D005 | Uninstall routine (`Uninstall()`) | behavior | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred, tightly coupled to D002's registry/context-menu cleanup. | |
| D006 | Full translation catalogs (`lang/*.ini` localized string packs beyond a default English set) | docs | docs+code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred — content is >95% GUI dialog text, coupled to D001. | |
| D007 | `/afterupdate`, `/update`, `/updatehelper`, `/updatehelpers` CLI verbs | interface | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: dispatch into D003 (auto-updater), deferred with it. | |
| D008 | `/plugins` CLI verb | interface | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: dispatches into D001 (GUI plugin manager), deferred with it. | |
| D009 | `/uninstall`, `/removeuserdata` CLI verbs | interface | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: dispatch into D005/D002, deferred with them. | |
| D010 | GUI-only preferences: `notraycon`, `nostatusbox`, `hidestatusboxiffullscreen`, `openfolderafterextr`, `keepopen`, `storeguiposition`+`posx`/`posy`/`GuiWidth`/`GuiHeight`, `topmost`, `statusposx`/`statusposy`, GUI-text-field `%VAR%` env-placeholder expansion (`EnvParse`) | config | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred with D001 (GUI-only settings, no effect on core engine output). | |
| D011 | Context-menu/file-association preferences: `addassocenabled`, `addassoc`, `addassocallusers` | config | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred with D002. | |
| D012 | Telemetry/updater preferences: `feedbackprompt`, `sendstats`, `updateinterval`, `lastupdate`, `ID` (per-install GUID), `Statistics` ini section | config | code | none found (directory check only) | OUT-OF-SCOPE | User decision, 2026-08-17: deferred with D003/D004. | |

| C001 | Positional file argument: path to extract/scan, resolved to a full path; nonexistent path terminates with exit code 5 | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:635 (ParseCommandLine) |
| C002 | Positional destination argument: output directory, or the literal tokens `/sub`, `/last`, `/scan` | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:643-649 |
| C003 | `/scan` — scan-only mode: detect and report file type, do not extract | interface | code+docs | none found (directory check only) | REQUIRED | | UniExtract.au3:640-642; README.md "Scan only mode" |
| C004 | `/sub` destination token — extract into a subdirectory named after the archive | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:527-528,645 |
| C005 | `/last` destination token — extract into the most-recently-used output directory (history) | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:529-530,872-880 |
| C006 | `/type[=value]` override — force a specific extractor, bypassing all detection; with no value, present a candidate list for the caller to choose | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:652-682,420 |
| C007 | `/silent` — suppress all interactive prompts for this run | interface | code+docs | none found (directory check only) | REQUIRED | | UniExtract.au3:601; README.md "Silent mode" |
| C008 | `/nolog` — suppress the per-run log file for this invocation, overriding the persisted `log` preference | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:602 |
| C009 | `/nostats` flag parsing — accepted without error; actual stats-send behavior is D004 | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:603 |
| C010 | `/help`, `/?`, `-h`, `/h`, `-?`, `--help` — print CLI usage/help text, exit 0 | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:605-606 |
| C011 | `/batch` — queue the file for later processing instead of extracting immediately | interface | code+docs | none found (directory check only) | REQUIRED | | UniExtract.au3:687-690; README.md "Batch mode" |
| C012 | `/batchclear` — clear the batch queue | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:630-632 |
| C013 | `/close` — exit silently (used to signal a running instance to close) | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:693 |
| C014 | Directory passed as the file argument — queues every file within as a batch instead of erroring or treating the directory itself as input | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:380-383 |
| C015 | Single-running-instance behavior — a second invocation while one is running is queued via the batch mechanism instead of running concurrently | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:365-369 |
| C016 | Exit code contract — internal status maps to a specific numeric process exit code (0 success, 1 failed, 3 unknown exe, 4 unknown ext/scan-fail, 5 invalid file/dir, 6 not packed, 7 not supported, 8 missing exe/plugin, 9 timeout, 10 wrong password, 11 missing definition, 12 move failed, 13 no free space, 14 missing archive part; syntax/batch/silent paths exit 0) | interface | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4098-4213 (terminate) |

| C017 | `language` preference — selects the message-string language file; auto-detected from OS locale on first run if unset/invalid | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:728,780-786 |
| C018 | `batchqueue` preference — overrides the batch queue file path (default `%settingsdir%\batch.queue`) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:729-730,721 |
| C019 | `filescanlogfile` preference — overrides the scan-only result log path (default `%settingsdir%\log\filescan.txt`) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:731-732,725 |
| C020 | `batchenabled` — persisted flag driving batch-queue continuation logic on process exit | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:733 |
| C021 | `history` preference — records the last 10 used files/output directories | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:734,844-869 |
| C022 | `appendext` preference — controls whether an extension is appended to extracted output | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:735 |
| C023 | `warnexecute` preference — warn before running/executing self-extracting content | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:736 |
| C024 | `deletesourcefile` preference — keep/delete/ask/move source archive after a successful extraction | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:741,4204-4207 |
| C025 | `freespacecheck` preference — enable/disable a disk-space check before extraction | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:742 |
| C026 | `Timeout` preference — extraction timeout in seconds; minimum enforced 10s, default 60s if invalid | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:744-746 |
| C027 | `keepoutputdir` preference | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:748 |
| C028 | `log` preference — enable/disable a per-extraction debug log file; overridable per-run by `/nolog` (C008) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:751,4231-4233 |
| C029 | `extract` preference — persisted default for extract-vs-scan-only; overridden per-run by `/scan` (C003) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:753 |
| C030 | `unicodecheck` preference — enables detection/handling of non-ASCII filenames requiring temp rename | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:754 |
| C031 | `extractvideotrack` preference — controls whether video-track extraction is attempted for media files | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:755 |
| C032 | `silentmode` preference — persisted default for silent mode, independent of the per-run `/silent` flag (C007) | config | code+docs | none found (directory check only) | REQUIRED | | UniExtract.au3:756 |
| C033 | `cleanup` preference — post-extraction cleanup mode (move-to-subfolder vs delete) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:758 |
| C034 | `BatchRecurse` preference — recurse into subdirectories when batch-adding a directory (C014) | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:6611 |
| C035 | Password list file — plain text, one password per line, default `%settingsdir%\passwords.txt`, falls back to `@ScriptDir\passwords.txt` | config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:726,4855-4860 |
| C036 | Third-party detector tool silencing — force PEiD/Exeinfo-adjacent bundled tools into non-interactive mode via their own registry keys, with backup/restore of prior values, so automated detection doesn't hang on a tool's own GUI | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1286-1324,1822-1850 |

| C037 | Layered detection cascade order: extension pre-check → PE-aware (Exeinfo/PEiD) for `.exe`/`.dll` → TrID → Unix `file` as second opinion → extension fallback → blind 7z probe. Order itself is behavior-significant. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:376-463, comment 424-430 |
| C038 | TrID signature-based detection (TrIDLib.dll via DllCall) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:901-1030 |
| C039 | TrID match dispatch table — maps TrID's textual output substrings to extractor types (~130 cases) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1490-1801 (tridcompare) |
| C040 | Unix `file` tool as secondary/second-opinion detector, run automatically after TrID | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1033-1051,938 |
| C041 | Unix `file` match dispatch table (~25 cases), including flagging un-extractable text/media types as "not packed" | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1398-1487 (filecompare) |
| C042 | Exeinfo PE detection, run before TrID for `.exe`/`.dll` files | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1096-1281,466 |
| C043 | Exeinfo PE match dispatch table (~45 cases), including explicit "not supported"/"not packed" terminal cases | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1141-1278 |
| C044 | PEiD fallback packer-signature detection for executables Exeinfo PE couldn't classify (run twice: extension mode then hard mode) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1284-1398,478-479 |
| C045 | MediaInfo scan — informational only, used in scan-only mode display, never drives extraction dispatch | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1054-1093 |
| C046 | Extension-based pre-check (split files `.001`, compound tar variants, unreliable-signature disk images) run before any signature scan | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2193-2215 (InitialCheckExt) |
| C047 | Extension-based fallback dispatch — last resort via `def/registry.ini`'s `[Extensions]` section when no detector matched | behavior | code | none found (directory check only) | DONE | | PR [#185](https://github.com/baileyrd/rusty_extract/pull/185), test `detection::registry::tests::resolves_every_extension_in_the_bundled_registry` |
| C048 | Blind 7-Zip probe fallback — attempt a 7z listing regardless of detected type, as a final catch-all | behavior | code | none found (directory check only) | DONE | | PR [#188](https://github.com/baileyrd/rusty_extract/pull/188), test `detection::sevenzip_probe::tests` (7 tests covering `route`'s branches and `probe_invocation`) |
| C049 | Central extractor dispatcher — single function, ~70-case switch keyed on extractor-type constant | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2269-3441 |
| C050 | Case-Else → `def/*.ini` plugin-engine fallback dispatch — any extractor-type string not hardcoded is treated as a plugin-ini filename stem | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3400-3402,3468-3520 |
| C051 | `def/registry.ini` detector-to-plugin mapping (`[Trid]`/`[File]`/`[Exeinfo]` sections) — bridges detector string output to plugin-ini stems | behavior+config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1804-1819 (UserDefCompare); def/registry.ini |
| C052 | `def/*.ini` plugin definition schema — `[Plugin]` section: display/executable/parameters/workingdir/runInTempOutdir/hide/log/patternSearch/initialShow/requireNetFramework/cleanup, with `%file%`/`%outdir%`/`%filename%`/`%tempoutdir%` placeholders | config | code | none found (directory check only) | REQUIRED | | def/adf.ini, def/bitrock.ini, def/bsa.ini, def/godot.ini (representative; all 17 files share this schema) |
| C053 | Manual disambiguation — when multiple candidate extractors exist for one format and automatic testing can't pick one, present the candidates for the caller to choose (GUI dialog itself is D001; the disambiguation *data*/policy is core) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:7499 (GUI_MethodSelect), used at 2663,2705,2854,2893,3337 |
| C054 | Chained/composite recursive dispatch — a format's handling internally re-invokes the dispatcher with a different type for nested/container formats | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2362,2543,2816,2999,3173,3385 |
| C055 | Game-archive BMS-script lookup — SQLite-backed `BMS.db` mapping file extension → per-game QuickBMS script (150+ entries), separate from the `def/*.ini` mechanism | behavior+config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1997-2049 (CheckGame), 3544-3569 (BmsExtract) |

| C056 | 7-Zip (`7z.exe`) integration — 7z/ZIP/GZIP/BZIP2/XZ/TAR/CPIO/AR/LZH/RPM/DEB/ARJ/LZMA/WIM/etc., plus the universal blind-probe fallback (C048); includes the 7z-SFX-stub splitter sub-step (`7ZSplit.exe`) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2289-2343 |
| C057 | acefile (`acefile.exe`) — ACE archives, ACE SFX | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2346 |
| C058 | AspackDie (`AspackDie.exe`) — ASPack-packed executables | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3624 |
| C059 | unalz (`unalz.exe`) — ALZip `.alz` archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1945; def/alz.ini |
| C060 | ARC (`arc.exe`) — `.arc` ARC-format archives | behavior | code | none found (directory check only) | REQUIRED | | def/arc.ini |
| C061 | ARJ SFX verification (`arj.exe`) — verify/list only, actual extraction delegates to 7-Zip (C056) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1959 (checkArj) |
| C062 | BCM (`bcm.exe`) — `.bcm` BCM-compressed files | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2418 |
| C063 | bootimg (`bootimg.exe`) — Android boot images (`.img`) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2421 |
| C064 | Windows `expand.exe` — Microsoft CAB archives (`.cab`), MSU updates, including the self-extracting Type-1 CAB path | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2431-2439,2911-2950 |
| C065 | chdman (`chdman.exe`) — MAME CHD compressed hard disk images | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2441 |
| C066 | ci-extractor (`ci-extractor.exe`) — CreateInstall installers, GUI-automated | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2459 |
| C067 | cicdec (`cicdec.exe`) — Clickteam Install Creator installers | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2472 |
| C068 | GARbro (`GARbro.Console.exe`) — 500+ visual-novel/game-engine archive formats | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2049,2565 |
| C069 | Exeinfo PE resource-extraction automation (`exeinfope.exe`) — GUI-automated resource ripping for Ghost Installer, MSCF, SWF-in-EXE, and some Wise/InstallShield MSI cache extraction (distinct from its detection use, C042) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1861,1822,1897; used at 2568,2814,3100,2719,3357 |
| C070 | xor (`xor.exe`) — byte-XOR decode of Ghost Installer's overlay-extracted CAB blob | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2598 |
| C071 | FreeArc (`unarc.exe`) — FreeArc `.arc` archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2556 |
| C072 | FSB extractor (`fsbext.exe`) — FMOD Sample Bank (`.fsb`) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2559 |
| C073 | helpdeco (`helpdeco.exe`) — Windows `.hlp` help files, with RTF reconstruction pass | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2605 |
| C074 | innounp + innoextract (`innounp.exe`, `innoextract.exe`) — Inno Setup installers, GOG installers; primary/fallback pair | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2085,2614 |
| C075 | InstallShield CAB fallback chain (`unshield.exe`, `i6comp.exe`, `i5comp.exe`, `iscab.exe`) — 4-tool chain, user/caller-selectable per C053 | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2655-2691 |
| C076 | IsXunpack (`IsXunpack.exe`) — legacy InstallShield installers | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2709 |
| C077 | QuickBMS (`quickbms.exe`) + WCX plugin fan-out (`gaup_pro.wcx`, `InstExpl.wcx`, `Iso.wcx`, `msi.wcx`, `TotalObserver.wcx`, `PDunSIS.wcx`) + `BMS.db`-selected `.bms` scripts (150+ game-specific) | behavior+config | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2984,2007,2071,2120,2161,3544 |
| C078 | unisz (`unisz.exe`) — ISZ compressed ISO (stage 1 of disk-image conversion) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2775 |
| C079 | KGB Archiver (`kgb2_console.exe`) — `.kgb`/`.kge` archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2780 |
| C080 | lzip (`lzip.exe`) — `.lz` LZIP compressed files | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2783 |
| C081 | lzop (`lzop.exe`) — `.lzo` LZO compressed files | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2786 |
| C082 | unlzx (`unlzx.exe`) — `.lzx` LZX (Amiga) archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2789 |
| C083 | demoleition / MoleBox (`demoleition.exe`) — MoleBox-packaged executables | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2792 |
| C084 | lessmsi (`lessmsi.exe`) — Windows Installer `.msi`, primary extractor | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2132,2841-2848 |
| C085 | jsMSIx (`jsMSIx.exe`) — MSI fallback method 1 | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2858 |
| C086 | MsiX (`MsiX.exe`) — MSI fallback 2, MSM merge modules, MSP patches | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2864,2887,2908 |
| C087 | Windows `msiexec.exe` — MSI administrative-install fallback | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2883 |
| C088 | NBHextract (`NBHextract.exe`) — HTC NBH ROM images | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2952 |
| C089 | Xpdf tools (`pdfdetach.exe`, `pdftohtml.exe`, `pdftopng.exe`, `pdftotext.exe`) — PDF content/attachment/text/image extraction, 4 sequential invocations | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2967-2971 |
| C090 | PeaZip (`pea.exe`) — `.pea` PEA archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2973 |
| C091 | RAIU (`RAIU.exe`) — Reflexive Arcade Installer wrapper, chains into Inno Setup extraction (C074) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2994 |
| C092 | UnRAR (`UnRAR.exe`) — RAR archives, RAR SFX | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3003 |
| C093 | RGSS Decryptor (`RgssDecrypter.exe`) — RPG Maker RGSS(2/3)A archives | behavior | code | none found (directory check only) | DONE | | PR [#186](https://github.com/baileyrd/rusty_extract/pull/186), test `extract::rgss::tests::matches_source_invocation` |
| C094 | unrpa (`unrpa.exe`) — Ren'Py `.rpa` archives | behavior | code | none found (directory check only) | DONE | | PR [#190](https://github.com/baileyrd/rusty_extract/pull/190), test `extract::rpa::tests::matches_source_invocation` |
| C095 | sfarkxtc (`sfarkxtc.exe`) — sfArk compressed SoundFont | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3019 |
| C096 | extsis (`extsis.exe`) — Symbian OS `.sis`/`.sisx` installers | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3022-3030 |
| C097 | SQLite (`sqlite3.exe`) — SQLite database dump | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3032 |
| C098 | swfextract (`swfextract.exe`) — Shockwave Flash (`.swf`) content extraction (sounds, images, streams) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3045-3098 |
| C099 | Thinstall/ThinApp extractor (`Extractor.exe`) — virtualized executables, GUI-automated | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3107-3128 |
| C100 | ttarchext (`ttarchext.exe`) — Telltale Games `.ttarch` archives, game selected via candidate list (C053) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3130-3151 |
| C101 | UHARC 3-version fallback chain (`UNUHARC06.EXE`, `UHARC04.EXE`, `UHARC02.EXE`) — `.uha` archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3153-3159 |
| C102 | uif2iso (`uif2iso.exe`) — MagicISO `.uif` images (stage-1 conversion to ISO) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3161 |
| C103 | umodel (`umodel.exe`/`unreal.exe`) — Unreal Engine packages | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3211 |
| C104 | ffmpeg (`ffmpeg.exe`) — audio conversion to WAV, video/audio track extraction from containers | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2414,3216,3285 |
| C105 | VIS3Ext (`VIS3Ext.exe`) — Visionaire Engine v3 archives, two-pass invocation | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3290-3322 |
| C106 | Wise Installer 4-method fallback (`e_wise_w.exe`, `wun.exe`, self-extract switch, exeinfope MSI rip, `unzip.exe`) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3332-3371 |
| C107 | dark / WiX Toolset (`dark.exe`) — WiX MSI-based installers | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3373 |
| C108 | WolfDec (`WolfDec.exe`) — Wolf RPG Editor game archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3377 |
| C109 | Info-ZIP UnZip (`unzip.exe`) — generic ZIP fallback when 7-Zip fails | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3384-3388 |
| C110 | unzoo (`unzoo.exe`) — `.zoo` Zoo archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3390 |
| C111 | zpaq (`zpaq.exe`) — `.zpaq` ZPAQ archives | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3396 |
| C112 | upx (`upx.exe`) — UPX-packed executables (unpack) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3617-3623 |
| C113 | arc_conv (`arc_conv.exe`, UniExtract-authored) — KiriKiri/ERISA/YU-RIS engine archive conversion, GUI-automated | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2392-2412 |
| C114 | Actual Installer inner-blob handling (reuses `unzip.exe` + `7z.exe`, plus embedded `aisetup.ini` rename manifest) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2351-2383 |
| C115 | Advanced Installer self-extraction (native `/extract:` switch, no external binary) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2385-2390 |
| C116 | Excelsior Installer self-extraction (native switches) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2514-2516 |
| C117 | Netopsystems FEAD self-extraction (native switches) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2530-2536 |
| C118 | SuperDAT Updater self-extraction (native switches) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3038-3043 |
| C119 | InstallForge (wraps `7z.exe` + base64 path-rename logic) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2538-2554 |
| C120 | MSCF Cab installer (wraps `7z.exe` + Exeinfo-PE GUI rip) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2814-2839 |
| C121 | Unity `.unitypackage` decoder (wraps `7z.exe` + custom path-remapping logic) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3168-3209 |
| C122 | unadf (`unadf.exe`) — Amiga Disk Format (`.adf`) images | behavior | code | none found (directory check only) | REQUIRED | | def/adf.ini |
| C123 | bitrock-unpacker (`bitrock-unpacker.exe`) — BitRock InstallBuilder installers | behavior | code | none found (directory check only) | REQUIRED | | def/bitrock.ini |
| C124 | BSA Browser (`bsab.exe`) — Bethesda Archive (`.bsa`/`.ba2`/`.pex`) | behavior | code | none found (directory check only) | REQUIRED | | def/bsa.ini |
| C125 | godotdec (`godotdec.exe`) — Godot Engine packages (`.pck`, embedded in `.exe`) | behavior | code | none found (directory check only) | REQUIRED | | def/godot.ini |
| C126 | lbrate (`lbrate.exe`) — `.lbr`/`.lzr`/`.lqr` LBR archives | behavior | code | none found (directory check only) | REQUIRED | | def/lbr.ini |
| C127 | ConvertLIT (`clit.exe`) — Microsoft Reader `.lit` eBooks. **Known quirk, verify still present**: `todo.txt` documents an output-path-with-spaces bug for this extractor. | behavior | code | none found (directory check only) | REQUIRED | | def/lit.ini; todo.txt:29 |
| C128 | GNU gettext (`msgunfmt.exe`) — compiled GNU Gettext `.mo` message catalogs | behavior | code | none found (directory check only) | REQUIRED | | def/mo.ini |
| C129 | Champollion (`Champollion.exe`) — compiled Papyrus scripts (`.pex`, Bethesda) | behavior | code | none found (directory check only) | REQUIRED | | def/pex.ini |
| C130 | Qt Linguist (`lconvert.exe`) — Qt `.qm` compiled message translations | behavior | code | none found (directory check only) | REQUIRED | | def/qm.ini |
| C131 | rmvdec (`rmvdec.exe`) — RPG Maker MV encrypted resources (`.rpgmvp`/`.rpgmvo`/`.rpgmvm`) | behavior | code | none found (directory check only) | REQUIRED | | def/rpgmvp.ini |
| C132 | sgbdec (`sgbdec.exe`) — Smile Game Builder `.sgbpack` archives | behavior | code | none found (directory check only) | REQUIRED | | def/sgb.ini |
| C133 | simdec (`simdec.exe`) — Smart Install Maker installers | behavior | code | none found (directory check only) | REQUIRED | | def/sim.ini |
| C134 | unar / TheUnarchiver (`unar.exe`) — StuffIt (`.sit`/`.sitx`) archives | behavior | code | none found (directory check only) | REQUIRED | | def/sit.ini |
| C135 | spoondec (`spoondec.exe`) — Spoon Installer/Streaming installers | behavior | code | none found (directory check only) | REQUIRED | | def/spoon.ini |
| C136 | utagedec (`utagedec.exe`) — UTAGE visual novel engine files | behavior | code | none found (directory check only) | REQUIRED | | def/utage.ini |
| C137 | UUDeview (`uudeview.exe`) — UUencode/yEnc encoded files | behavior | code | none found (directory check only) | REQUIRED | | def/uu.ini |

| C138 | Output-subfolder default resolution — `/sub` resolves to a same-name subfolder, suffixed `_unpacked` if the input has no extension; falls back to an underscore-replaced name on collision | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:500-523 |
| C139 | Output-directory relative-path resolution — relative/partial outdir paths resolve against the input file's directory; a leading single backslash inherits the input's drive letter | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:531-537 |
| C140 | Output-directory trailing-slash handling. **Known quirk, verify still present**: `ValidateOutputDirectory()` always appends a trailing `\`, `extract()` strips it immediately then re-appends only at function end, so mid-extraction code sees no trailing slash — todo.txt documents this as a known inconsistency. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:540,2278,3413; todo.txt:35 |
| C141 | Drive-root output directory behavior. **Known quirk, verify still present**: stripping the trailing `\` from a drive-root outdir (e.g. `C:\`→`C:`) causes the spawned extractor to resolve to Windows' ambiguous drive-relative cwd — reproduces the documented "extracting to C:/ creates a file in the script dir" bug. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2278; todo.txt:25 |
| C142 | Output-directory creation and validation — create if missing (tracked for possible removal on failure); terminate with exit 5 if the path exists but isn't a writable directory | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3968-3978 |
| C143 | No centralized overwrite policy. **Known quirk, verify still present**: overwrite behavior is fully delegated to each helper binary's own default; no global overwrite-all flag is injected anywhere. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2269-3403; todo.txt:53 |
| C144 | Overwrite/replace message treated as extraction success, not failure, in output-log evaluation | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4819-4823 |
| C145 | Overwrite/password/no-space/new-filename prompt live-detection — streamed subprocess output is scanned for these substrings; on match, the extractor's window is force-shown for manual response (no auto-answer) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4930-4933 |
| C146 | DAA→ISO conversion has no pre-existing-output-file check. **Known quirk, verify still present**: matches the documented "converting to iso fails when iso already exists" bug. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2505-2508; todo.txt:52 |
| C147 | Batch queue file format and duplicate handling — re-invocable command lines appended to a queue file; exact duplicates prompt the user; multipart archive volumes collapse to one queue entry | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4389-4416,4354-4367 |
| C148 | Batch-item-per-process execution model — each queued item spawns a brand-new process rather than looping in-process; chaining driven by the terminating status | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4444-4462 |
| C149 | Batch stall on blocking user-input prompts. **Known quirk, verify still present**: if an extractor blocks on a modal prompt, the batch chain stalls indefinitely with no timeout — matches the documented bug. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4235-4237,4925-4958; todo.txt:54 |
| C150 | InstallShield-cab batch crash risk. **Known quirk, verify still present**: `is6cab.exe` still runs via blocking `RunWait` with no crash guard, matching the documented "might crash, stopping batch processing" bug. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2668-2674; todo.txt:27 |
| C151 | Batch-completion summary — on queue exhaustion, opens accumulated scan results (if any) and shows an error-log summary | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4451-4453 |
| C152 | Scan-only-mode short-circuit — bypasses the extraction dispatcher entirely, terminates with a file-info status | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:444-448 |
| C153 | Scan-only full-detail output — concatenation of every scanner's results (TrID, Unix file, Exeinfo PE, PEiD, MediaInfo), not just a single type name | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:5292-5313 |
| C154 | Scan-only silent-mode file output — results appended to the scan log file (path, blank line, scan text, separator), accumulated across a batch run | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4139-4142 |
| C155 | Generic post-extraction cleanup utility — delete or move-to-subfolder modes, wildcard expansion, folder-vs-file handling, invoked per-extractor for installer cruft | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3645-3703 |
| C156 | Per-run temp output directory always removed at end of extraction, success or failure | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3412 |
| C157 | Freshly-created, still-empty output directory removed on extraction failure; non-empty failed output directories are left in place | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4224 |
| C158 | Source-file deletion-on-success policy (keep/delete/ask/move), driven by C024; "ask" only prompts outside silent mode | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4204-4207 |
| C159 | Unicode-relocation reversion at end of run — temp/ASCII working copies are moved back (or recycled) regardless of outcome | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4101-4114 |
| C160 | Automated password-list trial — limited to 7z, DGCA, and RAR extraction; probes whether an archive is encrypted, then tries each password in the list until one succeeds or the list is exhausted | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4848-4877; used at 2290,2501,3004 |
| C161 | ACE archives excluded from the password-list trial. **Known quirk, verify still present**: explicitly marked `TODO: _FindArchivePassword` in source; wrong/missing password just fails generically for this one extractor. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2347 |
| C162 | Generic password-failure detection via output-text matching, for extractor types with no automated list trial | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4782-4787 |
| C163 | Live password-prompt detection — "password" substring in streamed output force-shows the extractor's window as a manual-entry fallback | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4931 |
| C164 | Debug-line accumulation (timestamped, buffered in memory for the full run) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:5352-5357 |
| C165 | Per-run log file naming/location/encoding — `<settingsdir>\log\YYYY-MM-DD_HH-MM-SS_[STATUS_]<filename>.<ext>.log`, UTF-16, status omitted from the filename on success | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4764-4775 |
| C166 | Teelog dual-output mechanism — subprocess stdout/stderr piped through a tee-like helper into a fixed live file, read incrementally for progress/prompt detection, folded into the run log and deleted at the end | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4880-5008,4883,4960-4965; todo.txt:19,37 |
| C167 | Output-log evaluation and warning extraction — classifies success/failure/cancel/no-space/missing-part from tee output; extracts tool-specific warning blocks | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4778-4825 (EvaluateLog), 4832-4845 (ParseWarnings) |
| C168 | QuickBMS-specific interactive-prompt detection ("choose a new filename", "insert disk with") via the same live-output pattern match as C145/C163 | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4932; todo.txt:34 |
| C169 | Batch-mode error-log append — one line per non-zero-exit failure during silent/batch runs (timestamp, filename, status, extractor type) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4216-4221 |
| C170 | Log-writing suppressed for certain terminal statuses (silent/syntax/non-silent-fileinfo/not-packed/batch) even when logging is enabled | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4231-4233 |
| C171 | Generic success/failure fallback heuristic — when an extractor case doesn't explicitly report success, infer it from output-directory size/mtime change relative to a pre-extraction snapshot | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3415-3430 |
| C172 | Undifferentiated failure messaging. **Known quirk, verify still present**: the same generic failure prompt is shown regardless of total vs. partial failure — matches a documented open TODO. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4195; todo.txt:48 |
| C173 | Batch continues past an ordinary (clean-exit) per-item failure — only a hang/crash stalls the chain, not a normal failed extraction | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:4235-4237 |
| C174 | Per-extractor timeout handling (implemented case-by-case, not globally; most cases have none) | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2398-2399 (representative case) |
| C175 | Non-ASCII filename/path relocation before processing — rename in place for a non-ASCII filename, copy/move to a temp dir for a non-ASCII directory (aborts with a warning if the temp dir itself is non-ASCII); multipart archive members are exempted | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2218-2266 |
| C176 | UNC-path input relocation — files reached via a UNC path are unconditionally relocated to a temp directory before processing | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2219,2236-2237 |
| C177 | Unicode-move bookkeeping lost on nested re-entry. **Known quirk, verify still present**: a recursive re-scan (e.g. after unpacking a self-extracting packer) resets unicode-mode state, discarding the outer run's relocation bookkeeping mid-flight. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:378,3603-3642,3633-3636; todo.txt:26 |
| C178 | TrID UNC-path detection reliability. **Known quirk, verify still present**: the TrID DLL load call uses ANSI string marshalling, consistent with the documented UNC-path detection-failure report. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:944-957,949; todo.txt:28 |
| C179 | Free-space check blocks unattended continuation — insufficient space terminates immediately in silent mode; interactive mode offers abort/retry/ignore | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3782-3808 |
| C180 | QuickBMS game-listing hang risk. **Known quirk, verify still present**: the blocking game-database listing call used during detection (not just extraction) has no timeout, consistent with a documented hang-with-high-CPU report. | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:1997-2049,2007; todo.txt:55 |
| C181 | Multi-stage/recursive extraction for nested container formats — several extractor types re-invoke the dispatcher on their own intermediate output | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:2351-2383,2477-2497,2994-3001 |
| C182 | Plugin extension point placeholder substitution (`%filename%`, `%fileext%`, `%filedir%`, `%file%`, `%outdir%`, translation-key placeholders) for `def/*.ini`-declared extractors | behavior | code | none found (directory check only) | REQUIRED | | UniExtract.au3:3523-3541 (ReplacePlaceholders) |
