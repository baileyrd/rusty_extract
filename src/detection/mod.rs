//! File-type detection: the cascade of detectors UniExtract2 runs before
//! dispatching to an extractor. See `ARCHITECTURE.md` (ADR-0120) — detection
//! produces evidence/candidates, not a single asserted "true type".

pub mod alz_probe;
pub mod arj_probe;
pub mod cascade;
pub mod detector_mapping;
pub mod initial_ext_check;
pub mod registry;
pub mod sevenzip_probe;
