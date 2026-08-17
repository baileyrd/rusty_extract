//! Blind ALZip probe: UniExtract2's `CheckAlz` (UniExtract.au3:1945-1956)
//! — attempt an ALZ listing and see if `unalz.exe` recognizes the file as
//! an ALZip archive, the same shape as `detection::sevenzip_probe`'s
//! `check7z` (C048) for 7-Zip.

use crate::extract::{Invocation, WindowMode};

/// Builds the probe invocation `CheckAlz` (UniExtract.au3:1949) makes:
/// `<unalz> -l "<file>"`, run in the file's own directory with the window
/// hidden. Listing, not extracting — this step only asks unalz "can you
/// open this at all?".
pub fn probe_invocation(program: &str, file: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-l".to_string(), file.to_string()],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// UniExtract.au3:1953's exact predicate: `Listing archive:` must appear,
/// and neither `corrupted file` nor `file open error` may appear in the
/// captured `-l` output — matching `StringInStr($return, "Listing
/// archive:") And Not (StringInStr($return, "corrupted file") Or
/// StringInStr($return, "file open error"))`.
///
/// The routing decision this feeds — `extract($TYPE_ALZ, -1)` when `true`
/// (UniExtract.au3:1954) — is the extractor dispatcher's job (C049,
/// already done: `$TYPE_ALZ` has no hardcoded case, so it falls through to
/// the plugin path, `extract::alz`), not this probe's.
pub fn is_alz_archive(listing_output: &str) -> bool {
    listing_output.contains("Listing archive:")
        && !listing_output.contains("corrupted file")
        && !listing_output.contains("file open error")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C059: the probe invocation matches
    /// `CheckAlz`'s `FetchStdout($alz & ' -l "' & $file & '"', $filedir,
    /// @SW_HIDE)` exactly.
    #[test]
    fn probe_invocation_matches_source() {
        let inv = probe_invocation(
            r"C:\UniExtract\bin\unalz.exe",
            r"C:\downloads\archive.alz",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\unalz.exe");
        assert_eq!(
            inv.args,
            vec!["-l".to_string(), r"C:\downloads\archive.alz".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn recognizes_a_valid_alz_listing() {
        assert!(is_alz_archive(
            "unalz 0.61\nListing archive: archive.alz\n\nfilename1\nfilename2\n"
        ));
    }

    #[test]
    fn rejects_output_missing_the_listing_header() {
        assert!(!is_alz_archive("unalz 0.61\nnot an alz file\n"));
    }

    #[test]
    fn rejects_a_listing_reporting_a_corrupted_file() {
        assert!(!is_alz_archive(
            "Listing archive: archive.alz\ncorrupted file\n"
        ));
    }

    #[test]
    fn rejects_a_listing_reporting_a_file_open_error() {
        assert!(!is_alz_archive(
            "Listing archive: archive.alz\nfile open error\n"
        ));
    }
}
