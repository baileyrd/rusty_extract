//! ARJ SFX verification: UniExtract2's `checkArj` (UniExtract.au3:1958-1972)
//! — attempt an ARJ listing and see if `arj.exe` recognizes the file as a
//! self-extracting ARJ archive, the same shape as `detection::alz_probe`'s
//! `CheckAlz` (C059) and `detection::sevenzip_probe`'s `check7z` (C048).

use crate::extract::{Invocation, WindowMode};

/// Builds the probe invocation `checkArj` (UniExtract.au3:1963) makes:
/// `<arj> l "<file>"`, run in the file's own directory with the window
/// hidden. Listing, not extracting — this step only asks arj "can you open
/// this at all?".
pub fn probe_invocation(program: &str, file: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["l".to_string(), file.to_string()],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// UniExtract.au3:1965's exact predicate: `Archive created:` must appear in
/// the captured `l` output — matching `StringInStr($return, "Archive
/// created:", 0)`. Case-insensitive: the source's explicit `0`
/// case-sensitivity argument, the same AutoIt default already documented
/// for every other bare/explicit-`0` `StringInStr` call this port has
/// encountered (C007-C013, C144, C145, C147).
///
/// The routing decision this feeds — `extract($TYPE_7Z, ...)` when `true`
/// (UniExtract.au3:1966) — is a recursive re-dispatch into the 7-Zip
/// extractor (composite/recursive dispatch, capability C054, not yet
/// ported), not this probe's job, the same "probe vs. dispatch" boundary
/// `detection::alz_probe::is_alz_archive` already draws.
pub fn is_arj_sfx(listing_output: &str) -> bool {
    listing_output.to_lowercase().contains("archive created:")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C061: the probe invocation matches
    /// `checkArj`'s `FetchStdout($arj & ' l "' & $file & '"', $filedir,
    /// @SW_HIDE)` exactly.
    #[test]
    fn probe_invocation_matches_source() {
        let inv = probe_invocation(
            r"C:\UniExtract\bin\arj.exe",
            r"C:\downloads\archive.exe",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\arj.exe");
        assert_eq!(
            inv.args,
            vec!["l".to_string(), r"C:\downloads\archive.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn recognizes_a_valid_arj_listing() {
        assert!(is_arj_sfx(
            "ARJ32 2.87\nArchive created: 2001-01-01\n\nfilename1\n"
        ));
    }

    #[test]
    fn matches_case_insensitively() {
        assert!(is_arj_sfx("ARCHIVE CREATED: 2001-01-01\n"));
    }

    #[test]
    fn rejects_output_missing_the_created_header() {
        assert!(!is_arj_sfx("ARJ32 2.87\nnot an arj sfx\n"));
        assert!(!is_arj_sfx(""));
    }
}
