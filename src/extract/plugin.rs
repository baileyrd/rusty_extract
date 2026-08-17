//! Case-Else → `def/*.ini` plugin-engine fallback dispatch: resolving which
//! `.ini` file to load for an extractor-type key `extract::dispatch`
//! doesn't have a hardcoded case for.
//!
//! Ports the file-resolution half of `pluginExtract`
//! (UniExtract.au3:3468-3476) — checking a user-override directory first,
//! then the bundled directory. Parsing the resolved file's `[Plugin]`
//! section (C052, `extract::plugin_config`) and substituting its
//! `%placeholder%` values (C182) are separate capabilities; this module
//! only answers "which file, if any."

use std::path::{Path, PathBuf};

/// Where `resolve_plugin_ini` found (or didn't find) a plugin definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginResolution {
    /// A plugin `.ini` file exists at this path.
    Found(PathBuf),
    /// Neither the user-override nor the bundled directory has this file.
    /// `attempted` is the bundled-directory path — matching the source,
    /// which by this point has overwritten its own path variable with the
    /// bundled location and reports only that one in
    /// `terminate($STATUS_MISSINGDEF, $sPluginFile, ...)`, not the
    /// user-override path it tried first.
    Missing { attempted: PathBuf },
}

/// Resolves `plugin_stem` (e.g. `"rpa"`, the extractor-type key that
/// [`crate::extract::dispatch::dispatch`] reported as [`Plugin`]) to a
/// plugin `.ini` file path, matching `pluginExtract`'s exact order: the
/// user-override directory first, then the bundled directory. This is the
/// version to call in real use — [`resolve_plugin_ini_with`] takes the
/// existence check as a parameter for testing without real file I/O.
///
/// [`Plugin`]: crate::extract::dispatch::DispatchTarget::Plugin
pub fn resolve_plugin_ini(
    plugin_stem: &str,
    user_def_dir: &Path,
    def_dir: &Path,
) -> PluginResolution {
    resolve_plugin_ini_with(plugin_stem, user_def_dir, def_dir, |p| p.exists())
}

/// Same resolution as [`resolve_plugin_ini`], but `exists` replaces the
/// real filesystem check — the seam that keeps this testable without
/// creating real files (see `ARCHITECTURE.md` on keeping domain logic free
/// of I/O).
pub fn resolve_plugin_ini_with(
    plugin_stem: &str,
    user_def_dir: &Path,
    def_dir: &Path,
    exists: impl Fn(&Path) -> bool,
) -> PluginResolution {
    let user_path = user_def_dir.join(format!("{plugin_stem}.ini"));
    if exists(&user_path) {
        return PluginResolution::Found(user_path);
    }

    let bundled_path = def_dir.join(format!("{plugin_stem}.ini"));
    if exists(&bundled_path) {
        return PluginResolution::Found(bundled_path);
    }

    PluginResolution::Missing {
        attempted: bundled_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C050: resolution order matches
    /// `pluginExtract` (UniExtract.au3:3471-3475) exactly — user-override
    /// directory checked first, then the bundled directory.
    #[test]
    fn prefers_user_override_over_bundled() {
        let resolution = resolve_plugin_ini_with(
            "rpa",
            Path::new("/user/def"),
            Path::new("/bundled/def"),
            |p| p == Path::new("/user/def/rpa.ini") || p == Path::new("/bundled/def/rpa.ini"),
        );
        assert_eq!(
            resolution,
            PluginResolution::Found(PathBuf::from("/user/def/rpa.ini"))
        );
    }

    #[test]
    fn falls_back_to_bundled_when_user_override_absent() {
        let resolution = resolve_plugin_ini_with(
            "rpa",
            Path::new("/user/def"),
            Path::new("/bundled/def"),
            |p| p == Path::new("/bundled/def/rpa.ini"),
        );
        assert_eq!(
            resolution,
            PluginResolution::Found(PathBuf::from("/bundled/def/rpa.ini"))
        );
    }

    #[test]
    fn missing_reports_only_the_bundled_path_matching_source_error() {
        let resolution = resolve_plugin_ini_with(
            "nonexistent",
            Path::new("/user/def"),
            Path::new("/bundled/def"),
            |_| false,
        );
        assert_eq!(
            resolution,
            PluginResolution::Missing {
                attempted: PathBuf::from("/bundled/def/nonexistent.ini")
            }
        );
    }

    #[test]
    fn resolve_plugin_ini_uses_real_filesystem_by_default() {
        // Neither directory exists in the test environment, so this
        // exercises the real std::path::Path::exists() code path and
        // should still report Missing rather than panicking.
        let resolution = resolve_plugin_ini(
            "definitely-not-a-real-plugin",
            Path::new("/definitely/not/real/user/def"),
            Path::new("/definitely/not/real/bundled/def"),
        );
        assert_eq!(
            resolution,
            PluginResolution::Missing {
                attempted: PathBuf::from(
                    "/definitely/not/real/bundled/def/definitely-not-a-real-plugin.ini"
                )
            }
        );
    }
}
