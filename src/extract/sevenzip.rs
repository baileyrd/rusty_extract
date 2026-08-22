//! 7-Zip (`$TYPE_7Z`) integration — the main archive extraction, plus
//! its follow-up handling for RPM/DEB/gzip-family inner payloads and
//! the SFX-splitter fallback.
//!
//! ```autoit
//! Case $TYPE_7Z
//!     Local $sPassword = _FindArchivePassword($7z & ' l -p -slt "' & $file & '"', $7z & ' t -p"%PASSWORD%" "' & $file & '"', "Encrypted = +", "Wrong password?", 0, "Everything is Ok")
//!     _Run($7z & ' x ' & ($sPassword == 0? '"': '-p"' & $sPassword & '" "') & $file & '"', $outdir, @SW_HIDE, True, True, True, True)
//!     If @error = 3 Then terminate($STATUS_MISSINGPART)
//!     If @extended Then terminate($STATUS_PASSWORD, $file, $arctype, $arcdisp)
//!
//!     If FileExists($outdir & "\.text") Then
//!         ; Generic .exe extraction should not be considered successful
//!         $success = $RESULT_FAILED
//!     ElseIf StringInStr($sFileType, "RPM Linux Package", 0) Then
//!         ; Extract inner CPIO for RPMs
//!         Local $sPath = $outdir & "\" & $filename & ".cpio"
//!         If FileExists($sPath) Then
//!             _Run($7z & ' x "' & $sPath & '"', $outdir)
//!             FileDelete($sPath)
//!         EndIf
//!     ElseIf StringInStr($sFileType, "Debian Linux Package", 0) Then
//!         ; Extract inner tarball for DEBs
//!         Local $sPath = $outdir & "\data.tar"
//!         If FileExists($sPath) Then
//!             _Run($7z & ' x "' & $sPath & '"', $outdir)
//!             FileDelete($sPath)
//!         EndIf
//!     ElseIf $additionalParameters == "bz2" Or $additionalParameters == "gz" Or $additionalParameters == "xz" Or $additionalParameters == "Z" Then
//!         ; Extract inner tarball for GZipped files
//!         Local $sPath = $outdir & "\" & $filename
//!         If FileExists($sPath) Then
//!             Local $sReturn = TridLib_Analyse_Simple($sPath)
//!             If StringInStr($sReturn, "Tape ARchive") Or StringRight($sPath, 3) = "tar" Then
//!                 _Run($7z & ' x "' & $sPath & '"', $outdir)
//!                 FileDelete($sPath)
//!             EndIf
//!         EndIf
//!     ElseIf StringInStr($sFileType, "SFX") And Not StringInStr($sFileType, "CAB") Then
//!         ; 7z SFX Archives splitter GUI automation — see module scope note
//!     EndIf
//! ```
//!
//! **Password handling reuses `password_search`, C160/C161 (already
//! `DONE`)** — not duplicated here. `_FindArchivePassword`'s exact
//! arguments at this call site: `protected_text = "Encrypted = +"`,
//! `protected_text2 = Some("Wrong password?")`, `line = 0` (searches
//! the whole probe output, per `password_search::probe_shows_protected`'s
//! own doc comment, which already cites this exact call site),
//! `success_text = "Everything is Ok"`.
//!
//! **Operator precision — three different case-sensitivity rules in one
//! `Case` block.** Bare `StringInStr` and `StringInStr(..., 0)` are both
//! case-*insensitive* (`0` is AutoIt's documented default `casesense`
//! value — the same fact PR #387 corrected in `extract::ctar` after
//! getting it backwards there). Single-`=` string comparison
//! (`StringRight($sPath, 3) = "tar"`) is also case-insensitive, per this
//! script's default `StringCompareMode` (the same rule already
//! documented in `cli`/`dest_arg`/`outdir`/`type_override`). Double-`==`
//! (`$additionalParameters == "bz2"` etc.) is **always** case-sensitive,
//! unconditionally, regardless of `StringCompareMode` — verified against
//! AutoIt's own operator documentation before writing this module, given
//! the `StringInStr` mistake just made in `extract::ctar`.
//!
//! **The SFX-splitter branch is now ported too**, PR [#414](https://github.com/baileyrd/rusty_extract/pull/414),
//! built on C069's `automation::GuiAutomation` infrastructure —
//! [`sfx_splitter_extract`]. `classify_post_extraction` still reports
//! this branch as [`PostExtractionBranch::SfxSplitter`]; the caller
//! drives [`sfx_splitter_extract`] once it sees that variant.
//!
//! ```autoit
//! Run(_MakeCommand($7zsplit & ' "' & $file & '"'), $outdir, @SW_HIDE)
//! WinWait("7z SFX Archives splitter")
//! ControlClick("7z SFX Archives splitter", "", "Button8")
//! ControlClick("7z SFX Archives splitter", "", "Button1")
//! $TimerStart = TimerInit()
//!
//! Do
//!     Sleep(100)
//!     If WinExists("7z SFX Archives splitter warning") Then WinClose("7z SFX Archives splitter warning")
//!     $TimerDiff = TimerDiff($TimerStart)
//!     If $TimerDiff > $Timeout Then ExitLoop
//! Until FileExists($filedir & "\" & $filename & ".txt") Or WinExists("7z SFX Archives splitter error")
//!
//! ProcessClose("7ZSplit.exe")
//!
//! Local $sPath = $filedir & "\" & $filename & ".txt"
//! If FileExists($sPath) Then _FileMove($sPath, $outdir & "\" & $filename & "_SFX-Script.txt")
//! ```
//!
//! **A genuine, preserved hang-risk quirk**, the same shape already found
//! for C044's PEiD scan: `WinWait("7z SFX Archives splitter")` here also
//! passes no timeout argument — modeled as `u64::MAX`, not "fixed" into
//! a bounded wait.
//!
//! **`_MakeCommand`'s own bindir-prefixing isn't modeled** — the same
//! scope note already made in `extract::iscab`/`extract::expand`.
//! `sevenzip_split` is `_MakeCommand`'s own result, taken as an opaque,
//! already-resolved path.
//!
//! **Scope — the final `_FileMove` stays out of scope.** [`sfx_splitter_extract`]
//! does live-poll `FileExists` through the automation seam (the loop's
//! own exit condition genuinely needs it, unlike every other filesystem
//! check in this crate, which is a plain pre-computed function argument
//! — see `automation::GuiAutomation::file_exists`'s own doc comment) but
//! reports whether/where a script file was found as data
//! ([`SfxSplitterOutcome`]) rather than performing the final rename
//! itself, matching this crate's usual split between deciding and
//! mutating the filesystem. Carries the same honesty caveat as C069:
//! fake-backed tests prove the decision logic, not that the real Win32
//! backend drives an actual 7ZSplit window.

use crate::automation::{GuiAutomation, WindowHandle};
use crate::extract::{Invocation, WindowMode};
use crate::status::Status;

/// Builds the main extraction invocation (UniExtract.au3:2291):
/// `<7z> x "<file>"` when no password was found, or `<7z> x -p"<password>"
/// "<file>"` when one was — `password` is
/// `password_search::find_password`'s own `Option<&str>` output (`None`
/// standing in for the source's `$sPassword == 0` sentinel), run in
/// `outdir` with the window hidden.
pub fn extract_invocation(
    program: &str,
    file: &str,
    outdir: &str,
    password: Option<&str>,
) -> Invocation {
    let mut args = vec!["x".to_string()];
    if let Some(pw) = password {
        args.push(format!("-p\"{pw}\""));
    }
    args.push(file.to_string());
    Invocation {
        program: program.to_string(),
        args,
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports the `@error`/`@extended` classification after the main
/// extraction call (UniExtract.au3:2292-2293): `@error = 3` (checked
/// first — a sequential `If`, not `ElseIf`, but since `terminate()`
/// exits the process, only one of these two statuses is ever actually
/// reached) maps to `MissingPart`; otherwise `@extended` maps to
/// `Password`; otherwise neither fires.
pub fn classify_run_error(error_is_missing_part: bool, extended: bool) -> Option<Status> {
    if error_is_missing_part {
        Some(Status::MissingPart)
    } else if extended {
        Some(Status::Password)
    } else {
        None
    }
}

/// What the post-extraction `ElseIf` chain (UniExtract.au3:2295-2320)
/// decides, given the cached file-type string and this call's
/// `$additionalParameters`. Each variant carries just enough data for
/// the caller to know which candidate inner path to check
/// (`FileExists`, real I/O, left to the caller) before extracting it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostExtractionBranch {
    /// `FileExists($outdir & "\.text")` — a generic `.exe`'s extraction
    /// should not be considered successful.
    GenericExeGuard,
    /// RPM Linux Package: candidate inner path
    /// `<outdir>\<filename>.cpio`.
    Rpm { inner_path: String },
    /// Debian Linux Package: candidate inner path `<outdir>\data.tar`.
    Debian { inner_path: String },
    /// `$additionalParameters` in `{bz2, gz, xz, Z}`: candidate inner
    /// path `<outdir>\<filename>` — extracting it further needs
    /// [`should_extract_gz_family_inner`]'s own probe-then-classify
    /// gate, not just `FileExists`.
    GzFamily { inner_path: String },
    /// File type mentions "SFX" but not "CAB" — drive
    /// [`sfx_splitter_extract`] (see module doc comment).
    SfxSplitter,
    /// None of the above matched.
    NoAction,
}

/// Ports the branch selection itself (UniExtract.au3:2295-2320).
pub fn classify_post_extraction(
    text_placeholder_exists: bool,
    file_type: &str,
    additional_parameters: &str,
    outdir: &str,
    filename: &str,
) -> PostExtractionBranch {
    let file_type_lower = file_type.to_lowercase();
    if text_placeholder_exists {
        PostExtractionBranch::GenericExeGuard
    } else if file_type_lower.contains("rpm linux package") {
        PostExtractionBranch::Rpm {
            inner_path: format!("{outdir}\\{filename}.cpio"),
        }
    } else if file_type_lower.contains("debian linux package") {
        PostExtractionBranch::Debian {
            inner_path: format!("{outdir}\\data.tar"),
        }
    } else if matches!(additional_parameters, "bz2" | "gz" | "xz" | "Z") {
        PostExtractionBranch::GzFamily {
            inner_path: format!("{outdir}\\{filename}"),
        }
    } else if file_type_lower.contains("sfx") && !file_type_lower.contains("cab") {
        PostExtractionBranch::SfxSplitter
    } else {
        PostExtractionBranch::NoAction
    }
}

/// Ports the gz-family inner-tar gate (UniExtract.au3:2312):
/// `StringInStr($sReturn, "Tape ARchive") Or StringRight($sPath, 3) =
/// "tar"` — case-insensitive on both halves (bare `StringInStr`; `=` per
/// this script's default `StringCompareMode`).
pub fn should_extract_gz_family_inner(trid_probe_output: &str, inner_path: &str) -> bool {
    trid_probe_output.to_lowercase().contains("tape archive")
        || inner_path.to_lowercase().ends_with("tar")
}

/// Builds the inner-archive extraction invocation shared by the
/// RPM/Debian/gz-family branches (UniExtract.au3:2303,2309,2314):
/// `<7z> x "<inner_path>"`, run in `outdir`. No `$show_flag` argument is
/// passed at any of these three call sites, so `_Run`'s own default
/// (`@SW_MINIMIZE`) applies — the same convention `extract::raiu`/
/// `extract::ctar` already document for their own bare `_Run($cmd,
/// $dir)` calls. The source deletes `inner_path` afterward
/// (`FileDelete`) — real filesystem I/O, out of scope here.
pub fn inner_extract_invocation(program: &str, inner_path: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), inner_path.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

const SFX_SPLITTER_WINDOW_TITLE: &str = "7z SFX Archives splitter";
const SFX_SPLITTER_WARNING_TITLE: &str = "7z SFX Archives splitter warning";
const SFX_SPLITTER_ERROR_TITLE: &str = "7z SFX Archives splitter error";
const SFX_SPLITTER_PROCESS_NAME: &str = "7ZSplit.exe";

/// What [`sfx_splitter_extract`] found once its polling loop ended
/// (UniExtract.au3:2338-2339): whether `<filedir>\<filename>.txt` exists,
/// and if so, where the source moves it (`_FileMove`, real filesystem
/// I/O left to the caller — see module doc comment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfxSplitterOutcome {
    /// No script file appeared before the polling loop ended.
    NoScript,
    /// A script file exists at `from`; the source renames it to `to`.
    ScriptFound { from: String, to: String },
}

/// Ports the SFX-splitter branch (UniExtract.au3:2331-2343): launches
/// `sevenzip_split` against `file`, clicks its two buttons, then polls
/// for either the expected script file to appear or an error window,
/// closing any warning window that pops up along the way, before closing
/// the splitter process. See the module doc comment for the preserved
/// no-timeout `WinWait` quirk and the `_FileMove` scope note.
pub fn sfx_splitter_extract<A: GuiAutomation>(
    automation: &mut A,
    sevenzip_split: &str,
    file: &str,
    outdir: &str,
    filedir: &str,
    filename: &str,
    timeout_ms: u64,
) -> SfxSplitterOutcome {
    automation.run(&Invocation {
        program: sevenzip_split.to_string(),
        args: vec![file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    });
    let window = automation
        .win_wait(SFX_SPLITTER_WINDOW_TITLE, u64::MAX)
        .unwrap_or(WindowHandle(0));
    automation.control_click(window, "Button8");
    automation.control_click(window, "Button1");

    let script_path = format!("{filedir}\\{filename}.txt");
    let timer = automation.timer_init();
    loop {
        automation.sleep(100);
        if automation.win_exists(SFX_SPLITTER_WARNING_TITLE) {
            automation.win_close_by_title(SFX_SPLITTER_WARNING_TITLE);
        }
        if automation.file_exists(&script_path)
            || automation.win_exists(SFX_SPLITTER_ERROR_TITLE)
            || automation.elapsed_ms(timer) > timeout_ms
        {
            break;
        }
    }
    automation.process_close(SFX_SPLITTER_PROCESS_NAME);

    if automation.file_exists(&script_path) {
        SfxSplitterOutcome::ScriptFound {
            to: format!("{outdir}\\{filename}_SFX-Script.txt"),
            from: script_path,
        }
    } else {
        SfxSplitterOutcome::NoScript
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_invocation_without_password() {
        let inv = extract_invocation(
            r"C:\bin\7z.exe",
            r"C:\downloads\archive.7z",
            r"C:\downloads\unpacked",
            None,
        );
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\downloads\archive.7z".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn extract_invocation_with_password() {
        let inv = extract_invocation(
            r"C:\bin\7z.exe",
            r"C:\downloads\archive.7z",
            r"C:\downloads\unpacked",
            Some("hunter2"),
        );
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                "-p\"hunter2\"".to_string(),
                r"C:\downloads\archive.7z".to_string(),
            ]
        );
    }

    /// Parity test for capability C056: `@error = 3` takes priority over
    /// `@extended` — matches the source's sequential (not `ElseIf`) `If`
    /// statements, since only one status is ever actually reached.
    #[test]
    fn missing_part_takes_priority_over_extended() {
        assert_eq!(classify_run_error(true, true), Some(Status::MissingPart));
    }

    #[test]
    fn extended_alone_maps_to_password() {
        assert_eq!(classify_run_error(false, true), Some(Status::Password));
    }

    #[test]
    fn neither_error_nor_extended_is_no_status() {
        assert_eq!(classify_run_error(false, false), None);
    }

    #[test]
    fn generic_exe_guard_takes_priority() {
        assert_eq!(
            classify_post_extraction(true, "RPM Linux Package", "", r"C:\out", "f"),
            PostExtractionBranch::GenericExeGuard
        );
    }

    #[test]
    fn rpm_branch_builds_cpio_path() {
        assert_eq!(
            classify_post_extraction(false, "RPM Linux Package v3", "", r"C:\out", "pkg"),
            PostExtractionBranch::Rpm {
                inner_path: r"C:\out\pkg.cpio".to_string()
            }
        );
    }

    /// Parity test for capability C056: the RPM/Debian file-type checks
    /// are case-insensitive despite their explicit `0` third argument.
    #[test]
    fn rpm_branch_is_case_insensitive() {
        assert_eq!(
            classify_post_extraction(false, "rpm linux package", "", r"C:\out", "pkg"),
            PostExtractionBranch::Rpm {
                inner_path: r"C:\out\pkg.cpio".to_string()
            }
        );
    }

    #[test]
    fn debian_branch_builds_data_tar_path() {
        assert_eq!(
            classify_post_extraction(false, "Debian Linux Package", "", r"C:\out", "pkg"),
            PostExtractionBranch::Debian {
                inner_path: r"C:\out\data.tar".to_string()
            }
        );
    }

    #[test]
    fn gz_family_branch_builds_filename_path() {
        for param in ["bz2", "gz", "xz", "Z"] {
            assert_eq!(
                classify_post_extraction(false, "plain", param, r"C:\out", "movie.avi"),
                PostExtractionBranch::GzFamily {
                    inner_path: r"C:\out\movie.avi".to_string()
                }
            );
        }
    }

    /// Parity test for capability C056: `$additionalParameters` matching
    /// is case-*sensitive* (`==`), unlike every `StringInStr`/`=` check
    /// in this same block.
    #[test]
    fn gz_family_branch_requires_exact_case() {
        assert_eq!(
            classify_post_extraction(false, "plain", "GZ", r"C:\out", "f"),
            PostExtractionBranch::NoAction
        );
    }

    #[test]
    fn sfx_branch_requires_sfx_without_cab() {
        assert_eq!(
            classify_post_extraction(false, "SFX Archive", "", r"C:\out", "f"),
            PostExtractionBranch::SfxSplitter
        );
        assert_eq!(
            classify_post_extraction(false, "SFX CAB Archive", "", r"C:\out", "f"),
            PostExtractionBranch::NoAction
        );
    }

    #[test]
    fn no_branch_matches_falls_through() {
        assert_eq!(
            classify_post_extraction(false, "Zip Archive", "", r"C:\out", "f"),
            PostExtractionBranch::NoAction
        );
    }

    #[test]
    fn gz_family_inner_extracts_when_probe_shows_tar_archive() {
        assert!(should_extract_gz_family_inner(
            "Format: Tape ARchive",
            r"C:\out\movie.avi"
        ));
    }

    #[test]
    fn gz_family_inner_extracts_when_path_ends_in_tar() {
        assert!(should_extract_gz_family_inner(
            "unrecognized",
            r"C:\out\archive.tar"
        ));
    }

    #[test]
    fn gz_family_inner_does_not_extract_otherwise() {
        assert!(!should_extract_gz_family_inner(
            "unrecognized",
            r"C:\out\movie.avi"
        ));
    }

    #[test]
    fn inner_extract_invocation_matches_source() {
        let inv = inner_extract_invocation(r"C:\bin\7z.exe", r"C:\out\data.tar", r"C:\out");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\out\data.tar".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\out");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    mod sfx_splitter {
        use super::*;
        use crate::automation::fake::{Call, FakeGuiAutomation};

        #[test]
        fn launches_splitter_and_clicks_both_buttons() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_win_wait(SFX_SPLITTER_WINDOW_TITLE, Some(WindowHandle(1)));

            sfx_splitter_extract(
                &mut fake,
                r"C:\bin\7ZSplit.exe",
                r"C:\downloads\sfx.exe",
                r"C:\downloads\unpacked",
                r"C:\downloads",
                "sfx",
                1_000,
            );

            assert!(fake.calls().iter().any(|c| matches!(
                c,
                Call::Run(inv) if inv.program == r"C:\bin\7ZSplit.exe"
                    && inv.args == vec![r"C:\downloads\sfx.exe".to_string()]
                    && inv.working_dir == r"C:\downloads\unpacked"
                    && inv.window == WindowMode::Hidden
            )));
            assert!(fake
                .calls()
                .contains(&Call::ControlClick(WindowHandle(1), "Button8".to_string())));
            assert!(fake
                .calls()
                .contains(&Call::ControlClick(WindowHandle(1), "Button1".to_string())));
            // No timeout on the initial WinWait -- see module doc comment.
            assert!(fake.calls().contains(&Call::WinWait(
                SFX_SPLITTER_WINDOW_TITLE.to_string(),
                u64::MAX
            )));
        }

        #[test]
        fn reports_the_script_path_when_it_appears() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_win_wait(SFX_SPLITTER_WINDOW_TITLE, Some(WindowHandle(1)));
            fake.script_file_appears_after(r"C:\downloads\sfx.txt", 2);

            let outcome = sfx_splitter_extract(
                &mut fake,
                "7ZSplit.exe",
                r"C:\downloads\sfx.exe",
                r"C:\downloads\unpacked",
                r"C:\downloads",
                "sfx",
                5_000,
            );

            assert_eq!(
                outcome,
                SfxSplitterOutcome::ScriptFound {
                    from: r"C:\downloads\sfx.txt".to_string(),
                    to: r"C:\downloads\unpacked\sfx_SFX-Script.txt".to_string(),
                }
            );
            assert!(fake
                .calls()
                .contains(&Call::ProcessClose(SFX_SPLITTER_PROCESS_NAME.to_string())));
        }

        #[test]
        fn closes_a_warning_window_that_appears_mid_poll() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_win_wait(SFX_SPLITTER_WINDOW_TITLE, Some(WindowHandle(1)));
            fake.script_win_exists(SFX_SPLITTER_WARNING_TITLE, true);
            fake.script_file_appears_after(r"C:\downloads\sfx.txt", 1);

            sfx_splitter_extract(
                &mut fake,
                "7ZSplit.exe",
                r"C:\downloads\sfx.exe",
                r"C:\downloads\unpacked",
                r"C:\downloads",
                "sfx",
                5_000,
            );

            assert!(fake.calls().contains(&Call::WinCloseByTitle(
                SFX_SPLITTER_WARNING_TITLE.to_string()
            )));
        }

        /// Parity test for capability C056: no script file ever appears
        /// and no error window shows up -- the loop gives up on timeout
        /// rather than hanging, and reports `NoScript`.
        #[test]
        fn gives_up_on_timeout_when_nothing_ever_appears() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_win_wait(SFX_SPLITTER_WINDOW_TITLE, Some(WindowHandle(1)));

            let outcome = sfx_splitter_extract(
                &mut fake,
                "7ZSplit.exe",
                r"C:\downloads\sfx.exe",
                r"C:\downloads\unpacked",
                r"C:\downloads",
                "sfx",
                500,
            );

            assert_eq!(outcome, SfxSplitterOutcome::NoScript);
            let sleeps = fake
                .calls()
                .iter()
                .filter(|c| matches!(c, Call::Sleep(_)))
                .count();
            assert!(
                sleeps >= 5,
                "expected several polling iterations before timeout, got {sleeps}"
            );
        }

        #[test]
        fn stops_polling_once_an_error_window_appears() {
            let mut fake = FakeGuiAutomation::new();
            fake.script_win_wait(SFX_SPLITTER_WINDOW_TITLE, Some(WindowHandle(1)));
            fake.script_win_exists(SFX_SPLITTER_ERROR_TITLE, true);

            let outcome = sfx_splitter_extract(
                &mut fake,
                "7ZSplit.exe",
                r"C:\downloads\sfx.exe",
                r"C:\downloads\unpacked",
                r"C:\downloads",
                "sfx",
                5_000,
            );

            assert_eq!(outcome, SfxSplitterOutcome::NoScript);
        }
    }
}
