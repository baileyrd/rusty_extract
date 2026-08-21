//! The process-spawning half of C148 ("Batch-item-per-process execution
//! model"), `batch::pop_batch_queue`'s own doc comment flags as missing:
//! `BatchQueuePop()`'s (UniExtract.au3:4444-4462) "spawn the next queued
//! item" branch. The source pops the next queued command line and spawns
//! it as a brand-new, non-waited process running the same script
//! (`Run(@ScriptFullPath & " " & $element)`) and returns immediately —
//! the *next* item is only popped once *that* new process's own
//! `terminate()` reaches the C173 continuation check
//! (`batch::should_continue_batch`) and calls `BatchQueuePop()` again.
//! There is no in-process loop; the chain is driven entirely by each
//! process's own exit.
//!
//! Splits real process-spawning I/O from a testable core, the same
//! pattern `extract::runner`'s `CommandExtractorRunner`/
//! `FakeExtractorRunner` split already uses.

use std::cell::RefCell;

/// A port over "spawn this program with these arguments and don't wait
/// for it" — `Run()`, not `RunWait()`.
pub trait BatchProcessLauncher {
    fn launch(&self, program: &str, args: &[String]) -> bool;
}

/// Real adapter: spawns `program` with `args` via
/// [`std::process::Command::spawn`], which — unlike
/// `extract::runner::CommandExtractorRunner`'s `.output()` — does not
/// wait for the child to finish, matching `Run()`'s fire-and-forget
/// semantics.
pub struct RealBatchProcessLauncher;

impl BatchProcessLauncher for RealBatchProcessLauncher {
    fn launch(&self, program: &str, args: &[String]) -> bool {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .is_ok()
    }
}

/// Test double: records every `(program, args)` pair it's called with
/// and reports a caller-configured canned success/failure, so code that
/// depends on a [`BatchProcessLauncher`] can be tested without spawning
/// a real process.
#[derive(Default)]
pub struct FakeBatchProcessLauncher {
    result: bool,
    calls: RefCell<Vec<(String, Vec<String>)>>,
}

impl FakeBatchProcessLauncher {
    pub fn new(result: bool) -> Self {
        Self {
            result,
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.borrow().clone()
    }
}

impl BatchProcessLauncher for FakeBatchProcessLauncher {
    fn launch(&self, program: &str, args: &[String]) -> bool {
        self.calls
            .borrow_mut()
            .push((program.to_string(), args.to_vec()));
        self.result
    }
}

/// Splits one popped batch-queue command line — built by
/// `batch::build_command_line` (C147) as `"<file>" [/sub|"<outdir>"|
/// /scan] [/silent]` — back into argv, the reverse operation the
/// relaunch needs before handing it to [`BatchProcessLauncher::launch`].
/// Only understands that shape: a run of tokens each either a
/// double-quoted string or a bare, whitespace-free `/`-prefixed flag —
/// not a general Windows command-line tokenizer (the source doesn't need
/// one either, since `Run()` re-parses a string *it* built in exactly
/// this format).
pub fn split_batch_command_line(cmdline: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = cmdline.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '"' {
            chars.next();
            let mut token = String::new();
            for ch in chars.by_ref() {
                if ch == '"' {
                    break;
                }
                token.push(ch);
            }
            args.push(token);
        } else {
            let mut token = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    break;
                }
                token.push(ch);
                chars.next();
            }
            args.push(token);
        }
    }
    args
}

/// Reproduces `BatchQueuePop()`'s "spawn the next item" branch
/// (UniExtract.au3:4455-4460): pops `queue` via `batch::pop_batch_queue`
/// and, if an element was popped, spawns `current_exe` with that
/// element's own argv (via [`split_batch_command_line`]) appended.
/// Returns the remaining queue, or `None` when `queue` was already
/// empty — mirroring `pop_batch_queue`'s own `None` so a caller can
/// drive C151's completion-summary branch instead of relaunching.
///
/// Whether the spawn itself succeeded doesn't change the return value:
/// the source's own `Run()` call doesn't check its success either — a
/// batch chain that fails to relaunch just silently stalls, matching
/// C149's documented "stalls indefinitely" quirk family.
pub fn pop_and_relaunch_next_batch_item(
    queue: &[String],
    current_exe: &str,
    launcher: &dyn BatchProcessLauncher,
) -> Option<Vec<String>> {
    let (element, rest) = crate::batch::pop_batch_queue(queue)?;
    let args = split_batch_command_line(&element);
    launcher.launch(current_exe, &args);
    Some(rest)
}

/// What `BatchQueuePop()`'s "queue empty" branch does once the batch
/// queue is exhausted (UniExtract.au3:4448-4454) — capability C151.
/// The branch this doc comment on [`pop_and_relaunch_next_batch_item`]
/// flagged as still missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchCompletionActions {
    /// `If FileExists($fileScanLogFile) Then ShellExecute($fileScanLogFile)`
    /// (line 4451) — open the accumulated scan-results log, but only if
    /// scan-only mode ever actually wrote one.
    pub open_scan_log: bool,
    /// `If $return <> "" Then MsgBox(...)` (line 4453) — show the
    /// error-log summary dialog, but only if `errorlog.txt` actually
    /// has content.
    pub show_error_summary: bool,
    /// `If $bOptKeepOpen Then Run(@ScriptFullPath)` (line 4454) —
    /// relaunch the app with no arguments. Gated on `$bOptKeepOpen`
    /// alone here, **not** the same condition as `terminate()`'s own
    /// (unrelated) keep-open relaunch at UniExtract.au3:4238, which
    /// also requires `$cmdline[0] = 0` and a non-`$STATUS_SILENT`
    /// status — the two call sites are easy to conflate but genuinely
    /// distinct.
    pub relaunch_self: bool,
}

/// Ports `BatchQueuePop()`'s "queue empty" branch decision
/// (UniExtract.au3:4448-4454), given the real-I/O facts the caller has
/// already gathered: whether `$fileScanLogFile` exists, and
/// `errorlog.txt`'s content (empty means "nothing to summarize").
pub fn decide_batch_completion_actions(
    scan_log_exists: bool,
    error_log_content: &str,
    keep_open: bool,
) -> BatchCompletionActions {
    BatchCompletionActions {
        open_scan_log: scan_log_exists,
        show_error_summary: !error_log_content.is_empty(),
        relaunch_self: keep_open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_batch_command_line_quoted_file_and_sub_token() {
        assert_eq!(
            split_batch_command_line(r#""C:\downloads\archive.zip" /sub"#),
            vec![r"C:\downloads\archive.zip".to_string(), "/sub".to_string()]
        );
    }

    #[test]
    fn split_batch_command_line_quoted_file_and_quoted_outdir_and_silent() {
        assert_eq!(
            split_batch_command_line(r#""C:\a.zip" "C:\out" /silent"#),
            vec![
                r"C:\a.zip".to_string(),
                r"C:\out".to_string(),
                "/silent".to_string()
            ]
        );
    }

    #[test]
    fn split_batch_command_line_scan_only() {
        assert_eq!(
            split_batch_command_line(r#""C:\a.zip" /scan"#),
            vec![r"C:\a.zip".to_string(), "/scan".to_string()]
        );
    }

    #[test]
    fn split_batch_command_line_file_only() {
        assert_eq!(
            split_batch_command_line(r#""C:\a.zip""#),
            vec![r"C:\a.zip".to_string()]
        );
    }

    #[test]
    fn pop_and_relaunch_spawns_current_exe_with_popped_element_argv() {
        let queue = vec![
            r#""C:\a.zip" /sub"#.to_string(),
            r#""C:\b.zip" /scan"#.to_string(),
        ];
        let launcher = FakeBatchProcessLauncher::new(true);

        let rest =
            pop_and_relaunch_next_batch_item(&queue, r"C:\UniExtract\UniExtract.exe", &launcher)
                .unwrap();

        assert_eq!(rest, vec![r#""C:\b.zip" /scan"#.to_string()]);
        let calls = launcher.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, r"C:\UniExtract\UniExtract.exe");
        assert_eq!(
            calls[0].1,
            vec![r"C:\a.zip".to_string(), "/sub".to_string()]
        );
    }

    #[test]
    fn pop_and_relaunch_returns_none_for_empty_queue_without_launching() {
        let launcher = FakeBatchProcessLauncher::new(true);
        let rest =
            pop_and_relaunch_next_batch_item(&[], r"C:\UniExtract\UniExtract.exe", &launcher);
        assert_eq!(rest, None);
        assert!(launcher.calls().is_empty());
    }

    #[test]
    fn pop_and_relaunch_pops_regardless_of_spawn_success() {
        let queue = vec![r#""C:\a.zip" /sub"#.to_string()];
        let launcher = FakeBatchProcessLauncher::new(false);

        let rest =
            pop_and_relaunch_next_batch_item(&queue, r"C:\UniExtract\UniExtract.exe", &launcher);

        assert_eq!(rest, Some(vec![]));
        assert_eq!(launcher.calls().len(), 1);
    }

    #[test]
    fn real_launcher_spawns_a_process_without_waiting() {
        let launcher = RealBatchProcessLauncher;
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "exit".to_string()])
        } else {
            ("sh", vec!["-c".to_string(), "exit 0".to_string()])
        };
        assert!(launcher.launch(program, &args));
    }

    #[test]
    fn real_launcher_reports_launch_failure() {
        let launcher = RealBatchProcessLauncher;
        assert!(!launcher.launch("definitely-not-a-real-program-xyz", &[]));
    }

    #[test]
    fn completion_opens_scan_log_only_when_it_exists() {
        assert!(decide_batch_completion_actions(true, "", false).open_scan_log);
        assert!(!decide_batch_completion_actions(false, "", false).open_scan_log);
    }

    /// Parity test for capability C151: the error summary only shows
    /// when `errorlog.txt` actually has content.
    #[test]
    fn completion_shows_error_summary_only_when_log_has_content() {
        assert!(decide_batch_completion_actions(false, "3 files failed", false).show_error_summary);
        assert!(!decide_batch_completion_actions(false, "", false).show_error_summary);
    }

    #[test]
    fn completion_relaunches_only_when_keep_open_is_set() {
        assert!(decide_batch_completion_actions(false, "", true).relaunch_self);
        assert!(!decide_batch_completion_actions(false, "", false).relaunch_self);
    }

    #[test]
    fn completion_actions_are_independent_of_each_other() {
        let actions = decide_batch_completion_actions(true, "some error", true);
        assert_eq!(
            actions,
            BatchCompletionActions {
                open_scan_log: true,
                show_error_summary: true,
                relaunch_self: true,
            }
        );
    }
}
