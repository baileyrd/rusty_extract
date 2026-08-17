//! lzop (`lzop.exe`) — `.lzo` LZO compressed files.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_LZO`
/// (UniExtract.au3:2786-2787) makes: `<program> -d -p"<outdir>" "<file>"`,
/// run in `file_dir` (the input file's own directory).
///
/// The source's `_Run` call omits the third positional `$show_flag`
/// argument, so `_Run`'s own default applies: `@SW_MINIMIZE`. No window
/// flag appears literally in this `Case`, but that omission is itself what
/// selects `Minimized` here — it isn't a guess.
///
/// Note the `-p"<outdir>"` argument: like several other extractors, the
/// source concatenates `-p` directly onto the quoted outdir with no space,
/// producing a single argument token that includes the embedded quote
/// characters — not two separate args and not an unquoted `-p<outdir>`.
pub fn invocation(program: &str, file_dir: &str, outdir: &str, file: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-d".to_string(),
            format!("-p\"{outdir}\""),
            file.to_string(),
        ],
        working_dir: file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C081: matches UniExtract.au3:2786-2787's
    /// `_Run($lzo & ' -d -p"' & $outdir & '" "' & $file & '"', $filedir)` —
    /// same program, same argument order (including the single
    /// `-p"<outdir>"` token), the `$filedir` working directory, and
    /// `Minimized`: the source passes no third `$show_flag` argument to
    /// `_Run`, so `_Run`'s own default (`@SW_MINIMIZE`) applies — this is
    /// not a guess, it's what the omission means.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\lzop.exe",
            r"C:\downloads",
            r"C:\downloads\app_unpacked",
            r"C:\downloads\app.lzo",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\lzop.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                r#"-p"C:\downloads\app_unpacked""#.to_string(),
                r"C:\downloads\app.lzo".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
