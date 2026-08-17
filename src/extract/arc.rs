//! ARC (`arc.exe`) — `.arc` ARC-format archives, dispatched entirely
//! through the `def/*.ini` plugin path: extension routing (C047) resolves
//! `.arc` to the `arc` type, `extract::dispatch` has no hardcoded
//! `Case $TYPE_...` for it (there is none in the source — `arc` is
//! `.ini`-only), so it falls through to the plugin engine
//! (`extract::plugin`, C050), whose `def/arc.ini` is parsed by
//! `extract::plugin_config` (C052) and substituted by
//! `extract::placeholder` (C182).
//!
//! This capability has no new production code of its own — it's proven by
//! composing those already-built primitives against the bundled
//! `def/arc.ini` asset and checking the resulting command line matches
//! what `pluginExtract` (UniExtract.au3:3468-3520) would build.
//!
//! **Scope note:** this verifies the *substituted command-line string*
//! (`$sBinary & " " & $sParameters`, matching `pluginExtract`'s own
//! construction before handing it to `_Run`), not a tokenized
//! `Invocation.args` the way every hardcoded `extract::*` module builds.
//! Turning that string into pre-split argument tokens needs a quote-aware
//! command-line tokenizer this port hasn't built yet for the plugin
//! path — a real gap, not something this capability's own one-line
//! manifest description asks it to solve.

/// The bundled `def/arc.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/arc.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C060: parsing the bundled `def/arc.ini`
    /// and substituting its `parameters` for a representative extraction
    /// produces the exact command line `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "arc");

        assert_eq!(config.executable, "arc.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.require_net_framework, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\archive.arc",
            outdir: r"C:\downloads\archive_unpacked",
            filename: "archive",
            fileext: "arc",
            filedir: r"C:\downloads",
        };

        // pluginExtract prefixes parameters with a leading space:
        // `Local $sParameters = " " & _ArrayGet(...)`.
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(command_line, r#"arc.exe x "C:\downloads\archive.arc""#);

        // workingdir is absent, so pluginExtract falls back to $outdir
        // (`If Not $sWorkingDir Or $sWorkingDir = "" Then $sWorkingDir =
        // $outdir`).
        assert_eq!(config.workingdir, None);
    }
}
