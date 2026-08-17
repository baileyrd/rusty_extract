//! utagedec (`utagedec.exe`) — UTAGE visual novel engine files, dispatched
//! entirely through the `def/*.ini` plugin path (see `extract::arc`'s
//! module doc comment for the full C047→C050→C052→C182 chain this
//! composes, and for why this verifies a raw command-line string rather
//! than a tokenized `Invocation`).

/// The bundled `def/utage.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/utage.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C136: parsing the bundled
    /// `def/utage.ini` and substituting its `parameters` for a
    /// representative extraction produces the exact command line
    /// `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "utage");

        assert_eq!(config.executable, "utagedec.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert!(config.pattern_search);
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\game.dat",
            outdir: r"C:\downloads\game_unpacked",
            filename: "game",
            fileext: "dat",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"utagedec.exe "C:\downloads\game.dat" "C:\downloads\game_unpacked""#
        );
    }
}
