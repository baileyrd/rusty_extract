//! Blind 7-Zip probe fallback: the final catch-all UniExtract2 tries when
//! every other detector has failed to identify a file — attempt a 7-Zip
//! listing and see if 7-Zip itself recognizes the file as *something*.

use crate::extract::{Invocation, WindowMode};

/// Builds the probe invocation UniExtract2's `check7z`
/// (UniExtract.au3:1917-1942) makes: `<7z> l "<file>"`, run in the file's
/// own directory with the window hidden. Listing, not extracting — this
/// step only asks 7-Zip "can you open this at all?".
pub fn probe_invocation(seven_zip_program: &str, file: &str, file_dir: &str) -> Invocation {
    Invocation {
        program: seven_zip_program.to_string(),
        args: vec!["l".to_string(), file.to_string()],
        working_dir: file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// The routing decision `check7z` makes from `l`'s captured stdout, without
/// actually calling `extract()`/`extractDiskImage()` — wiring those up is
/// the responsibility of the extractor dispatcher (C049) and disk-image
/// chaining (C054) capabilities, not this probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// 7-Zip could not open the file — `check7z` returns `False`.
    NotAnArchive,
    /// `is_disk_image` was set: dispatch through `extractDiskImage`.
    DiskImage,
    /// A custom display string was supplied by the caller: dispatch with
    /// that display name.
    CustomDisplay(String),
    /// The file is a `.exe` whose listing mentions "InstallShield": the
    /// source also runs `CheckInstallShieldCab()` before treating it as a
    /// 7-Zip-extractable installer package.
    InstallerPackage { needs_installshield_check: bool },
    /// Plain archive, no special-casing.
    GenericArchive,
}

/// Reimplements `check7z`'s branching (UniExtract.au3:1924-1936) purely
/// from its inputs: the captured `7z l` stdout, whether this probe was made
/// on the disk-image path, an optional caller-supplied display string, and
/// the file's extension (already lowercased, as the source always has it
/// by this point — see C175/C176's `$fileext = StringLower(...)`).
pub fn route(
    listing_output: &str,
    is_disk_image: bool,
    arcdisp: Option<&str>,
    file_ext: &str,
) -> ProbeOutcome {
    if !is_valid_archive(listing_output) {
        return ProbeOutcome::NotAnArchive;
    }
    if is_disk_image {
        return ProbeOutcome::DiskImage;
    }
    if let Some(display) = arcdisp {
        return ProbeOutcome::CustomDisplay(display.to_string());
    }
    if file_ext == "exe" {
        return ProbeOutcome::InstallerPackage {
            needs_installshield_check: listing_output.contains("InstallShield"),
        };
    }
    ProbeOutcome::GenericArchive
}

/// UniExtract.au3:1924's exact predicate: `Listing archive:` must appear,
/// and it must NOT be the case that both `Errors: ` and `Can not open the
/// file as ` appear (7-Zip prints a listing header even for some files it
/// then reports it couldn't actually open).
fn is_valid_archive(listing_output: &str) -> bool {
    let has_listing_header = listing_output.contains("Listing archive:");
    let has_open_error =
        listing_output.contains("Errors: ") && listing_output.contains("Can not open the file as ");
    has_listing_header && !has_open_error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_invocation_matches_source() {
        let inv = probe_invocation(
            r"C:\UniExtract\bin\7z.exe",
            r"C:\downloads\mystery.bin",
            r"C:\downloads",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec!["l".to_string(), r"C:\downloads\mystery.bin".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    const VALID_LISTING: &str =
        "7-Zip [64] ...\n\nListing archive: mystery.bin\n\n--\nPath = mystery.bin\nType = 7z\n";

    /// Parity test for capability C048: routing matches UniExtract.au3's
    /// `check7z` branch-by-branch.
    #[test]
    fn not_an_archive_when_listing_header_missing() {
        let outcome = route(
            "Errors: 1\nCan not open the file as archive",
            false,
            None,
            "bin",
        );
        assert_eq!(outcome, ProbeOutcome::NotAnArchive);
    }

    #[test]
    fn not_an_archive_when_open_error_present_despite_listing_header() {
        let output = "Listing archive: foo\n\nErrors: 1\nCan not open the file as archive\n";
        assert_eq!(
            route(output, false, None, "bin"),
            ProbeOutcome::NotAnArchive
        );
    }

    #[test]
    fn disk_image_takes_precedence_over_custom_display() {
        assert_eq!(
            route(VALID_LISTING, true, Some("ignored"), "iso"),
            ProbeOutcome::DiskImage
        );
    }

    #[test]
    fn custom_display_used_when_supplied() {
        assert_eq!(
            route(VALID_LISTING, false, Some("Weird Archive"), "bin"),
            ProbeOutcome::CustomDisplay("Weird Archive".to_string())
        );
    }

    #[test]
    fn exe_extension_flags_installshield_check_only_when_mentioned() {
        let with_is = format!("{VALID_LISTING}InstallShield Cabinet\n");
        assert_eq!(
            route(&with_is, false, None, "exe"),
            ProbeOutcome::InstallerPackage {
                needs_installshield_check: true
            }
        );
        assert_eq!(
            route(VALID_LISTING, false, None, "exe"),
            ProbeOutcome::InstallerPackage {
                needs_installshield_check: false
            }
        );
    }

    #[test]
    fn plain_archive_falls_through_to_generic() {
        assert_eq!(
            route(VALID_LISTING, false, None, "dat"),
            ProbeOutcome::GenericArchive
        );
    }
}
