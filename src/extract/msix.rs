//! MsiX (`MsiX.exe`) — MSI fallback 2, MSM merge modules, MSP patches.

use super::{Invocation, WindowMode};

/// Builds the invocation shared by three dispatch cases that all shell
/// out to MsiX the same way: `<program> "<file>" /out "<outdir>"
/// [/ext]`, run in `filedir` with the window minimized (none of the
/// three call sites pass `_Run`'s optional show-flag argument, so its
/// `@SW_MINIMIZE` default applies).
///
/// - `$TYPE_MSI`'s "MsiX" fallback candidate (UniExtract.au3:2862-2864):
///   `append_ext` should be the resolved `appendext` preference (C022).
/// - `$TYPE_MSM` (MSM merge modules, UniExtract.au3:2887-2889): same —
///   `append_ext` should be the resolved `appendext` preference.
/// - `$TYPE_MSP`'s "MsiX" fallback candidate (UniExtract.au3:2907-2908):
///   `/ext` is a literal in the source, unconditional — pass `append_ext
///   = true` regardless of the `appendext` preference's actual value.
///
/// **Behavioral finding, flagged but not asserted as fact — a real
/// inconsistency worth a caller's attention.** Every other inline
/// ternary this source embeds inside a `&` concatenation chain is
/// wrapped in parentheses (e.g. `($sPassword == 0? '"': ...)` at
/// UniExtract.au3:2291, 2502, 3005; `($bDouble? '""': '"')` at 3599;
/// `($aPluginInfo[5] == ''? ' x': ' e')` at 7881) — a consistent idiom
/// across six other call sites. The `$TYPE_MSM` line
/// (UniExtract.au3:2889) is the *only* one missing those parentheses:
/// `... & '" ' & $appendext? '/ext': ''`. Depending on AutoIt's actual
/// `?:` precedence relative to `&` (not conclusively verified here),
/// this could mean `$appendext`'s value never actually gates `/ext` for
/// this one dispatch case the way the parenthesized form the rest of
/// the file uses does. This function models the *intended* behavior
/// (an `append_ext` boolean the caller resolves and passes through) —
/// whoever wires up `$TYPE_MSM`'s real dispatch should treat this as an
/// open question about what value to actually pass, not settled.
pub fn invocation(
    program: &str,
    file: &str,
    outdir: &str,
    filedir: &str,
    append_ext: bool,
) -> Invocation {
    let mut args = vec![file.to_string(), "/out".to_string(), outdir.to_string()];
    if append_ext {
        args.push("/ext".to_string());
    }
    Invocation {
        program: program.to_string(),
        args,
        working_dir: filedir.to_string(),
        window: WindowMode::Minimized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C086: `append_ext = false` omits the
    /// `/ext` argument.
    #[test]
    fn matches_source_invocation_without_ext() {
        let inv = invocation(
            r"C:\UniExtract\bin\MsiX.exe",
            r"C:\downloads\installer.msi",
            r"C:\downloads\installer",
            r"C:\downloads",
            false,
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\MsiX.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\installer.msi".to_string(),
                "/out".to_string(),
                r"C:\downloads\installer".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C086: `append_ext = true` appends
    /// `/ext`, matching `$TYPE_MSP`'s unconditional case
    /// (UniExtract.au3:2908) and an `appendext`-enabled `$TYPE_MSI`/
    /// `$TYPE_MSM` case.
    #[test]
    fn matches_source_invocation_with_ext() {
        let inv = invocation(
            r"C:\UniExtract\bin\MsiX.exe",
            r"C:\downloads\patch.msp",
            r"C:\downloads\patch",
            r"C:\downloads",
            true,
        );
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\patch.msp".to_string(),
                "/out".to_string(),
                r"C:\downloads\patch".to_string(),
                "/ext".to_string(),
            ]
        );
    }
}
