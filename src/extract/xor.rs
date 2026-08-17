//! xor (`xor.exe`) — byte-XOR decode of Ghost Installer's overlay-extracted
//! CAB blob.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract.au3:2598 makes from inside
/// `Case $TYPE_GHOST`: `<program> "<overlay_file>" "<outdir>\<filename>.cab"
/// 0x8D`.
///
/// Unlike the other modules in `extract`, this isn't a top-level
/// `$arctype` dispatch case — there is no `$TYPE_XOR` constant in the
/// source. It's an internal helper call the Ghost Installer case makes
/// itself, after unpacking an overlay blob, to XOR-decode that blob into a
/// `.cab` before handing it to the CAB extractor. So it's intentionally
/// absent from `extract::dispatch::HARDCODED_CASES`, the same way
/// `extract::plugin` is absent — that table is only for the `Switch`'s own
/// top-level keys.
///
/// The source's call, `_Run($xor & ' "' & $ret2 & '" "' & $outdir & '\' &
/// $filename & '.cab" 0x8D')`, omits all three of `_Run`'s optional
/// arguments, so its signature's own defaults apply: `Func _Run($f,
/// $sWorkingDir = $outdir, $show_flag = @SW_MINIMIZE, ...)`. That's why
/// `working_dir` here is `outdir` (the same outer-scope `$outdir` this
/// call's other arguments reference, not `overlay_file`'s directory or any
/// other file-specific path) and `window` is [`WindowMode::Minimized`] —
/// both are `_Run`'s defaults, not a guess made for this port. The literal
/// `0x8D` XOR key byte is preserved verbatim as its own trailing argument
/// token, exactly as the source concatenates it onto the command line.
pub fn invocation(program: &str, overlay_file: &str, outdir: &str, filename: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            overlay_file.to_string(),
            format!("{outdir}\\{filename}.cab"),
            "0x8D".to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C070: matches UniExtract.au3:2598's
    /// `_Run($xor & ' "' & $ret2 & '" "' & $outdir & '\' & $filename &
    /// '.cab" 0x8D')` — program, argument order (including the literal
    /// `0x8D` key byte), and `_Run`'s own defaults for working directory
    /// (`$outdir`) and window (`@SW_MINIMIZE`), since all three optional
    /// `_Run` arguments are omitted in the source.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\xor.exe",
            r"C:\downloads\archive_unpacked\overlay.bin",
            r"C:\downloads\archive_unpacked",
            "archive",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\xor.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive_unpacked\overlay.bin".to_string(),
                r"C:\downloads\archive_unpacked\archive.cab".to_string(),
                "0x8D".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
