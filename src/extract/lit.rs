//! ConvertLIT (`clit.exe`) — Microsoft Reader `.lit` eBooks, dispatched
//! entirely through the `def/*.ini` plugin path (see `extract::arc`'s
//! module doc comment for the full C047→C050→C052→C182 chain this
//! composes, and for why this verifies a raw command-line string rather
//! than a tokenized `Invocation`).
//!
//! **Known quirk, preserved:** `todo.txt:29` documents an output-path-
//! with-spaces bug for this extractor. Nothing in this capability's own
//! scope (parsing `def/lit.ini` and substituting its placeholders) adds
//! any path-escaping or quoting beyond what `extract::placeholder`
//! already does uniformly for every plugin — so the quirk is preserved by
//! this row simply not inventing a special case for `clit.exe`, not by any
//! code addition.

/// The bundled `def/lit.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/lit.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C127: parsing the bundled `def/lit.ini`
    /// and substituting its `parameters` for a representative extraction
    /// produces the exact command line `pluginExtract` would run.
    #[test]
    fn bundled_ini_produces_source_matching_command_line() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "lit");

        assert_eq!(config.executable, "clit.exe");
        assert!(!config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.workingdir, None);

        let ctx = PlaceholderContext {
            file: r"C:\downloads\book.lit",
            outdir: r"C:\downloads\book_unpacked",
            filename: "book",
            fileext: "lit",
            filedir: r"C:\downloads",
        };
        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line,
            r#"clit.exe "C:\downloads\book.lit" "C:\downloads\book_unpacked""#
        );
    }
}
