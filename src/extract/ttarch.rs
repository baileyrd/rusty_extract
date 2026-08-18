//! ttarchext (`ttarchext.exe`) — Telltale Games `.ttarch` archives.

use super::{Invocation, WindowMode};

/// Builds the invocation `Case $TYPE_TTARCH`'s game-selected extraction
/// step makes (UniExtract.au3:3147): `<program> -m <game_index> "<file>"
/// "<outdir>"`, run in `outdir` with the window hidden.
///
/// **Scope — invocation only, GUI game selection out of scope.** The
/// source first runs `ttarchext.exe` alone to list every game it
/// supports, then presents that list via `GUI_MethodSelectList` (C053,
/// deferred GUI subsystem, D001) so the caller can pick which one `file`
/// belongs to; `game_index` is that selection's index into the game
/// list, already resolved by the time this invocation is built. The
/// listing step and the game-list parsing that produces the index are
/// out of scope here — this only builds the final extraction command
/// line. A `None` selection (`$iChoice == 0`, "no matching game" in the
/// source's own list) means the source never calls `_Run` at all —
/// composite, conditional dispatch, not registered in
/// `extract::dispatch::HARDCODED_CASES`.
pub fn invocation(program: &str, game_index: u32, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-m".to_string(),
            game_index.to_string(),
            file.to_string(),
            outdir.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C100: the constructed invocation
    /// matches UniExtract.au3:3147's `ttarchext.exe -m <index> "<file>"
    /// "<outdir>"` call — program, args (including the game index as a
    /// bare numeric token, not quoted), `outdir` as both the working
    /// directory and the trailing destination argument, and the hidden
    /// window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\ttarchext.exe",
            7,
            r"C:\downloads\archive.ttarch",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\ttarchext.exe");
        assert_eq!(
            inv.args,
            vec![
                "-m".to_string(),
                "7".to_string(),
                r"C:\downloads\archive.ttarch".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
