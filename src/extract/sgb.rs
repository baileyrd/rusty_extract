//! sgbdec (`sgbdec.exe`) — Smile Game Builder `.sgbpack` archives,
//! dispatched entirely through the `def/*.ini` plugin path (see
//! `extract::arc`'s module doc comment for the full C047→C050→C052→C182
//! chain this composes, and for why this verifies a raw command-line
//! string rather than a tokenized `Invocation`).

/// The bundled `def/sgb.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/sgb.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C132: parsing the bundled `def/sgb.ini`
    /// and substituting its `parameters` for a representative extraction
    /// produces the exact command line `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "sgb");

        assert_eq!(config.executable, "sgbdec.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert!(config.pattern_search);
        assert!(config.initial_show);
        assert_eq!(config.require_net_framework.as_deref(), Some("4.5"));
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\game.sgbpack",
            outdir: r"C:\downloads\game_unpacked",
            filename: "game",
            fileext: "sgbpack",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"sgbdec.exe "C:\downloads\game.sgbpack" "C:\downloads\game_unpacked""#
        );
    }
}
