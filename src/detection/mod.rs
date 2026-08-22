//! File-type detection: the cascade of detectors UniExtract2 runs before
//! dispatching to an extractor. See `ARCHITECTURE.md` (ADR-0120) — detection
//! produces evidence/candidates, not a single asserted "true type".

pub mod alz_probe;
pub mod arj_probe;
pub mod cascade;
pub mod detector_mapping;
pub mod exeinfo_dispatch;
pub mod exeinfo_scan;
pub mod file_dispatch;
pub mod initial_ext_check;
pub mod mediainfo_scan;
pub mod peid_dispatch;
pub mod peid_scan;
pub mod registry;
pub mod sevenzip_probe;
pub mod trid_dispatch;
pub mod trid_scan;
pub mod unixfile_scan;
