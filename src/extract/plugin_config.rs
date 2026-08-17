//! `def/*.ini` plugin definition schema — the `[Plugin]` section format
//! shared by every `def/*.ini` file `extract::plugin::resolve_plugin_ini`
//! can resolve.
//!
//! Ports the schema half of `pluginExtract` (UniExtract.au3:3468-3520):
//! reading the `[Plugin]` section's recognized keys with their exact
//! defaults and type coercions (mirroring every `_ArrayGet(...)` call at
//! lines 3480-3501, plus the standalone `cleanup` read at line 3515).
//! Verified against all 18 non-`registry.ini` files under `def/` — the
//! `[Plugin]` keys actually in use are exactly `display`, `executable`,
//! `hide`, `initialShow`, `log`, `parameters`, `patternSearch`,
//! `requireNetFramework`, `runInTempOutdir`, `useCmd`, `workingdir`; none of
//! the bundled files set `cleanup`, but the source still reads it (default:
//! none), so this schema does too.
//!
//! This module answers "what does this plugin definition say", not "what
//! `Invocation` does it produce": turning a [`PluginConfig`] into a real
//! `Invocation` needs its `parameters`/`workingdir` strings run through
//! `%placeholder%` substitution first (`ReplacePlaceholders`,
//! UniExtract.au3:3523-3541 — capability C182, not yet ported), plus the
//! resolved executable path from `extract::plugin::resolve_plugin_ini`
//! (C050). Wiring parsed config + substituted placeholders into an
//! `Invocation` is integration work for once both exist, not this row.

use super::WindowMode;
use crate::ini::IniFile;

/// A parsed `[Plugin]` section, field-for-field matching `pluginExtract`'s
/// own `_ArrayGet` reads (UniExtract.au3:3480-3501,3515).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    /// UI label for this plugin. Defaults to the plugin stem (the `.ini`
    /// filename without extension) when absent — `_ArrayGet(..., "display",
    /// $sPlugin)` at line 3489.
    pub display: String,
    /// The helper binary's filename. Defaults to the plugin stem when
    /// absent — `_ArrayGet(..., "executable", $sPlugin)` at line 3480.
    pub executable: String,
    /// Raw, unsubstituted command-line parameters (may contain
    /// `%file%`/`%outdir%`/etc. placeholders — see C182). Defaults to an
    /// empty string when absent.
    pub parameters: String,
    /// Raw, unsubstituted working directory. `None` when the key is absent
    /// *or* present-but-empty — both cases mean "fall back to `outdir`"
    /// per `If Not $sWorkingDir Or $sWorkingDir = "" Then $sWorkingDir =
    /// $outdir` (line 3503); this type collapses that two-way check into
    /// one `Option`, since both inputs mean the same thing to every caller.
    pub workingdir: Option<String>,
    /// Whether to stage output in a temp directory and move it into place
    /// afterward (`_RunInTempOutdir` vs plain `_Run`) — defaults to `false`
    /// when absent.
    pub run_in_temp_outdir: bool,
    /// Whether the helper binary's window should be hidden rather than
    /// minimized — defaults to `false` when absent. See [`Self::window_mode`].
    pub hide: bool,
    /// Whether to route the invocation through `cmd.exe` — defaults to
    /// `true` when absent.
    pub use_cmd: bool,
    /// Whether to tee the helper binary's output to the log — defaults to
    /// `true` when absent.
    pub log: bool,
    /// Whether the caller should pattern-search the helper binary's output
    /// for known result markers — defaults to `false` when absent.
    pub pattern_search: bool,
    /// Whether the helper binary's window starts visible — defaults to
    /// `true` when absent.
    pub initial_show: bool,
    /// Minimum required .NET Framework version, if any. `None` when the key
    /// is absent or its value isn't a positive number — matching
    /// `If $ret > 0 Then HasNetFramework($ret)` (line 3485), which only
    /// acts on a positive value.
    pub require_net_framework: Option<String>,
    /// Post-extraction cleanup glob patterns, `|`-separated in the source
    /// (`IniRead(..., "cleanup", 0)` then `StringSplit($sCleanup, "|", 2)`
    /// at lines 3515-3519). Empty when the key is absent — none of the
    /// bundled `def/*.ini` files set it today.
    pub cleanup: Vec<String>,
}

impl PluginConfig {
    /// Parses `ini`'s `[Plugin]` section, using `plugin_stem` (the `.ini`
    /// filename without extension) as the fallback for `display` and
    /// `executable` per the source's own `$sPlugin` fallback. A missing
    /// `[Plugin]` section parses as if every key were absent — matching
    /// `IniReadSection` returning an empty result the source would apply
    /// the same per-key defaults to, not an error.
    pub fn parse(ini: &IniFile, plugin_stem: &str) -> Self {
        let entries = ini.section("Plugin").unwrap_or(&[]);
        let get = |key: &str| -> Option<&str> {
            entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let get_bool =
            |key: &str, default: bool| -> bool { get(key).map(|v| v == "1").unwrap_or(default) };

        PluginConfig {
            display: get("display").unwrap_or(plugin_stem).to_string(),
            executable: get("executable").unwrap_or(plugin_stem).to_string(),
            parameters: get("parameters").unwrap_or("").to_string(),
            workingdir: get("workingdir")
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            run_in_temp_outdir: get_bool("runInTempOutdir", false),
            hide: get_bool("hide", false),
            use_cmd: get_bool("useCmd", true),
            log: get_bool("log", true),
            pattern_search: get_bool("patternSearch", false),
            initial_show: get_bool("initialShow", true),
            require_net_framework: get("requireNetFramework")
                .filter(|v| v.parse::<f64>().is_ok_and(|n| n > 0.0))
                .map(str::to_string),
            cleanup: get("cleanup")
                .map(|s| s.split('|').map(str::to_string).collect())
                .unwrap_or_default(),
        }
    }

    /// The window mode this config selects — `_ArrayGet(..., "hide", 0,
    /// True) == 1? @SW_HIDE: @SW_MINIMIZE` (line 3497). Plugin-driven
    /// extraction never shows a window normally (`@SW_SHOW`); only hidden
    /// or minimized are reachable from this schema.
    pub fn window_mode(&self) -> WindowMode {
        if self.hide {
            WindowMode::Hidden
        } else {
            WindowMode::Minimized
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C052: every key present in a real bundled
    /// `def/*.ini` file (`def/bsa.ini`) parses to the value the source's
    /// `_ArrayGet` calls would read.
    #[test]
    fn parses_every_key_from_a_bundled_plugin_file() {
        let (ini, skipped) = IniFile::parse(
            "[Plugin]\n\
             display=Bethesda %TERM_ARCHIVE%\n\
             useCmd=1\n\
             executable=bsab.exe\n\
             parameters=/e %file% %outdir%\n\
             hide=1\n\
             log=1\n\
             patternSearch=1\n\
             initialShow=0\n\
             requireNetFramework=4\n",
        );
        assert!(skipped.is_empty());
        let config = PluginConfig::parse(&ini, "bsa");

        assert_eq!(config.display, "Bethesda %TERM_ARCHIVE%");
        assert_eq!(config.executable, "bsab.exe");
        assert_eq!(config.parameters, "/e %file% %outdir%");
        assert_eq!(config.workingdir, None);
        assert!(!config.run_in_temp_outdir);
        assert!(config.hide);
        assert!(config.use_cmd);
        assert!(config.log);
        assert!(config.pattern_search);
        assert!(!config.initial_show);
        assert_eq!(config.require_net_framework.as_deref(), Some("4"));
        assert!(config.cleanup.is_empty());
        assert_eq!(config.window_mode(), WindowMode::Hidden);
    }

    /// A missing `[Plugin]` section, or a section missing every key, uses
    /// exactly the defaults `_ArrayGet` would apply — `display`/`executable`
    /// fall back to the plugin stem, booleans fall back per-key
    /// (`useCmd`/`log`/`initialShow` default true, the rest default false),
    /// `workingdir`/`requireNetFramework`/`cleanup` default to "unset".
    #[test]
    fn missing_keys_use_array_get_defaults() {
        let (ini, _) = IniFile::parse("[Plugin]\n");
        let config = PluginConfig::parse(&ini, "myplugin");

        assert_eq!(config.display, "myplugin");
        assert_eq!(config.executable, "myplugin");
        assert_eq!(config.parameters, "");
        assert_eq!(config.workingdir, None);
        assert!(!config.run_in_temp_outdir);
        assert!(!config.hide);
        assert!(config.use_cmd);
        assert!(config.log);
        assert!(!config.pattern_search);
        assert!(config.initial_show);
        assert_eq!(config.require_net_framework, None);
        assert!(config.cleanup.is_empty());
        assert_eq!(config.window_mode(), WindowMode::Minimized);
    }

    /// A missing `[Plugin]` section entirely (not just an empty one) parses
    /// the same as a present-but-empty section — no special-case error.
    #[test]
    fn missing_plugin_section_parses_as_all_defaults() {
        let (ini, _) = IniFile::parse("[SomethingElse]\nkey=value\n");
        let config = PluginConfig::parse(&ini, "fallback-name");
        assert_eq!(config.display, "fallback-name");
        assert_eq!(config.executable, "fallback-name");
    }

    /// `workingdir=` present but empty means the same as absent — "fall
    /// back to outdir" — matching `If Not $sWorkingDir Or $sWorkingDir = ""`
    /// (line 3503) treating both cases identically.
    #[test]
    fn empty_workingdir_value_is_treated_as_absent() {
        let (ini, _) = IniFile::parse("[Plugin]\nworkingdir=\n");
        let config = PluginConfig::parse(&ini, "stem");
        assert_eq!(config.workingdir, None);
    }

    /// `workingdir=%tempoutdir%` (used by e.g. `def/adf.ini`) is preserved
    /// verbatim — placeholder substitution is a separate capability (C182),
    /// not this one's job.
    #[test]
    fn nonempty_workingdir_is_preserved_unsubstituted() {
        let (ini, _) = IniFile::parse("[Plugin]\nworkingdir=%tempoutdir%\n");
        let config = PluginConfig::parse(&ini, "stem");
        assert_eq!(config.workingdir.as_deref(), Some("%tempoutdir%"));
    }

    /// `requireNetFramework=0` (or any non-positive value) is `None`,
    /// matching `If $ret > 0 Then HasNetFramework($ret)` only acting on a
    /// positive value — zero means "no requirement", not "requires version
    /// 0".
    #[test]
    fn require_net_framework_zero_is_none() {
        let (ini, _) = IniFile::parse("[Plugin]\nrequireNetFramework=0\n");
        let config = PluginConfig::parse(&ini, "stem");
        assert_eq!(config.require_net_framework, None);
    }

    /// `cleanup=*.tmp|*.log` splits on `|`, matching `StringSplit($sCleanup,
    /// "|", 2)` (line 3518).
    #[test]
    fn cleanup_splits_on_pipe() {
        let (ini, _) = IniFile::parse("[Plugin]\ncleanup=*.tmp|*.log\n");
        let config = PluginConfig::parse(&ini, "stem");
        assert_eq!(
            config.cleanup,
            vec!["*.tmp".to_string(), "*.log".to_string()]
        );
    }
}
