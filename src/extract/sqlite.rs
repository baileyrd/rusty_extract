//! SQLite (`sqlite3.exe`) — SQLite database dump.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_SQLITE`
/// (UniExtract.au3:3032-3033) makes to dump a SQLite database to SQL text:
/// `<program> "<file>" .dump`, run in `filedir` with the window hidden.
///
/// The source calls this through `FetchStdout($sqlite & ' "' & $file & '"
/// .dump"', $filedir, @SW_HIDE, 0)`, which routes through `_MakeCommand`'s
/// generic `cmd.exe /d /c` shell-wrapping (`FetchStdout`'s `$bUseCmd`
/// defaults to `True`). That wrapping has no effect on the arguments
/// `sqlite3.exe` itself actually receives for this call — no output
/// redirection or piping happens at this call site, unlike
/// `FetchStdout`'s tee-log caller (UniExtract.au3:4885), so `cmd.exe` here
/// is a transparent passthrough — and this port's [`Invocation`] model
/// (program + decomposed args, matching every other module in this crate)
/// targets `sqlite3.exe` directly rather than reproducing the shell
/// wrapper.
///
/// **Source quirk, not reproduced as a literal character:** the source's
/// string literal is `' "' & $file & '" .dump"'` — a stray, unbalanced
/// double quote follows `.dump` with nothing after it. Windows' standard
/// command-line argument parsing (which both `cmd.exe` and a typical C
/// runtime's `argv` startup code use) toggles "inside quotes" on each
/// unescaped `"` — an unmatched trailing one with no further characters
/// contributes no literal quote to the parsed token, so the actual third
/// argument `sqlite3.exe` receives is exactly `.dump`, matching the
/// well-known `sqlite3 <db> ".dump"` CLI usage this case is clearly
/// invoking. This function's `args` reflect that effective argument, not
/// the source's literal (and inert) stray character.
///
/// **Scope — invocation only.** Capturing `sqlite3.exe`'s stdout and
/// writing it to `<outdir>\<filename>.sql` (UniExtract.au3:3034-3037) is
/// separate runtime behavior — committing captured output to the
/// destination — not part of building this invocation, matching every
/// other module in this crate (see the `extract` module doc comment).
pub fn invocation(program: &str, file: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), ".dump".to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C097: the constructed invocation matches
    /// UniExtract.au3:3032-3033's effective `sqlite3.exe "<file>" .dump`
    /// call — program, args, the `$filedir` working directory (not
    /// `$outdir`), and the hidden window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\sqlite3.exe",
            r"C:\downloads\data.db",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\sqlite3.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\data.db".to_string(), ".dump".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
