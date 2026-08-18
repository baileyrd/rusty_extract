//! Batch queue file format and duplicate handling: ports `GetCmd()`
//! (UniExtract.au3:4370-4386), `AddToBatch()`'s add-vs-skip decision
//! (UniExtract.au3:4389-4416), and `IsMultipartArchive()`/`__TestMultipart()`
//! (UniExtract.au3:4354-4367) — capability C147.

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Ports `StringRegExpReplace($filenamefull, '(.*?\.part)(\d+\.rar)',
/// "$1", 1)` + `@extended > 0`: the earliest `.part<digits>.rar`
/// occurrence, replaced by its own `.part`-inclusive prefix (matching
/// AutoIt's non-greedy `.*?` leftmost-match semantics) — `None` if the
/// pattern never matches anywhere in `name`.
fn match_part_rar_volume(name: &str) -> Option<String> {
    if !name.is_ascii() {
        return None;
    }
    let bytes = name.as_bytes();
    let marker = b".part";
    let mut start = 0;
    while let Some(rel) = find_bytes(&bytes[start..], marker) {
        let marker_end = start + rel + marker.len();
        let mut digit_end = marker_end;
        while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
            digit_end += 1;
        }
        if digit_end > marker_end && bytes[digit_end..].starts_with(b".rar") {
            let match_end = digit_end + 4;
            return Some(format!("{}{}", &name[..marker_end], &name[match_end..]));
        }
        start = marker_end;
    }
    None
}

/// Ports `StringRegExpReplace($filenamefull, '(.*?\.7z.)(\d{3})', "$1",
/// 1)` + `@extended > 0`: `.7z` followed by exactly one arbitrary
/// character (the unescaped `.` wildcard in the source's own regex) and
/// three digits, e.g. `.7z.001` — group 1 includes that wildcard
/// character.
fn match_7z_volume(name: &str) -> Option<String> {
    if !name.is_ascii() {
        return None;
    }
    let bytes = name.as_bytes();
    let marker = b".7z";
    let mut start = 0;
    while let Some(rel) = find_bytes(&bytes[start..], marker) {
        let marker_end = start + rel + marker.len();
        if marker_end < bytes.len() {
            let wildcard_end = marker_end + 1;
            let digits_end = wildcard_end + 3;
            if digits_end <= bytes.len()
                && bytes[wildcard_end..digits_end]
                    .iter()
                    .all(u8::is_ascii_digit)
            {
                return Some(format!("{}{}", &name[..wildcard_end], &name[digits_end..]));
            }
        }
        start = marker_end;
    }
    None
}

/// Ports `StringRegExpReplace($filenamefull, '(.*?\.r)((\d{2})|ar)',
/// "$1", 1)` + `@extended > 0`: `.r` followed by either two digits
/// (`.r00`) or the literal `ar` — the second alternative means a plain
/// solo `.rar` file (no digits at all) also matches this pattern, since
/// `.r` + `ar` decomposes any `....rar` ending exactly. That's a real
/// quirk in the source, not something this port introduces: a
/// single-volume `.rar` is treated the same as a multipart one for batch
/// collision purposes. The two-digit alternative is tried first, then
/// falls back to `ar`, matching how a PCRE-style engine tries
/// alternatives left-to-right.
fn match_r_volume(name: &str) -> Option<String> {
    if !name.is_ascii() {
        return None;
    }
    let bytes = name.as_bytes();
    let marker = b".r";
    let mut start = 0;
    while let Some(rel) = find_bytes(&bytes[start..], marker) {
        let marker_end = start + rel + marker.len();
        if marker_end + 2 <= bytes.len()
            && bytes[marker_end..marker_end + 2]
                .iter()
                .all(u8::is_ascii_digit)
        {
            let match_end = marker_end + 2;
            return Some(format!("{}{}", &name[..marker_end], &name[match_end..]));
        }
        if bytes[marker_end..].starts_with(b"ar") {
            let match_end = marker_end + 2;
            return Some(format!("{}{}", &name[..marker_end], &name[match_end..]));
        }
        start = marker_end;
    }
    None
}

/// Ports `IsMultipartArchive()` (UniExtract.au3:4354-4359): `true` if
/// `filenamefull` matches any of the three multipart-volume patterns
/// above *and* that match's prefix already appears somewhere in
/// `queue_content` — i.e. some other volume of the same archive (or, per
/// the `match_r_volume` quirk, the same solo archive) is already queued.
///
/// `StringInStr` here has no case-sensitivity argument, so — like every
/// other `StringInStr`/`StringComparison` this port has encountered
/// (C007-C013, C144, C145) — it defaults to case-insensitive.
pub fn is_multipart_archive_already_queued(filenamefull: &str, queue_content: &str) -> bool {
    let queue_lower = queue_content.to_lowercase();
    [
        match_part_rar_volume(filenamefull),
        match_7z_volume(filenamefull),
        match_r_volume(filenamefull),
    ]
    .into_iter()
    .flatten()
    .any(|prefix| queue_lower.contains(&prefix.to_lowercase()))
}

/// Ports `GetCmd($silent = True)` (UniExtract.au3:4370-4386): builds the
/// re-invocable command line UniExtract2 appends to the batch queue
/// file for the current run's file — `Quote($file)` (always
/// double-quoted, UniExtract.au3:3598-3600) followed by the destination
/// argument (`/sub` verbatim, a quoted `outdir`, or nothing if `outdir`
/// is empty, only when `extract` is true — `/scan` otherwise) and a
/// trailing `/silent` if either `silentmode` or `silent` is set.
pub fn build_command_line(
    file: &str,
    extract: bool,
    outdir: &str,
    silentmode: bool,
    silent: bool,
) -> String {
    let mut cmd = format!("\"{file}\"");
    if extract {
        if outdir == "/sub" {
            cmd.push_str(" /sub");
        } else if !outdir.is_empty() {
            cmd.push_str(&format!(" \"{outdir}\""));
        }
    } else {
        cmd.push_str(" /scan");
    }
    if silentmode || silent {
        cmd.push_str(" /silent");
    }
    cmd
}

/// C148: ports the queue-popping half of `BatchQueuePop()`
/// (UniExtract.au3:4444-4462) — removes and returns the first queued
/// command line, leaving the rest as the new persisted queue (the
/// source's `_ArrayDelete($queueArray, 0)` followed by
/// `SaveBatchQueue()`), or `None` if the queue is already empty.
///
/// This is only the FIFO queue-management half of C148's own
/// description. The other half — "each queued item spawns a brand-new
/// process rather than looping in-process; chaining driven by the
/// terminating status" — describes the source's actual execution model:
/// `BatchQueuePop()` spawns the popped command line as a fresh process
/// (`Run(@ScriptFullPath & " " & $element)`) and returns immediately;
/// the *next* item in the queue is only popped when *that* new process's
/// own `terminate()` call reaches its `$batchEnabled = 1 And $status <>
/// $STATUS_SILENT` check (UniExtract.au3:4235) and calls
/// `BatchQueuePop()` again — so the chain is driven entirely by each
/// process's own exit, never by a loop inside a single running process.
/// That process-spawning-and-chaining architecture is this port's own
/// runtime concern (not yet built) rather than portable pure logic, so
/// it isn't reproduced by this function — only the queue-array mechanics
/// are.
pub fn pop_batch_queue(queue: &[String]) -> Option<(String, Vec<String>)> {
    let (first, rest) = queue.split_first()?;
    Some((first.clone(), rest.to_vec()))
}

/// Ports the add-vs-skip decision inside `AddToBatch()`
/// (UniExtract.au3:4398-4404): an exact-duplicate command line already
/// present in the queue defers to `user_confirmed_duplicate` (standing
/// in for `CustomPrompt('BATCH_DUPLICATE', ...)`, out of scope, deferred
/// GUI subsystem); otherwise, a multipart-archive match against the
/// existing queue content silently suppresses the add — no prompt at
/// all in that branch, matching the source exactly.
///
/// The exact-duplicate check (`StringInStr($sBatchQueueContent,
/// $cmdline)`) is also case-insensitive by default, same as
/// [`is_multipart_archive_already_queued`].
pub fn should_add_to_batch(
    queue_content: &str,
    cmdline: &str,
    filenamefull: &str,
    user_confirmed_duplicate: bool,
) -> bool {
    if queue_content
        .to_lowercase()
        .contains(&cmdline.to_lowercase())
    {
        user_confirmed_duplicate
    } else {
        !is_multipart_archive_already_queued(filenamefull, queue_content)
    }
}

/// C173: ports the batch-continuation gate inside `terminate()`
/// (UniExtract.au3:4235-4237): `If $batchEnabled = 1 And $status <>
/// $STATUS_SILENT Then BatchQueuePop()`. A `Failed` status (or any other
/// ordinary terminal status) still satisfies `status != Silent`, so a
/// normal, clean-exit per-item failure does **not** stop the chain — only
/// `$STATUS_SILENT` (used when the GUI itself has been closed/aborted)
/// does. This is the condition [`pop_batch_queue`]'s own doc comment
/// already describes in prose (the next process's own `terminate()` call
/// reaching this check before popping again); this function is that
/// check itself, ported.
///
/// **Not modeled here:** whether an extraction *hangs or crashes*
/// instead of exiting cleanly — that's a process-liveness concern for
/// this port's not-yet-built runtime orchestration (the same territory
/// `pop_batch_queue`'s own doc comment already flags), not something a
/// status-comparison function can observe.
pub fn should_continue_batch(batch_enabled: bool, status: crate::status::Status) -> bool {
    batch_enabled && status != crate::status::Status::Silent
}

#[cfg(test)]
mod tests {
    use super::{
        build_command_line, is_multipart_archive_already_queued, pop_batch_queue,
        should_add_to_batch, should_continue_batch,
    };
    use crate::status::Status;

    /// Parity test for capability C147: `GetCmd`'s command-line shape
    /// across its `extract`/`/sub`/plain-outdir/`/scan`/`/silent`
    /// branches.
    #[test]
    fn build_command_line_matches_source_shapes() {
        assert_eq!(
            build_command_line(r"C:\downloads\archive.zip", true, "/sub", false, true),
            r#""C:\downloads\archive.zip" /sub /silent"#
        );
        assert_eq!(
            build_command_line(r"C:\downloads\archive.zip", true, r"D:\out", false, false),
            r#""C:\downloads\archive.zip" "D:\out""#
        );
        assert_eq!(
            build_command_line(r"C:\downloads\archive.zip", true, "", false, false),
            r#""C:\downloads\archive.zip""#
        );
        assert_eq!(
            build_command_line(r"C:\downloads\archive.zip", false, "", true, false),
            r#""C:\downloads\archive.zip" /scan /silent"#
        );
    }

    /// Parity test for capability C147: a `.partN.rar` volume whose
    /// prefix is already queued is detected as a multipart duplicate.
    #[test]
    fn detects_queued_part_rar_volume() {
        assert!(is_multipart_archive_already_queued(
            "archive.part2.rar",
            r#""C:\downloads\archive.part1.rar""#
        ));
        assert!(!is_multipart_archive_already_queued(
            "archive.part2.rar",
            r#""C:\downloads\other.zip""#
        ));
    }

    /// Parity test for capability C147: a `.7zNNN` volume whose prefix
    /// is already queued is detected.
    #[test]
    fn detects_queued_7z_volume() {
        assert!(is_multipart_archive_already_queued(
            "archive.7z.002",
            r#""C:\downloads\archive.7z.001""#
        ));
    }

    /// Parity test for capability C147: the `.r` pattern's documented
    /// quirk — a plain solo `.rar` (no digits) also matches, via the
    /// `ar` alternative, and collides with any other queued name sharing
    /// the same `.r`-prefix.
    #[test]
    fn solo_rar_matches_via_ar_alternative_quirk() {
        assert!(is_multipart_archive_already_queued(
            "archive.rar",
            r#""C:\downloads\archive.rar""#
        ));
    }

    /// Parity test for capability C147: `should_add_to_batch`'s two
    /// branches — exact duplicate defers to the (simulated) prompt
    /// answer; a multipart match silently suppresses with no prompt.
    #[test]
    fn should_add_to_batch_matches_source_branches() {
        let cmdline = r#""C:\downloads\archive.zip""#;
        assert!(should_add_to_batch(cmdline, cmdline, "archive.zip", true));
        assert!(!should_add_to_batch(cmdline, cmdline, "archive.zip", false));
        assert!(!should_add_to_batch(
            r#""C:\downloads\archive.part1.rar""#,
            r#""C:\downloads\archive.part2.rar""#,
            "archive.part2.rar",
            false
        ));
        assert!(should_add_to_batch(
            r#""C:\downloads\other.zip""#,
            r#""C:\downloads\archive.zip""#,
            "archive.zip",
            false
        ));
    }

    /// Parity test for capability C148: popping a non-empty queue
    /// returns the first item and the remaining items in order.
    #[test]
    fn pop_batch_queue_returns_first_and_rest() {
        let queue = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (first, rest) = pop_batch_queue(&queue).unwrap();
        assert_eq!(first, "a");
        assert_eq!(rest, vec!["b".to_string(), "c".to_string()]);
    }

    /// Parity test for capability C148: popping the last item leaves an
    /// empty queue.
    #[test]
    fn pop_batch_queue_last_item_leaves_empty_queue() {
        let queue = vec!["only".to_string()];
        let (first, rest) = pop_batch_queue(&queue).unwrap();
        assert_eq!(first, "only");
        assert!(rest.is_empty());
    }

    /// Parity test for capability C148: popping an empty queue returns
    /// `None`, matching `BatchQueuePop`'s "queue empty" branch.
    #[test]
    fn pop_batch_queue_empty_queue_returns_none() {
        assert_eq!(pop_batch_queue(&[]), None);
    }

    /// Parity test for capability C173: an ordinary terminal status (not
    /// `Silent`) does not stop the chain when batch mode is enabled —
    /// covers `Failed`, `Success`, and `NotPacked` as representative
    /// non-`Silent` statuses.
    #[test]
    fn should_continue_batch_continues_on_ordinary_statuses() {
        assert!(should_continue_batch(true, Status::Failed));
        assert!(should_continue_batch(true, Status::Success));
        assert!(should_continue_batch(true, Status::NotPacked));
    }

    /// Parity test for capability C173: `$STATUS_SILENT` stops the chain
    /// regardless of whether batch mode is enabled.
    #[test]
    fn should_continue_batch_stops_on_silent_status() {
        assert!(!should_continue_batch(true, Status::Silent));
        assert!(!should_continue_batch(false, Status::Silent));
    }

    /// Parity test for capability C173: batch mode disabled stops the
    /// chain regardless of status.
    #[test]
    fn should_continue_batch_stops_when_batch_disabled() {
        assert!(!should_continue_batch(false, Status::Failed));
        assert!(!should_continue_batch(false, Status::Success));
    }
}
