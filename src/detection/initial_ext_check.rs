//! Extension-based pre-check (`InitialCheckExt`, UniExtract.au3:2193-2215)
//! — capability C046. Runs before any signature scan, for extensions
//! whose file magic is unreliable or ambiguous on its own: split-archive
//! first parts, compound tar variants that need multi-step handling, and
//! disk images where signature detection isn't trustworthy.
//!
//! ```autoit
//! Func InitialCheckExt()
//!     If Not $extract Then Return
//!
//!     Switch $fileext
//!         ; Split files have no additional file magic and will be misdetected
//!         Case "001"
//!             If FileExists($filedir & "\" & $filename & ".002") Then check7z()
//!         ; Compound compressed files that require multiple actions
//!         Case "ipk", "tbz2", "tgz", "tz", "tlz", "txz"
//!             extract($TYPE_CTAR, 'Compressed Tar ' & t('TERM_ARCHIVE'))
//!         ; Disk images - file type identification is not always reliable
//!         Case "bin", "cdi", "mdf"
//!             CheckIso()
//!             check7z(t('TERM_DISK_IMAGE'), True)
//!         Case "dmg"
//!             extract($TYPE_7Z, 'DMG ' & t('TERM_IMAGE'))
//!         Case "cue", "gdi", "iso", "mds"
//!             check7z(t('TERM_DISK_IMAGE'), True)
//!             CheckIso()
//!         Case "unitypackage"
//!             extract($TYPE_UNITYPACKAGE, "Unity Engine Asset Package")
//!     EndSwitch
//! EndFunc
//! ```
//!
//! **`If Not $extract Then Return`** is the same scan-only-mode gate
//! already ported as `entry_gate::scan_only_gate` (C152) — not
//! duplicated here; a caller checks that first and only reaches
//! [`route`] when it's `false`.
//!
//! **Every routing target already has a home in this port**, so this
//! capability is purely the *order and grouping* `Switch $fileext`
//! decides, not new extraction logic: `check7z()` is the blind 7-Zip
//! probe (`detection::sevenzip_probe`, C048, `DONE`); `CheckIso()` is
//! `extract::qbms`'s ISO detector (C077, partial but this call shape is
//! covered); `extract($TYPE_CTAR/$TYPE_7Z/$TYPE_UNITYPACKAGE, ...)` are
//! `extract::ctar`/`extract::sevenzip`/`extract::unity` (C181, C056,
//! C121/C054). [`$fileext`'s own lowercasing](https://github.com) already
//! happens upstream of this function (documented in
//! `detection::sevenzip_probe`'s own doc comment, citing C175/C176), so
//! `route`'s `fileext` parameter is assumed already-lowercased — no
//! case-folding is done here.
//!
//! **A real, easy-to-miss ordering difference between two disk-image
//! groups.** `{bin, cdi, mdf}` calls `CheckIso()` *then* `check7z(...)`;
//! `{cue, gdi, iso, mds}` calls `check7z(...)` *then* `CheckIso()` — the
//! reverse order. [`Routing`] models these as two distinct variants
//! rather than one shared "disk image" outcome, so the order can't be
//! silently lost.

/// What `InitialCheckExt` decides for a given (already-lowercased)
/// `$fileext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Routing {
    /// `"001"`: check for a `.002` sibling (real filesystem I/O, left to
    /// the caller — see [`split_file_sibling_path`]) and only then
    /// invoke the blind 7-Zip probe.
    SplitFileCheck7zIfSiblingExists,
    /// `"ipk"`/`"tbz2"`/`"tgz"`/`"tz"`/`"tlz"`/`"txz"`: dispatch straight
    /// to `extract::ctar`.
    CompressedTar,
    /// `"bin"`/`"cdi"`/`"mdf"`: `CheckIso()` first, then the blind 7-Zip
    /// probe (with `is_disk_image = true`).
    DiskImageIsoThenCheck7z,
    /// `"dmg"`: dispatch straight to `extract::sevenzip`.
    DmgImage,
    /// `"cue"`/`"gdi"`/`"iso"`/`"mds"`: the blind 7-Zip probe (with
    /// `is_disk_image = true`) first, then `CheckIso()` — the reverse
    /// order from [`Routing::DiskImageIsoThenCheck7z`].
    Check7zThenDiskImageIso,
    /// `"unitypackage"`: dispatch straight to `extract::unity`.
    UnityPackage,
    /// No case matched — `InitialCheckExt` does nothing further.
    NoAction,
}

/// Ports `Switch $fileext`'s case selection (UniExtract.au3:2196-2213).
/// `fileext` is expected already-lowercased (see module doc comment).
pub fn route(fileext: &str) -> Routing {
    match fileext {
        "001" => Routing::SplitFileCheck7zIfSiblingExists,
        "ipk" | "tbz2" | "tgz" | "tz" | "tlz" | "txz" => Routing::CompressedTar,
        "bin" | "cdi" | "mdf" => Routing::DiskImageIsoThenCheck7z,
        "dmg" => Routing::DmgImage,
        "cue" | "gdi" | "iso" | "mds" => Routing::Check7zThenDiskImageIso,
        "unitypackage" => Routing::UnityPackage,
        _ => Routing::NoAction,
    }
}

/// Builds the `.002` sibling path `Case "001"` checks
/// (UniExtract.au3:2199): `<filedir>\<filename>.002`. `FileExists` on it
/// is real filesystem I/O, left to the caller.
pub fn split_file_sibling_path(filedir: &str, filename: &str) -> String {
    format!("{filedir}\\{filename}.002")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_file_extension_routes_to_sibling_check() {
        assert_eq!(route("001"), Routing::SplitFileCheck7zIfSiblingExists);
    }

    #[test]
    fn compressed_tar_extensions_all_route_the_same_way() {
        for ext in ["ipk", "tbz2", "tgz", "tz", "tlz", "txz"] {
            assert_eq!(route(ext), Routing::CompressedTar);
        }
    }

    /// Parity test for capability C046: the preserved ordering
    /// difference between the two disk-image extension groups.
    #[test]
    fn disk_image_extensions_preserve_their_distinct_call_order() {
        for ext in ["bin", "cdi", "mdf"] {
            assert_eq!(route(ext), Routing::DiskImageIsoThenCheck7z);
        }
        for ext in ["cue", "gdi", "iso", "mds"] {
            assert_eq!(route(ext), Routing::Check7zThenDiskImageIso);
        }
    }

    #[test]
    fn dmg_routes_directly_to_sevenzip() {
        assert_eq!(route("dmg"), Routing::DmgImage);
    }

    #[test]
    fn unitypackage_routes_directly_to_unity() {
        assert_eq!(route("unitypackage"), Routing::UnityPackage);
    }

    #[test]
    fn unrecognized_extension_takes_no_action() {
        assert_eq!(route("zip"), Routing::NoAction);
        assert_eq!(route(""), Routing::NoAction);
    }

    #[test]
    fn split_file_sibling_path_matches_source_shape() {
        assert_eq!(
            split_file_sibling_path(r"C:\downloads", "archive.001"),
            r"C:\downloads\archive.001.002"
        );
    }
}
