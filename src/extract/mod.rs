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

pub mod ace;
pub mod bcm;
pub mod chdman;
pub mod cic;
pub mod dispatch;
pub mod extsis;
pub mod freearc;
pub mod fsb;
pub mod garbro;
pub mod isz;
pub mod kgb;
pub mod lzip;
pub mod lzop;
pub mod plugin;
pub mod rgss;
pub mod rpa;
pub mod sfark;
pub mod uif;
pub mod upx;
pub mod xor;

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
/// actually passes to `_Run`/`RunWait`/`Run` when launching an extractor
/// helper binary.
///
/// Verified exhaustive by grepping every `@SW_*` occurrence in
/// UniExtract.au3 (audit finding F2): `@SW_HIDE`, `@SW_MINIMIZE`, and
/// `@SW_SHOW` are the only ones reaching an extractor invocation (the
/// numeric literal `True`/`1` some `_Run` calls pass — e.g.
/// `extract::rpa` — is AutoIt's `@SW_SHOWNORMAL`, mapped to `Show` here).
/// The two other `@SW_*` constants in the source, `@SW_SHOWNORMAL` and
/// `@SW_SHOWNOACTIVATE`, appear only in `GUISetState(...)` calls governing
/// the main window — out of scope per this migration's deferred GUI
/// subsystem (manifest row D001) — so this enum has no missing variant to
/// add for any capability still to be ported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Hidden,
    Minimized,
    Show,
}
