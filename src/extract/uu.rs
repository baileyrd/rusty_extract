//! UUDeview (`uudeview.exe`) — UUencode/yEnc encoded files, dispatched
//! entirely through the `def/*.ini` plugin path (see `extract::arc`'s
//! module doc comment for the full C047→C050→C052→C182 chain this
//! composes, and for why this verifies a raw command-line string rather
//! than a tokenized `Invocation`).
//!
//! `def/uu.ini` sets `workingdir=%filedir%` — unlike the majority of
//! `def/*.ini` files (which omit `workingdir`, falling back to `outdir`),
//! this one sets it explicitly to the input file's own directory.
//! `%filedir%` is one of `extract::placeholder::replace_placeholders`'s
//! five named substitutions (unlike `def/adf.ini`'s `%tempoutdir%`), so no
//! special two-step handling is needed here — a single
//! `replace_placeholders` call resolves it.

/// The bundled `def/uu.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/uu.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C137: parsing the bundled `def/uu.ini`
    /// and substituting its `parameters`/`workingdir` for a representative
    /// extraction produces the exact command line and working directory
    /// `pluginExtract` would use.
    #[test]
    fn bundled_ini_produces_source_matching_command_line_and_workingdir() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "uu");

        assert_eq!(config.executable, "uudeview.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.workingdir.as_deref(), Some("%filedir%"));

        let ctx = PlaceholderContext {
            file: r"C:\downloads\encoded.uu",
            outdir: r"C:\downloads\encoded_unpacked",
            filename: "encoded",
            fileext: "uu",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"uudeview.exe -p "C:\downloads\encoded_unpacked" -i "C:\downloads\encoded.uu""#
        );

        // %filedir% is one of the five named substitutions, so a single
        // replace_placeholders call (no manual %tempoutdir%-style
        // pre-step) resolves the working directory.
        let workingdir =
            replace_placeholders(config.workingdir.as_deref().unwrap(), true, ctx, |k| {
                k.to_string()
            });
        assert_eq!(workingdir, r"C:\downloads");
    }
}
