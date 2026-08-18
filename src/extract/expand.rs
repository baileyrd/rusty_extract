//! Windows `expand.exe` — Microsoft CAB archives (`.cab`), MSU updates,
//! including the self-extracting Type-1 CAB path.

use super::{Invocation, WindowMode};

/// Builds the `expand.exe` invocation shared by both call sites that use
/// it — the non-self-extracting `Case $TYPE_CAB` branch
/// (UniExtract.au3:2438) and `Case $TYPE_MSU`'s two `expand.exe` calls
/// (UniExtract.au3:2916,2927): `<program> -F:* "<file>" "<destdir>"`, run
/// in `filedir` with the window hidden.
///
/// **Scope note — shell wrapping not modeled as a literal string:** the
/// source builds this as `$cmd & $expand & ' -F:* "' & $file & '" "' &
/// $destdir & '"'` — a literal `cmd.exe /d /c ` prefix concatenated
/// directly onto the command string (bypassing `_Run`'s own
/// `_MakeCommand` bindir-prefixing, since `$expand` is already a fully
/// resolved, pre-quoted `@SystemDir` path, not a bare bindir-relative
/// name). Functionally this still runs exactly `<expand> -F:* "<file>"
/// "<destdir>"`, so this port's `Invocation` targets the exe directly,
/// the same as every other module in this crate.
///
/// **Not modeled here:** the CAB call site's preceding `check7z($arcdisp)`
/// probe and `HasPlugin($expand)` precondition check; the MSU call site's
/// surrounding orchestration (temp-directory staging, extracting a nested
/// `.cab` found inside the first expansion, and sorting the second
/// expansion's output into `x86`/`x64`/`WOW64`/`MSIL` subfolders by
/// filename prefix) — all separate runtime behavior, not part of building
/// either `expand.exe` call.
pub fn invocation(program: &str, file: &str, destdir: &str, filedir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-F:*".to_string(), file.to_string(), destdir.to_string()],
        working_dir: filedir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the self-extracting-CAB invocation `Case $TYPE_CAB`'s "Type 1"
/// branch makes (UniExtract.au3:2432-2433) when `$sFileType` reports "Type
/// 1" — the archive is itself a self-extracting CAB executable, so it's
/// run directly: `<file> /q /x:<outdir>`, run in `outdir`.
///
/// `/x:<outdir>` is a single concatenated-flag argument token (flag
/// directly joined to the destination, no space), the same pattern
/// already established in `extract::bcm`/`extract::lzop`/`extract::unreal`.
///
/// The source builds this as `Warn_Execute(Quote($file & '" /q /x:"' &
/// $outdir))`, run via `RunWait` with no explicit `show_flag`, so it takes
/// `RunWait`'s own default, `@SW_SHOWNORMAL` — mapped to `WindowMode::Show`,
/// the same mapping `extract::nbh` already uses for the same default.
///
/// **Not modeled here:** `Warn_Execute`'s "you're about to run an
/// executable, continue?" confirmation gate (`warnexecute` preference,
/// C023) — a deferred-GUI-subsystem concern (manifest row D001) that
/// either passes the command through unchanged or aborts the run entirely;
/// this function reproduces only the command it passes through.
pub fn cab_self_extract_invocation(file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: file.to_string(),
        args: vec!["/q".to_string(), format!("/x:{outdir}")],
        working_dir: outdir.to_string(),
        window: WindowMode::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C064: the shared `expand.exe` invocation
    /// matches both call sites' effective `expand.exe -F:* "<file>"
    /// "<destdir>"` shape.
    #[test]
    fn matches_source_invocation() {
        let inv = invocation(
            r"C:\Windows\System32\expand.exe",
            r"C:\downloads\update.cab",
            r"C:\downloads\update_unpacked",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\Windows\System32\expand.exe");
        assert_eq!(
            inv.args,
            vec![
                "-F:*".to_string(),
                r"C:\downloads\update.cab".to_string(),
                r"C:\downloads\update_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C064: the self-extracting Type-1 CAB
    /// invocation matches UniExtract.au3:2432-2433's effective `<file> /q
    /// /x:<outdir>` call.
    #[test]
    fn cab_self_extract_matches_source_invocation() {
        let inv = cab_self_extract_invocation(
            r"C:\downloads\selfextract.cab",
            r"C:\downloads\selfextract_unpacked",
        );
        assert_eq!(inv.program, r"C:\downloads\selfextract.cab");
        assert_eq!(
            inv.args,
            vec![
                "/q".to_string(),
                r"/x:C:\downloads\selfextract_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\selfextract_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }
}
