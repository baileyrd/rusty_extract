//! Composition root: the first real, end-to-end wiring of this port's
//! extraction pipeline (previously a collection of pure functions with
//! nothing driving them — `main` was a one-line stub). Proves the wiring
//! pattern for exactly two extractors (`rgss`, `ace`) rather than the full
//! ~70-case dispatch table; see ARCHITECTURE.md and `extract::dispatch`'s
//! module doc comment for why a uniform dispatch call doesn't exist yet.
//!
//! **Deliberately out of scope for this phase** (left for later, separately
//! scoped PRs, not dropped):
//! - The `/type` override (C006) and the CLI flags in `cli` (C007-C013):
//!   this binary only takes positional arguments.
//! - The detection cascade (C037-046) picking `extractor_type`
//!   automatically: this phase requires it as an explicit argument, since
//!   no detector is wired up yet.
//! - `def/*.ini` plugin-engine dispatch (`extract::plugin`,
//!   `DispatchTarget::Plugin`): only the two hardcoded cases below run.
//! - Batch-queue execution/process chaining (C011, C015, the remainder of
//!   C148) and `ExtractionTransaction`/ADR-0119 staged-commit hardening
//!   (extractors here still write straight to `outdir`, matching today's
//!   documented behavior).

use rusty_extract::extract::dispatch::{dispatch, DispatchTarget};
use rusty_extract::extract::runner::{CommandExtractorRunner, ExtractorRunner, RunOutcome};
use rusty_extract::extract::table::{self, Ctx};
use rusty_extract::log_eval::is_overwrite_success_message;
use rusty_extract::outdir::{
    decide_outdir_outcome, default_output_subfolder, reappend_trailing_backslash_after_extraction,
    resolve_output_directory, strip_trailing_backslash_for_extraction, OutdirOutcome,
};
use rusty_extract::status::{exit_code, Status};

const USAGE: &str = "usage: rusty_extract <extractor-type> <program> <file> [outdir]\n\n\
    <extractor-type> is one of: rgss, ace (the only extractors this phase wires up)\n\
    <program>        path to the helper binary to invoke\n\
    <file>           the archive to extract\n\
    [outdir]         output directory, or the token /sub (default: a\n\
                     subfolder named after <file>) — /last is not yet\n\
                     supported (no output-directory history in this phase)";

/// Splits a file path into `(dir, stem, has_extension)`, mirroring the
/// inputs `outdir::default_output_subfolder` (C138) expects: `stem` is the
/// filename with only its *last* `.`-delimited extension trimmed.
///
/// Splits on `\` directly rather than going through `std::path::Path`:
/// this port's paths are always Windows paths (this binary is Windows-only,
/// same as the rest of the port — see `RELEASE_NOTES.md`, "CI: target
/// windows-latest"), and `Path`'s separator handling is platform-dependent,
/// which would silently misparse a `\`-separated path on a non-Windows
/// build host. The rest of this codebase (e.g. `outdir::
/// resolve_output_directory`) already treats paths as plain `\`-delimited
/// strings for the same reason.
fn split_file_path(file: &str) -> (String, String, bool) {
    let (dir, filename) = match file.rfind('\\') {
        Some(idx) => (file[..idx].to_string(), file[idx + 1..].to_string()),
        None => (String::new(), file.to_string()),
    };
    match filename.rsplit_once('.') {
        Some((stem, _ext)) if !stem.is_empty() => (dir, stem.to_string(), true),
        _ => (dir, filename, false),
    }
}

/// Resolves and prepares the output directory for `file`: computes the
/// `/sub` default (C138), resolves the `outdir` argument against it (C004,
/// C139, C140), then ensures the resulting directory exists and is usable
/// (C142). Returns the resolved, trailing-slash-terminated directory path.
fn prepare_outdir(file: &str, outdir_arg: &str) -> Result<String, String> {
    if outdir_arg.eq_ignore_ascii_case("/last") {
        return Err(
            "/last requires output-directory history, not yet wired up in this phase".to_string(),
        );
    }

    let (filedir, stem, has_extension) = split_file_path(file);
    let collision_path = format!("{filedir}\\{stem}");
    let initoutdir_collision = std::fs::metadata(&collision_path)
        .map(|m| m.is_file())
        .unwrap_or(false);
    let initoutdir = default_output_subfolder(
        &filedir,
        &stem,
        has_extension,
        initoutdir_collision,
        "unpacked",
    );

    let resolved = resolve_output_directory(outdir_arg, &initoutdir, &filedir, None);
    let resolved_no_slash = resolved.strip_suffix('\\').unwrap_or(&resolved);

    let metadata = std::fs::metadata(resolved_no_slash);
    let exists = metadata.is_ok();
    let is_directory = metadata.map(|m| m.is_dir()).unwrap_or(false);
    let can_access = !exists || std::fs::read_dir(resolved_no_slash).is_ok();
    let dir_create_succeeded = !exists && std::fs::create_dir_all(resolved_no_slash).is_ok();

    match decide_outdir_outcome(exists, is_directory, can_access, dir_create_succeeded) {
        OutdirOutcome::AlreadyValid | OutdirOutcome::Created => Ok(resolved),
        OutdirOutcome::ExistsButNotADirectory => {
            Err(format!("{resolved_no_slash} exists but is not a directory"))
        }
        OutdirOutcome::ExistsButNotAccessible => {
            Err(format!("{resolved_no_slash} exists but is not accessible"))
        }
        OutdirOutcome::CreateFailed => Err(format!("failed to create {resolved_no_slash}")),
    }
}

/// Builds the `Invocation` for `extractor_type` and runs it through
/// `runner`, sandwiching the call in the C140 trailing-backslash
/// strip/reappend cycle `extract()` performs around every extraction.
/// `outdir` must already be resolved and validated (see
/// [`prepare_outdir`]).
fn run_extraction(
    extractor_type: &str,
    program: &str,
    file: &str,
    outdir: &str,
    runner: &dyn ExtractorRunner,
) -> Result<(RunOutcome, String), String> {
    let case = match dispatch(extractor_type) {
        DispatchTarget::Hardcoded(case) if case.module == "extract::rgss" => case,
        DispatchTarget::Hardcoded(case) if case.module == "extract::ace" => case,
        _ => {
            return Err(format!(
                "'{extractor_type}' is not one of the extractors this phase wires up (rgss, ace)"
            ))
        }
    };

    let stripped_outdir = strip_trailing_backslash_for_extraction(outdir);

    let format_name = match case.module {
        "extract::rgss" => "rgss",
        "extract::ace" => "ace",
        _ => unreachable!("filtered to rgss/ace above"),
    };
    let invocation = table::build(
        format_name,
        &Ctx {
            program,
            file,
            outdir: &stripped_outdir,
            ..Default::default()
        },
    )
    .unwrap_or_else(|| unreachable!("'{format_name}' is a row in extract::table::FORMATS"));

    let outcome = runner.run(&invocation);
    let final_outdir = reappend_trailing_backslash_after_extraction(&stripped_outdir);
    Ok((outcome, final_outdir))
}

/// Ports `ParseCommandLine`'s own zero-args branch (UniExtract.au3:588-
/// 591): no CLI arguments at all means interactive mode, i.e. launch the
/// main window (capability C183) rather than treat it as a usage error.
/// Windows-only, same as the GUI module itself; on any other host this
/// falls through to the ordinary usage-error path below, since there is
/// no window to launch.
#[cfg(windows)]
fn launch_gui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "UniExtract",
        options,
        Box::new(|_cc| Ok(Box::new(rusty_extract::gui::app::MainWindow::new()))),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    #[cfg(windows)]
    if args.is_empty() {
        launch_gui();
        return;
    }

    let [extractor_type, program, file, rest @ ..] = args.as_slice() else {
        eprintln!("{USAGE}");
        std::process::exit(exit_code(Status::Syntax));
    };
    let outdir_arg = rest.first().map(String::as_str).unwrap_or("/sub");

    let outdir = match prepare_outdir(file, outdir_arg) {
        Ok(outdir) => outdir,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(exit_code(Status::InvalidDir));
        }
    };

    let (outcome, final_outdir) = match run_extraction(
        extractor_type,
        program,
        file,
        &outdir,
        &CommandExtractorRunner,
    ) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(exit_code(Status::Syntax));
        }
    };

    let combined_log = format!("{}\n{}", outcome.stdout, outcome.stderr);
    let succeeded = outcome.exit_status == Some(0) || is_overwrite_success_message(&combined_log);

    if succeeded {
        println!("extracted to {final_outdir}");
        std::process::exit(exit_code(Status::Success));
    } else {
        eprintln!("extraction failed:\n{combined_log}");
        std::process::exit(exit_code(Status::Failed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_extract::extract::runner::FakeExtractorRunner;

    /// Proves the dispatch-and-run wiring for `rgss`: the constructed
    /// `Invocation` matches `rgss::invocation`'s own parity test
    /// expectations, and the runner's canned outcome flows back out.
    #[test]
    fn wires_rgss_through_dispatch_to_the_runner() {
        let fake = FakeExtractorRunner::new(RunOutcome {
            exit_status: Some(0),
            stdout: "OK".to_string(),
            stderr: String::new(),
        });

        let (outcome, final_outdir) = run_extraction(
            "rgss",
            r"C:\UniExtract\bin\RgssDecrypter.exe",
            r"C:\games\Game.rgss3a",
            r"C:\games\Game_unpacked\",
            &fake,
        )
        .unwrap();

        assert_eq!(outcome.stdout, "OK");
        assert_eq!(final_outdir, r"C:\games\Game_unpacked\");
        let calls = fake.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].program, r"C:\UniExtract\bin\RgssDecrypter.exe");
        assert_eq!(calls[0].working_dir, r"C:\games\Game_unpacked");
    }

    /// Proves the dispatch-and-run wiring for `ace`, including its
    /// reversed `(program, outdir, file)` argument order versus `rgss`.
    #[test]
    fn wires_ace_through_dispatch_to_the_runner() {
        let fake = FakeExtractorRunner::new(RunOutcome {
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        });

        let (_outcome, _final_outdir) = run_extraction(
            "ace",
            r"C:\UniExtract\bin\acefile.exe",
            r"C:\downloads\archive.ace",
            r"C:\downloads\archive_unpacked\",
            &fake,
        )
        .unwrap();

        let calls = fake.calls();
        assert_eq!(calls.len(), 1);
        assert!(calls[0]
            .args
            .contains(&r"C:\downloads\archive_unpacked".to_string()));
        assert!(calls[0]
            .args
            .contains(&r"C:\downloads\archive.ace".to_string()));
    }

    /// An extractor type not wired up in this phase (even one `dispatch`
    /// itself recognizes, like `rpa`) is rejected rather than silently
    /// mishandled.
    #[test]
    fn rejects_extractor_types_not_wired_up_yet() {
        let fake = FakeExtractorRunner::new(RunOutcome {
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        });

        let err = run_extraction("rpa", "prog", "file", r"C:\out\", &fake).unwrap_err();
        assert!(err.contains("rpa"));
        assert!(fake.calls().is_empty());
    }

    /// `split_file_path` extracts the same `(filedir, stem, has_extension)`
    /// shape `default_output_subfolder`'s own parity tests exercise.
    #[test]
    fn split_file_path_separates_dir_stem_and_extension() {
        assert_eq!(
            split_file_path(r"C:\downloads\archive.zip"),
            (r"C:\downloads".to_string(), "archive".to_string(), true)
        );
        assert_eq!(
            split_file_path(r"C:\downloads\noext"),
            (r"C:\downloads".to_string(), "noext".to_string(), false)
        );
    }
}
