//! AspackDie (`AspackDie.exe`) — ASPack-packed executables.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract.au3:3624-3625 makes from inside `Case
/// $PACKER_ASPACK`: `<program> "<file>" "<dest_path>" /NO_PROMPT`, run in
/// `$filedir` with the window minimized.
///
/// Unlike the other modules in `extract`, this isn't a top-level `$arctype`
/// dispatch case — there is no `$TYPE_ASPACK` constant in the source's main
/// extractor `Switch $arctype`. `Case $PACKER_ASPACK` belongs to a wholly
/// separate `Switch $packer` (a post-extraction "unpack a packed
/// executable" routine keyed on `$PACKER_UPX`/`$PACKER_ASPACK`, not
/// `$arctype`/`$TYPE_*`), so it's intentionally absent from
/// `extract::dispatch::HARDCODED_CASES` — that table represents only the
/// main `extract($arctype, ...)` dispatch, the same reason
/// `extract::upx` (the sibling `$PACKER_UPX` case) is absent from it — see
/// that module's doc comment.
///
/// The source's call, `_Run($aspack & ' "' & $file & '" "' & $sPath & '"
/// /NO_PROMPT', $filedir)`, passes only two of `_Run`'s arguments (`$f`,
/// `$sWorkingDir`), omitting `$show_flag`. Per `_Run`'s signature, `Func
/// _Run($f, $sWorkingDir = $outdir, $show_flag = @SW_MINIMIZE, ...)`, the
/// omitted `$show_flag` takes its own default, `@SW_MINIMIZE` — mapped
/// here to [`WindowMode::Minimized`], not a guess made for this port.
pub fn invocation(program: &str, file: &str, dest_path: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            file.to_string(),
            dest_path.to_string(),
            "/NO_PROMPT".to_string(),
        ],
        working_dir: file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C058: matches UniExtract.au3:3624-3625's
    /// `_Run($aspack & ' "' & $file & '" "' & $sPath & '" /NO_PROMPT',
    /// $filedir)` — program, argument order (`<file> <dest_path>
    /// /NO_PROMPT`), working directory (`$filedir`, the explicit second
    /// argument), and `_Run`'s own default for window (`@SW_MINIMIZE`),
    /// since `$show_flag` is omitted in the source call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\AspackDie.exe",
            r"C:\downloads\archive_unpacked\packed.exe",
            r"C:\downloads\archive_unpacked\packed_unpacked.exe",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\AspackDie.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive_unpacked\packed.exe".to_string(),
                r"C:\downloads\archive_unpacked\packed_unpacked.exe".to_string(),
                "/NO_PROMPT".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
