//! TrID match dispatch table (`tridcompare`, UniExtract.au3:1490-1801) —
//! capability C039. Maps TrID's textual output to the action `tridcompare`
//! takes once the scan itself (C038) has produced it.
//!
//! ```autoit
//! Func tridcompare($sFileType)
//!     Select
//!         Case StringInStr($sFileType, "7-Zip compressed archive")
//!             extract($TYPE_7Z, '7-Zip ' & t('TERM_ARCHIVE'))
//!         ; ... ~90 more Cases, matched top to bottom, first match wins ...
//!         Case Not $isexe And (StringInStr($sFileType, 'Executable') Or StringInStr($sFileType, '(.EXE)', 1))
//!             IsExe()
//!         Case Else
//!             UserDefCompare($aTridDefinitions, $sFileType, "Trid")
//!     EndSelect
//! EndFunc
//! ```
//!
//! Every `StringInStr` call here is case-insensitive **except one**:
//! `StringInStr($sFileType, '(.EXE)', 1)` (UniExtract.au3:1795) passes an
//! explicit `1`, AutoIt's case-*sensitive* mode — the only case-sensitive
//! comparison in this entire dispatch table, easy to miss among ~90
//! otherwise-uniform case-insensitive `Case`s. [`classify`] preserves it
//! exactly: its final branch compares `"(.EXE)"` against `file_type`
//! directly, without lowercasing either side, while every other needle
//! in this function goes through the shared lowercased `has` closure.
//!
//! **Two genuine preserved quirks, not modeling artifacts** (the same
//! kind already found for C041):
//! - `"null bytes"` appears twice — once in the disk-image group
//!   (UniExtract.au3:1573, checked first) and again in the "Not packed"
//!   group (1782). The disk-image case always wins; the "Not packed"
//!   mention of it is unreachable.
//! - `"Executable"` (bare, case-insensitive) is a literal substring of
//!   `"ELF Executable and Linkable format"` — but that's harmless here,
//!   since the "Not packed" `Case` containing it (1782) is checked
//!   *before* the final `Case Not $isexe And (... "Executable" ...)`
//!   (1795), so ELF binaries are correctly classified as not-packed
//!   before the generic executable check ever runs.
//!
//! **`Case Else` is already covered**: it falls through to
//! `UserDefCompare`, ported as
//! [`detection::detector_mapping::DetectorMapping::resolve_trid`] (C051)
//! — [`Action::Fallback`] just signals that this classification reached
//! it.
//!
//! **What isn't modeled here**: the exact `t('TERM_X')`-composed display
//! text, and the internals of `CheckAlz`/`checkIE`/`CheckTotalObserver`/
//! `CheckGame`/`CheckGarbro`/`check7z`/`IsExe` — each is its own
//! separate, already-referenced capability.

/// What `tridcompare`'s `Select` (UniExtract.au3:1493-1799) decides for a
/// given TrID output string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Dispatch to `extract($type_key, ..., additional_parameters)`.
    /// `additional_parameters` is the source's third positional
    /// argument to `extract()` — empty when the source call omits it,
    /// otherwise a forced-format hint (`"bz2"`/`"gz"`/`"tar"`/`"xz"`/
    /// `"Z"`) feeding `extract::sevenzip`'s `classify_post_extraction`
    /// (C056).
    Extract {
        type_key: &'static str,
        additional_parameters: &'static str,
    },
    /// `extractDiskImage($type_key, ...)` — a distinct dispatch entry
    /// point from `extract()` (also used by C041).
    ExtractDiskImage { type_key: &'static str },
    /// UniExtract.au3:1592-1593's "Windows Help File" case:
    /// `extract($TYPE_HLP, ..., "", False, True)` first — `returnFail =
    /// True` means a failure returns `false` instead of terminating
    /// (`extract::completion::resolve_completion`, C054/C181) — falling
    /// through to `extract($TYPE_CHM, ...)` only when the HLP attempt
    /// fails.
    ExtractHlpThenChm,
    /// `CheckAlz()`.
    CheckAlz,
    /// UniExtract.au3:1572-1576's disk-image group: `CheckIso()` then
    /// `check7z(disp, True)` (`is_disk_image = true`), unconditionally
    /// in sequence.
    CheckIsoThenDiskImageCheck7z,
    /// `checkIE()`.
    CheckIe,
    /// `CheckTotalObserver(...)`.
    CheckTotalObserver,
    /// UniExtract.au3:1601-1603's "InstallShield Z archive" case:
    /// `CreateRenamedCopy("z")` runs first only if `$fileext <> "z"`
    /// (case-insensitive `=`), then `CheckTotalObserver(...)`
    /// unconditionally either way.
    InstallShieldZArchive { needs_rename: bool },
    /// UniExtract.au3:1658's "Broken Age package" case:
    /// `CheckGame(False, False)` — explicit non-default arguments
    /// (source default is `CheckGame($bUseGaup = True, $bUseGarbro =
    /// True)`), disabling both the Gaup and Garbro sub-probes for this
    /// one signature.
    CheckGameNoGaupNoGarbro,
    /// `CheckGarbro(...)` alone, no fallback.
    CheckGarbro,
    /// `CheckGarbro(msg)` followed unconditionally by
    /// `extract($TYPE_ARC_CONV, msg)` — UniExtract.au3:1663-1666 and
    /// three other cases share this exact two-step shape.
    CheckGarbroThenExtractArcConv,
    /// `check7z(...)`.
    Check7z,
    /// `IsExe()` — UniExtract.au3:1796, the final named case before
    /// `Case Else`. Only reachable when `is_exe` is `false` (the
    /// source's own `Not $isexe` guard) and the file-type text mentions
    /// an executable, matched per the module doc comment's mixed
    /// case-sensitivity note.
    IsExe,
    /// `terminate($STATUS_NOTSUPPORTED, ...)`.
    NotSupported,
    /// `terminate($STATUS_NOTPACKED, ...)`.
    NotPacked,
    /// `Case Else` — falls through to `UserDefCompare`
    /// (`detection::detector_mapping::resolve_trid`, C051).
    Fallback,
}

/// Ports `tridcompare`'s `Select` (UniExtract.au3:1493-1799), matched top
/// to bottom exactly as the source orders it. `is_exe` is `$isexe`
/// (already-known state from earlier in the cascade, C037); `fileext` is
/// the (assumed already-lowercased) file extension, needed only for the
/// `InstallShield Z archive` case.
pub fn classify(file_type: &str, is_exe: bool, fileext: &str) -> Action {
    let s = file_type.to_lowercase();
    let has = |needle: &str| s.contains(&needle.to_lowercase());

    if has("7-Zip compressed archive")
        || has("Android Package")
        || has("ARJ compressed archive")
        || has("asar Electron Archive")
        || has("BZA compressed")
        || has("GZA compressed")
    {
        // Five distinct source Cases, each dispatching to $TYPE_7Z with
        // no forced-format hint -- combined since the outcome is
        // identical.
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("bzip2 compressed archive") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "bz2",
        }
    } else if has("HBuilder language package")
        || has("CPIO Archive")
        || has("Debian Linux Package")
        || has("Disk Image (Macintosh)")
    {
        // Four distinct source Cases, each dispatching to $TYPE_7Z with
        // no forced-format hint -- combined since the outcome is
        // identical.
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("GZipped") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "gz",
        }
    } else if has("LHARC/LZARK compressed archive") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("UNIX Compressed") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "Z",
        }
    } else if has("RPM Package") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("TAR - Tape ARchive") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "tar",
        }
    } else if has("VirtualBox Disk Image")
        || has("Virtual HD image")
        || has("VMware 4 Virtual Disk")
        || has("Windows Imaging Format")
    {
        // Four distinct source Cases, each dispatching to $TYPE_7Z with
        // no forced-format hint -- combined since the outcome is
        // identical.
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("xz compressed container") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "xz",
        }
    } else if has("ACE compressed archive") || has("ACE Self-Extracting Archive") {
        Action::Extract {
            type_key: "ace",
            additional_parameters: "",
        }
    } else if has("ALZip compressed archive") {
        Action::CheckAlz
    } else if has("BCM compressed") {
        Action::Extract {
            type_key: "bcm",
            additional_parameters: "",
        }
    } else if has("Android boot image") {
        Action::Extract {
            type_key: "bootimg",
            additional_parameters: "",
        }
    } else if has("LZIP compressed archive") {
        Action::Extract {
            type_key: "lz",
            additional_parameters: "",
        }
    } else if has("Microsoft Cabinet Archive") || has("IncrediMail letter/ecard") {
        Action::Extract {
            type_key: "cab",
            additional_parameters: "",
        }
    } else if has("Magic ISO Universal Image Format") {
        Action::ExtractDiskImage { type_key: "uif" }
    } else if has("MAME Compressed Hard Disk image") {
        Action::ExtractDiskImage { type_key: "chd" }
    } else if has("Generic PC disk image")
        || has("WinImage compressed disk image")
        || has("CDImage")
        || has("CD image")
        || has("null bytes")
        || has("Nero Burning ROM")
        || has("Error Code Modeler")
    {
        Action::CheckIsoThenDiskImageCheck7z
    } else if has("PowerISO Direct-Access-Archive") || has("gBurner Image") {
        Action::ExtractDiskImage { type_key: "daa" }
    } else if has("DGCA Digital G Codec Archiver") {
        Action::Extract {
            type_key: "dgca",
            additional_parameters: "",
        }
    } else if has("FMOD Sample Bank Format") {
        Action::Extract {
            type_key: "fsb",
            additional_parameters: "",
        }
    } else if has("Gentee Installer executable")
        || has("Installer VISE executable")
        || has("Setup Factory")
    {
        Action::CheckIe
    } else if has("Windows Help File") {
        Action::ExtractHlpThenChm
    } else if has("Reflexive Arcade Installer") {
        Action::Extract {
            type_key: "rai",
            additional_parameters: "",
        }
    } else if has("InstallForge Installer") {
        Action::Extract {
            type_key: "installforge",
            additional_parameters: "",
        }
    } else if has("InstallShield Z archive") {
        Action::InstallShieldZArchive {
            needs_rename: !fileext.eq_ignore_ascii_case("z"),
        }
    } else if has("InstallShield compressed archive") {
        Action::Extract {
            type_key: "iscab",
            additional_parameters: "",
        }
    } else if has("ISo Zipped format") {
        Action::ExtractDiskImage { type_key: "isz" }
    } else if has("KGB archive") {
        Action::Extract {
            type_key: "kgb",
            additional_parameters: "",
        }
    } else if has("lzop compressed") {
        Action::Extract {
            type_key: "lzo",
            additional_parameters: "",
        }
    } else if has("LZX Amiga compressed archive") {
        Action::Extract {
            type_key: "lzx",
            additional_parameters: "",
        }
    } else if has("MIME HTML archive format") || has("E-Mail message") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("Microsoft Windows Installer merge module") {
        Action::Extract {
            type_key: "msm",
            additional_parameters: "",
        }
    } else if has("Microsoft Windows Installer") || has("Generic OLE2 / Multistream Compound") {
        Action::Extract {
            type_key: "msi",
            additional_parameters: "",
        }
    } else if has("Windows Installer Patch") {
        Action::Extract {
            type_key: "msp",
            additional_parameters: "",
        }
    } else if has("MPQ Archive - Blizzard game data") {
        Action::CheckTotalObserver
    } else if has("HTC NBH ROM Image") {
        Action::Extract {
            type_key: "nbh",
            additional_parameters: "",
        }
    } else if has("Outlook Express Database") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("Portable Document Format") {
        Action::Extract {
            type_key: "pdf",
            additional_parameters: "",
        }
    } else if has("PEA compressed archive") {
        Action::Extract {
            type_key: "pea",
            additional_parameters: "",
        }
    } else if has("RAR compressed archive") {
        Action::Extract {
            type_key: "rar",
            additional_parameters: "",
        }
    } else if has("Artemis engine resource archive")
        || has("BGI (Buriko General Interpreter) engine")
    {
        Action::CheckGarbro
    } else if has("Broken Age package") {
        Action::CheckGameNoGaupNoGarbro
    } else if has("Bruns Engine encrypted") || has("Ultramarine 3 encrypted audio file") {
        Action::CheckGarbro
    } else if has("ClsFileLink")
        || has("ERISA archive file")
        || has("KiriKiri Adventure Game System Package")
    {
        Action::CheckGarbroThenExtractArcConv
    } else if has("Livemaker Engine main game executable")
        || has("NScripter archive, version 1")
        || has("Pajamas Adventure System game data archive")
    {
        Action::CheckGarbro
    } else if has("Ren'Py data file") {
        Action::Extract {
            type_key: "rpa",
            additional_parameters: "",
        }
    } else if has("RPG Maker") && !has("MV encrypted") {
        Action::Extract {
            type_key: "rgss",
            additional_parameters: "",
        }
    } else if has("Telltale Games ressource archive") {
        Action::Extract {
            type_key: "ttarch",
            additional_parameters: "",
        }
    } else if has("Unreal Package") {
        Action::Extract {
            type_key: "unreal",
            additional_parameters: "",
        }
    } else if has("Valve package")
        || has("WAD3 game data")
        || has("Valve Source map")
        || has("Valve Source BSP")
    {
        Action::CheckTotalObserver
    } else if has("Visionaire Studio V3 archive") {
        Action::Extract {
            type_key: "visionaire3",
            additional_parameters: "",
        }
    } else if has("Wintermute Engine data") {
        Action::Extract {
            type_key: "dcp",
            additional_parameters: "",
        }
    } else if has("Wolf RPG Editor") || has("YU-RIS Script Engine") {
        Action::CheckGarbroThenExtractArcConv
    } else if has("sfArk compressed SoundFont") {
        Action::Extract {
            type_key: "sfark",
            additional_parameters: "",
        }
    } else if has("EPOC Installation package") {
        Action::Extract {
            type_key: "sis",
            additional_parameters: "",
        }
    } else if has("MacBinary") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("Macromedia Flash Player") {
        Action::Extract {
            type_key: "swf",
            additional_parameters: "",
        }
    } else if has("UHARC compressed archive") {
        Action::Extract {
            type_key: "uha",
            additional_parameters: "",
        }
    } else if has("BinHex encoded") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("PHP source") {
        Action::Check7z
    } else if has("Web ARChive") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("Windows Update Package") {
        Action::Extract {
            type_key: "msu",
            additional_parameters: "",
        }
    } else if has("Wise Installer Executable") {
        Action::Extract {
            type_key: "wise",
            additional_parameters: "",
        }
    } else if has("ZIP compressed archive") || has("Winzip Win32 self-extracting archive") {
        Action::Extract {
            type_key: "zip",
            additional_parameters: "",
        }
    } else if has("ZOO compressed archive") {
        Action::Extract {
            type_key: "zoo",
            additional_parameters: "",
        }
    } else if has("ZPAQ compressed archive") {
        Action::Extract {
            type_key: "zpaq",
            additional_parameters: "",
        }
    } else if has("LZMA compressed archive") || has("Windows Thumbnail Database") {
        // "Forced to bottom of list due to false positives" -- source's
        // own comment (line 1756).
        Action::Check7z
    } else if has("Enigma Virtual Box virtualized executable") {
        Action::Extract {
            type_key: "enigma",
            additional_parameters: "",
        }
    } else if has("FreeArc compressed archive") {
        Action::Extract {
            type_key: "freearc",
            additional_parameters: "",
        }
    } else if has("InstallShield setup") {
        Action::Extract {
            type_key: "isexe",
            additional_parameters: "",
        }
    } else if has("audio") || has("FLAC lossless") {
        Action::Extract {
            type_key: "audio",
            additional_parameters: "",
        }
    } else if has("Smacker movie/video") || has("Bink video") {
        Action::Extract {
            type_key: "videoconv",
            additional_parameters: "",
        }
    } else if has("Video")
        || has("QuickTime Movie")
        || has("Matroska")
        || has("Material Exchange Format")
        || has("Windows Media (generic)")
        || has("GIF animated")
        || has("MPEG-2 Transport Stream")
    {
        Action::Extract {
            type_key: "video",
            additional_parameters: "",
        }
    } else if has("null bytes")
        || has("phpMyAdmin SQL dump")
        || has("ELF Executable and Linkable format")
        || has("Generic XML")
        || has("Microsoft Program DataBase")
        || has("Windows Minidump")
        || has("Windows Shortcut")
        || has("JPEG bitmap")
        || has("Windows Registry Data")
        || has("X509 Certificate")
        || has("Linux/UNIX shell script")
    {
        // "null bytes" here is unreachable -- see the module doc
        // comment's quirk note; kept to document the source's own dead
        // case rather than silently dropping it.
        Action::NotPacked
    } else if has("Long Range ZIP") || has("Kremlin Encrypted File") || has("Foxit Reader Add-on") {
        Action::NotSupported
    } else if !is_exe && (has("Executable") || file_type.contains("(.EXE)")) {
        // The only case-sensitive comparison in this whole table: the
        // source's own `StringInStr($sFileType, '(.EXE)', 1)` -- see
        // the module doc comment.
        Action::IsExe
    } else {
        Action::Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_zip_compressed_archive_routes_to_7z() {
        assert_eq!(
            classify("7-Zip compressed archive data", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: ""
            }
        );
    }

    #[test]
    fn matching_is_case_insensitive_except_the_one_documented_exception() {
        assert_eq!(
            classify("7-ZIP COMPRESSED ARCHIVE", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: ""
            }
        );
    }

    /// Parity test for capability C039: bzip2/gzip/tar/xz/UNIX-compress
    /// pass a forced-format hint as `extract()`'s third argument.
    #[test]
    fn compressed_stream_cases_forward_the_format_hint() {
        assert_eq!(
            classify("bzip2 compressed archive data", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "bz2"
            }
        );
        assert_eq!(
            classify("GZipped data", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "gz"
            }
        );
        assert_eq!(
            classify("TAR - Tape ARchive", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "tar"
            }
        );
        assert_eq!(
            classify("xz compressed container", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "xz"
            }
        );
        assert_eq!(
            classify("UNIX Compressed data", false, ""),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "Z"
            }
        );
    }

    #[test]
    fn disk_image_extract_cases_use_extract_disk_image() {
        assert_eq!(
            classify("Magic ISO Universal Image Format", false, ""),
            Action::ExtractDiskImage { type_key: "uif" }
        );
        assert_eq!(
            classify("PowerISO Direct-Access-Archive", false, ""),
            Action::ExtractDiskImage { type_key: "daa" }
        );
    }

    /// Parity test for capability C039: "null bytes" is checked in the
    /// disk-image group first (checked well before the "Not packed"
    /// group), so it always routes to CheckIso+check7z, never to
    /// NotPacked -- a genuine dead-code quirk in the source, preserved
    /// rather than removed.
    #[test]
    fn null_bytes_is_shadowed_by_the_earlier_disk_image_group() {
        assert_eq!(
            classify("null bytes data", false, ""),
            Action::CheckIsoThenDiskImageCheck7z
        );
    }

    #[test]
    fn windows_help_file_extracts_hlp_then_falls_back_to_chm() {
        assert_eq!(
            classify("Windows Help File", false, ""),
            Action::ExtractHlpThenChm
        );
    }

    /// Parity test for capability C039: `$fileext = "z"` (case-
    /// insensitive) suppresses the renamed-copy step.
    #[test]
    fn installshield_z_archive_skips_rename_when_already_dot_z() {
        assert_eq!(
            classify("InstallShield Z archive", false, "z"),
            Action::InstallShieldZArchive {
                needs_rename: false
            }
        );
        assert_eq!(
            classify("InstallShield Z archive", false, "Z"),
            Action::InstallShieldZArchive {
                needs_rename: false
            }
        );
        assert_eq!(
            classify("InstallShield Z archive", false, "exe"),
            Action::InstallShieldZArchive { needs_rename: true }
        );
    }

    #[test]
    fn broken_age_disables_gaup_and_garbro() {
        assert_eq!(
            classify("Broken Age package", false, ""),
            Action::CheckGameNoGaupNoGarbro
        );
    }

    #[test]
    fn game_engine_cases_split_between_plain_and_arc_conv_fallback() {
        assert_eq!(
            classify("Artemis engine resource archive", false, ""),
            Action::CheckGarbro
        );
        assert_eq!(
            classify("ClsFileLink data", false, ""),
            Action::CheckGarbroThenExtractArcConv
        );
        assert_eq!(
            classify("Wolf RPG Editor archive", false, ""),
            Action::CheckGarbroThenExtractArcConv
        );
    }

    /// Parity test for capability C039: `"RPG Maker"` excludes the
    /// `"MV encrypted"` variant.
    #[test]
    fn rpg_maker_excludes_mv_encrypted_variant() {
        assert_eq!(
            classify("RPG Maker VX Ace archive", false, ""),
            Action::Extract {
                type_key: "rgss",
                additional_parameters: ""
            }
        );
        assert_eq!(
            classify("RPG Maker MV encrypted asset", false, ""),
            Action::Fallback
        );
    }

    #[test]
    fn php_source_and_lzma_both_route_to_check7z() {
        assert_eq!(
            classify("PHP source, ASCII text", false, ""),
            Action::Check7z
        );
        assert_eq!(
            classify("LZMA compressed archive", false, ""),
            Action::Check7z
        );
    }

    #[test]
    fn not_supported_group_matches_any_listed_signature() {
        for sample in [
            "Long Range ZIP",
            "Kremlin Encrypted File",
            "Foxit Reader Add-on",
        ] {
            assert_eq!(classify(sample, false, ""), Action::NotSupported);
        }
    }

    #[test]
    fn not_packed_group_matches_any_listed_signature() {
        for sample in [
            "phpMyAdmin SQL dump",
            "ELF Executable and Linkable format",
            "Generic XML document",
            "Microsoft Program DataBase",
            "Windows Minidump",
            "Windows Shortcut",
            "JPEG bitmap",
            "Windows Registry Data",
            "X509 Certificate",
            "Linux/UNIX shell script",
        ] {
            assert_eq!(classify(sample, false, ""), Action::NotPacked);
        }
    }

    /// Parity test for capability C039: the final `IsExe()` case
    /// requires `is_exe` to be `false` (the source's `Not $isexe`
    /// guard).
    #[test]
    fn executable_case_requires_is_exe_false() {
        assert_eq!(
            classify("Win32 Executable (generic)", false, ""),
            Action::IsExe
        );
        assert_eq!(
            classify("Win32 Executable (generic)", true, ""),
            Action::Fallback
        );
    }

    /// Parity test for capability C039: the one case-sensitive check in
    /// this whole table -- `"(.EXE)"` must match exact case, unlike
    /// every other needle here.
    #[test]
    fn exe_marker_case_sensitivity_is_the_one_documented_exception() {
        assert_eq!(
            classify("Some file (.EXE) format", false, ""),
            Action::IsExe
        );
        assert_eq!(
            classify("Some file (.exe) format", false, ""),
            Action::Fallback
        );
    }

    #[test]
    fn unrecognized_text_falls_back_to_userdefcompare() {
        assert_eq!(
            classify("some completely unknown signature", false, ""),
            Action::Fallback
        );
    }
}
