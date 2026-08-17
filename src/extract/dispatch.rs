//! Central extractor dispatcher: routes an extractor-type key to whichever
//! Rust module implements it, or signals that no hardcoded case exists and
//! the caller should fall through to the `def/*.ini` plugin engine.
//!
//! Mirrors UniExtract.au3:2269-3441's `extract($arctype, ...)` — a single
//! function with a `Switch $arctype` of ~70 `Case`s, falling through to
//! `pluginExtract($arctype, ...)` (`Case Else`, ported separately as C050)
//! for any type string not hardcoded. This module ports the *routing
//! decision*, not the invocation-building call: each extractor case takes
//! different explicit inputs (compare `extract::rgss::invocation`'s
//! `(program, file, outdir)` against `extract::rpa::invocation`'s
//! `(program, script_dir, file, outdir)`, which needs one more parameter
//! the source reads from its `@ScriptDir` global) — the source hides that
//! divergence behind global variables every `Case` reads from directly;
//! this port keeps every function's inputs explicit instead (see
//! `ARCHITECTURE.md`), so there is no single uniform call signature to
//! dispatch to yet. Wiring a resolved [`HardcodedCase`] to its actual
//! invocation call is left to a future integration point once enough
//! extractors exist to know what that call site should look like.
//!
//! [`HARDCODED_CASES`] is a hand-maintained list, not reflection or
//! auto-registration — each extractor-integration capability's own PR adds
//! its one line here, the same way the source's `Switch` grew `Case` by
//! `Case`. **Only the extractor-type keys actually ported so far are
//! listed**; a type string the *source* hardcodes but this port hasn't
//! reached yet correctly reports [`DispatchTarget::Plugin`] here — that's
//! this capability's honest current coverage, not a claim that every
//! source `Case` is done (track that in `capability-manifest.md` instead).

/// One dispatch-table entry: an extractor-type key this port recognizes as
/// hardcoded, naming the Rust module that implements it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardcodedCase {
    /// The `$arctype` string constant from the source (e.g. `$TYPE_RGSS`'s
    /// value `"rgss"`).
    pub type_key: &'static str,
    /// The module implementing this case, as it would be written in a
    /// `use` path (e.g. `"extract::rgss"`).
    pub module: &'static str,
}

/// Extractor-type keys this port has a hardcoded case for. Add one entry
/// per extractor-integration capability as it's ported — see the module
/// doc comment for why this is a plain list rather than auto-registration.
pub const HARDCODED_CASES: &[HardcodedCase] = &[
    HardcodedCase {
        type_key: "rgss",
        module: "extract::rgss",
    },
    HardcodedCase {
        type_key: "rpa",
        module: "extract::rpa",
    },
    HardcodedCase {
        type_key: "sfark",
        module: "extract::sfark",
    },
    HardcodedCase {
        type_key: "sis",
        module: "extract::extsis",
    },
    HardcodedCase {
        type_key: "ace",
        module: "extract::ace",
    },
    HardcodedCase {
        type_key: "bcm",
        module: "extract::bcm",
    },
    HardcodedCase {
        type_key: "cic",
        module: "extract::cic",
    },
    HardcodedCase {
        type_key: "chd",
        module: "extract::chdman",
    },
    HardcodedCase {
        type_key: "daa",
        module: "extract::daa",
    },
    HardcodedCase {
        type_key: "freearc",
        module: "extract::freearc",
    },
    HardcodedCase {
        type_key: "fsb",
        module: "extract::fsb",
    },
    HardcodedCase {
        type_key: "garbro",
        module: "extract::garbro",
    },
    HardcodedCase {
        type_key: "isz",
        module: "extract::isz",
    },
    HardcodedCase {
        type_key: "kgb",
        module: "extract::kgb",
    },
    HardcodedCase {
        type_key: "lz",
        module: "extract::lzip",
    },
    HardcodedCase {
        type_key: "lzo",
        module: "extract::lzop",
    },
    HardcodedCase {
        type_key: "lzx",
        module: "extract::lzx",
    },
    HardcodedCase {
        type_key: "mole",
        module: "extract::mole",
    },
    HardcodedCase {
        type_key: "uif",
        module: "extract::uif",
    },
    HardcodedCase {
        type_key: "unreal",
        module: "extract::unreal",
    },
    HardcodedCase {
        type_key: "wix",
        module: "extract::wix",
    },
    HardcodedCase {
        type_key: "wolf",
        module: "extract::wolf",
    },
    HardcodedCase {
        type_key: "zoo",
        module: "extract::zoo",
    },
    HardcodedCase {
        type_key: "zpaq",
        module: "extract::zpaq",
    },
];

/// Where `dispatch` routes an extractor-type key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchTarget {
    /// A hardcoded case exists — matches a `Case $TYPE_*` in the source.
    Hardcoded(HardcodedCase),
    /// No hardcoded case: matches the source's `Case Else`, which falls
    /// through to `pluginExtract` (C050).
    Plugin,
}

/// Routes `extractor_type` the way UniExtract.au3:2288's `Switch $arctype`
/// does: an exact match against a hardcoded case, or `Plugin` for anything
/// else (the `Case Else` fallthrough).
pub fn dispatch(extractor_type: &str) -> DispatchTarget {
    match HARDCODED_CASES
        .iter()
        .find(|c| c.type_key == extractor_type)
    {
        Some(case) => DispatchTarget::Hardcoded(*case),
        None => DispatchTarget::Plugin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C049: every extractor-type key this port
    /// has actually implemented routes to its own module; everything else
    /// — including type keys the *source* hardcodes but this port hasn't
    /// reached — falls through to the plugin path, matching `Case Else`.
    #[test]
    fn routes_ported_extractors_to_their_module() {
        assert_eq!(
            dispatch("rgss"),
            DispatchTarget::Hardcoded(HardcodedCase {
                type_key: "rgss",
                module: "extract::rgss"
            })
        );
        assert_eq!(
            dispatch("rpa"),
            DispatchTarget::Hardcoded(HardcodedCase {
                type_key: "rpa",
                module: "extract::rpa"
            })
        );
        assert_eq!(
            dispatch("sfark"),
            DispatchTarget::Hardcoded(HardcodedCase {
                type_key: "sfark",
                module: "extract::sfark"
            })
        );
        assert_eq!(
            dispatch("ace"),
            DispatchTarget::Hardcoded(HardcodedCase {
                type_key: "ace",
                module: "extract::ace"
            })
        );
    }

    #[test]
    fn falls_through_to_plugin_for_unrecognized_or_not_yet_ported_types() {
        // "7z" has a hardcoded Case in the source (UniExtract.au3) but not
        // yet in this port — Plugin here reflects this port's real current
        // coverage, not a parity gap in C049 itself (see the module doc
        // comment).
        assert_eq!(dispatch("7z"), DispatchTarget::Plugin);
        assert_eq!(dispatch("nonsense-not-a-real-type"), DispatchTarget::Plugin);
    }

    #[test]
    fn dispatch_is_case_sensitive_matching_the_source() {
        // $arctype is always a lowercase string constant in the source
        // (e.g. $TYPE_RGSS = "rgss") — the Switch's string comparison is
        // exact, not case-folded, so an unexpected-case key is genuinely
        // unrecognized rather than a variant spelling of a known one.
        assert_eq!(dispatch("RGSS"), DispatchTarget::Plugin);
    }
}
