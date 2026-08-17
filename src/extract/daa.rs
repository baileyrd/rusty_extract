//! DAA→ISO conversion (`daa2iso.exe`) — PowerISO DAA disk images, stage-1
//! conversion to ISO.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract2's `Case $TYPE_DAA`
/// (UniExtract.au3:2505-2508) makes: `<program> "<file>" "<outdir>\<filename>.iso"`,
/// run in `outdir` with the window minimized.
///
/// `$daa` (UniExtract.au3:214) is `Const $daa = "daa2iso.exe"` — the
/// DAA-to-ISO conversion tool.
///
/// The source's `_Run` call passes no explicit show-flag argument, so
/// `_Run`'s own default applies (`Func _Run($f, $sWorkingDir = $outdir,
/// $show_flag = @SW_MINIMIZE, ...)`, UniExtract.au3:4880) — the window is
/// minimized because that omission *is* the source selecting
/// `@SW_MINIMIZE`, not a guess made for this port.
///
/// **Deliberately preserved quirk — do not add an existence check here.**
/// This capability (C146) exists specifically because the source builds
/// `$sFile` (the target `.iso` path, `$outdir & "\" & $filename & ".iso"`)
/// and passes it straight to `_Run` with no check for whether that file
/// already exists. A pre-existing `<filename>.iso` in `outdir` is silently
/// overwritten, or `daa2iso.exe` does whatever it does when its target
/// already exists — UniExtract2 itself performs no pre-check either way.
/// This is a documented bug in the source itself (`todo.txt:52`,
/// "Converting to iso failes when iso file already exists (helper binary
/// not terminating correctly)"). The migration's job is to preserve
/// UniExtract2's observable behavior, not to fix bugs it never fixed —
/// adding a `Path::exists` guard (or any other existence check) before
/// building this invocation would be "fixing" behavior this capability is
/// explicitly required to preserve as-is. If this ever needs fixing, that's
/// a separate, deliberately-scoped capability, not a change to this one.
///
/// **Scope note:** `_CreateTrayMessageBox(...)` (UniExtract.au3:2506), the
/// "Extracting... DAA disk image (stage 1)" progress notification the
/// source shows before building this invocation, is separate, out-of-scope
/// GUI-subsystem behavior (manifest row D001) — not part of this row.
pub fn invocation(program: &str, file: &str, outdir: &str, filename: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), format!("{outdir}\\{filename}.iso")],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C146: matches UniExtract.au3:2505-2508's
    /// `Local $sFile = $outdir & "\" & $filename & ".iso"` followed by
    /// `_Run($daa & ' "' & $file & '" "' & $sFile & '"', $outdir)` — program,
    /// argument order (including the explicit `.iso` output path, built and
    /// used with no pre-existing-file check, matching the source's own
    /// `todo.txt:52`-documented bug), the `$outdir` working directory, and
    /// `_Run`'s own default of `@SW_MINIMIZE` for the omitted show-flag
    /// argument.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\daa2iso.exe",
            r"C:\downloads\image.daa",
            r"C:\downloads\image_unpacked",
            "image",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\daa2iso.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\image.daa".to_string(),
                r"C:\downloads\image_unpacked\image.iso".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\image_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
