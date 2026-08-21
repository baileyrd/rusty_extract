//! PEiD match dispatch table (`FileScan_Peid`'s `Select`,
//! UniExtract.au3:1332-1394) — the classification half of capability
//! C044. Maps PEiD's scanned file-type text to the action
//! `FileScan_Peid` takes once the scan itself has produced it.
//!
//! ```autoit
//! Select
//!     Case StringInStr($sFileType, "Enigma Virtual Box")
//!         extract($TYPE_ENIGMA, 'Enigma Virtual Box ' & t('TERM_PACKAGE'))
//!     ; ... ~18 more Cases, matched top to bottom, first match wins ...
//!     Case StringInStr($sFileType, "Unable to open file", 0)
//!         $isexe = False
//! EndSelect
//! ```
//!
//! **Not modeled: the PEiD scan itself.** `FileScan_Peid` drives PEiD
//! through real Win32 GUI automation (`Run`/`WinWait("PEiD v")`/
//! `ControlGetText`/`WinClose`, plus backing up and restoring three
//! registry values around the call) — the same deferred-GUI-subsystem
//! blocker already found for C069/C106/C054's `$TYPE_MSCF` fallback/
//! C056's SFX splitter (manifest row D001). [`classify`] only covers the
//! dispatch table that runs *after* a scan result text is already in
//! hand; manifest row C044 stays `REQUIRED` for that reason — this is a
//! partial port, the same shape as C056/C077's own partial coverage.
//!
//! **No `Case Else` here** — unlike C039/C041/C043's dispatch tables,
//! this `Select` has no fallback case at all: an unmatched scan result
//! simply falls through `EndSelect` with no action. [`Action::NoMatch`]
//! models that directly; there's no `UserDefCompare`/registry-mapping
//! fallback to point to for PEiD.
//!
//! **The one case-sensitive comparison in this table**:
//! `StringInStr($sFileType, "PEtite", 1)` (UniExtract.au3:1361) passes
//! AutoIt's explicit case-sensitive mode — every other `Case` here is
//! case-insensitive (bare or explicit `0`), the same kind of
//! easy-to-miss exception already found for C039's `'(.EXE)', 1`.
//!
//! **What isn't modeled here**: the exact `t('TERM_X')`-composed display
//! text, and the internals of `checkIE`/`checkInno`/`checkNSIS`/`unpack`
//! — each is its own separate capability. `checkArj()`'s own result
//! (real process I/O, C061) is left to the caller — see
//! [`Action::CheckArjThenExtractAceIfNotArj`].

/// What `FileScan_Peid`'s `Select` (UniExtract.au3:1332-1394) decides
/// for a given scanned PEiD file-type string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Dispatch to `extract($type_key, ...)`. None of this table's
    /// cases pass `extract()` a third (`additionalParameters`)
    /// argument, unlike C039/C041's tables.
    Extract { type_key: &'static str },
    /// `checkIE()`.
    CheckIe,
    /// `checkInno()`.
    CheckInno,
    /// `checkNSIS()`.
    CheckNsis,
    /// UniExtract.au3:1361-1362's "PEtite" case: `If Not checkArj()
    /// Then extract($TYPE_ACE, ...)`. `checkArj`'s own result (real
    /// process I/O, already ported as `detection::arj_probe`, C061) is
    /// left to the caller — this variant just signals that the
    /// `$TYPE_ACE` dispatch is conditional on it being `false`.
    CheckArjThenExtractAceIfNotArj,
    /// `unpack($PACKER_X)` — `packer` is the `$PACKER_*` constant's own
    /// name (`"upx"`/`"aspack"`).
    Unpack { packer: &'static str },
    /// `$isexe = False` — UniExtract.au3:1392, a plain state mutation,
    /// not an `extract`/`terminate` call.
    ClearIsExe,
    /// No `Case` matched. There is no `Case Else` in this `Select` —
    /// `FileScan_Peid` simply returns with no further action.
    NoMatch,
}

/// Ports `FileScan_Peid`'s `Select` (UniExtract.au3:1332-1394), matched
/// top to bottom exactly as the source orders it.
pub fn classify(file_type: &str) -> Action {
    let s = file_type.to_lowercase();
    let has = |needle: &str| s.contains(&needle.to_lowercase());

    if has("Enigma Virtual Box") {
        Action::Extract { type_key: "enigma" }
    } else if has("ARJ SFX") {
        Action::Extract { type_key: "7z" }
    } else if has("Gentee Installer") {
        Action::CheckIe
    } else if has("Inno Setup") {
        Action::CheckInno
    } else if has("Installer VISE") {
        Action::Extract { type_key: "ie" }
    } else if has("KGB SFX") {
        Action::Extract { type_key: "kgb" }
    } else if has("Microsoft Visual C++ 7.0") && has("Custom") && !has("Hotfix") {
        Action::Extract { type_key: "vssfx" }
    } else if has("Microsoft Visual C++ 6.0") && has("Custom") {
        Action::Extract {
            type_key: "vssfxpath",
        }
    } else if has("Nullsoft PiMP SFX") {
        Action::CheckNsis
    } else if file_type.contains("PEtite") {
        // The one case-sensitive comparison in this table -- see the
        // module doc comment.
        Action::CheckArjThenExtractAceIfNotArj
    } else if has("RAR SFX") {
        Action::Extract { type_key: "rar" }
    } else if has("RoboForm Installer") {
        Action::Extract { type_key: "robo" }
    } else if has("Setup Factory 6.x") {
        Action::Extract { type_key: "ie" }
    } else if has("SPx Method") || has("CAB SFX") {
        Action::Extract { type_key: "cab" }
    } else if has("SuperDAT") {
        Action::Extract {
            type_key: "superdat",
        }
    } else if has("Wise") || has("PEncrypt 4.0") {
        Action::Extract { type_key: "wise" }
    } else if has("ZIP SFX") {
        Action::Extract { type_key: "zip" }
    } else if has("upx") {
        Action::Unpack { packer: "upx" }
    } else if has("aspack") {
        Action::Unpack { packer: "aspack" }
    } else if has("Unable to open file") {
        Action::ClearIsExe
    } else {
        Action::NoMatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enigma_virtual_box_routes_to_enigma() {
        assert_eq!(
            classify("Enigma Virtual Box protected"),
            Action::Extract { type_key: "enigma" }
        );
    }

    #[test]
    fn arj_sfx_routes_to_7z_not_arj_type() {
        assert_eq!(
            classify("ARJ SFX archive"),
            Action::Extract { type_key: "7z" }
        );
    }

    /// Parity test for capability C044: two distinct cases both use the
    /// literal `extract("ie", ...)` call rather than a `$TYPE_*`
    /// constant.
    #[test]
    fn installer_vise_and_setup_factory_both_use_the_literal_ie_type() {
        assert_eq!(
            classify("Installer VISE 3.x"),
            Action::Extract { type_key: "ie" }
        );
        assert_eq!(
            classify("Setup Factory 6.x installer"),
            Action::Extract { type_key: "ie" }
        );
    }

    #[test]
    fn visual_cpp_cases_use_distinct_type_keys() {
        assert_eq!(
            classify("Microsoft Visual C++ 7.0 Custom SFX"),
            Action::Extract { type_key: "vssfx" }
        );
        assert_eq!(
            classify("Microsoft Visual C++ 7.0 Custom Hotfix SFX"),
            Action::NoMatch
        );
        assert_eq!(
            classify("Microsoft Visual C++ 6.0 Custom SFX"),
            Action::Extract {
                type_key: "vssfxpath"
            }
        );
    }

    /// Parity test for capability C044: `"PEtite"` is the one
    /// case-sensitive comparison in this table.
    #[test]
    fn petite_case_is_case_sensitive() {
        assert_eq!(
            classify("PEtite 2.x compressed"),
            Action::CheckArjThenExtractAceIfNotArj
        );
        assert_eq!(classify("petite 2.x compressed"), Action::NoMatch);
        assert_eq!(classify("PETITE 2.x compressed"), Action::NoMatch);
    }

    #[test]
    fn upx_and_aspack_route_to_unpack_with_distinct_packers() {
        assert_eq!(classify("upx compressed"), Action::Unpack { packer: "upx" });
        assert_eq!(
            classify("aspack compressed"),
            Action::Unpack { packer: "aspack" }
        );
    }

    #[test]
    fn unable_to_open_file_clears_isexe() {
        assert_eq!(classify("Unable to open file"), Action::ClearIsExe);
    }

    /// Parity test for capability C044: unlike C039/C041/C043, there is
    /// no `Case Else` here -- an unrecognized scan result takes no
    /// action at all.
    #[test]
    fn unrecognized_text_has_no_fallback() {
        assert_eq!(
            classify("some completely unknown signature"),
            Action::NoMatch
        );
    }
}
