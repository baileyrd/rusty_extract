//! GNU gettext (`msgunfmt.exe`) — compiled GNU Gettext `.mo` message
//! catalogs, dispatched entirely through the `def/*.ini` plugin path (see
//! `extract::arc`'s module doc comment for the full C047→C050→C052→C182
//! chain this composes, and for why this verifies a raw command-line
//! string rather than a tokenized `Invocation`).
//!
//! `def/mo.ini`'s `parameters` value concatenates `%outdir%\%filename%.po`
//! with no space between the quoted `%outdir%` substitution and the
//! literal `\` — preserved exactly as the source builds it, not split into
//! separate tokens.

/// The bundled `def/mo.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/mo.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C128: parsing the bundled `def/mo.ini`
    /// and substituting its `parameters` for a representative extraction
    /// produces the exact command line `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "mo");

        assert_eq!(config.executable, "msgunfmt.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\catalog.mo",
            outdir: r"C:\downloads\catalog_unpacked",
            filename: "catalog",
            fileext: "mo",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"msgunfmt.exe "C:\downloads\catalog.mo" -o "C:\downloads\catalog_unpacked"\catalog.po"#
        );
    }
}
