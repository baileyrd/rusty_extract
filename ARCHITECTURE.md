# Architecture

## Overview
`rusty_extract` identifies the type of an input file (archive, installer, disk
image, game/media container, ...) and drives the correct external helper
binary to extract its contents, mirroring UniExtract2's core engine. It is not
a GUI, not a context-menu shell extension, not an auto-updater, and (per the
source project's own stated non-goal) not a compressor/archiver — extraction
only.

## Governing standards
Two capabilities this port centers on already have accepted architectural
decisions in `Rusty-Mill/rusty_foundation_akb` that predate this repo and
apply directly, so this repo conforms to them rather than inventing its own
detection/extraction model:

- **[ADR-0120 — Content identification is evidence, not intrinsic truth or use
  authority.](https://github.com/Rusty-Mill/rusty_foundation_akb/blob/main/docs/adr/0120-content-identification-is-evidence-not-intrinsic-truth-or-use-authority.md)**
  File-type detection (UniExtract2's TrIDLib-based signature scanning plus its
  `def/*.ini` heuristics) produces scored, possibly-conflicting evidence, never
  a single asserted "true type." Extension, magic-signature, and container-brand
  detections are represented and reported independently; the extractor
  selection step is the policy layer that picks an interpretation, not the
  detector.
- **[ADR-0119 — Extraction is a validated filesystem transaction.](https://github.com/Rusty-Mill/rusty_foundation_akb/blob/main/docs/adr/0119-extraction-is-a-validated-filesystem-transaction.md)**
  Archive entries are attacker-controlled (paths, links, sizes). Enumeration/
  validation of an archive's contents is kept separate from committing entries
  to disk — no writing a path straight out of an archive header without
  validating it stays under the destination root first. This is an
  implementation-hardening detail of *how* extraction is carried out, not a
  behavior change from the source: the observable outcome (files land in the
  chosen output directory) is preserved per the migration's boundary contract.

Both are cited per capability in the relevant issue/PR when a detection- or
extraction-path capability lands, rather than restated as this repo's own
invention. `ATLAS-300` (Rust workspace/Cargo architecture) is still `Seed`
status in `Atlas_Engineering_Standards_Library` and explicitly defers workspace
splitting until there's a second real crate — this repo starts as a single
crate (`src/lib.rs` + `src/main.rs`) accordingly.

## Boundaries
Ports-and-adapters, greenfield default (no repo-specific override yet):

| Port | Adapter(s) | Notes |
| ---- | ---------- | ----- |
| `TypeDetector` | signature/magic scanner, extension map, `def/*.ini`-equivalent format definitions (ported from source) | produces evidence per ADR-0120, not a single verdict |
| `ExtractorRunner` | one adapter per external helper binary (7-Zip, innoextract, unrar, ...) | shells out to the same helpers as the source, per the migration's own scope decision |
| `ExtractionTransaction` | staged-then-committed filesystem writer | enforces ADR-0119; domain code never writes an archive-supplied path directly to its final destination |

## Structure
Modular monolith, single crate for now (see "Governing standards" above for
why). A component only gets split into its own crate for a concrete forcing
function — e.g. the extractor-adapter layer growing large enough to want
independent versioning — not preemptively.

## Data flow
1. CLI/library caller passes a file path.
2. `TypeDetector` returns type evidence (candidate formats, confidence, source
   of each signal).
3. Policy selects an extractor adapter for the winning candidate.
4. `ExtractorRunner` invokes the corresponding external helper binary.
5. `ExtractionTransaction` validates and commits the helper's output into the
   destination directory.

## Key decisions
See [docs/adr/](./docs/adr/) for this repo's own decisions, and the two
`rusty_foundation_akb` ADRs cited above for the ones it inherits rather than
re-litigates.

## Non-goals
- Not a compressor/archiver (matches the source project's own stated
  non-goal).
- Not (yet, this phase) the AutoIt GUI, Windows context-menu integration,
  auto-updater, or feedback/telemetry system — staged for a later migration
  pass; see `capability-manifest.md`.
- Not cross-platform in this phase — Windows-only parity target, matching the
  source.
