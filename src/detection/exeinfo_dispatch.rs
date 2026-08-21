//! Exeinfo PE match dispatch table (`FileScan_ExeInfo`'s `Select`,
//! UniExtract.au3:1141-1278) — capability C043. Maps Exeinfo PE's scanned
//! file-type text to the action `FileScan_ExeInfo` takes once the scan
//! itself (C042) has produced it.
//!
//! ```autoit
//! Select
//!     Case StringInStr($sFileType, "Inno Setup")
//!         checkInno()
//!     Case StringInStr($sFileType, "WinAce / SFX Factory")
//!         extract($TYPE_ACE, ...)
//!     ; ... ~40 more Cases, matched top to bottom, first match wins ...
//!     Case StringInStr($sFileType, "upx") And Not StringInStr($sFileType, "sign like")
//!         unpack($PACKER_UPX)
//!     Case Else
//!         UserDefCompare($aExeinfoDefinitions, $sFileType, "Exeinfo")
//! EndSelect
//! ```
//!
//! Every `StringInStr($sFileType, "...")` call here uses either the bare
//! 2-argument form or an explicit `0` third argument — both
//! case-insensitive (`$STR_NOCASESENSE`, AutoIt's documented default) —
//! so [`classify`] lowercases both sides rather than modeling any
//! case-sensitive branch.
//!
//! **`Case Else` is already covered**: it falls through to
//! `UserDefCompare`, ported as
//! [`detection::detector_mapping::DetectorMapping::resolve_exeinfo`]
//! (C051) — [`Action::Fallback`] just signals that this classification
//! reached it, not a duplicate implementation.
//!
//! **What isn't modeled here**: the exact `t('TERM_X')`-composed display
//! text passed alongside most `extract(...)` calls (translation/
//! formatting only, not a decision), and the internals of `checkInno`/
//! `checkIE`/`checkNSIS`/`CheckTotalObserver`/`unpack`/`BmsExtract` —
//! each is its own separate capability or mechanism; [`Action`] only
//! signals which one this dispatch reaches.

/// What `FileScan_ExeInfo`'s dispatch table decides for a given scanned
/// file-type string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `checkInno()` — Inno Setup detection.
    CheckInno,
    /// `checkIE()` — Installer VISE/Gentee-family detection.
    CheckIe,
    /// `checkNSIS()` — NSIS (Nullsoft) detection.
    CheckNsis,
    /// UniExtract.au3:1168-1170's "Setup Factory" case:
    /// `CheckTotalObserver(label)` runs first, unconditionally followed
    /// by `checkIE()` regardless of what `CheckTotalObserver` did.
    CheckTotalObserverThenCheckIe,
    /// `BmsExtract("install4j")` — the same SQLite-backed `.bms` lookup
    /// ambiguity already backed off for C055/C077/C180.
    BmsExtractInstall4j,
    /// Dispatch to `extract($type_key, ...)` for a recognized
    /// packer/installer/container signature. `type_key` is the
    /// `$TYPE_*` constant's own string value (e.g. `"vssfxpath"` for
    /// `$TYPE_VSSFX_PATH` — deliberately not `"vssfx_path"`).
    Extract(&'static str),
    /// UniExtract.au3:1214-1221's "SPx Method"/"Microsoft SFX CAB" case,
    /// when the scanned text additionally mentions renaming the file to
    /// `.cab`: `CreateRenamedCopy("cab")` runs before the blind 7-Zip
    /// probe, instead of a direct `extract($TYPE_CAB, ...)`.
    RenameToCabThenCheck7z,
    /// `unpack($PACKER_X)` — generic packer unwrapping (already covered
    /// for UPX by C112's `extract::table` entry); `packer` is the
    /// `$PACKER_*` constant's own name (`"upx"`/`"aspack"`).
    Unpack(&'static str),
    /// `terminate($STATUS_NOTSUPPORTED, ...)`.
    NotSupported,
    /// `terminate($STATUS_NOTPACKED, ...)`.
    NotPacked,
    /// `Case Else` — falls through to `UserDefCompare`
    /// (`detection::detector_mapping::resolve_exeinfo`, C051).
    Fallback,
}

/// Ports `FileScan_ExeInfo`'s `Select` (UniExtract.au3:1141-1278),
/// matched top to bottom exactly as the source orders it — several
/// comments in the source call out order dependencies (`InstallAware`
/// must precede `InstallShield`; the `upx` case must be last before
/// `Case Else`) that this preserves.
pub fn classify(file_type: &str) -> Action {
    let s = file_type.to_lowercase();
    let has = |needle: &str| s.contains(&needle.to_lowercase());

    if has("Inno Setup") {
        Action::CheckInno
    } else if has("WinAce / SFX Factory") {
        Action::Extract("ace")
    } else if has("Actual Installer") {
        Action::Extract("actual")
    } else if has("Advanced Installer") {
        Action::Extract("ai")
    } else if has("FreeArc") {
        Action::Extract("freearc")
    } else if has("CreateInstall") {
        Action::Extract("ci")
    } else if has("Excelsior Installer") {
        Action::Extract("ei")
    } else if has("Ghost Installer Studio") {
        Action::Extract("ghost")
    } else if has("Gentee Installer") || has("Installer VISE") {
        Action::CheckIe
    } else if has("Setup Factory") {
        Action::CheckTotalObserverThenCheckIe
    } else if has("install4j") {
        Action::BmsExtractInstall4j
    } else if has("InstallAware") {
        // Must precede "InstallShield" -- InstallAware's scanned text
        // also contains "InstallShield"-adjacent wording upstream.
        Action::Extract("7z")
    } else if has("Install Creator/Pro") {
        Action::Extract("cic")
    } else if has("InstallScript Setup Launcher") {
        Action::Extract("installscript")
    } else if has("InstallShield") {
        Action::Extract("isexe")
    } else if has("KGB SFX") {
        Action::Extract("kgb")
    } else if has("Microsoft Visual C++ 7.0") && has("Custom") && !has("Hotfix") {
        Action::Extract("vssfx")
    } else if has("Microsoft Visual C++ 6.0") && has("Custom") {
        Action::Extract("vssfxpath")
    } else if has("www.molebox.com") {
        Action::Extract("mole")
    } else if has("Netopsystems AG INSTALLER FEAD") {
        Action::Extract("fead")
    } else if has("Nullsoft") {
        Action::CheckNsis
    } else if has("RAR SFX") {
        Action::Extract("rar")
    } else if has("RoboForm Installer") {
        Action::Extract("robo")
    } else if has("WiX Installer") {
        Action::Extract("wix")
    } else if has("SPx Method") || has("Microsoft SFX CAB") {
        if has("rename file *.exe as *.cab") {
            Action::RenameToCabThenCheck7z
        } else {
            Action::Extract("cab")
        }
    } else if has("Overlay :  SWF flash object ver") {
        Action::Extract("swfexe")
    } else if has("VMware ThinApp") || has("Thinstall") || has("ThinyApp Packager") {
        Action::Extract("thinstall")
    } else if has("Wise") || has("PEncrypt 4.0") {
        Action::Extract("wise")
    } else if has("ZIP SFX") || (has("WinZip") && has("Sfx ver")) {
        Action::Extract("zip")
    } else if has("Enigma Virtual Box") {
        Action::Extract("enigma")
    } else if has(".dmg  Mac OS")
        || has(".pak  Chromium format")
        || has("Explorer cache file")
        || has("PyInstaller")
    {
        // Four distinct source Cases, each dispatching to $TYPE_7Z --
        // combined here since the outcome is identical; order among
        // these four doesn't matter, only their combined position
        // relative to the surrounding cases (preserved).
        Action::Extract("7z")
    } else if has("MSCF Cab file detected") || has("VirtualBox Installer") {
        Action::Extract("mscf")
    } else if has("aspack") {
        Action::Unpack("aspack")
    } else if has("Astrum InstallWizard")
        || has("clickteam")
        || has("NE <- Windows 16bit")
        || has("Enigma Protector")
    {
        Action::NotSupported
    } else if (has("Not packed") && !has("Microsoft Visual C++"))
        || has("ELF executable")
        || has("Microsoft Visual C# / Basic.NET")
        || has("Autoit")
        || has("LE <- Linear Executable")
        || has("NOT EXE - Empty file")
        || has("Native - System driver")
        || has("Denuvo protector")
        || has("Kaspersky AV Pack")
        || has("TASM / MASM / FASM - assembler")
    {
        Action::NotPacked
    } else if has("upx") && !has("sign like") {
        // Must be last before Case Else -- several already-matched
        // cases above (e.g. "aspack") could otherwise be shadowed if
        // this ran earlier, per the source's own ordering comment.
        Action::Unpack("upx")
    } else {
        Action::Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inno_setup_routes_to_check_inno() {
        assert_eq!(classify("Inno Setup 5.x installer"), Action::CheckInno);
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(classify("inno setup"), Action::CheckInno);
        assert_eq!(classify("WINACE / SFX FACTORY"), Action::Extract("ace"));
    }

    /// Parity test for capability C043: `InstallAware` must be checked
    /// before `InstallShield`, matching the source's own ordering
    /// comment, even though the scanned text for an InstallAware
    /// package could plausibly also satisfy the `InstallShield` check.
    #[test]
    fn installaware_takes_priority_over_installshield() {
        assert_eq!(
            classify("InstallAware InstallShield-compatible wrapper"),
            Action::Extract("7z")
        );
        assert_eq!(classify("InstallShield 2012"), Action::Extract("isexe"));
    }

    /// Parity test for capability C043: `$TYPE_VSSFX_PATH`'s string
    /// value is `"vssfxpath"`, not `"vssfx_path"` -- verified against
    /// the source's own `Const $TYPE_VSSFX_PATH = "vssfxpath"`.
    #[test]
    fn visual_cpp_6_custom_uses_the_vssfxpath_type_key() {
        assert_eq!(
            classify("Microsoft Visual C++ 6.0 Custom SFX"),
            Action::Extract("vssfxpath")
        );
    }

    /// Parity test for capability C043: the 7.0 case additionally
    /// excludes a "Hotfix" variant.
    #[test]
    fn visual_cpp_7_custom_excludes_hotfix_variant() {
        assert_eq!(
            classify("Microsoft Visual C++ 7.0 Custom SFX"),
            Action::Extract("vssfx")
        );
        assert_eq!(
            classify("Microsoft Visual C++ 7.0 Custom Hotfix SFX"),
            Action::Fallback
        );
    }

    /// Parity test for capability C043: `$TYPE_ISCRIPT`'s string value
    /// is `"installscript"`.
    #[test]
    fn installscript_setup_launcher_uses_installscript_type_key() {
        assert_eq!(
            classify("InstallScript Setup Launcher"),
            Action::Extract("installscript")
        );
    }

    /// Parity test for capability C043: "Setup Factory" is a two-step
    /// dispatch, not a single one.
    #[test]
    fn setup_factory_checks_total_observer_then_check_ie() {
        assert_eq!(
            classify("Setup Factory 9.x"),
            Action::CheckTotalObserverThenCheckIe
        );
    }

    /// Parity test for capability C043: the SFX-CAB rename sub-branch
    /// only fires when the scanned text mentions the rename hint;
    /// otherwise it's a plain `$TYPE_CAB` dispatch.
    #[test]
    fn spx_method_rename_hint_selects_the_renamed_copy_branch() {
        assert_eq!(
            classify("SPx Method: rename file *.exe as *.cab and extract"),
            Action::RenameToCabThenCheck7z
        );
        assert_eq!(
            classify("Microsoft SFX CAB installer"),
            Action::Extract("cab")
        );
    }

    /// Parity test for capability C043: `upx` must be checked last --
    /// an `aspack`-tagged file that also happens to mention `upx`
    /// still matches `aspack` first, per the source's own ordering
    /// comment ("Needs to be at the end").
    #[test]
    fn upx_case_is_checked_after_every_earlier_case() {
        assert_eq!(
            classify("aspack, also mentions upx"),
            Action::Unpack("aspack")
        );
        assert_eq!(classify("upx compressed"), Action::Unpack("upx"));
        assert_eq!(classify("upx sign like structure"), Action::Fallback);
    }

    #[test]
    fn not_supported_group_matches_any_listed_signature() {
        for sample in [
            "Astrum InstallWizard",
            "clickteam engine",
            "NE <- Windows 16bit",
            "Enigma Protector",
        ] {
            assert_eq!(classify(sample), Action::NotSupported);
        }
    }

    #[test]
    fn not_packed_group_matches_any_listed_signature() {
        for sample in [
            "Not packed",
            "ELF executable",
            "Microsoft Visual C# / Basic.NET",
            "Autoit",
            "LE <- Linear Executable",
            "NOT EXE - Empty file",
            "Native - System driver",
            "Denuvo protector",
            "Kaspersky AV Pack",
            "TASM / MASM / FASM - assembler",
        ] {
            assert_eq!(classify(sample), Action::NotPacked);
        }
    }

    /// Parity test for capability C043: "Not packed" is excluded when
    /// the text also mentions "Microsoft Visual C++" (that combination
    /// is a real signature elsewhere, not an unpacked file).
    #[test]
    fn not_packed_excludes_the_visual_cpp_combination() {
        assert_eq!(
            classify("Not packed, Microsoft Visual C++ runtime"),
            Action::Fallback
        );
    }

    #[test]
    fn unrecognized_text_falls_back_to_userdefcompare() {
        assert_eq!(
            classify("some completely unknown signature"),
            Action::Fallback
        );
    }
}
