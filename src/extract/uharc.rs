//! UHARC 3-version fallback chain (`UNUHARC06.EXE`, `UHARC04.EXE`,
//! `UHARC02.EXE`) — `.uha` archives.

use super::{Invocation, WindowMode};

/// Builds the first attempt UniExtract2's `Case $TYPE_UHA`
/// (UniExtract.au3:3154) makes: `<program> x -t"<outdir>" "<file>"`, run in
/// `outdir` with the window minimized (`_Run`'s own default for the
/// omitted `$show_flag` argument). `-t"<outdir>"` is a single
/// concatenated-flag argument token (flag directly joined to a quoted
/// value, no space), the same pattern already established in
/// `extract::bcm`/`extract::lzop`/`extract::unreal`.
pub fn uharc_invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), format!("-t{outdir}"), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Builds the second-attempt fallback UniExtract2 makes
/// (UniExtract.au3:3156) when [`uharc_invocation`]'s run didn't succeed:
/// same shape, a different (older) UHARC binary — `<program> x
/// -t"<outdir>" "<file>"`, run in `outdir` with the window minimized.
pub fn uharc04_invocation(program: &str, outdir: &str, file: &str) -> Invocation {
    uharc_invocation(program, outdir, file)
}

/// Builds the third-and-last fallback UniExtract2 makes
/// (UniExtract.au3:3158) when both prior attempts didn't succeed:
/// `<program> x -t<outdir_short> <file_short>`, run in `outdir` with the
/// window minimized.
///
/// **`outdir_short`/`file_short` are caller-supplied.** The source computes
/// these via `FileGetShortName($outdir)`/`FileGetShortName($file)` — the
/// Windows 8.3 short-path-name API — a real OS call this pure function
/// can't perform itself, the same "caller supplies an OS-dependent fact"
/// pattern already used for `outdir::decide_outdir_outcome`'s filesystem
/// booleans. Unlike the other two attempts, neither argument is quoted
/// here: 8.3 short names never contain spaces, so the source omits the
/// quoting it uses everywhere else — preserved as written, not "fixed"
/// into the quoted style the other two functions use.
pub fn uharc02_invocation(program: &str, outdir_short: &str, file_short: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "x".to_string(),
            format!("-t{outdir_short}"),
            file_short.to_string(),
        ],
        working_dir: outdir_short.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_common_shape(inv: &Invocation, program: &str, outdir: &str) {
        assert_eq!(inv.program, program);
        assert_eq!(inv.working_dir, outdir);
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C101: the first attempt matches
    /// UniExtract.au3:3154's `UNUHARC06.EXE x -t"<outdir>" "<file>"`.
    #[test]
    fn uharc_matches_source_invocation() {
        let inv = uharc_invocation(
            r"C:\UniExtract\bin\UNUHARC06.EXE",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.uha",
        );
        assert_common_shape(
            &inv,
            r"C:\UniExtract\bin\UNUHARC06.EXE",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.uha".to_string(),
            ]
        );
    }

    /// Parity test for capability C101: the second fallback attempt
    /// matches UniExtract.au3:3156 — same shape as the first, a different
    /// binary.
    #[test]
    fn uharc04_matches_source_invocation() {
        let inv = uharc04_invocation(
            r"C:\UniExtract\bin\UHARC04.EXE",
            r"C:\downloads\archive_unpacked",
            r"C:\downloads\archive.uha",
        );
        assert_common_shape(
            &inv,
            r"C:\UniExtract\bin\UHARC04.EXE",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.uha".to_string(),
            ]
        );
    }

    /// Parity test for capability C101: the third fallback attempt matches
    /// UniExtract.au3:3158 — 8.3 short-form paths, unquoted (unlike the
    /// other two attempts).
    #[test]
    fn uharc02_matches_source_invocation_and_is_unquoted() {
        let inv = uharc02_invocation(
            r"C:\UniExtract\bin\UHARC02.EXE",
            r"C:\DOWNLO~1\ARCHIV~1",
            r"C:\DOWNLO~1\ARCHIV~2.UHA",
        );
        assert_common_shape(
            &inv,
            r"C:\UniExtract\bin\UHARC02.EXE",
            r"C:\DOWNLO~1\ARCHIV~1",
        );
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\DOWNLO~1\ARCHIV~1".to_string(),
                r"C:\DOWNLO~1\ARCHIV~2.UHA".to_string(),
            ]
        );
    }
}
