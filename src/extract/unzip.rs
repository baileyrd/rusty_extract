//! Info-ZIP UnZip (`unzip.exe`) — generic ZIP fallback when 7-Zip fails.

use super::{Invocation, WindowMode};

/// Builds the fallback invocation UniExtract.au3:3384-3388 makes from
/// inside `Case $TYPE_ZIP`, once the initial 7-Zip attempt has failed:
/// `<program> -x "<file>"`, run in `outdir` with the window minimized.
///
/// The source's full case is:
///
/// ```text
/// Case $TYPE_ZIP
///     If Not extract($TYPE_7Z, -1, $additionalParameters, False, True) Then
///         If $arcdisp > -1 Then _CreateTrayMessageBox(t('EXTRACTING') & @CRLF & $arcdisp)
///         _Run($zip & ' -x "' & $file & '"', $outdir, @SW_MINIMIZE, True, False)
///     EndIf
/// ```
///
/// This function ports only the innermost `_Run(...)` call — the `-x`
/// switch plus the quoted `$file` becomes `args`, the explicit `$outdir`
/// second argument becomes `working_dir`, and the explicit `@SW_MINIMIZE`
/// third argument becomes [`WindowMode::Minimized`].
///
/// Two things this function deliberately does *not* cover:
///
/// - The enclosing `If Not extract($TYPE_7Z, ...) Then ... EndIf`: `$TYPE_ZIP`
///   first recursively calls `extract()` with `$TYPE_7Z`, and only runs this
///   Info-ZIP UnZip fallback if that 7-Zip attempt fails. That
///   conditional-recursive-dispatch mechanism — try 7-Zip first, fall back
///   to Info-ZIP UnZip on failure — is a separate, already-tracked
///   composite/recursive-dispatch capability, not this row.
/// - `_CreateTrayMessageBox(t('EXTRACTING') & @CRLF & $arcdisp)`: a UI
///   progress notification belonging to the deferred GUI subsystem
///   (manifest row D001), out of scope here.
///
/// Because the real `$TYPE_ZIP` dispatch behavior is that composite
/// try-7z-then-fall-back-to-unzip sequence rather than a single flat call,
/// this is intentionally **absent from `extract::dispatch::HARDCODED_CASES`**
/// for now: that table maps one `$arctype` key to one fully-hardcoded case,
/// and a bare `"zip" -> extract::unzip` entry would misrepresent the
/// source's actual dispatch logic, misleading a reader of `dispatch.rs` into
/// thinking `dispatch("zip")` fully implements the case when it doesn't yet
/// include the 7z-first branch. Registering it accurately requires the
/// composite/recursive dispatch capability to exist first — the same
/// reasoning `extract::xor` uses for the same kind of exclusion (see its
/// module doc comment).
pub fn invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-x".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C109: the constructed invocation matches
    /// UniExtract.au3:3384-3388's `_Run($zip & ' -x "' & $file & '"',
    /// $outdir, @SW_MINIMIZE, True, False)` — same program, same `-x`
    /// switch and file argument order, same working directory, same
    /// minimized window.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\UniExtract\bin\unzip.exe",
            r"C:\downloads\archive.zip",
            r"C:\downloads\archive_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unzip.exe");
        assert_eq!(
            inv.args,
            vec!["-x".to_string(), r"C:\downloads\archive.zip".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }
}
