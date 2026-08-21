//! Unix `file` tool match dispatch table (`filecompare`,
//! UniExtract.au3:1398-1487) — capability C041. Maps the `file` tool's
//! textual output to the action `filecompare` takes once the scan itself
//! (C040) has produced it.
//!
//! ```autoit
//! Func filecompare($sFileType)
//!     Select
//!         Case StringInStr($sFileType, "7 zip archive data") Or StringInStr($sFileType, "7-zip archive data")
//!             extract($TYPE_7Z, '7-Zip ' & t('TERM_ARCHIVE'))
//!         ; ... ~24 more Cases, matched top to bottom, first match wins ...
//!         Case StringInStr($sFileType, "ISO", 0) And StringInStr($sFileType, "filesystem", 0)
//!             CheckIso()
//!         Case Else
//!             UserDefCompare($aFileDefinitions, $sFileType, "File")
//!     EndSelect
//!
//!     ; Not extractable filetypes
//!     If StringInStr($sFileType, "CDF V2 document") Then Return
//!
//!     If (StringInStr($sFileType, "text") And (...)) Or ... Then
//!         terminate($STATUS_NOTPACKED, $file, $fileext, $sFileType)
//!
//!     If StringInStr($sFileType, "MS-DOS executable") Then terminate($STATUS_NOTSUPPORTED, $file, $sFileType, $sFileType)
//! EndFunc
//! ```
//!
//! Every `StringInStr($sFileType, "...")` call here uses either the bare
//! 2-argument form or an explicit `0` third argument — both
//! case-insensitive (`$STR_NOCASESENSE`, AutoIt's documented default) —
//! so [`classify`] and [`trailing_termination`] both lowercase rather
//! than modeling any case-sensitive branch.
//!
//! **A genuinely important difference from `exeinfo_dispatch` (C043)**:
//! there, every `Select` outcome ends in a call that itself always
//! terminates the process, so the source having no code after its
//! `EndSelect` was never a modeling concern. Here it's different —
//! `filecompare` has *two more `If` checks after the `Select`*
//! (`CDF V2 document`, then the not-packed/not-supported groups), and
//! not every `Select` outcome is guaranteed to terminate before reaching
//! them: `CheckTotalObserver()`/`check7z()`/`CheckIso()` are themselves
//! detection cascades (their own separate capabilities) that can fail to
//! dispatch and simply return, letting control fall through to this
//! trailing logic exactly as the source's straight-line function body
//! does. [`classify`] and [`trailing_termination`] are kept as two
//! separate functions for this reason — a caller applies
//! [`trailing_termination`] after any [`Action`] that doesn't itself
//! terminate (in practice: [`Action::Fallback`] always, and
//! [`Action::CheckTotalObserver`]/[`Action::Check7z`]/[`Action::CheckIso`]
//! whenever *their own* cascade doesn't find a match — that "did it
//! match" question belongs to those separate capabilities, not this
//! one). [`Action::Extract`]/[`Action::ExtractDiskImage`] always
//! terminate via a nested `extract(...)`/`extractDiskImage(...)` call
//! (the same completion contract as C054/C181), so the trailing checks
//! never apply after those.
//!
//! **A genuine quirk in the source itself**: `"POSIX tar archive"`
//! (line 1428) is checked *after* `"ar archive"` (line 1424) — but
//! `"tar archive"` always contains `"ar archive"` as a literal
//! substring (`"tar"`'s last two letters plus the following `"
//! archive"`), so the more specific, later case is unreachable in
//! practice. Preserved exactly rather than "fixed" — see
//! [`classify`]'s own test for the shadowed case.
//!
//! **`Case Else` is already covered**: it falls through to
//! `UserDefCompare`, ported as
//! [`detection::detector_mapping::DetectorMapping::resolve_file`] (C051)
//! — [`Action::Fallback`] just signals that this classification reached
//! it.
//!
//! **What isn't modeled here**: the exact `t('TERM_X')`-composed display
//! text passed alongside `extract(...)` calls (translation/formatting
//! only), and the internals of `CheckTotalObserver`/`check7z`/`CheckIso`
//! — each is its own separate, already-referenced capability.

use crate::status::Status;

/// What `filecompare`'s `Select` (UniExtract.au3:1400-1467) decides for a
/// given `file`-tool output string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Dispatch to `extract($type_key, ..., additional_parameters)`.
    /// `additional_parameters` is the source's third positional
    /// argument to `extract()` — empty when the source call omits it,
    /// otherwise a forced-format hint (`"bz2"`/`"gz"`/`"xz"`/`"tar"`)
    /// that feeds `extract::sevenzip`'s already-ported
    /// `classify_post_extraction` (C056) for the `bz2`/`gz`/`xz` cases.
    Extract {
        type_key: &'static str,
        additional_parameters: &'static str,
    },
    /// `extractDiskImage($type_key, ...)` — UniExtract.au3:1435, a
    /// distinct dispatch entry point from `extract()`.
    ExtractDiskImage { type_key: &'static str },
    /// `CheckTotalObserver('MPQ ' & t('TERM_ARCHIVE'))`.
    CheckTotalObserver,
    /// `check7z("Base 64" & t('TERM_ENCODED'))`.
    Check7z,
    /// `CheckIso()`.
    CheckIso,
    /// `Case Else` — falls through to `UserDefCompare`
    /// (`detection::detector_mapping::resolve_file`, C051).
    Fallback,
}

/// Ports `filecompare`'s `Select` (UniExtract.au3:1400-1467), matched top
/// to bottom exactly as the source orders it. Notably, "MIME entity
/// text"/"mhtml" (→ `extract($TYPE_7Z, ...)`) is checked *before* the
/// plain "MIME entity" case (→ `check7z(...)`), so a `file` output
/// mentioning "MIME entity text" never reaches the plainer case.
pub fn classify(file_type: &str) -> Action {
    let s = file_type.to_lowercase();
    let has = |needle: &str| s.contains(&needle.to_lowercase());

    if has("7 zip archive data") || has("7-zip archive data") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("RAR archive data") {
        Action::Extract {
            type_key: "rar",
            additional_parameters: "",
        }
    } else if has("lzip compressed data") {
        Action::Extract {
            type_key: "lz",
            additional_parameters: "",
        }
    } else if has("Zip archive data") && !has("7") {
        Action::Extract {
            type_key: "zip",
            additional_parameters: "",
        }
    } else if has("UHarc archive data") {
        Action::Extract {
            type_key: "uha",
            additional_parameters: "",
        }
    } else if has("Symbian installation file") {
        Action::Extract {
            type_key: "sis",
            additional_parameters: "",
        }
    } else if has("Zoo archive data") {
        Action::Extract {
            type_key: "zoo",
            additional_parameters: "",
        }
    } else if has("MS Outlook Express DBX file") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("bzip2 compressed data") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "bz2",
        }
    } else if has("ASCII cpio archive") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("gzip compressed") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "gz",
        }
    } else if has("LZX compressed archive") {
        Action::Extract {
            type_key: "lzx",
            additional_parameters: "",
        }
    } else if has("ar archive") || has("ARJ archive") {
        // Two distinct source Cases, each dispatching to $TYPE_7Z with
        // no forced-format hint -- combined since the outcome is
        // identical. This is also the case that shadows "POSIX tar
        // archive" below (see that case's own comment).
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("POSIX tar archive") {
        // Unreachable in practice -- "tar archive" always contains "ar
        // archive" as a substring, so the case above always matches
        // first. Kept (not merged away) to document the source's own
        // dead case rather than silently dropping it; see
        // posix_tar_archive_case_is_shadowed_by_the_earlier_ar_archive_case.
        Action::Extract {
            type_key: "7z",
            additional_parameters: "tar",
        }
    } else if has("LHa") && has("archive data") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("Macromedia Flash data") {
        Action::Extract {
            type_key: "swf",
            additional_parameters: "",
        }
    } else if has("PowerISO Direct-Access-Archive") {
        Action::ExtractDiskImage { type_key: "daa" }
    } else if has("sfArk compressed Soundfont") {
        Action::Extract {
            type_key: "sfark",
            additional_parameters: "",
        }
    } else if has("SQLite") {
        Action::Extract {
            type_key: "sqlite",
            additional_parameters: "",
        }
    } else if has("XZ compressed data") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "xz",
        }
    } else if has("MS Windows HtmlHelp Data") {
        Action::Extract {
            type_key: "chm",
            additional_parameters: "",
        }
    } else if has("MIME entity text") || has("mhtml") {
        Action::Extract {
            type_key: "7z",
            additional_parameters: "",
        }
    } else if has("MoPaQ") {
        Action::CheckTotalObserver
    } else if has("MIME entity") {
        Action::Check7z
    } else if (has("RIFF") && !has("WAVE audio"))
        || has("MPEG v")
        || has("MPEG sequence")
        || has("Microsoft ASF")
        || has("GIF image")
        || has("PNG image")
        || has("MNG video")
        || has("ISO Media, MP4")
    {
        Action::Extract {
            type_key: "video",
            additional_parameters: "",
        }
    } else if has("AAC,")
        || has("FLAC audio")
        || has("Ogg data, Vorbis audio")
        || has("Audio file")
        || has("Dolby Digital stream")
    {
        // Four distinct source Cases, each dispatching to $TYPE_AUDIO --
        // combined here since the outcome is identical; order among
        // these four doesn't matter, only their combined position
        // relative to the surrounding cases (preserved).
        Action::Extract {
            type_key: "audio",
            additional_parameters: "",
        }
    } else if has("ISO") && has("filesystem") {
        Action::CheckIso
    } else {
        Action::Fallback
    }
}

/// Ports `filecompare`'s two trailing `If` checks (UniExtract.au3:1470-
/// 1486), which run after the `Select` in the source's own straight-line
/// function body. A caller applies this after any [`Action`] that
/// doesn't itself terminate — see the module doc comment for which.
///
/// `"CDF V2 document"` is checked first and, when it matches, skips both
/// of the checks below it entirely (`Return`) — modeled as `None` here,
/// the same outcome as no match at all, but the *priority* is real:
/// preserved by checking it before the not-packed group rather than
/// folding it into the same boolean expression.
pub fn trailing_termination(file_type: &str) -> Option<Status> {
    let s = file_type.to_lowercase();
    let has = |needle: &str| s.contains(&needle.to_lowercase());

    if has("CDF V2 document") {
        return None;
    }

    let not_packed = (has("text") && (has("CRLF") || has("long lines") || has("ASCII")))
        || has("batch file")
        || has("XML")
        || has("HTML")
        || has("source")
        || has("Rich ")
        || has("icon resource")
        || (has("bitmap") && !has("MGR bitmap"))
        || has("WAVE audio")
        || has("boot sector;")
        || has("shortcut")
        || has("empty")
        || has("directory")
        || has("BitTorrent file")
        || has("Standard MIDI data")
        || has("MSVC program database");
    if not_packed {
        return Some(Status::NotPacked);
    }

    if has("MS-DOS executable") {
        return Some(Status::NotSupported);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_zip_archive_data_routes_to_7z() {
        assert_eq!(
            classify("7-zip archive data, version 0.4"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: ""
            }
        );
    }

    /// Parity test for capability C041: plain ZIP requires "Zip archive
    /// data" with no "7" anywhere in the description -- a variant that
    /// also mentions "7" (e.g. `file` describing it as 7-Zip-adjacent
    /// without matching the exact "7-zip archive data"/"7 zip archive
    /// data" wording case 1 checks for) falls through to `Fallback`
    /// rather than being misclassified as a plain zip.
    #[test]
    fn zip_archive_data_excludes_the_7_marker() {
        assert_eq!(
            classify("Zip archive data, at least v2.0"),
            Action::Extract {
                type_key: "zip",
                additional_parameters: ""
            }
        );
        assert_eq!(
            classify("Zip archive data, 7-zip SFX wrapped"),
            Action::Fallback
        );
    }

    /// Parity test for capability C041: bzip2/gzip/xz pass a
    /// forced-format hint as `extract()`'s third argument, feeding
    /// `extract::sevenzip`'s gzip-family branch (C056).
    #[test]
    fn compressed_stream_cases_forward_the_format_hint() {
        assert_eq!(
            classify("bzip2 compressed data, block size = 900k"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "bz2"
            }
        );
        assert_eq!(
            classify("gzip compressed data, from Unix"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "gz"
            }
        );
        assert_eq!(
            classify("XZ compressed data"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: "xz"
            }
        );
    }

    /// Parity test for capability C041, a genuine quirk in the source
    /// itself rather than a modeling choice: `"tar archive"` always
    /// contains `"ar archive"` as a literal substring (`"tar"`'s last
    /// two letters plus the following `" archive"`), and the `"ar
    /// archive"` case (UniExtract.au3:1424-1425) is checked three
    /// cases before `"POSIX tar archive"` (1428-1429). So the later,
    /// more specific case is unreachable in practice -- any `file`
    /// output naming a POSIX tar archive matches `"ar archive"` first
    /// and gets no forced-format hint, not `"tar"`. Preserved exactly
    /// rather than "fixed", since a faithful port keeps the source's
    /// unreachable code unreachable too.
    #[test]
    fn posix_tar_archive_case_is_shadowed_by_the_earlier_ar_archive_case() {
        assert_eq!(
            classify("POSIX tar archive"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: ""
            }
        );
    }

    #[test]
    fn disk_image_case_uses_extract_disk_image_not_extract() {
        assert_eq!(
            classify("PowerISO Direct-Access-Archive"),
            Action::ExtractDiskImage { type_key: "daa" }
        );
    }

    /// Parity test for capability C041: "MIME entity text"/"mhtml" is
    /// checked before the plainer "MIME entity" case, so a match on
    /// the more specific text never falls through to `check7z`.
    #[test]
    fn mime_entity_text_takes_priority_over_plain_mime_entity() {
        assert_eq!(
            classify("MIME entity text, ISO-8859 text"),
            Action::Extract {
                type_key: "7z",
                additional_parameters: ""
            }
        );
        assert_eq!(classify("MIME entity, with headers"), Action::Check7z);
    }

    #[test]
    fn mopaq_routes_to_check_total_observer() {
        assert_eq!(classify("MoPaQ (MPQ) archive"), Action::CheckTotalObserver);
    }

    /// Parity test for capability C041: the RIFF/media group excludes
    /// RIFF's own WAVE-audio subtype (handled separately below it).
    #[test]
    fn riff_media_group_excludes_wave_audio() {
        for sample in [
            "RIFF (little-endian) data",
            "MPEG v4 system",
            "MPEG sequence, v2",
            "Microsoft ASF",
            "GIF image data",
            "PNG image data",
            "MNG video data",
            "ISO Media, MP4 Base Media",
        ] {
            assert_eq!(
                classify(sample),
                Action::Extract {
                    type_key: "video",
                    additional_parameters: ""
                }
            );
        }
        assert_eq!(
            classify("RIFF (little-endian) data, WAVE audio"),
            Action::Fallback
        );
    }

    #[test]
    fn iso_filesystem_routes_to_check_iso() {
        assert_eq!(
            classify("ISO 9660 CD-ROM filesystem data"),
            Action::CheckIso
        );
    }

    #[test]
    fn unrecognized_text_falls_back_to_userdefcompare() {
        assert_eq!(
            classify("some completely unknown signature"),
            Action::Fallback
        );
    }

    #[test]
    fn cdf_v2_document_skips_both_trailing_checks() {
        // Even though "CDF V2 document" text also happens to satisfy
        // nothing else here, this asserts the *priority*: it must be
        // checked, and short-circuit, before the not-packed group.
        assert_eq!(trailing_termination("CDF V2 document, Little Endian"), None);
    }

    #[test]
    fn not_packed_group_matches_any_listed_signature() {
        for sample in [
            "ASCII text, with CRLF line terminators",
            "a batch file for MS-DOS",
            "XML document text",
            "HTML document text",
            "C source, ASCII text",
            "Rich Text Format data",
            "MS Windows icon resource",
            "PC bitmap, Windows 3.x format",
            "RIFF (little-endian) data, WAVE audio",
            "x86 boot sector; partition",
            "MS Windows shortcut",
            "empty",
            "directory",
            "BitTorrent file",
            "Standard MIDI data",
            "MSVC program database",
        ] {
            assert_eq!(trailing_termination(sample), Some(Status::NotPacked));
        }
    }

    /// Parity test for capability C041: the "bitmap" not-packed rule
    /// excludes "MGR bitmap" specifically.
    #[test]
    fn not_packed_bitmap_rule_excludes_mgr_bitmap() {
        assert_eq!(trailing_termination("MGR bitmap, big-endian"), None);
    }

    #[test]
    fn ms_dos_executable_is_not_supported() {
        assert_eq!(
            trailing_termination("MS-DOS executable, MZ for MS-DOS"),
            Some(Status::NotSupported)
        );
    }

    #[test]
    fn unrelated_text_has_no_trailing_termination() {
        assert_eq!(trailing_termination("data"), None);
    }
}
