//! rmvdec (`rmvdec.exe`) — RPG Maker MV encrypted resources
//! (`.rpgmvp`/`.rpgmvo`/`.rpgmvm`), dispatched entirely through the
//! `def/*.ini` plugin path (see `extract::arc`'s module doc comment for
//! the full C047→C050→C052→C182 chain this composes, and for why this
//! verifies a raw command-line string rather than a tokenized
//! `Invocation`).

/// The bundled `def/rpgmvp.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/rpgmvp.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C131: parsing the bundled
    /// `def/rpgmvp.ini` and substituting its `parameters` for a
    /// representative extraction produces the exact command line
    /// `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "rpgmvp");

        assert_eq!(config.executable, "rmvdec.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert!(config.pattern_search);
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\resource.rpgmvp",
            outdir: r"C:\downloads\resource_unpacked",
            filename: "resource",
            fileext: "rpgmvp",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"rmvdec.exe "C:\downloads\resource.rpgmvp" "C:\downloads\resource_unpacked""#
        );
    }
}
