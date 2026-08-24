//! Command-line flag detection: ports the flag-presence checks
//! `ParseCommandLine` (UniExtract.au3:589-694) makes directly against the
//! raw argv array (`$cmdline`), before any positional-argument-dependent
//! parsing happens.
//!
//! **Every check here is case-insensitive**, matching the AutoIt
//! functions the source uses: `_ArraySearch($cmdline, "...")` defaults
//! its `$iCase` parameter to `0` (not case sensitive), and a plain `=`
//! comparison (`$cmdline[1] = "..."`) is itself case-insensitive by
//! default (the script never calls `Opt("StringCompareMode", 1)` to
//! change that). `/SILENT`, `/Silent`, and `/silent` are all the same
//! flag to UniExtract2 — a real, easy-to-miss quirk this port preserves
//! rather than "fixes" into a conventional case-sensitive CLI.

/// C007: `/silent` — ports `_ArraySearch($cmdline, "/silent") > -1`
/// (UniExtract.au3:601): suppress all interactive prompts for this run.
pub fn has_silent_flag(args: &[String]) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case("/silent"))
}

/// C008: `/nolog` — ports `_ArraySearch($cmdline, "/nolog") > -1`
/// (UniExtract.au3:602): suppress the per-run log file for this
/// invocation, overriding the persisted `log` preference (C028).
pub fn has_nolog_flag(args: &[String]) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case("/nolog"))
}

/// C009: `/nostats` — ports `_ArraySearch($cmdline, "/nostats") > -1`
/// (UniExtract.au3:603): accepted without error here; the actual
/// stats-send suppression this flag drives is a separate, deferred
/// capability (manifest row D004).
pub fn has_nostats_flag(args: &[String]) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case("/nostats"))
}

/// C010: `/help`, `/?`, `-h`, `/h`, `-?`, `--help` — ports the six-way
/// equality check against `$cmdline[1]` (UniExtract.au3:605): print CLI
/// usage/help text, exit 0. `first_arg` is `$cmdline[1]`, the first
/// positional argument, not the full argv.
pub fn is_help_flag(first_arg: &str) -> bool {
    ["/help", "/?", "-h", "/h", "-?", "--help"]
        .iter()
        .any(|f| first_arg.eq_ignore_ascii_case(f))
}

/// C012: `/batchclear` — ports the `$cmdline[1] = "/batchclear"` branch
/// check (UniExtract.au3:630): clear the batch queue. `first_arg` is
/// `$cmdline[1]`.
pub fn is_batchclear_flag(first_arg: &str) -> bool {
    first_arg.eq_ignore_ascii_case("/batchclear")
}

/// C013: `/close` — ports `_ArraySearch($cmdline, "/close") > -1`
/// (UniExtract.au3:693): exit silently (used to signal a running
/// instance to close).
pub fn has_close_flag(args: &[String]) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case("/close"))
}

/// C011: `/batch` — ports `_ArraySearch($cmdline, "/batch") > -1`
/// (UniExtract.au3:687-690): queue the file for later processing instead
/// of extracting immediately. The source's branch calls `AddToBatch()`
/// then `terminate($STATUS_SILENT)` — real queue-file I/O and process
/// exit, so this function covers only the flag detection; adding the
/// queued entry is `batch::build_command_line` (C147) and the caller's
/// job.
pub fn has_batch_flag(args: &[String]) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case("/batch"))
}

/// C205: `/update`, `/updatehelper`, `/updatehelpers`, `/afterupdate` —
/// ports the update-related `ElseIf` branches of `ParseCommandLine`'s
/// dispatch chain (UniExtract.au3:608-617), covering D007. `first_arg`
/// is `$cmdline[1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateVerb {
    /// `_AfterUpdate()` — post-update cleanup/relaunch.
    AfterUpdate,
    /// `CheckUpdate()` then `terminate($STATUS_SILENT)` — a plain,
    /// interactive-capable update check.
    Update,
    /// `CheckUpdate($UPDATEMSG_SILENT, False, $UPDATE_HELPER)` then
    /// `$prompt = True` — a silent helper-only update check, followed by
    /// the main GUI (matching the source's own zero-args prompt branch).
    UpdateHelper,
    /// None of the above matched.
    None,
}

/// Ports UniExtract.au3:608-617's dispatch.
///
/// **Verified quirk, preserved rather than "fixed"**: `/updatehelper`
/// is matched case-insensitively (`$cmdline[1] = "/updatehelper"`, a
/// bare `=`), but `/updatehelpers` is matched case-*sensitively*
/// (`$cmdline[1] == "/updatehelpers"`, AutoIt's explicit `==` operator
/// overriding the script's default case-insensitive comparison mode) —
/// a real inconsistency between the two spellings, not obviously
/// intentional. One concrete consequence: `/UpdateHelpers` (any casing
/// other than the exact lowercase spelling) matches *neither* branch and
/// falls all the way through to the `Else` case, silently attempting to
/// treat it as a file to extract, rather than being recognized as an
/// update verb at all.
pub fn match_update_verb(first_arg: &str) -> UpdateVerb {
    if first_arg.eq_ignore_ascii_case("/afterupdate") {
        UpdateVerb::AfterUpdate
    } else if first_arg.eq_ignore_ascii_case("/update") {
        UpdateVerb::Update
    } else if first_arg.eq_ignore_ascii_case("/updatehelper") || first_arg == "/updatehelpers" {
        UpdateVerb::UpdateHelper
    } else {
        UpdateVerb::None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        has_batch_flag, has_close_flag, has_nolog_flag, has_nostats_flag, has_silent_flag,
        is_batchclear_flag, is_help_flag, match_update_verb, UpdateVerb,
    };

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// Parity test for capability C007.
    #[test]
    fn silent_flag_detected_case_insensitively() {
        assert!(has_silent_flag(&args(&["/silent"])));
        assert!(has_silent_flag(&args(&["/SILENT"])));
        assert!(has_silent_flag(&args(&["file.zip", "/Silent"])));
        assert!(!has_silent_flag(&args(&["file.zip"])));
        assert!(!has_silent_flag(&args(&[])));
    }

    /// Parity test for capability C008.
    #[test]
    fn nolog_flag_detected_case_insensitively() {
        assert!(has_nolog_flag(&args(&["/nolog"])));
        assert!(has_nolog_flag(&args(&["/NoLog"])));
        assert!(!has_nolog_flag(&args(&["/silent"])));
    }

    /// Parity test for capability C009: accepted without error, i.e. its
    /// presence is simply detectable like every other flag here — the
    /// actual stats-suppression behavior is out of scope (D004).
    #[test]
    fn nostats_flag_detected_case_insensitively() {
        assert!(has_nostats_flag(&args(&["/nostats"])));
        assert!(has_nostats_flag(&args(&["/NOSTATS"])));
        assert!(!has_nostats_flag(&args(&["/silent"])));
    }

    /// Parity test for capability C010: all six spellings match, and all
    /// case-insensitively; anything else does not.
    #[test]
    fn help_flag_matches_all_six_spellings_case_insensitively() {
        for f in ["/help", "/?", "-h", "/h", "-?", "--help"] {
            assert!(is_help_flag(f), "{f} should be recognized as help");
            assert!(
                is_help_flag(&f.to_uppercase()),
                "{f} uppercased should still be recognized as help"
            );
        }
        assert!(!is_help_flag("file.zip"));
        assert!(!is_help_flag(""));
    }

    /// Parity test for capability C012.
    #[test]
    fn batchclear_flag_matches_case_insensitively() {
        assert!(is_batchclear_flag("/batchclear"));
        assert!(is_batchclear_flag("/BatchClear"));
        assert!(!is_batchclear_flag("/batch"));
    }

    /// Parity test for capability C013.
    #[test]
    fn close_flag_detected_case_insensitively() {
        assert!(has_close_flag(&args(&["/close"])));
        assert!(has_close_flag(&args(&["/CLOSE"])));
        assert!(!has_close_flag(&args(&["/silent"])));
    }

    /// Parity test for capability C011: `/batch` is detected
    /// case-insensitively, and doesn't false-positive on `/batchclear`
    /// (a different, distinct flag, not a substring match).
    #[test]
    fn batch_flag_detected_case_insensitively() {
        assert!(has_batch_flag(&args(&["/batch"])));
        assert!(has_batch_flag(&args(&["/BATCH"])));
        assert!(has_batch_flag(&args(&["file.zip", "/Batch"])));
        assert!(!has_batch_flag(&args(&["/batchclear"])));
        assert!(!has_batch_flag(&args(&["file.zip"])));
        assert!(!has_batch_flag(&args(&[])));
    }

    /// Parity test for capability C205: `/afterupdate` and `/update`
    /// match case-insensitively, same as every other verb here.
    #[test]
    fn afterupdate_and_update_match_case_insensitively() {
        assert_eq!(match_update_verb("/afterupdate"), UpdateVerb::AfterUpdate);
        assert_eq!(match_update_verb("/AfterUpdate"), UpdateVerb::AfterUpdate);
        assert_eq!(match_update_verb("/update"), UpdateVerb::Update);
        assert_eq!(match_update_verb("/UPDATE"), UpdateVerb::Update);
    }

    /// Parity test: `/updatehelper` (no trailing "s") matches
    /// case-insensitively.
    #[test]
    fn updatehelper_matches_case_insensitively() {
        assert_eq!(match_update_verb("/updatehelper"), UpdateVerb::UpdateHelper);
        assert_eq!(match_update_verb("/UpdateHelper"), UpdateVerb::UpdateHelper);
    }

    /// Parity test for the verified case-sensitivity inconsistency:
    /// `/updatehelpers` only matches in its exact lowercase spelling —
    /// any other casing matches neither branch and falls through to
    /// `None`, unlike every other verb in this dispatch.
    #[test]
    fn updatehelpers_matches_only_the_exact_lowercase_spelling() {
        assert_eq!(
            match_update_verb("/updatehelpers"),
            UpdateVerb::UpdateHelper
        );
        assert_eq!(match_update_verb("/UpdateHelpers"), UpdateVerb::None);
        assert_eq!(match_update_verb("/UPDATEHELPERS"), UpdateVerb::None);
    }

    #[test]
    fn unrelated_argument_matches_none() {
        assert_eq!(match_update_verb("file.zip"), UpdateVerb::None);
        assert_eq!(match_update_verb(""), UpdateVerb::None);
    }
}
