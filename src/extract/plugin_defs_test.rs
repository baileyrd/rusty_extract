//! Regression tests for every `def/*.ini`-only plugin definition: formats
//! with no hardcoded `extract::*` module of their own, dispatched entirely
//! through the `def/*.ini` plugin path — extension routing (C047) resolves
//! the extension to a plugin stem, `extract::dispatch` has no hardcoded
//! `Case $TYPE_...` for it, so it falls through to the plugin engine
//! (`extract::plugin`, C050), whose bundled `.ini` is parsed by
//! `extract::plugin_config` (C052) and substituted by `extract::placeholder`
//! (C182).
//!
//! None of these 18 capabilities has any production code of its own — each
//! is proven purely by composing those already-built primitives against its
//! bundled `def/*.ini` asset and checking the resulting command line
//! matches what `pluginExtract` (UniExtract.au3:3468-3520) would build.
//! This module verifies the *substituted command-line string*
//! (`$sBinary & " " & $sParameters`, matching `pluginExtract`'s own
//! construction before handing it to `_Run`), not a tokenized
//! `Invocation.args`, the way every hardcoded `extract::*` module builds —
//! turning that string into pre-split argument tokens needs a quote-aware
//! command-line tokenizer this port hasn't built yet for the plugin path, a
//! real gap, not something any one of these capabilities' own one-line
//! manifest descriptions ask it to solve.
//!
//! Two rows need extra handling beyond the shared assertions:
//! - `adf` (C122) sets `workingdir=%tempoutdir%`. `%tempoutdir%` is *not*
//!   one of `replace_placeholders`'s five named substitutions — in the
//!   source, `pluginExtract` replaces it with a separate, direct
//!   `StringReplace($sWorkingDir, "%tempoutdir%", $tempoutdir)` call
//!   (UniExtract.au3:3506) before running the result through
//!   `ReplacePlaceholders` (line 3507). [`WorkingDir::TempOutdir`] performs
//!   that same two-step substitution.
//! - `uu` (C137) sets `workingdir=%filedir%`, which *is* one of the five
//!   named substitutions, so a single `replace_placeholders` call resolves
//!   it — [`WorkingDir::Named`].
//!
//! Other per-row quirks worth flagging (each still just a table value here,
//! not special-cased code): `mo` (C128) and `qm` (C130) both concatenate
//! `%outdir%\%filename%.po`/`.ts` with no space between the quoted
//! `%outdir%` substitution and the literal `\`, preserved exactly as the
//! source builds it. `lit` (C127) is the one `todo.txt:29` flags for an
//! output-path-with-spaces bug in `clit.exe` itself — nothing in this row
//! adds path-escaping beyond what `extract::placeholder` already does
//! uniformly for every plugin, so the quirk is preserved by this row simply
//! not inventing a special case, not by any code addition.

use crate::extract::placeholder::{replace_placeholders, PlaceholderContext};
use crate::extract::plugin_config::PluginConfig;
use crate::extract::WindowMode;
use crate::ini::IniFile;

const ADF_INI: &str = include_str!("../../def/adf.ini");
const ALZ_INI: &str = include_str!("../../def/alz.ini");
const ARC_INI: &str = include_str!("../../def/arc.ini");
const BITROCK_INI: &str = include_str!("../../def/bitrock.ini");
const BSA_INI: &str = include_str!("../../def/bsa.ini");
const GODOT_INI: &str = include_str!("../../def/godot.ini");
const LBR_INI: &str = include_str!("../../def/lbr.ini");
const LIT_INI: &str = include_str!("../../def/lit.ini");
const MO_INI: &str = include_str!("../../def/mo.ini");
const PEX_INI: &str = include_str!("../../def/pex.ini");
const QM_INI: &str = include_str!("../../def/qm.ini");
const RPGMVP_INI: &str = include_str!("../../def/rpgmvp.ini");
const SGB_INI: &str = include_str!("../../def/sgb.ini");
const SIM_INI: &str = include_str!("../../def/sim.ini");
const SIT_INI: &str = include_str!("../../def/sit.ini");
const SPOON_INI: &str = include_str!("../../def/spoon.ini");
const UTAGE_INI: &str = include_str!("../../def/utage.ini");
const UU_INI: &str = include_str!("../../def/uu.ini");

/// How a case's `workingdir` should be checked. Every row but `adf`/`uu`
/// omits the key entirely, falling back to `outdir` per `If Not
/// $sWorkingDir Or $sWorkingDir = "" Then $sWorkingDir = $outdir`
/// (UniExtract.au3:3503).
enum WorkingDir {
    /// No `workingdir` key.
    Absent,
    /// `workingdir=%tempoutdir%` (`adf`): a manual `%tempoutdir%` replace,
    /// then the general `replace_placeholders` pass.
    TempOutdir {
        raw: &'static str,
        tempoutdir: &'static str,
        resolved: &'static str,
    },
    /// `workingdir=%filedir%` (`uu`): one of the five named placeholders,
    /// resolved by a single `replace_placeholders` call.
    Named {
        raw: &'static str,
        resolved: &'static str,
    },
}

/// One row: a bundled `def/*.ini` plugin definition plus the fixture
/// [`PlaceholderContext`] and expectations the original per-format test
/// checked. `pattern_search`/`initial_show`/`require_net_framework` are
/// `Some` only where the original test asserted them; `None` means the
/// original test didn't check that field, so this one doesn't either.
struct PluginCase {
    /// capability-manifest.md ID this row proves parity for.
    capability: &'static str,
    stem: &'static str,
    ini: &'static str,
    executable: &'static str,
    run_in_temp_outdir: bool,
    pattern_search: Option<bool>,
    initial_show: Option<bool>,
    require_net_framework: Option<Option<&'static str>>,
    workingdir: WorkingDir,
    ctx: PlaceholderContext<'static>,
    expected_command_line: &'static str,
}

fn cases() -> Vec<PluginCase> {
    vec![
        // C122 — unadf: %tempoutdir% workingdir, two-step substitution.
        PluginCase {
            capability: "C122",
            stem: "adf",
            ini: ADF_INI,
            executable: "unadf.exe",
            run_in_temp_outdir: true,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::TempOutdir {
                raw: "%tempoutdir%",
                tempoutdir: r"C:\downloads\archive_unpacked\tmp123456",
                resolved: r"C:\downloads\archive_unpacked\tmp123456",
            },
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.adf",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "adf",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"unadf.exe "C:\downloads\archive.adf""#,
        },
        // C059 — unalz.
        PluginCase {
            capability: "C059",
            stem: "alz",
            ini: ALZ_INI,
            executable: "unalz.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.alz",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "alz",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"unalz.exe -d "C:\downloads\archive_unpacked" "C:\downloads\archive.alz""#,
        },
        // C060 — arc.exe. require_net_framework explicitly checked as None.
        PluginCase {
            capability: "C060",
            stem: "arc",
            ini: ARC_INI,
            executable: "arc.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: Some(None),
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.arc",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "arc",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"arc.exe x "C:\downloads\archive.arc""#,
        },
        // C123 — bitrock-unpacker.
        PluginCase {
            capability: "C123",
            stem: "bitrock",
            ini: BITROCK_INI,
            executable: "bitrock-unpacker.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\installer.exe",
                outdir: r"C:\downloads\installer_unpacked",
                filename: "installer",
                fileext: "exe",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"bitrock-unpacker.exe "C:\downloads\installer.exe" "C:\downloads\installer_unpacked""#,
        },
        // C124 — BSA Browser.
        PluginCase {
            capability: "C124",
            stem: "bsa",
            ini: BSA_INI,
            executable: "bsab.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: Some(Some("4")),
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.bsa",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "bsa",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"bsab.exe /e "C:\downloads\archive.bsa" "C:\downloads\archive_unpacked""#,
        },
        // C125 — godotdec.
        PluginCase {
            capability: "C125",
            stem: "godot",
            ini: GODOT_INI,
            executable: "godotdec.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: Some(Some("4.5")),
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\game.pck",
                outdir: r"C:\downloads\game_unpacked",
                filename: "game",
                fileext: "pck",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"godotdec.exe -c "C:\downloads\game.pck" "C:\downloads\game_unpacked""#,
        },
        // C126 — lbrate.
        PluginCase {
            capability: "C126",
            stem: "lbr",
            ini: LBR_INI,
            executable: "lbrate.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.lbr",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "lbr",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"lbrate.exe "C:\downloads\archive.lbr""#,
        },
        // C127 — ConvertLIT (todo.txt:29 output-path-with-spaces quirk).
        PluginCase {
            capability: "C127",
            stem: "lit",
            ini: LIT_INI,
            executable: "clit.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\book.lit",
                outdir: r"C:\downloads\book_unpacked",
                filename: "book",
                fileext: "lit",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"clit.exe "C:\downloads\book.lit" "C:\downloads\book_unpacked""#,
        },
        // C128 — GNU gettext msgunfmt (%outdir%\%filename%.po, no space).
        PluginCase {
            capability: "C128",
            stem: "mo",
            ini: MO_INI,
            executable: "msgunfmt.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\catalog.mo",
                outdir: r"C:\downloads\catalog_unpacked",
                filename: "catalog",
                fileext: "mo",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"msgunfmt.exe "C:\downloads\catalog.mo" -o "C:\downloads\catalog_unpacked"\catalog.po"#,
        },
        // C129 — Champollion.
        PluginCase {
            capability: "C129",
            stem: "pex",
            ini: PEX_INI,
            executable: "Champollion.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\script.pex",
                outdir: r"C:\downloads\script_unpacked",
                filename: "script",
                fileext: "pex",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"Champollion.exe "C:\downloads\script.pex" -p "C:\downloads\script_unpacked" -a "C:\downloads\script_unpacked""#,
        },
        // C130 — Qt Linguist lconvert (%outdir%\%filename%.ts, no space).
        PluginCase {
            capability: "C130",
            stem: "qm",
            ini: QM_INI,
            executable: "lconvert.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\translation.qm",
                outdir: r"C:\downloads\translation_unpacked",
                filename: "translation",
                fileext: "qm",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"lconvert.exe "C:\downloads\translation.qm" -o "C:\downloads\translation_unpacked"\translation.ts"#,
        },
        // C131 — rmvdec (patternSearch=1).
        PluginCase {
            capability: "C131",
            stem: "rpgmvp",
            ini: RPGMVP_INI,
            executable: "rmvdec.exe",
            run_in_temp_outdir: false,
            pattern_search: Some(true),
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\resource.rpgmvp",
                outdir: r"C:\downloads\resource_unpacked",
                filename: "resource",
                fileext: "rpgmvp",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"rmvdec.exe "C:\downloads\resource.rpgmvp" "C:\downloads\resource_unpacked""#,
        },
        // C132 — sgbdec (patternSearch=1, initialShow=1, .NET 4.5).
        PluginCase {
            capability: "C132",
            stem: "sgb",
            ini: SGB_INI,
            executable: "sgbdec.exe",
            run_in_temp_outdir: false,
            pattern_search: Some(true),
            initial_show: Some(true),
            require_net_framework: Some(Some("4.5")),
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\game.sgbpack",
                outdir: r"C:\downloads\game_unpacked",
                filename: "game",
                fileext: "sgbpack",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"sgbdec.exe "C:\downloads\game.sgbpack" "C:\downloads\game_unpacked""#,
        },
        // C133 — simdec (.NET 4).
        PluginCase {
            capability: "C133",
            stem: "sim",
            ini: SIM_INI,
            executable: "simdec.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: Some(Some("4")),
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\installer.exe",
                outdir: r"C:\downloads\installer_unpacked",
                filename: "installer",
                fileext: "exe",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"simdec.exe "C:\downloads\installer.exe" "C:\downloads\installer_unpacked""#,
        },
        // C134 — unar / TheUnarchiver.
        PluginCase {
            capability: "C134",
            stem: "sit",
            ini: SIT_INI,
            executable: "unar.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\archive.sit",
                outdir: r"C:\downloads\archive_unpacked",
                filename: "archive",
                fileext: "sit",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"unar.exe -o "C:\downloads\archive_unpacked" "C:\downloads\archive.sit""#,
        },
        // C135 — spoondec.
        PluginCase {
            capability: "C135",
            stem: "spoon",
            ini: SPOON_INI,
            executable: "spoondec.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\installer.exe",
                outdir: r"C:\downloads\installer_unpacked",
                filename: "installer",
                fileext: "exe",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"spoondec.exe "C:\downloads\installer.exe" "C:\downloads\installer_unpacked""#,
        },
        // C136 — utagedec (patternSearch=1).
        PluginCase {
            capability: "C136",
            stem: "utage",
            ini: UTAGE_INI,
            executable: "utagedec.exe",
            run_in_temp_outdir: false,
            pattern_search: Some(true),
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Absent,
            ctx: PlaceholderContext {
                file: r"C:\downloads\game.dat",
                outdir: r"C:\downloads\game_unpacked",
                filename: "game",
                fileext: "dat",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"utagedec.exe "C:\downloads\game.dat" "C:\downloads\game_unpacked""#,
        },
        // C137 — UUDeview: %filedir% workingdir, single-step substitution.
        PluginCase {
            capability: "C137",
            stem: "uu",
            ini: UU_INI,
            executable: "uudeview.exe",
            run_in_temp_outdir: false,
            pattern_search: None,
            initial_show: None,
            require_net_framework: None,
            workingdir: WorkingDir::Named {
                raw: "%filedir%",
                resolved: r"C:\downloads",
            },
            ctx: PlaceholderContext {
                file: r"C:\downloads\encoded.uu",
                outdir: r"C:\downloads\encoded_unpacked",
                filename: "encoded",
                fileext: "uu",
                filedir: r"C:\downloads",
            },
            expected_command_line: r#"uudeview.exe -p "C:\downloads\encoded_unpacked" -i "C:\downloads\encoded.uu""#,
        },
    ]
}

/// Parity test for capabilities C059, C060, C122-C137: parsing each
/// bundled plugin-only `def/*.ini` and substituting its
/// `parameters`/`workingdir` for a representative extraction produces the
/// exact command line (and, for `adf`/`uu`, working directory)
/// `pluginExtract` would use.
#[test]
fn bundled_plugin_only_inis_produce_source_matching_command_lines() {
    for case in cases() {
        let (ini, skipped) = IniFile::parse(case.ini);
        assert!(
            skipped.is_empty(),
            "{} ({}): unexpected skipped ini lines",
            case.stem,
            case.capability
        );
        let config = PluginConfig::parse(&ini, case.stem);

        assert_eq!(
            config.executable, case.executable,
            "{} ({}): executable",
            case.stem, case.capability
        );
        assert_eq!(
            config.run_in_temp_outdir, case.run_in_temp_outdir,
            "{} ({}): run_in_temp_outdir",
            case.stem, case.capability
        );
        assert_eq!(
            config.window_mode(),
            WindowMode::Hidden,
            "{} ({}): window_mode",
            case.stem,
            case.capability
        );

        if let Some(expected) = case.pattern_search {
            assert_eq!(
                config.pattern_search, expected,
                "{} ({}): pattern_search",
                case.stem, case.capability
            );
        }
        if let Some(expected) = case.initial_show {
            assert_eq!(
                config.initial_show, expected,
                "{} ({}): initial_show",
                case.stem, case.capability
            );
        }
        if let Some(expected) = case.require_net_framework {
            assert_eq!(
                config.require_net_framework.as_deref(),
                expected,
                "{} ({}): require_net_framework",
                case.stem,
                case.capability
            );
        }

        let params =
            replace_placeholders(&format!(" {}", config.parameters), true, case.ctx, |k| {
                k.to_string()
            });
        let command_line = format!("{}{params}", config.executable);
        assert_eq!(
            command_line, case.expected_command_line,
            "{} ({}): command line",
            case.stem, case.capability
        );

        match case.workingdir {
            WorkingDir::Absent => {
                assert_eq!(
                    config.workingdir, None,
                    "{} ({}): workingdir",
                    case.stem, case.capability
                );
            }
            WorkingDir::TempOutdir {
                raw,
                tempoutdir,
                resolved,
            } => {
                assert_eq!(
                    config.workingdir.as_deref(),
                    Some(raw),
                    "{} ({}): raw workingdir",
                    case.stem,
                    case.capability
                );
                // %tempoutdir% first (a direct StringReplace in
                // pluginExtract, not part of ReplacePlaceholders), then the
                // general substitution pass.
                let workingdir_raw = config
                    .workingdir
                    .as_deref()
                    .unwrap()
                    .replace("%tempoutdir%", tempoutdir);
                let workingdir =
                    replace_placeholders(&workingdir_raw, true, case.ctx, |k| k.to_string());
                assert_eq!(
                    workingdir, resolved,
                    "{} ({}): resolved workingdir",
                    case.stem, case.capability
                );
            }
            WorkingDir::Named { raw, resolved } => {
                assert_eq!(
                    config.workingdir.as_deref(),
                    Some(raw),
                    "{} ({}): raw workingdir",
                    case.stem,
                    case.capability
                );
                // %filedir% is one of the five named substitutions, so a
                // single replace_placeholders call (no manual
                // %tempoutdir%-style pre-step) resolves it.
                let workingdir = replace_placeholders(
                    config.workingdir.as_deref().unwrap(),
                    true,
                    case.ctx,
                    |k| k.to_string(),
                );
                assert_eq!(
                    workingdir, resolved,
                    "{} ({}): resolved workingdir",
                    case.stem, case.capability
                );
            }
        }
    }
}
