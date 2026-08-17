//! umodel (`umodel.exe`/`unreal.exe`) — Unreal Engine packages.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_UNREAL`
/// (UniExtract.au3:3211-3214) makes: `<program> -export -all -sounds
/// -3rdparty -path="<file_dir>" -out="<outdir>" *`, run in `outdir` with the
/// window minimized.
///
/// `-path="<file_dir>"` and `-out="<outdir>"` are each built as a single
/// argument token with the flag directly concatenated to a quoted value (no
/// space, embedded quote characters included) — the same concatenated-flag
/// pattern already established in `extract::bcm`/`extract::lzop` for
/// similar arguments. The trailing `*` is a literal wildcard argument
/// token, passed through as-is.
///
/// Matching the source's own comment on this `Case`, umodel extracts files
/// from *all* packages in `file_dir`, not only the one the user selected —
/// a documented quirk of the source's behavior, preserved here rather than
/// "fixed".
///
/// Scope note: the source's preceding `HasPlugin($unreal)` call is a
/// precondition check — separate runtime behavior, not part of building
/// this invocation, and out of scope for this row.
pub fn invocation(program: &str, file_dir: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-export".to_string(),
            "-all".to_string(),
            "-sounds".to_string(),
            "-3rdparty".to_string(),
            format!("-path=\"{file_dir}\""),
            format!("-out=\"{outdir}\""),
            "*".to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C103: matches UniExtract.au3:3211-3214's
    /// `_Run($unreal & ' -export -all -sounds -3rdparty -path="' & $filedir
    /// & '" -out="' & $outdir & '" *', $outdir, @SW_MINIMIZE, True, True,
    /// False)` — program, argument order (including the concatenated
    /// `-path="..."`/`-out="..."` tokens and the trailing `*`), the
    /// `$outdir` working directory, and the minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\umodel.exe",
            r"C:\downloads",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\umodel.exe");
        assert_eq!(
            inv.args,
            vec![
                "-export".to_string(),
                "-all".to_string(),
                "-sounds".to_string(),
                "-3rdparty".to_string(),
                r#"-path="C:\downloads""#.to_string(),
                r#"-out="C:\downloads\archive_unpacked""#.to_string(),
                "*".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
