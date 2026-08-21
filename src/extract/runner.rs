//! `ExtractorRunner` port (see `ARCHITECTURE.md`): the seam between building
//! an [`Invocation`] and actually running the external helper binary it
//! describes. Splits real process-spawning I/O from a testable core, the
//! same pattern `extract::plugin`'s `resolve_plugin_ini`/
//! `resolve_plugin_ini_with` split already uses, so callers can be tested
//! without spawning a real process — necessary since CI can't run the real
//! Windows helper binaries (see this module's parent doc comment).

use std::cell::RefCell;
use std::process::Command;

use super::Invocation;

/// The captured result of running one [`Invocation`]. No attempt is made
/// here to classify success/failure — `log_eval` does that, applied to
/// `stdout`/`stderr` by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// A port over "run this external helper binary and capture its output".
pub trait ExtractorRunner {
    fn run(&self, invocation: &Invocation) -> RunOutcome;
}

/// Real adapter: spawns `invocation.program` with `invocation.args` in
/// `invocation.working_dir`, waits for it to finish, and captures its full
/// stdout/stderr via [`Command::output`] — not a streaming read. The
/// source's only use of *streamed* output is force-showing the extractor's
/// window on a detected prompt (`log_eval::needs_manual_input`), and that
/// windowing is itself out of scope (deferred GUI subsystem, manifest row
/// D001), so there's no phase-1 reason to build a streaming loop.
///
/// `invocation.window` is not applied here: window visibility is the same
/// out-of-scope GUI concern.
pub struct CommandExtractorRunner;

impl ExtractorRunner for CommandExtractorRunner {
    fn run(&self, invocation: &Invocation) -> RunOutcome {
        let output = Command::new(&invocation.program)
            .args(&invocation.args)
            .current_dir(&invocation.working_dir)
            .output();

        match output {
            Ok(output) => RunOutcome {
                exit_status: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(err) => RunOutcome {
                exit_status: None,
                stdout: String::new(),
                stderr: err.to_string(),
            },
        }
    }
}

/// Test double: records every [`Invocation`] it's called with and returns a
/// caller-configured canned [`RunOutcome`], so code that depends on an
/// [`ExtractorRunner`] can be tested without spawning a real process.
pub struct FakeExtractorRunner {
    outcome: RunOutcome,
    calls: RefCell<Vec<Invocation>>,
}

impl FakeExtractorRunner {
    pub fn new(outcome: RunOutcome) -> Self {
        Self {
            outcome,
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<Invocation> {
        self.calls.borrow().clone()
    }
}

impl ExtractorRunner for FakeExtractorRunner {
    fn run(&self, invocation: &Invocation) -> RunOutcome {
        self.calls.borrow_mut().push(invocation.clone());
        self.outcome.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::WindowMode;

    /// The real adapter actually spawns a process and captures its output —
    /// exercised against a trivial, always-present shell command rather than
    /// a real UniExtract2 helper binary (none are installed on CI, see this
    /// module's parent doc comment).
    #[test]
    fn command_runner_captures_a_real_process_output() {
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "echo hello".to_string()])
        } else {
            ("sh", vec!["-c".to_string(), "echo hello".to_string()])
        };
        let invocation = Invocation {
            program: program.to_string(),
            args,
            working_dir: std::env::temp_dir().to_string_lossy().into_owned(),
            window: WindowMode::Hidden,
        };

        let outcome = CommandExtractorRunner.run(&invocation);

        assert_eq!(outcome.exit_status, Some(0));
        assert!(outcome.stdout.contains("hello"));
    }

    /// Parity test for capability C150 ("InstallShield-cab batch crash
    /// risk" — is6comp.exe's blocking `RunWait` call,
    /// `extract::iscab::is6comp_extract_invocation`,
    /// UniExtract.au3:2668-2674): `run()` blocks until the child process
    /// actually exits and returns its real exit code — there's no
    /// timeout parameter anywhere in this port's execution model for a
    /// caller to opt into. A crashed or hung extractor (is6comp
    /// included) would stall this port's own call the same way the
    /// source's blocking `RunWait` with no crash guard stalls its batch
    /// chain — a documented bug (`todo.txt:27`), reproduced here rather
    /// than fixed.
    #[test]
    fn command_runner_blocks_until_exit_with_no_timeout_escape() {
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "exit 7".to_string()])
        } else {
            ("sh", vec!["-c".to_string(), "exit 7".to_string()])
        };
        let invocation = Invocation {
            program: program.to_string(),
            args,
            working_dir: std::env::temp_dir().to_string_lossy().into_owned(),
            window: WindowMode::Hidden,
        };

        let outcome = CommandExtractorRunner.run(&invocation);

        assert_eq!(outcome.exit_status, Some(7));
    }

    /// The real adapter reports a non-existent program as a captured
    /// failure (`exit_status: None`, the launch error in `stderr`) rather
    /// than panicking.
    #[test]
    fn command_runner_reports_launch_failure_without_panicking() {
        let invocation = Invocation {
            program: "definitely-not-a-real-program-xyz".to_string(),
            args: vec![],
            working_dir: std::env::temp_dir().to_string_lossy().into_owned(),
            window: WindowMode::Hidden,
        };

        let outcome = CommandExtractorRunner.run(&invocation);

        assert_eq!(outcome.exit_status, None);
        assert!(!outcome.stderr.is_empty());
    }

    #[test]
    fn fake_runner_records_calls_and_returns_canned_outcome() {
        let fake = FakeExtractorRunner::new(RunOutcome {
            exit_status: Some(0),
            stdout: "done".to_string(),
            stderr: String::new(),
        });
        let invocation = Invocation {
            program: "prog".to_string(),
            args: vec!["a".to_string()],
            working_dir: "dir".to_string(),
            window: WindowMode::Hidden,
        };

        let outcome = fake.run(&invocation);

        assert_eq!(outcome.stdout, "done");
        assert_eq!(fake.calls(), vec![invocation]);
    }
}
