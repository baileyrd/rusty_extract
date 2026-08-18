//! Core file-type detection and extraction-orchestration engine, ported from
//! UniExtract2 (AutoIt). See ARCHITECTURE.md for the port's boundaries.

pub mod batch;
pub mod batch_runner;
pub mod cleanup;
pub mod cli;
pub mod detection;
pub mod extract;
pub mod filetype_report;
pub mod free_space;
pub mod ini;
pub mod log_eval;
pub mod outdir;
pub mod password_search;
pub mod prefs;
pub mod result_heuristic;
pub mod run_log;
pub mod status;
