//! File-type detection: the cascade of detectors UniExtract2 runs before
//! dispatching to an extractor. See `ARCHITECTURE.md` (ADR-0120) — detection
//! produces evidence/candidates, not a single asserted "true type".

pub mod registry;
