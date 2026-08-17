//! godotdec (`godotdec.exe`) — Godot Engine packages (`.pck`, embedded in
//! `.exe`), dispatched entirely through the `def/*.ini` plugin path (see
//! `extract::arc`'s module doc comment for the full C047→C050→C052→C182
//! chain this capability composes rather than duplicates, and for why this
//! verifies a raw command-line string rather than a tokenized
//! `Invocation`).

/// The bundled `def/godot.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/godot.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C125: parsing the bundled
    /// `def/godot.ini` and substituting its `parameters` for a
    /// representative extraction produces the exact command line
    /// `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "godot");

        assert_eq!(config.executable, "godotdec.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.require_net_framework.as_deref(), Some("4.5"));
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\game.pck",
            outdir: r"C:\downloads\game_unpacked",
            filename: "game",
            fileext: "pck",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"godotdec.exe -c "C:\downloads\game.pck" "C:\downloads\game_unpacked""#
        );
    }
}
