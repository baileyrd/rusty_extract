//! UnRAR (`UnRAR.exe`) — RAR archives, RAR SFX.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_RAR`
/// (UniExtract.au3:3005) makes to extract a RAR archive:
/// `<program> x -kb [-p<password>] "<file>"`, run in `outdir` with the
/// window shown.
///
/// The source's literal command-line string is `' x -kb ' &
/// ($sPassword == 0? '"': '-p"' & $sPassword & '" "') & $file & '"'` —
/// when no password was found (`$sPassword == 0`), that ternary
/// contributes just an opening quote before `$file`, giving `x -kb
/// "<file>"`; when a password was found, it contributes `-p"<password>"
/// "`, giving `x -kb -p"<password>" "<file>"`. Either way, after
/// standard Windows command-line argument parsing removes the quoting,
/// `UnRAR.exe` receives `password` (when present) as part of the same
/// argument as the `-p` flag — this function's `args` reflect that
/// effective argument vector, matching this crate's `Invocation` model
/// (decomposed program + args, not a literal command-line string).
///
/// **Scope — invocation only, given an already-resolved password.**
/// Resolving `password` itself is `_FindArchivePassword()`'s job
/// (UniExtract.au3:4848-4877, capability C160's automated password-list
/// trial) — a separate capability this function doesn't attempt;
/// `password = None` reproduces `$sPassword == 0`, the "no password
/// found or needed" case. Interpreting the run's result — `@error = 3`
/// meaning a missing archive part (`$STATUS_MISSINGPART`), `@extended`
/// meaning the password was wrong (`$STATUS_PASSWORD`) — is real
/// process-execution outcome handling, not part of building the
/// invocation, matching every other module in this crate (see the
/// `extract` module doc comment).
pub fn invocation(program: &str, file: &str, outdir: &str, password: Option<&str>) -> Invocation {
    let mut args = vec!["x".to_string(), "-kb".to_string()];
    if let Some(password) = password {
        args.push(format!("-p{password}"));
    }
    args.push(file.to_string());
    Invocation {
        program: program.to_string(),
        args,
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C092: no password resolved builds
    /// `x -kb "<file>"`, matching `$sPassword == 0`.
    #[test]
    fn matches_source_invocation_without_password() {
        let inv = invocation(
            r"C:\UniExtract\bin\UnRAR.exe",
            r"C:\downloads\archive.rar",
            r"C:\downloads",
            None,
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\UnRAR.exe");
        assert_eq!(
            inv.args,
            vec!["x", "-kb", r"C:\downloads\archive.rar"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C092: a resolved password is folded
    /// into the `-p<password>` argument, matching the source's
    /// effective (post-quote-parsing) argument shape.
    #[test]
    fn matches_source_invocation_with_password() {
        let inv = invocation(
            r"C:\UniExtract\bin\UnRAR.exe",
            r"C:\downloads\archive.rar",
            r"C:\downloads",
            Some("hunter2"),
        );
        assert_eq!(
            inv.args,
            vec!["x", "-kb", "-phunter2", r"C:\downloads\archive.rar"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
