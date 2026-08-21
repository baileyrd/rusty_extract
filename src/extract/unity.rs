//! Unity `.unitypackage` decoder (wraps `7z.exe` + custom
//! path-remapping logic).
//!
//! ```autoit
//! Case $TYPE_UNITYPACKAGE
//!     ; Unitypackages are tar.gz files with a specific internal structure. First, extract them normally.
//!     Local $oldoutdir = $outdir
//!     $outdir = $tempoutdir
//!
//!     extract($TYPE_7Z, -1, "gz", True, False)
//!
//!     ; Newer files contain 'archtemp.tar', old version are standard tar.gz archives
//!     Local $sFile = $tempoutdir & "archtemp.tar"
//!     If FileExists($sFile) Then
//!         _Run($7z & ' x "' & $sFile & '"', $tempoutdir)
//!         FileDelete($sFile)
//!     EndIf
//!
//!     $outdir = $oldoutdir
//!     ; ... per-asset rename/restructure loop, see resolve_asset_destination/
//!     ; is_destination_within_outdir ...
//! ```
//!
//! The primary extraction is a recursive `extract($TYPE_7Z, -1, "gz",
//! True, False)` call (UniExtract.au3:3173) — `return_success = true,
//! return_fail = false`, the same shape `extract::forge`'s and
//! `extract::raiu`'s recursive calls use. `$outdir` is redirected to
//! `$tempoutdir` first (the same dance `extract::forge` documents for
//! its own recursive call) so the primary extraction lands in the
//! scratch directory rather than the real output, and is restored
//! afterward. Per `extract::completion` (C054/C181), a `return_fail =
//! false` call still terminates the whole process on failure — so,
//! like `extract::forge`/`extract::raiu`, everything after this call
//! (the conditional inner-tar unpack, the `$outdir` restore, the
//! per-asset rename loop) is **not** unconditional: a failed primary
//! extraction terminates right there. This call site's own return
//! value (`1` on success) is otherwise unused, the same as
//! `extract::forge`'s/`extract::raiu`'s.

use super::{Invocation, WindowMode};
use crate::prefs::resolve_relative_path;

/// Builds the conditional inner-tar extraction invocation UniExtract2's
/// `Case $TYPE_UNITYPACKAGE` (UniExtract.au3:3177) makes when the initial
/// `.gz` extraction produced an `archtemp.tar` (newer-format packages):
/// `<program> x "<tar_file>"`, run in `tempoutdir` with the window
/// minimized (`_Run`'s own default for the omitted `$show_flag`
/// argument).
///
/// **Not modeled here:** the recursive `extract($TYPE_7Z, -1, "gz", True,
/// False)` dispatch that runs first (see module doc comment); the
/// `FileExists`/`FileDelete` staging around this call; the per-asset
/// rename/restructure loop that follows (see
/// [`resolve_asset_destination`]/[`is_destination_within_outdir`] for
/// that half of this capability). All separate runtime behavior, not
/// part of building this one invocation.
pub fn inner_tar_invocation(program: &str, tar_file: &str, tempoutdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["x".to_string(), tar_file.to_string()],
        working_dir: tempoutdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// Ports the destination-path computation in the per-asset rename loop
/// (UniExtract.au3:3196): `_PathFull($sName, $outdir)`, where `$sName` is
/// the relative asset path read from the unitypackage's `pathname` file.
/// Approximated via [`resolve_relative_path`] — `_PathFull` isn't defined
/// anywhere in this port's source checkout, the same already-documented
/// gap behind `prefs::resolve_batchqueue_path`/
/// `prefs::resolve_filescanlogfile_path` (C018/C019).
pub fn resolve_asset_destination(pathname: &str, outdir: &str) -> String {
    resolve_relative_path(pathname, outdir)
}

/// Ports the safety check the rename loop makes before moving an asset
/// into place (UniExtract.au3:3197): `StringInStr($sDestination,
/// $outdir)` — bare, so it defaults case-insensitive, the same AutoIt
/// default already documented for every other bare `StringInStr` call
/// this port has encountered (C007-C013, C144, C145, C147, C061).
///
/// **Preserved quirk, not hardened.** This is a substring containment
/// check, not a proper path-prefix validation: `outdir` merely needs to
/// appear *somewhere* in `destination`, which a crafted relative
/// `pathname` could satisfy without the resolved path actually staying
/// under `outdir` (e.g. a pathname built so the resulting string reads
/// `..\..\evil\<outdir>\file`). This is exactly the class of directory-
/// escape risk `ExtractionTransaction` (ADR-0119) exists to close
/// properly once built; this function reproduces the source's weaker
/// check as written, not a fixed version of it.
pub fn is_destination_within_outdir(destination: &str, outdir: &str) -> bool {
    destination.to_lowercase().contains(&outdir.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::WindowMode;

    /// Parity test for capability C121: the inner-tar invocation matches
    /// UniExtract.au3:3177's effective `7z.exe x "<tar_file>"` call.
    #[test]
    fn inner_tar_invocation_matches_source() {
        let inv = inner_tar_invocation(
            r"C:\UniExtract\bin\7z.exe",
            r"C:\Temp\unity_tmp\archtemp.tar",
            r"C:\Temp\unity_tmp",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\Temp\unity_tmp\archtemp.tar".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\Temp\unity_tmp");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C121: a relative asset pathname resolves
    /// against `outdir`.
    #[test]
    fn resolve_asset_destination_resolves_relative_pathname() {
        assert_eq!(
            resolve_asset_destination(r"Assets\Scripts\Foo.cs", r"C:\downloads\unpacked"),
            r"C:\downloads\unpacked\Assets\Scripts\Foo.cs"
        );
    }

    /// Parity test for capability C121: a destination containing `outdir`
    /// as a substring passes the check.
    #[test]
    fn is_destination_within_outdir_accepts_contained_path() {
        assert!(is_destination_within_outdir(
            r"C:\downloads\unpacked\Assets\Foo.cs",
            r"C:\downloads\unpacked"
        ));
    }

    /// Parity test for capability C121: the check is case-insensitive,
    /// matching AutoIt's `StringInStr` default.
    #[test]
    fn is_destination_within_outdir_is_case_insensitive() {
        assert!(is_destination_within_outdir(
            r"C:\DOWNLOADS\UNPACKED\Assets\Foo.cs",
            r"C:\downloads\unpacked"
        ));
    }

    /// Parity test for capability C121: a destination that doesn't
    /// contain `outdir` at all is rejected.
    #[test]
    fn is_destination_within_outdir_rejects_unrelated_path() {
        assert!(!is_destination_within_outdir(
            r"C:\elsewhere\Assets\Foo.cs",
            r"C:\downloads\unpacked"
        ));
    }
}
