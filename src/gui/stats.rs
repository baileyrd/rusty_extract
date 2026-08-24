//! Local usage statistics — capability C196. Ports the pure decisions
//! inside `GUI_Stats` (UniExtract.au3:7983-8061) and the counter-write
//! sites inside `terminate()` (UniExtract.au3:4125-4127). Entirely
//! local — nothing here is ever transmitted, distinct from the network
//! `SendStats`/telemetry capability (C214/D012); kept clearly separated
//! by name from that capability, per this row's own manifest note.
//!
//! **The pie-chart rendering itself is out of scope, not just unwired.**
//! `_Pie_PrepareValues`/`_Pie_Draw`/`_Pie_Draw_Legend`/
//! `_MouseOverRotation`/`_Pie_Shutdown` are GDI+ chart-drawing helpers —
//! a rendering *dependency* this port replaces with whatever `egui`-native
//! charting approach it eventually uses, not a *behavior* to reproduce.
//! What's real and worth porting precisely is the data classification and
//! filtering feeding into that chart, which is exactly what this module
//! covers.

use crate::status::Status;

/// Maps a [`Status`] to the exact lowercase string
/// UniExtract.au3's own `$STATUS_*` constants use as both the
/// `terminate()` counter-write key and the `GUI_Stats` classification
/// key (UniExtract.au3:106-110) — the two sides of the same shared,
/// flat "Statistics" INI-section keyspace this row's manifest note
/// calls fragile.
pub fn status_ini_key(status: Status) -> &'static str {
    match status {
        Status::Syntax => "syntax",
        Status::FileInfo { .. } => "fileinfo",
        Status::UnknownExe => "unknownexe",
        Status::UnknownExt => "unknownext",
        Status::InvalidFile => "invalidfile",
        Status::InvalidDir => "invaliddir",
        Status::NotPacked => "notpacked",
        Status::Batch => "batch",
        Status::NotSupported => "notsupported",
        Status::MissingExe => "missingexe",
        Status::Timeout => "timeout",
        Status::Password => "password",
        Status::MissingDef => "missingdef",
        Status::MoveFailed => "movefailed",
        Status::NoFreeSpace => "nofreespace",
        Status::MissingPart => "missingpart",
        Status::Failed => "failed",
        Status::Success => "success",
        Status::Silent => "silent",
        Status::TrayExit => "trayexit",
    }
}

/// Ports the `$status = $STATUS_SUCCESS` gate on the second counter
/// write (UniExtract.au3:4127): the archive-type counter only ever
/// increments alongside a successful extraction.
pub fn should_increment_arctype_counter(status: Status) -> bool {
    matches!(status, Status::Success)
}

/// Which of `GUI_Stats`'s four pie-chart buckets a "Statistics" INI key
/// falls into (UniExtract.au3:8008-8024) — or whether it's excluded
/// entirely, or (the `Case Else` fallthrough) treated as an archive-type
/// entry instead of a status at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsBucket {
    Success,
    Failed,
    FileInfo,
    Unknown,
    /// `batch`/`silent`/`syntax`: skipped from both pies entirely.
    Excluded,
    /// Not a recognized status key at all — either a genuine archive-type
    /// name, or (see this function's own doc comment) one of four status
    /// keys the source's own `Switch` never gives a case to.
    ArchiveType,
}

/// Ports `GUI_Stats`'s per-key `Switch` (UniExtract.au3:8008-8024).
///
/// **Verified quirk this row's own manifest note doesn't mention**: the
/// source's `Switch` has no `Case` at all for `"movefailed"`,
/// `"nofreespace"`, `"missingpart"`, or `"trayexit"` — four real
/// `$STATUS_*` values that any INI-key literal comparison against them
/// would need explicit cases to catch. Since none exist, all four fall
/// through to `Case Else` alongside genuine archive-type names, and get
/// counted as if they were archive types in the second pie chart rather
/// than skipped or bucketed into `Failed`. This looks like an oversight
/// in the source (all four are also absent from any of the *other*
/// three status cases), but it's real, shipped behavior — preserved
/// here exactly rather than "corrected" into whichever bucket seems
/// more sensible, per this migration's own parity contract.
pub fn classify_stats_key(key: &str) -> StatsBucket {
    match key {
        "fileinfo" => StatsBucket::FileInfo,
        "notsupported" | "unknownexe" | "unknownext" => StatsBucket::Unknown,
        "failed" | "invaliddir" | "invalidfile" | "missingdef" | "missingexe" | "timeout" => {
            StatsBucket::Failed
        }
        "success" | "notpacked" | "password" => StatsBucket::Success,
        "batch" | "silent" | "syntax" => StatsBucket::Excluded,
        _ => StatsBucket::ArchiveType,
    }
}

/// Ports `GUI_Stats`'s "enough data" gate (UniExtract.au3:7985-7986):
/// `IniReadSection`'s own `$aReturn[0][0]` is the number of *distinct
/// keys* in the section, not the sum of their values — an app that's
/// run 500 extractions of always the same 3 archive types would still
/// only ever have a handful of distinct keys and never reach this
/// threshold, no matter the real extraction volume. Verified against
/// `IniReadSection`'s own documented return shape rather than assumed,
/// resolving this row's own manifest uncertainty.
pub fn should_show_stats(distinct_key_count: usize) -> bool {
    distinct_key_count >= 10
}

/// Ports `_ArraySort($GUI_Stats_Types, 1)` (descending by count, the
/// default sort column) followed by `If UBound(...) > 9 Then ReDim
/// $GUI_Stats_Types[9][2]` (UniExtract.au3:8028-8029) — keep only the
/// highest-count entries, capped at `limit`. Also reused for
/// `$GUI_Stats_Status`'s own identical sort (UniExtract.au3:8027,
/// cosmetic slice ordering, not filtering, since that array only ever
/// has 4 entries).
pub fn top_n_by_count(mut entries: Vec<(String, i64)>, limit: usize) -> Vec<(String, i64)> {
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(limit);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_ini_key_matches_source_string_constants() {
        assert_eq!(status_ini_key(Status::Success), "success");
        assert_eq!(status_ini_key(Status::Failed), "failed");
        assert_eq!(
            status_ini_key(Status::FileInfo {
                silent_mode: false,
                filetype_identified: true
            }),
            "fileinfo"
        );
        assert_eq!(status_ini_key(Status::TrayExit), "trayexit");
        assert_eq!(status_ini_key(Status::MoveFailed), "movefailed");
        assert_eq!(status_ini_key(Status::NoFreeSpace), "nofreespace");
        assert_eq!(status_ini_key(Status::MissingPart), "missingpart");
    }

    #[test]
    fn arctype_counter_only_increments_on_success() {
        assert!(should_increment_arctype_counter(Status::Success));
        assert!(!should_increment_arctype_counter(Status::Failed));
        assert!(!should_increment_arctype_counter(Status::NotPacked));
    }

    #[test]
    fn classify_stats_key_buckets_match_source_switch() {
        assert_eq!(classify_stats_key("fileinfo"), StatsBucket::FileInfo);
        assert_eq!(classify_stats_key("notsupported"), StatsBucket::Unknown);
        assert_eq!(classify_stats_key("unknownexe"), StatsBucket::Unknown);
        assert_eq!(classify_stats_key("unknownext"), StatsBucket::Unknown);
        assert_eq!(classify_stats_key("failed"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("invaliddir"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("invalidfile"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("missingdef"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("missingexe"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("timeout"), StatsBucket::Failed);
        assert_eq!(classify_stats_key("success"), StatsBucket::Success);
        assert_eq!(classify_stats_key("notpacked"), StatsBucket::Success);
        assert_eq!(classify_stats_key("password"), StatsBucket::Success);
        assert_eq!(classify_stats_key("batch"), StatsBucket::Excluded);
        assert_eq!(classify_stats_key("silent"), StatsBucket::Excluded);
        assert_eq!(classify_stats_key("syntax"), StatsBucket::Excluded);
    }

    #[test]
    fn classify_stats_key_treats_real_archive_type_names_as_archive_type() {
        assert_eq!(classify_stats_key("7z"), StatsBucket::ArchiveType);
        assert_eq!(classify_stats_key("NSIS"), StatsBucket::ArchiveType);
    }

    /// Parity test for the verified quirk: four real status keys the
    /// source's own `Switch` never gives a case to fall through to
    /// `Case Else` and get miscounted as archive types.
    #[test]
    fn classify_stats_key_status_codes_missing_from_switch_fall_through() {
        assert_eq!(classify_stats_key("movefailed"), StatsBucket::ArchiveType);
        assert_eq!(classify_stats_key("nofreespace"), StatsBucket::ArchiveType);
        assert_eq!(classify_stats_key("missingpart"), StatsBucket::ArchiveType);
        assert_eq!(classify_stats_key("trayexit"), StatsBucket::ArchiveType);
    }

    #[test]
    fn should_show_stats_requires_at_least_ten_distinct_keys() {
        assert!(!should_show_stats(9));
        assert!(should_show_stats(10));
    }

    #[test]
    fn top_n_by_count_sorts_descending_and_truncates() {
        let entries = vec![
            ("zip".to_string(), 5),
            ("7z".to_string(), 20),
            ("rar".to_string(), 10),
        ];
        assert_eq!(
            top_n_by_count(entries, 2),
            vec![("7z".to_string(), 20), ("rar".to_string(), 10)]
        );
    }

    #[test]
    fn top_n_by_count_keeps_everything_under_the_limit() {
        let entries = vec![("zip".to_string(), 1), ("7z".to_string(), 2)];
        assert_eq!(top_n_by_count(entries.clone(), 9).len(), entries.len());
    }
}
