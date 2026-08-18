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
