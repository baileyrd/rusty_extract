//! unadf (`unadf.exe`) — Amiga Disk Format (`.adf`) images, dispatched
//! entirely through the `def/*.ini` plugin path (see `extract::arc`'s
//! module doc comment for the full C047→C050→C052→C182 chain this
//! capability composes rather than duplicates).
//!
//! **Scope note, `%tempoutdir%`:** `def/adf.ini` sets `workingdir=%tempoutdir%`
//! alongside `runInTempOutdir=1`. `%tempoutdir%` is *not* one of
//! `extract::placeholder::replace_placeholders`'s five named substitutions
//! — in the source, `pluginExtract` replaces it with a separate, direct
//! `StringReplace($sWorkingDir, "%tempoutdir%", $tempoutdir)` call
//! (UniExtract.au3:3506), before running the result through
//! `ReplacePlaceholders` (line 3507). This test does the same two-step
//! substitution to match.
//!
//! See `extract::arc`'s module doc comment for why this verifies a raw
//! command-line string rather than a tokenized `Invocation`.

/// The bundled `def/adf.ini` plugin definition, verbatim.
pub const BUNDLED_INI: &str = include_str!("../../def/adf.ini");

#[cfg(test)]
mod tests {
    use super::BUNDLED_INI;
    use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
    use crate::extract::plugin_config::PluginConfig;
    use crate::extract::WindowMode;
    use crate::ini::IniFile;

    /// Parity test for capability C122: parsing the bundled `def/adf.ini`
    /// and substituting its `parameters`/`workingdir` for a representative
    /// extraction produces the exact command line and working directory
    /// `pluginExtract` would use.
    #[test]
    fn bundled_ini_produces_source_matching_command_line_and_workingdir() {
        let (ini, skipped) = IniFile::parse(BUNDLED_INI);
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "adf");

        assert_eq!(config.executable, "unadf.exe");
        assert!(config.run_in_temp_outdir);
        assert_eq!(config.window_mode(), WindowMode::Hidden);
        assert_eq!(config.workingdir.as_deref(), Some("%tempoutdir%"));

        let ctx = PlaceholderContext {
            file: r"C:\downloads\archive.adf",
            outdir: r"C:\downloads\archive_unpacked",
            filename: "archive",
            fileext: "adf",
            filedir: r"C:\downloads",
        };
        let tempoutdir = r"C:\downloads\archive_unpacked\tmp123456";

        let params = replace_placeholders(&format!(" {}", config.parameters), true, ctx, |k| {
            k.to_string()
        });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(command_line, r#"unadf.exe "C:\downloads\archive.adf""#);

        // %tempoutdir% first (a direct StringReplace in pluginExtract, not
        // part of ReplacePlaceholders), then the general substitution pass
        // — which is a no-op here since no other % remains.
        let workingdir_raw = config
            .workingdir
            .as_deref()
            .unwrap()
            .replace("%tempoutdir%", tempoutdir);
        let workingdir = replace_placeholders(&workingdir_raw, true, ctx, |k| k.to_string());
        assert_eq!(workingdir, tempoutdir);
    }
}
