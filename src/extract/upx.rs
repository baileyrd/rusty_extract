//! upx (`upx.exe`) — UPX-packed executables (unpack).

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract.au3:3617-3623 makes from inside `Case
/// $PACKER_UPX`: `<program> -d -k "<file>"`, run in `$filedir` with the
/// window minimized.
///
/// Unlike the other modules in `extract`, this isn't a top-level `$arctype`
/// dispatch case — there is no `$TYPE_UPX` constant in the source's main
/// extractor `Switch $arctype`. `Case $PACKER_UPX` belongs to a wholly
/// separate `Switch $packer` (a post-extraction "unpack a packed
/// executable" routine keyed on `$PACKER_UPX`/`$PACKER_ASPACK`, not
/// `$arctype`/`$TYPE_*`), so it's intentionally absent from
/// `extract::dispatch::HARDCODED_CASES` — that table represents only the
/// main `extract($arctype, ...)` dispatch, the same reason
/// `extract::xor` is absent from it (see that module's doc comment).
///
/// The source's call, `_Run($upx & ' -d -k "' & $file & '"', $filedir)`,
/// passes only two of `_Run`'s arguments (`$f`, `$sWorkingDir`), omitting
/// `$show_flag`. Per `_Run`'s signature, `Func _Run($f, $sWorkingDir =
/// $outdir, $show_flag = @SW_MINIMIZE, ...)`, the omitted `$show_flag`
/// takes its own default, `@SW_MINIMIZE` — mapped here to
/// [`WindowMode::Minimized`], not a guess made for this port.
///
/// Scope note: the lines following `_Run` in the source —
/// `StringTrimRight($fileext, 1) & '~'`, `FileExists(...)`, and the two
/// `_FileMove(...)` calls — rename UPX's decompressed output file
/// (`<filename>.<fileext minus last char>~`) into place over the original.
/// That's separate runtime behavior, not part of building this
/// invocation, and is out of scope for this row.
pub fn invocation(program: &str, file: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-d".to_string(), "-k".to_string(), file.to_string()],
        working_dir: file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C112: matches UniExtract.au3:3617-3623's
    /// `_Run($upx & ' -d -k "' & $file & '"', $filedir)` — program,
    /// argument order (`-d -k <file>`), working directory (`$filedir`, the
    /// explicit second argument), and `_Run`'s own default for window
    /// (`@SW_MINIMIZE`), since `$show_flag` is omitted in the source call.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\upx.exe",
            r"C:\downloads\archive_unpacked\packed.exe",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\upx.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                "-k".to_string(),
                r"C:\downloads\archive_unpacked\packed.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
