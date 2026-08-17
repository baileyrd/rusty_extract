//! Xpdf tools (`pdfdetach.exe`, `pdftohtml.exe`, `pdftopng.exe`,
//! `pdftotext.exe`) — PDF content/attachment/text/image extraction, 4
//! sequential invocations.
//!
//! UniExtract.au3:2967-2971's `Case $TYPE_PDF` makes 4 independent `_Run`
//! calls in sequence, all sharing `outdir` as working directory and
//! `@SW_HIDE` as window mode:
//!
//! ```text
//! Case $TYPE_PDF
//!     _Run($pdfdetach & ' -saveall "' & $file & '"', $outdir, @SW_HIDE, True, True, False, False)
//!     _Run($pdftohtml & ' "' & $file & '" "' & $outdir & '\' & $filename & '-HTML"', $outdir, @SW_HIDE, True, True, False, False)
//!     _Run($pdftopng & ' "' & $file & '" "' & $outdir & '\' & $filename & '-' & t('TERM_PAGE') & '"', $outdir, @SW_HIDE, True, True, False, False)
//!     _Run($pdftotext & ' "' & $file & '" "' & $outdir & '\' & $filename & '.txt"', $outdir, @SW_HIDE, True, True, False, False)
//! ```
//!
//! Unlike most other extractor-integration modules in this repo, which
//! build a single [`Invocation`] per capability, this module exposes one
//! function per tool — `detach_invocation`, `to_html_invocation`,
//! `to_png_invocation`, `to_text_invocation` — since each of the source's
//! 4 `_Run` calls builds a genuinely separate `Invocation` for a distinct
//! program, not 4 variations of one shape.
//!
//! **No `extract::dispatch::HARDCODED_CASES` entry:** that table maps one
//! `$arctype` key to one Rust module/invocation shape (see its module doc
//! comment), and `$TYPE_PDF`'s 4-invocation case doesn't fit that model —
//! a bare `"pdf" -> extract::pdf` entry would misrepresent which of the 4
//! functions actually runs. Registering it accurately requires
//! `HARDCODED_CASES` to gain multi-invocation support first. This is the
//! same reasoning already used to exclude `extract::xor` and
//! `extract::unzip` (see their module doc comments) for the same kind of
//! composite/non-single-shape case.

use super::{Invocation, WindowMode};

/// Builds the invocation UniExtract.au3:2967's `_Run($pdfdetach & '
/// -saveall "' & $file & '"', $outdir, @SW_HIDE, True, True, False,
/// False)` makes: `<program> -saveall "<file>"`, run in `outdir` with the
/// window hidden. Saves any file attachments embedded in the PDF.
pub fn detach_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-saveall".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the invocation UniExtract.au3:2968's `_Run($pdftohtml & ' "' &
/// $file & '" "' & $outdir & '\' & $filename & '-HTML"', $outdir,
/// @SW_HIDE, True, True, False, False)` makes: `<program> "<file>"
/// "<outdir>\<filename>-HTML"`, run in `outdir` with the window hidden.
/// Converts the PDF to HTML.
pub fn to_html_invocation(program: &str, file: &str, outdir: &str, filename: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), format!("{outdir}\\{filename}-HTML")],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the invocation UniExtract.au3:2969's `_Run($pdftopng & ' "' &
/// $file & '" "' & $outdir & '\' & $filename & '-' & t('TERM_PAGE') &
/// '"', $outdir, @SW_HIDE, True, True, False, False)` makes: `<program>
/// "<file>" "<outdir>\<filename>-<term_page>"`, run in `outdir` with the
/// window hidden. Renders the PDF's pages to PNG images.
///
/// `term_page` is the caller-resolved value of the source's `t('TERM_PAGE')`
/// call — the localized UI string for "Page" (e.g. `"Page"` in English).
/// Resolving translation-catalog terms is out of scope for this migration
/// (see `capability-manifest.md`'s OUT-OF-SCOPE rows for the deferred
/// translation subsystem), so this function takes the already-resolved
/// string as a parameter rather than resolving it itself, the same way it
/// already takes `filename` and `outdir` as parameters instead of deriving
/// them internally — this keeps the invocation-builder translation-agnostic,
/// matching the source's own separation between `t(...)` resolution and the
/// `_Run` call site.
pub fn to_png_invocation(
    program: &str,
    file: &str,
    outdir: &str,
    filename: &str,
    term_page: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            file.to_string(),
            format!("{outdir}\\{filename}-{term_page}"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the invocation UniExtract.au3:2970's `_Run($pdftotext & ' "' &
/// $file & '" "' & $outdir & '\' & $filename & '.txt"', $outdir,
/// @SW_HIDE, True, True, False, False)` makes: `<program> "<file>"
/// "<outdir>\<filename>.txt"`, run in `outdir` with the window hidden.
/// Extracts the PDF's text content.
pub fn to_text_invocation(program: &str, file: &str, outdir: &str, filename: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![file.to_string(), format!("{outdir}\\{filename}.txt")],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C089: the constructed invocation matches
    /// UniExtract.au3:2967's `_Run($pdfdetach & ' -saveall "' & $file &
    /// '"', $outdir, @SW_HIDE, True, True, False, False)` — same program,
    /// same `-saveall` switch and file argument order, same working
    /// directory, same hidden window.
    #[test]
    fn detach_matches_source_invocation() {
        let inv = detach_invocation(
            r"C:\UniExtract\bin\pdfdetach.exe",
            r"C:\downloads\document.pdf",
            r"C:\downloads\document_unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\pdfdetach.exe");
        assert_eq!(
            inv.args,
            vec![
                "-saveall".to_string(),
                r"C:\downloads\document.pdf".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\document_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C089: the constructed invocation matches
    /// UniExtract.au3:2968's `_Run($pdftohtml & ' "' & $file & '" "' &
    /// $outdir & '\' & $filename & '-HTML"', $outdir, @SW_HIDE, True,
    /// True, False, False)` — same program, same file and explicit
    /// `<filename>-HTML` output path arguments, same working directory,
    /// same hidden window.
    #[test]
    fn to_html_matches_source_invocation() {
        let inv = to_html_invocation(
            r"C:\UniExtract\bin\pdftohtml.exe",
            r"C:\downloads\document.pdf",
            r"C:\downloads\document_unpacked",
            "document",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\pdftohtml.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\document.pdf".to_string(),
                r"C:\downloads\document_unpacked\document-HTML".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\document_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C089: the constructed invocation matches
    /// UniExtract.au3:2969's `_Run($pdftopng & ' "' & $file & '" "' &
    /// $outdir & '\' & $filename & '-' & t('TERM_PAGE') & '"', $outdir,
    /// @SW_HIDE, True, True, False, False)` — same program, same file and
    /// explicit `<filename>-<term_page>` output path arguments (using a
    /// representative caller-resolved `term_page` value in place of
    /// `t('TERM_PAGE')`), same working directory, same hidden window.
    #[test]
    fn to_png_matches_source_invocation() {
        let inv = to_png_invocation(
            r"C:\UniExtract\bin\pdftopng.exe",
            r"C:\downloads\document.pdf",
            r"C:\downloads\document_unpacked",
            "document",
            "Page",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\pdftopng.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\document.pdf".to_string(),
                r"C:\downloads\document_unpacked\document-Page".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\document_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C089: the constructed invocation matches
    /// UniExtract.au3:2970's `_Run($pdftotext & ' "' & $file & '" "' &
    /// $outdir & '\' & $filename & '.txt"', $outdir, @SW_HIDE, True, True,
    /// False, False)` — same program, same file and explicit
    /// `<filename>.txt` output path arguments, same working directory,
    /// same hidden window.
    #[test]
    fn to_text_matches_source_invocation() {
        let inv = to_text_invocation(
            r"C:\UniExtract\bin\pdftotext.exe",
            r"C:\downloads\document.pdf",
            r"C:\downloads\document_unpacked",
            "document",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\pdftotext.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\document.pdf".to_string(),
                r"C:\downloads\document_unpacked\document.txt".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\document_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }
}
