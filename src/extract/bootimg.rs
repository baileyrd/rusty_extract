//! bootimg (`bootimg.exe`) — Android boot images (`.img`).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_BOOTIMG`
/// (UniExtract.au3:2421-2429) makes: `<program> --unpack-bootimg`, run in
/// `outdir` with the window minimized (`@SW_MINIMIZE`, explicit).
///
/// `bootimg.exe` takes no file argument at all — `--unpack-bootimg`
/// implicitly operates on a file named exactly `boot.img` in its own
/// current working directory. That's why the source stages the input file
/// before running this: it copies `bootimg.exe` itself into `outdir`
/// (`FileCopy($bindir & $bootimg, $outdir)`) and renames the archive to
/// `outdir\boot.img` (`_FileMove($file, $outdir & '\boot.img')`) *before*
/// this call, then renames it back and deletes the copied exe afterward.
/// That staging is real filesystem I/O, separate runtime behavior from
/// building this invocation — the same "invocation vs. staging" boundary
/// every module in this crate already draws (see the `extract` module doc
/// comment) — so `program` here must already point at the exe *as copied
/// into `outdir`*, and the archive must already be sitting at
/// `outdir\boot.img` by the time this invocation runs.
///
/// **Scope note — shell wrapping not modeled as a literal string:** the
/// source builds this as `$cmd & '"' & $ret & ' --unpack-bootimg"'` —
/// `cmd.exe /d /c "<ret> --unpack-bootimg"`, the whole program+argument
/// pair inside one quoted token, the classic idiom for running a
/// space-containing path through `cmd.exe`. Functionally this still runs
/// exactly `<ret> --unpack-bootimg`, so this port's `Invocation` targets
/// the exe directly, the same as every other module in this crate.
///
/// **Not modeled here:** the preceding `HasPlugin($bootimg)` precondition
/// check, and the four staging/cleanup calls described above (`FileCopy`,
/// the two `_FileMove`s, `FileDelete`) — all separate runtime behavior,
/// not part of building this one invocation.
pub fn invocation(program: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["--unpack-bootimg".to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C063: the constructed invocation matches
    /// UniExtract.au3:2421-2429's effective `bootimg.exe --unpack-bootimg`
    /// call — program, args, the `$outdir` working directory, and a
    /// minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\downloads\image_unpacked\bootimg.exe",
            r"C:\downloads\image_unpacked",
        );
        assert_eq!(inv.program, r"C:\downloads\image_unpacked\bootimg.exe");
        assert_eq!(inv.args, vec!["--unpack-bootimg".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\image_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
