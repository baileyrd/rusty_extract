//! Undifferentiated failure messaging: `terminate()`'s `Case
//! $STATUS_FAILED` branch shows exactly one message key,
//! `'EXTRACT_FAILED'`, regardless of whether the run failed completely or
//! only partially — there is no total-vs-partial distinction anywhere in
//! the source before reaching this branch, a documented open TODO
//! (`todo.txt:48`).
//!
//! ```autoit
//! Case $STATUS_FAILED
//!     If Not $silentmode And Prompt(256 + 16 + 4, 'EXTRACT_FAILED', CreateArray($filenamefull, $arcdisp)) Then
//!         ShellExecute(SaveLog($status))
//!         $bLogSaved = True
//!     EndIf
//! ```
//!
//! **Scope — the gate only, not the prompt itself.** `Prompt(...)` is a
//! `MsgBox` GUI dialog — deferred under the same GUI-subsystem boundary
//! (manifest row D001) as every other interactive prompt in this port
//! (e.g. `free_space::FreeSpaceOutcome::PromptInteractive`,
//! `outdir`'s already-documented `MsgBox`). What this module pins down
//! is the quirk itself: [`FAILURE_MESSAGE_KEY`] is the single,
//! unconditional key every `Status::Failed` outcome uses, and
//! [`failure_prompt_should_fire`] ports the `Not $silentmode` half of
//! the guard that decides whether `Prompt(...)` is even reached — its
//! own return value (whether the user chose to save the log) governs
//! `SaveLog`/`ShellExecute`, real GUI I/O out of scope here.

/// The single message key `Case $STATUS_FAILED` passes to `Prompt()`
/// (UniExtract.au3:4195) — unconditional: nothing in the source
/// distinguishes a total failure from a partial one before reaching this
/// branch, so every `Status::Failed` outcome shows this same key.
pub const FAILURE_MESSAGE_KEY: &str = "EXTRACT_FAILED";

/// Ports the `Not $silentmode` half of `If Not $silentmode And
/// Prompt(...) Then` (UniExtract.au3:4195): the failure prompt is only
/// attempted when not running silently.
pub fn failure_prompt_should_fire(silent_mode: bool) -> bool {
    !silent_mode
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C172: the message key is the fixed
    /// literal `'EXTRACT_FAILED'` (UniExtract.au3:4195), with no
    /// total-vs-partial variant.
    #[test]
    fn failure_message_key_matches_source_literal() {
        assert_eq!(FAILURE_MESSAGE_KEY, "EXTRACT_FAILED");
    }

    /// Parity test for capability C172: the prompt is attempted when not
    /// silent.
    #[test]
    fn failure_prompt_fires_when_not_silent() {
        assert!(failure_prompt_should_fire(false));
    }

    /// Parity test for capability C172: silent mode suppresses the
    /// prompt outright, regardless of what caused `Status::Failed`.
    #[test]
    fn failure_prompt_suppressed_in_silent_mode() {
        assert!(!failure_prompt_should_fire(true));
    }
}
