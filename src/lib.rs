//! Core file-type detection and extraction-orchestration engine, ported from
//! UniExtract2 (AutoIt). See ARCHITECTURE.md for the port's boundaries.

pub mod batch;
pub mod cli;
pub mod detection;
pub mod extract;
pub mod ini;
pub mod log_eval;
pub mod outdir;
pub mod prefs;
pub mod run_log;
pub mod status;
