//! unalz (`unalz.exe`) — ALZip `.alz` archives, dispatched through the
//! `def/*.ini` plugin path once `detection::alz_probe::is_alz_archive`
//! (the other half of capability C059) has identified the file — there's
//! no hardcoded `Case $TYPE_ALZ` in the source's main dispatch `Switch`
//! (see `extract::arc`'s module doc comment for the full
//! C047→C050→C052→C182 chain this composes, and for why this verifies a
//! raw command-line string rather than a tokenized `Invocation`).

/// The bundled `def/alz.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/alz.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C059: parsing the bundled `def/alz.ini`
    /// and substituting its `parameters` for a representative extraction
    /// produces the exact command line `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "alz");

        assert_eq!(config.executable, "unalz.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\archive.alz",
            outdir: r"C:\downloads\archive_unpacked",
            filename: "archive",
            fileext: "alz",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"unalz.exe -d "C:\downloads\archive_unpacked" "C:\downloads\archive.alz""#
        );
    }
}
