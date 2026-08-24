//! Core file-type detection and extraction-orchestration engine, ported from
//! UniExtract2 (AutoIt). See ARCHITECTURE.md for the port's boundaries.

pub mod automation;
pub mod batch;
pub mod batch_runner;
pub mod bms;
pub mod cleanup;
pub mod cli;
pub mod dest_arg;
pub mod detection;
pub mod detector_silence;
pub mod dlllib;
pub mod entry_gate;
pub mod extract;
pub mod extractor_timeout;
pub mod failure_message;
pub mod file_arg;
pub mod filetype_report;
pub mod free_space;
pub mod gui;
pub mod ini;
pub mod log_eval;
pub mod method_select;
pub mod outdir;
pub mod password_search;
pub mod prefs;
pub mod result_heuristic;
pub mod run_log;
pub mod status;
pub mod teelog;
pub mod type_override;
pub mod unicode_relocation;
pub mod update_ffmpeg;
pub mod update_helpers;
pub mod update_index;
pub mod update_orchestration;
pub mod warn_execute;
