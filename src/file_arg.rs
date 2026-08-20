//! Positional file-argument resolution: ports the file-argument half of
//! `ParseCommandLine()` (UniExtract.au3:625-628) — `$file =
//! _PathFull($cmdline[1])`, then `If Not FileExists($file) Then
//! terminate($STATUS_INVALIDFILE, $file)`.
//!
//! **Not modeled — `_PathFull`'s own segment normalization.** Mirrors the
//! same documented gap already noted for `outdir::resolve_output_directory`
//! (C139) and `prefs::resolve_batchqueue_path`/`resolve_filescanlogfile_path`
//! (C018/C019): `_PathFull` isn't defined anywhere in this port's source
//! checkout, so [`resolve_file_argument_path`]'s relative-path branch does
//! a plain `cwd` join rather than reproducing whatever `.`/`..`-collapsing
//! AutoIt's single-argument `_PathFull` performs internally.

use crate::status::Status;

fn is_drive_absolute(path: &str) -> bool {
    path.as_bytes().get(1) == Some(&b':')
}

fn is_unc(path: &str) -> bool {
    path.starts_with(r"\\")
}

/// C001: resolves `arg` (`$cmdline[1]`) to a full path — drive-absolute
/// (`X:...`) or UNC (`\\...`) paths pass through unchanged; anything else
/// is joined onto `cwd`, mirroring `_PathFull`'s common-case behavior (see
/// the module doc comment for the one gap this doesn't reproduce).
pub fn resolve_file_argument_path(arg: &str, cwd: &str) -> String {
    if is_drive_absolute(arg) || is_unc(arg) {
        arg.to_string()
    } else {
        format!("{cwd}\\{arg}")
    }
}

/// C001: ports `FileExists($file)`'s validation branch
/// (UniExtract.au3:628) — `exists` is caller-supplied so this stays a pure
/// decision rather than real I/O, the same seam
/// `plugin::resolve_plugin_ini_with` uses for its own existence check. The
/// source's `terminate($STATUS_INVALIDFILE, $file)` call also logs `$file`
/// as a message argument; that's a `terminate`-side concern (real I/O),
/// not part of this decision.
pub fn validate_file_argument(exists: bool) -> Result<(), Status> {
    if exists {
        Ok(())
    } else {
        Err(Status::InvalidFile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C001: drive-absolute and UNC paths pass
    /// through `_PathFull` unchanged; anything else resolves against `cwd`.
    #[test]
    fn resolve_file_argument_path_matches_source_branches() {
        assert_eq!(
            resolve_file_argument_path(r"C:\downloads\archive.zip", r"C:\somewhere"),
            r"C:\downloads\archive.zip"
        );
        assert_eq!(
            resolve_file_argument_path(r"\\server\share\archive.zip", r"C:\somewhere"),
            r"\\server\share\archive.zip"
        );
        assert_eq!(
            resolve_file_argument_path("archive.zip", r"C:\downloads"),
            r"C:\downloads\archive.zip"
        );
    }

    /// Parity test for capability C001: an existing file validates; a
    /// missing one maps to `Status::InvalidFile` (exit code 5, per
    /// `status::tests::fixed_exit_codes_match_source`).
    #[test]
    fn validate_file_argument_matches_source_branches() {
        assert_eq!(validate_file_argument(true), Ok(()));
        assert_eq!(validate_file_argument(false), Err(Status::InvalidFile));
    }
}
