//! Plugin extension point placeholder substitution — the `%placeholder%`
//! mechanism `def/*.ini` plugin definitions use in their `parameters` and
//! `workingdir` values (see `extract::plugin_config::PluginConfig`, C052).
//!
//! Ports `ReplacePlaceholders` (UniExtract.au3:3523-3541) exactly: five
//! named placeholders are substituted directly from extraction-context
//! values, then every remaining `%...%` pair is treated as a
//! translation-key placeholder (e.g. `%TERM_ARCHIVE%`) and resolved via
//! `t($sPlaceholder)`.
//!
//! Translation-catalog resolution itself is a separate, deferred subsystem
//! (out of scope for this migration — see `capability-manifest.md`'s
//! OUT-OF-SCOPE rows), so [`replace_placeholders`] takes the resolver as a
//! `translate` closure rather than reading language files itself — the same
//! "caller supplies the resolved value" approach already used for
//! `extract::pdf::to_png_invocation`'s `term_page` parameter. Source's own
//! fallback when a translation is missing (`t`, UniExtract.au3:559-586)
//! returns the key unchanged (`$sDefault == 0? $t: $sDefault` with the
//! default `$sDefault` left at `0`), so a `translate` closure that mirrors
//! that — returning its input unchanged when it has nothing better — matches
//! source behavior for the "no catalog available" case.

/// Wraps `s` in literal double quotes, matching `Quote($sString, $bDouble =
/// False)` (UniExtract.au3:3598-3600) at its default (`$bDouble = False`):
/// `'"' & $sString & '"'`.
fn quote(s: &str) -> String {
    format!("\"{s}\"")
}

/// Returns every substring found between successive pairs of `%` in `s`, in
/// order, matching `_StringBetween($sString, "%", "%")`'s non-overlapping
/// pairing: scan for the next `%`, then the `%` after it, extract what's
/// between, and resume scanning immediately after the second `%` — not from
/// inside the extracted text.
fn percent_delimited_segments(s: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        let after_start = &rest[start + 1..];
        match after_start.find('%') {
            Some(end) => {
                segments.push(&after_start[..end]);
                rest = &after_start[end + 1..];
            }
            None => break,
        }
    }
    segments
}

/// The five extraction-context values `ReplacePlaceholders` substitutes
/// directly — `$file`/`$outdir`/`$filename`/`$fileext`/`$filedir`, the same
/// global variables every other `extract::*` module takes as explicit
/// parameters instead (see `ARCHITECTURE.md`). Bundled into one struct here
/// purely to keep [`replace_placeholders`]'s parameter list manageable —
/// this isn't a type the rest of the port shares.
#[derive(Debug, Clone, Copy)]
pub struct PlaceholderContext<'a> {
    pub file: &'a str,
    pub outdir: &'a str,
    pub filename: &'a str,
    pub fileext: &'a str,
    pub filedir: &'a str,
}

/// Substitutes every `%placeholder%` in `s`, matching `ReplacePlaceholders`
/// (UniExtract.au3:3523-3541) exactly:
///
/// 1. If `s` contains no `%` at all, it's returned unchanged (no-op fast
///    path, matching the source's own early return).
/// 2. `%filename%`, `%fileext%`, `%filedir%` are replaced with `ctx`'s
///    matching fields verbatim.
/// 3. `%file%` and `%outdir%` are replaced with `ctx.file`/`ctx.outdir`,
///    wrapped in literal double quotes when `quote_values` is `true`
///    (matching the source's `$bQuote? Quote($file): $file` — callers
///    building a quoted command-line argument pass `true`;
///    `ReplacePlaceholders` itself defaults `$bQuote` to `True`, so `true`
///    is the common case).
/// 4. Every remaining `%...%` pair (anything not one of the five named
///    placeholders above, which are already gone by this point) is treated
///    as a translation-key placeholder: if the text between the `%`s
///    contains a space, it's left untouched (matching `If StringInStr(...,
///    " ") Then ContinueLoop` — a real `%` used as a literal percent sign in
///    running text, not a placeholder); otherwise it's replaced with
///    `translate(key)`.
pub fn replace_placeholders(
    s: &str,
    quote_values: bool,
    ctx: PlaceholderContext,
    translate: impl Fn(&str) -> String,
) -> String {
    if !s.contains('%') {
        return s.to_string();
    }

    let mut result = s
        .replace("%filename%", ctx.filename)
        .replace("%fileext%", ctx.fileext)
        .replace("%filedir%", ctx.filedir);

    let file_value = if quote_values {
        quote(ctx.file)
    } else {
        ctx.file.to_string()
    };
    result = result.replace("%file%", &file_value);
    let outdir_value = if quote_values {
        quote(ctx.outdir)
    } else {
        ctx.outdir.to_string()
    };
    result = result.replace("%outdir%", &outdir_value);

    let remaining: Vec<String> = percent_delimited_segments(&result)
        .into_iter()
        .map(str::to_string)
        .collect();
    for placeholder in remaining {
        if placeholder.contains(' ') {
            continue;
        }
        let token = format!("%{placeholder}%");
        result = result.replace(&token, &translate(&placeholder));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_CTX: PlaceholderContext = PlaceholderContext {
        file: "/f/archive.rar",
        outdir: "/out",
        filename: "archive",
        fileext: "rar",
        filedir: "/f",
    };

    const PLAIN_CTX: PlaceholderContext = PlaceholderContext {
        file: "f",
        outdir: "o",
        filename: "n",
        fileext: "e",
        filedir: "d",
    };

    /// A string with no `%` at all is returned unchanged — the source's own
    /// fast path (`If Not StringInStr($sString, "%") Then Return $sString`).
    #[test]
    fn no_percent_sign_returns_unchanged() {
        assert_eq!(
            replace_placeholders("plain text", true, PLAIN_CTX, |k| k.to_string()),
            "plain text"
        );
    }

    /// Parity test for capability C182: `%filename%`/`%fileext%`/`%filedir%`
    /// substitute verbatim, matching `ReplacePlaceholders`'s first three
    /// `StringReplace` calls.
    #[test]
    fn substitutes_filename_fileext_filedir_verbatim() {
        let result =
            replace_placeholders("%filename%.%fileext% in %filedir%", true, REAL_CTX, |k| {
                k.to_string()
            });
        assert_eq!(result, "archive.rar in /f");
    }

    /// `%file%`/`%outdir%` are quoted when `quote_values` is `true` —
    /// matching `Quote($file)`/`Quote($outdir)` (`$bQuote` defaults to
    /// `True` in the source).
    #[test]
    fn quotes_file_and_outdir_when_quote_values_is_true() {
        let result =
            replace_placeholders("-x %file% -o %outdir%", true, REAL_CTX, |k| k.to_string());
        assert_eq!(result, "-x \"/f/archive.rar\" -o \"/out\"");
    }

    /// `%file%`/`%outdir%` are left unquoted when `quote_values` is `false`
    /// — matching a caller passing `$bQuote = False`.
    #[test]
    fn leaves_file_and_outdir_unquoted_when_quote_values_is_false() {
        let result = replace_placeholders("%file% %outdir%", false, REAL_CTX, |k| k.to_string());
        assert_eq!(result, "/f/archive.rar /out");
    }

    /// Any remaining `%...%` pair after the five named placeholders are gone
    /// is resolved via the `translate` closure — matching the source's
    /// fallback to `t($sPlaceholder)` for arbitrary translation-key
    /// placeholders like `%TERM_ARCHIVE%`.
    #[test]
    fn resolves_remaining_placeholders_via_translate_closure() {
        let result = replace_placeholders("Extracting %TERM_ARCHIVE%", true, PLAIN_CTX, |k| {
            if k == "TERM_ARCHIVE" {
                "Archive".to_string()
            } else {
                k.to_string()
            }
        });
        assert_eq!(result, "Extracting Archive");
    }

    /// A `%...%` pair whose contents include a space is left untouched —
    /// matching `If StringInStr($sPlaceholder, " ") Then ContinueLoop`,
    /// which treats it as a literal percent sign in running text rather
    /// than a placeholder — while a later, space-free pair in the same
    /// string still resolves normally.
    #[test]
    fn leaves_percent_pairs_containing_a_space_untouched() {
        let result = replace_placeholders("keep %A B% but replace %C%", true, PLAIN_CTX, |k| {
            format!("<{k}>")
        });
        assert_eq!(result, "keep %A B% but replace <C>");
    }

    /// A translate closure that mirrors the source's own missing-key
    /// fallback (return the key unchanged) is a valid, source-matching way
    /// to call this when no translation catalog is available — this is the
    /// intended default for callers that haven't ported the translation
    /// subsystem (out of scope for this migration).
    #[test]
    fn identity_translate_matches_source_fallback_for_missing_translations() {
        let result =
            replace_placeholders("%SOME_UNKNOWN_TERM%", true, PLAIN_CTX, |k| k.to_string());
        assert_eq!(result, "SOME_UNKNOWN_TERM");
    }
}
