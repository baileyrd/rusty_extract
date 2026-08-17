//! Process exit code contract: `terminate()` (UniExtract.au3:4098-4213)
//! maps an internal completion status to a specific numeric process exit
//! code before AutoIt's `Exit`. This module ports only that pure
//! status→exit-code mapping (capability C016) — the GUI prompts, logging,
//! statistics, and update-check side effects `terminate` also performs are
//! each their own capability (or, for the GUI prompts themselves, out of
//! scope under the deferred GUI subsystem, manifest row D001).

/// Mirrors UniExtract2's `$STATUS_*` constants (UniExtract.au3:106-110):
/// the internal completion status `terminate()` switches on to decide the
/// process exit code.
///
/// `FileInfo` carries the two booleans its exit code actually depends on
/// (`$silentmode` and whether `$aFiletype` came back non-empty,
/// UniExtract.au3:4139-4150) — every other variant's exit code is fixed
/// regardless of context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Syntax,
    FileInfo {
        silent_mode: bool,
        filetype_identified: bool,
    },
    UnknownExe,
    UnknownExt,
    InvalidFile,
    InvalidDir,
    NotPacked,
    Batch,
    NotSupported,
    MissingExe,
    Timeout,
    Password,
    MissingDef,
    MoveFailed,
    NoFreeSpace,
    MissingPart,
    Failed,
    Success,
    Silent,
    TrayExit,
}

/// Ports the `Switch $status` block's exit-code assignment
/// (UniExtract.au3:4132-4213) exactly: `$exitcode` starts at 0
/// (UniExtract.au3:4098) and only the cases below ever change it.
/// `Syntax`, `Batch`, `Success`, `Silent`, and `TrayExit` have no case in
/// the source's switch that touches `$exitcode`, so they keep 0.
pub fn exit_code(status: Status) -> i32 {
    match status {
        Status::Syntax => 0,
        Status::FileInfo {
            silent_mode,
            filetype_identified,
        } => {
            if !silent_mode && !filetype_identified {
                4
            } else {
                0
            }
        }
        Status::UnknownExe => 3,
        Status::UnknownExt => 4,
        Status::InvalidFile => 5,
        Status::InvalidDir => 5,
        Status::NotPacked => 6,
        Status::Batch => 0,
        Status::NotSupported => 7,
        Status::MissingExe => 8,
        Status::Timeout => 9,
        Status::Password => 10,
        Status::MissingDef => 11,
        Status::MoveFailed => 12,
        Status::NoFreeSpace => 13,
        Status::MissingPart => 14,
        Status::Failed => 1,
        Status::Success => 0,
        Status::Silent => 0,
        Status::TrayExit => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{exit_code, Status};

    /// Parity test for capability C016: every `$STATUS_*` case's exit code
    /// matches the numeric literal `terminate()` assigns to `$exitcode`
    /// for that case (UniExtract.au3:4132-4213).
    #[test]
    fn fixed_exit_codes_match_source() {
        assert_eq!(exit_code(Status::Syntax), 0);
        assert_eq!(exit_code(Status::UnknownExe), 3);
        assert_eq!(exit_code(Status::UnknownExt), 4);
        assert_eq!(exit_code(Status::InvalidFile), 5);
        assert_eq!(exit_code(Status::InvalidDir), 5);
        assert_eq!(exit_code(Status::NotPacked), 6);
        assert_eq!(exit_code(Status::Batch), 0);
        assert_eq!(exit_code(Status::NotSupported), 7);
        assert_eq!(exit_code(Status::MissingExe), 8);
        assert_eq!(exit_code(Status::Timeout), 9);
        assert_eq!(exit_code(Status::Password), 10);
        assert_eq!(exit_code(Status::MissingDef), 11);
        assert_eq!(exit_code(Status::MoveFailed), 12);
        assert_eq!(exit_code(Status::NoFreeSpace), 13);
        assert_eq!(exit_code(Status::MissingPart), 14);
        assert_eq!(exit_code(Status::Failed), 1);
        assert_eq!(exit_code(Status::Success), 0);
        assert_eq!(exit_code(Status::Silent), 0);
        assert_eq!(exit_code(Status::TrayExit), 0);
    }

    /// `$STATUS_FILEINFO` (UniExtract.au3:4138-4150) is the one case whose
    /// exit code isn't fixed: silent mode always writes the scan result to
    /// a file and exits 0 (the `If $silentmode Then` branch never touches
    /// `$exitcode`); interactively, an empty `$aFiletype` (nothing
    /// recognized at all — a scan fail) is the only sub-case that sets
    /// `$exitcode = 4`, while a non-empty one calls `_GUI_FileScan()` and
    /// leaves it at 0.
    #[test]
    fn fileinfo_exit_code_depends_on_silent_mode_and_filetype_identification() {
        assert_eq!(
            exit_code(Status::FileInfo {
                silent_mode: true,
                filetype_identified: false,
            }),
            0
        );
        assert_eq!(
            exit_code(Status::FileInfo {
                silent_mode: true,
                filetype_identified: true,
            }),
            0
        );
        assert_eq!(
            exit_code(Status::FileInfo {
                silent_mode: false,
                filetype_identified: true,
            }),
            0
        );
        assert_eq!(
            exit_code(Status::FileInfo {
                silent_mode: false,
                filetype_identified: false,
            }),
            4
        );
    }
}
