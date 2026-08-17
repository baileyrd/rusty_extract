//! Extractor integrations: one module per external helper binary
//! UniExtract2 shells out to. See ARCHITECTURE.md (ADR-0119) — extraction
//! is a validated filesystem transaction, so these modules build the
//! external invocation; committing its output to the destination is a
//! separate concern, not this module's job.
//!
//! CI can't run the real Windows helper binaries (not installed on the
//! runner — see `RELEASE_NOTES.md`, "CI: target windows-latest"), so parity
//! tests here verify the constructed [`Invocation`] matches the source's
//! `_Run(...)` call for the same capability, not an actual extraction.

pub mod rgss;

/// A single external helper-binary invocation, corresponding to one
/// UniExtract2 `_Run(...)`/`_RunInTempOutdir(...)` call: the command line
/// UniExtract2 builds as one raw string, decomposed into a program and its
/// argument vector, plus the working directory and window visibility the
/// source passes alongside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub window: WindowMode,
}

/// Mirrors the subset of AutoIt's `@SW_*` window-show flags UniExtract2
/// actually passes to `_Run`/`Run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Hidden,
    Minimized,
    Show,
}
