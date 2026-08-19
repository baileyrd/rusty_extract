//! Data-driven table for the extractor formats that each shell out to
//! one external helper binary with a fixed, non-branching argument pattern
//! (no fallback logic, no conditionals). Each format used to be its own
//! file — one `pub fn invocation(...) -> Invocation` plus one parity test —
//! but the ceremony (module doc comment, `use` line, `#[cfg(test)] mod
//! tests`) dwarfed the 5-10 lines of actual argument-building logic every
//! time. This module collapses them into one [`Ctx`] input struct, one
//! small builder fn per format, and one [`FORMATS`] table tying a format
//! name to its builder — same observable [`Invocation`] output as before
//! for the same inputs, just without 43x the boilerplate.
//!
//! Every format's `UniExtract.au3` source-line citation is kept as the
//! `citation` field on its [`FormatEntry`] row (and repeated as the doc
//! comment on the builder fn itself) so provenance isn't lost. The
//! multi-paragraph rationale essays the original per-file doc comments
//! carried (scope notes on what's deliberately *not* modeled, quoting
//! quirks, `_Run` default-argument reasoning, etc.) are condensed to a
//! one-line summary here — see git history for the original prose if any
//! of that reasoning is needed again.
//!
//! `Ctx` fields are a shared transport, not a 1:1 mirror of any one
//! format's source variable names: e.g. `filename` stands in for a bare
//! stem in most formats but a full `name.ext` in `uif`, and `file` stands
//! in for `xor`'s `overlay_file`. Each builder's doc comment says which
//! fields it reads and what they mean for that format.

use super::{Invocation, WindowMode};

/// Shared inputs a format's invocation builder may read. Not every field
/// is used by every format — see each builder fn's doc comment for which
/// apply and what they mean for that format. Fields a given format doesn't
/// use are left at their `Default` value (`""` / `None` / `false` / `0`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Ctx<'a> {
    pub program: &'a str,
    pub file: &'a str,
    pub outdir: &'a str,
    pub file_dir: &'a str,
    pub tempoutdir: &'a str,
    pub filename: &'a str,
    pub filename_full: &'a str,
    pub dest_path: &'a str,
    pub script_dir: &'a str,
    pub password: Option<&'a str>,
    pub append_ext: bool,
    pub game_index: u32,
}

/// One table row: a format name, its AutoIt source-line provenance, and
/// the builder fn that turns a [`Ctx`] into that format's [`Invocation`].
pub struct FormatEntry {
    pub name: &'static str,
    pub citation: &'static str,
    pub build: fn(&Ctx) -> Invocation,
}

/// Looks up `name` in [`FORMATS`] and runs its builder against `ctx`.
/// Returns `None` for a name not in the table.
pub fn build(name: &str, ctx: &Ctx) -> Option<Invocation> {
    FORMATS
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| (entry.build)(ctx))
}

/// C057 UniExtract.au3:2346-2349 — acefile: `-x -v -d <outdir> <file>`, in
/// `outdir`, hidden.
fn ace(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-x".to_string(),
            "-v".to_string(),
            "-d".to_string(),
            ctx.outdir.to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C115 UniExtract.au3:2385-2390 — Advanced Installer self-extraction:
/// `<file> /extract:<outdir>`, program is `ctx.file` itself, in `outdir`,
/// shown.
fn ai(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.file.to_string(),
        args: vec![format!("/extract:{}", ctx.outdir)],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C113 UniExtract.au3:2394 — arc_conv: `<file>`, in `outdir`, hidden.
fn arc_conv(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C058 UniExtract.au3:3624-3625 (`Case $PACKER_ASPACK`) — AspackDie:
/// `<file> <dest_path> /NO_PROMPT`, in `file_dir`, minimized. Deliberately
/// absent from `dispatch::HARDCODED_CASES` — see `upx`'s doc comment.
fn aspack(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            ctx.file.to_string(),
            ctx.dest_path.to_string(),
            "/NO_PROMPT".to_string(),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C062 UniExtract.au3:2418-2419 — bcm: `-d <file> <outdir>\<filename>`
/// (`filename` here is the stem), in `file_dir`, hidden.
fn bcm(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-d".to_string(),
            ctx.file.to_string(),
            format!("{}\\{}", ctx.outdir, ctx.filename),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C063 UniExtract.au3:2421-2429 — bootimg: `--unpack-bootimg` (no file
/// argument — operates on `boot.img` in its own cwd, staged by the
/// caller), in `outdir`, minimized.
fn bootimg(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["--unpack-bootimg".to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C065 UniExtract.au3:2441-2442 — chdman: `extracthd -i <file> -o
/// <outdir>\<filename>.img` (`filename` here is the stem), in `outdir`,
/// minimized.
fn chdman(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "extracthd".to_string(),
            "-i".to_string(),
            ctx.file.to_string(),
            "-o".to_string(),
            format!("{}\\{}.img", ctx.outdir, ctx.filename),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C067 UniExtract.au3:2472-2475 — cicdec: `-db <file> <outdir>`, in
/// `file_dir`, hidden.
fn cic(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-db".to_string(),
            ctx.file.to_string(),
            ctx.outdir.to_string(),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C146 UniExtract.au3:2505-2508 — daa2iso: `<file> <outdir>\<filename>.iso`,
/// in `outdir`, minimized. No pre-existing-file check, matching the
/// source's own `todo.txt:52`-documented bug — deliberately preserved.
fn daa(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            ctx.file.to_string(),
            format!("{}\\{}.iso", ctx.outdir, ctx.filename),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C116 UniExtract.au3:2514-2516 — Excelsior Installer self-extraction:
/// `<file> /batch /no-reg /no-postinstall /dest <outdir>`, program is
/// `ctx.file` itself, in `outdir`, shown.
fn ei(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.file.to_string(),
        args: vec![
            "/batch".to_string(),
            "/no-reg".to_string(),
            "/no-postinstall".to_string(),
            "/dest".to_string(),
            ctx.outdir.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C096 UniExtract.au3:3026 — extsis: `-x -xcsd <file> -d <tempoutdir>`, in
/// `tempoutdir`, minimized.
fn extsis(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-x".to_string(),
            "-xcsd".to_string(),
            ctx.file.to_string(),
            "-d".to_string(),
            ctx.tempoutdir.to_string(),
        ],
        working_dir: ctx.tempoutdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C117 UniExtract.au3:2530-2536 — Netopsystems FEAD self-extraction:
/// `<file> /s -nos_ne -nos_o<tempoutdir>\`, program is `ctx.file` itself,
/// in `file_dir`, shown.
fn fead(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.file.to_string(),
        args: vec![
            "/s".to_string(),
            "-nos_ne".to_string(),
            format!("-nos_o{}\\", ctx.tempoutdir),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Show,
    }
}

/// C071 UniExtract.au3:2556-2557 — unarc (FreeArc): `x -dp"<outdir>"
/// <file>`, in `file_dir`, hidden. `-dp"<outdir>"` is a single token with
/// the quote characters embedded literally — deliberate, matching the
/// source's own concatenation, not a typo.
fn freearc(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "x".to_string(),
            format!("-dp\"{}\"", ctx.outdir),
            ctx.file.to_string(),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C072 UniExtract.au3:2559-2562 — fsbext: `-o -1 -A -d <outdir> <file>`,
/// in `file_dir`, minimized.
fn fsb(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-o".to_string(),
            "-1".to_string(),
            "-A".to_string(),
            "-d".to_string(),
            ctx.outdir.to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C068 UniExtract.au3:2565-2566 — GARbro.Console: `x -ocu -if png -o
/// <outdir> <file>`, in `outdir`, minimized.
fn garbro(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "x".to_string(),
            "-ocu".to_string(),
            "-if".to_string(),
            "png".to_string(),
            "-o".to_string(),
            ctx.outdir.to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C076 UniExtract.au3:2711 — IsXunpack (one `$TYPE_ISEXE` GUI candidate):
/// `<outdir>\<filename_full>`, in `outdir`, shown.
fn isxunpack(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![format!("{}\\{}", ctx.outdir, ctx.filename_full)],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C078 UniExtract.au3:2775-2778 — unisz: `<file>`, in `tempoutdir`, shown.
fn isz(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string()],
        working_dir: ctx.tempoutdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C085 UniExtract.au3:2858 — jsMSIx (one `$TYPE_MSI` GUI candidate):
/// `<file>|<outdir>` collapses to one pipe-joined token, in `filedir`,
/// hidden.
fn jsmsix(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![format!("{}|{}", ctx.file, ctx.outdir)],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C079 UniExtract.au3:2780-2781 — kgb2_console: `<file>`, in `outdir`,
/// minimized.
fn kgb(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C084 UniExtract.au3:2843-2845 — lessmsi (`$TYPE_MSI`'s primary
/// attempt): `x <file> <outdir>\`, in `outdir`, hidden.
fn lessmsi(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "x".to_string(),
            ctx.file.to_string(),
            format!("{}\\", ctx.outdir),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C080 UniExtract.au3:2783-2784 — lzip: `-d -k -v -v <file>`, in
/// `tempoutdir`, shown.
fn lzip(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-d".to_string(),
            "-k".to_string(),
            "-v".to_string(),
            "-v".to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.tempoutdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C081 UniExtract.au3:2786-2787 — lzop: `-d -p"<outdir>" <file>`, in
/// `file_dir`, minimized.
fn lzop(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-d".to_string(),
            format!("-p\"{}\"", ctx.outdir),
            ctx.file.to_string(),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C082 UniExtract.au3:2789-2790 — unlzx: `-x <file>`, in `outdir`,
/// minimized.
fn lzx(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["-x".to_string(), ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C083 UniExtract.au3:2792-2811 — demoleition: `/nogui <file>`, in
/// `outdir` (not `tempoutdir` — see `wolf`'s doc comment for the same
/// `_RunInTempOutdir` quirk), hidden.
fn mole(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["/nogui".to_string(), ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C087 UniExtract.au3:2882-2883 — msiexec (`$TYPE_MSI`'s "administrative
/// install" candidate): `msiexec.exe /a <file> /qb TARGETDIR=<outdir>`,
/// program is the literal `msiexec.exe`, in `filedir`, shown.
fn msiexec(ctx: &Ctx) -> Invocation {
    Invocation {
        program: "msiexec.exe".to_string(),
        args: vec![
            "/a".to_string(),
            ctx.file.to_string(),
            "/qb".to_string(),
            format!("TARGETDIR={}", ctx.outdir),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Show,
    }
}

/// C086 UniExtract.au3:2862-2864 / 2887-2889 / 2907-2908 — MsiX (shared by
/// `$TYPE_MSI`'s fallback, `$TYPE_MSM`, `$TYPE_MSP`'s fallback): `<file>
/// /out <outdir> [/ext]`, in `filedir`, minimized. `ctx.append_ext` selects
/// the trailing `/ext`.
fn msix(ctx: &Ctx) -> Invocation {
    let mut args = vec![
        ctx.file.to_string(),
        "/out".to_string(),
        ctx.outdir.to_string(),
    ];
    if ctx.append_ext {
        args.push("/ext".to_string());
    }
    Invocation {
        program: ctx.program.to_string(),
        args,
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C088 UniExtract.au3:2952-2953 — NBHextract: `<file>`, in `outdir`,
/// shown.
fn nbh(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C092 UniExtract.au3:3005 — UnRAR: `x -kb [-p<password>] <file>`, in
/// `outdir`, shown. `ctx.password` folds into a single `-p<password>`
/// token when present, matching the source's effective post-quote-parsing
/// argument shape.
fn rar(ctx: &Ctx) -> Invocation {
    let mut args = vec!["x".to_string(), "-kb".to_string()];
    if let Some(password) = ctx.password {
        args.push(format!("-p{password}"));
    }
    args.push(ctx.file.to_string());
    Invocation {
        program: ctx.program.to_string(),
        args,
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C093 UniExtract.au3:3009-3011 — RgssDecrypter: `-p -o=<outdir> <file>`,
/// in `outdir`, hidden.
fn rgss(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-p".to_string(),
            format!("-o={}", ctx.outdir),
            ctx.file.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C094 UniExtract.au3:3016-3017 — unrpa: `-m -v --continue-on-error -p
/// <outdir> <file>`, in `script_dir` (`@ScriptDir` in the source, not
/// `outdir`), shown.
fn rpa(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-m".to_string(),
            "-v".to_string(),
            "--continue-on-error".to_string(),
            "-p".to_string(),
            ctx.outdir.to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.script_dir.to_string(),
        window: WindowMode::Show,
    }
}

/// C095 UniExtract.au3:3019-3020 — sfarkxtc: `<file>
/// <outdir>\<filename>.sf2` (`filename` here is the stem), in `file_dir`,
/// shown.
fn sfark(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            ctx.file.to_string(),
            format!("{}\\{}.sf2", ctx.outdir, ctx.filename),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Show,
    }
}

/// C097 UniExtract.au3:3032-3033 — sqlite3: `<file> .dump`, in `filedir`
/// (not `outdir`), hidden.
fn sqlite(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string(), ".dump".to_string()],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C118 UniExtract.au3:3038-3043 — SuperDAT Updater self-extraction:
/// `<file> /LOGFILE <outdir>\SuperDAT.log /e <outdir>`, program is
/// `ctx.file` itself, in `outdir`, shown.
fn superdat(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.file.to_string(),
        args: vec![
            "/LOGFILE".to_string(),
            format!("{}\\SuperDAT.log", ctx.outdir),
            "/e".to_string(),
            ctx.outdir.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C100 UniExtract.au3:3147 — ttarchext (game already selected via GUI):
/// `-m <game_index> <file> <outdir>`, in `outdir`, hidden.
fn ttarch(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-m".to_string(),
            ctx.game_index.to_string(),
            ctx.file.to_string(),
            ctx.outdir.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C101 UniExtract.au3:3154,3156,3158 — UHARC's 3-binary fallback chain
/// (`UNUHARC06.EXE`, then `UHARC04.EXE`, then `UHARC02.EXE`): all three
/// attempts build the identical shape, `x -t<outdir> <file>`, in `outdir`,
/// minimized — only the `program` binary (and, for the third attempt,
/// 8.3-short-form `outdir`/`file` strings the caller supplies) differ, so
/// one builder serves all three `FORMATS` rows.
fn uharc(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "x".to_string(),
            format!("-t{}", ctx.outdir),
            ctx.file.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C102 UniExtract.au3:3161-3163 — uif2iso: `<file> <outdir>\<filename>`
/// (`filename` here is a full `name.ext`, e.g. `image.iso`), in
/// `file_dir`, shown.
fn uif(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            ctx.file.to_string(),
            format!("{}\\{}", ctx.outdir, ctx.filename),
        ],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Show,
    }
}

/// C103 UniExtract.au3:3211-3214 — umodel: `-export -all -sounds
/// -3rdparty -path="<file_dir>" -out="<outdir>" *`, in `outdir`,
/// minimized.
fn unreal(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-export".to_string(),
            "-all".to_string(),
            "-sounds".to_string(),
            "-3rdparty".to_string(),
            format!("-path=\"{}\"", ctx.file_dir),
            format!("-out=\"{}\"", ctx.outdir),
            "*".to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C109 UniExtract.au3:3384-3388 — Info-ZIP unzip (`$TYPE_ZIP`'s fallback
/// once 7-Zip fails): `-x <file>`, in `outdir`, minimized.
fn unzip(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["-x".to_string(), ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C112 UniExtract.au3:3617-3623 (`Case $PACKER_UPX`) — upx: `-d -k
/// <file>`, in `file_dir`, minimized. Deliberately absent from
/// `dispatch::HARDCODED_CASES`: this belongs to the source's separate
/// post-extraction `Switch $packer`, not the main `Switch $arctype`.
fn upx(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["-d".to_string(), "-k".to_string(), ctx.file.to_string()],
        working_dir: ctx.file_dir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C107 UniExtract.au3:3373-3375 — dark (WiX Toolset): `-x <outdir>
/// <file>`, in `outdir`, minimized.
fn wix(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "-x".to_string(),
            ctx.outdir.to_string(),
            ctx.file.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C108 UniExtract.au3:3377-3382 — WolfDec: `<file>`, in `outdir` (not
/// `tempoutdir` — `_RunInTempOutdir`'s explicit third argument overrides
/// its own staging dir), minimized.
fn wolf(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![ctx.file.to_string()],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C070 UniExtract.au3:2598 — xor (internal Ghost-Installer helper, not a
/// top-level `$arctype`): `<overlay_file> <outdir>\<filename>.cab 0x8D`.
/// `ctx.file` stands in for the source's `overlay_file`. In `outdir`,
/// minimized (both `_Run` defaults, omitted in the source).
fn xor(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            ctx.file.to_string(),
            format!("{}\\{}.cab", ctx.outdir, ctx.filename),
            "0x8D".to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Minimized,
    }
}

/// C110 UniExtract.au3:3390-3394 — unzoo: `-x <filename_full>` (bare,
/// unquoted in the source — no observable difference in this crate's
/// already-split `Invocation` model), in `tempoutdir`, hidden.
fn zoo(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["-x".to_string(), ctx.filename_full.to_string()],
        working_dir: ctx.tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// C111 UniExtract.au3:3396-3399 — zpaq: `x <file> -to <outdir>`, in
/// `outdir`, shown.
fn zpaq(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec![
            "x".to_string(),
            ctx.file.to_string(),
            "-to".to_string(),
            ctx.outdir.to_string(),
        ],
        working_dir: ctx.outdir.to_string(),
        window: WindowMode::Show,
    }
}

/// C120 UniExtract.au3:2827 — per-`.cab`-file 7-Zip extraction inside
/// `$TYPE_MSCF` (after `RipExeInfo`'s GUI rip): `x <cab_file>`
/// (`ctx.file` stands in for the cab file), in `tempoutdir`, hidden.
fn mscf(ctx: &Ctx) -> Invocation {
    Invocation {
        program: ctx.program.to_string(),
        args: vec!["x".to_string(), ctx.file.to_string()],
        working_dir: ctx.tempoutdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// The single-invocation extractor formats, one row per format (`uharc`'s
/// 3-attempt fallback chain shares one builder across three rows — see
/// its doc comment). See the module doc comment for what each `Ctx` field
/// means and the per-fn doc comments above for each format's exact
/// argument shape and citation.
pub static FORMATS: &[FormatEntry] = &[
    FormatEntry {
        name: "ace",
        citation: "C057 UniExtract.au3:2346-2349",
        build: ace,
    },
    FormatEntry {
        name: "ai",
        citation: "C115 UniExtract.au3:2385-2390",
        build: ai,
    },
    FormatEntry {
        name: "arc_conv",
        citation: "C113 UniExtract.au3:2394",
        build: arc_conv,
    },
    FormatEntry {
        name: "aspack",
        citation: "C058 UniExtract.au3:3624-3625",
        build: aspack,
    },
    FormatEntry {
        name: "bcm",
        citation: "C062 UniExtract.au3:2418-2419",
        build: bcm,
    },
    FormatEntry {
        name: "bootimg",
        citation: "C063 UniExtract.au3:2421-2429",
        build: bootimg,
    },
    FormatEntry {
        name: "chdman",
        citation: "C065 UniExtract.au3:2441-2442",
        build: chdman,
    },
    FormatEntry {
        name: "cic",
        citation: "C067 UniExtract.au3:2472-2475",
        build: cic,
    },
    FormatEntry {
        name: "daa",
        citation: "C146 UniExtract.au3:2505-2508",
        build: daa,
    },
    FormatEntry {
        name: "ei",
        citation: "C116 UniExtract.au3:2514-2516",
        build: ei,
    },
    FormatEntry {
        name: "extsis",
        citation: "C096 UniExtract.au3:3026",
        build: extsis,
    },
    FormatEntry {
        name: "fead",
        citation: "C117 UniExtract.au3:2530-2536",
        build: fead,
    },
    FormatEntry {
        name: "freearc",
        citation: "C071 UniExtract.au3:2556-2557",
        build: freearc,
    },
    FormatEntry {
        name: "fsb",
        citation: "C072 UniExtract.au3:2559-2562",
        build: fsb,
    },
    FormatEntry {
        name: "garbro",
        citation: "C068 UniExtract.au3:2565-2566",
        build: garbro,
    },
    FormatEntry {
        name: "isxunpack",
        citation: "C076 UniExtract.au3:2711",
        build: isxunpack,
    },
    FormatEntry {
        name: "isz",
        citation: "C078 UniExtract.au3:2775-2778",
        build: isz,
    },
    FormatEntry {
        name: "jsmsix",
        citation: "C085 UniExtract.au3:2858",
        build: jsmsix,
    },
    FormatEntry {
        name: "kgb",
        citation: "C079 UniExtract.au3:2780-2781",
        build: kgb,
    },
    FormatEntry {
        name: "lessmsi",
        citation: "C084 UniExtract.au3:2843-2845",
        build: lessmsi,
    },
    FormatEntry {
        name: "lzip",
        citation: "C080 UniExtract.au3:2783-2784",
        build: lzip,
    },
    FormatEntry {
        name: "lzop",
        citation: "C081 UniExtract.au3:2786-2787",
        build: lzop,
    },
    FormatEntry {
        name: "lzx",
        citation: "C082 UniExtract.au3:2789-2790",
        build: lzx,
    },
    FormatEntry {
        name: "mole",
        citation: "C083 UniExtract.au3:2792-2811",
        build: mole,
    },
    FormatEntry {
        name: "msiexec",
        citation: "C087 UniExtract.au3:2882-2883",
        build: msiexec,
    },
    FormatEntry {
        name: "msix",
        citation: "C086 UniExtract.au3:2862-2864,2887-2889,2907-2908",
        build: msix,
    },
    FormatEntry {
        name: "nbh",
        citation: "C088 UniExtract.au3:2952-2953",
        build: nbh,
    },
    FormatEntry {
        name: "rar",
        citation: "C092 UniExtract.au3:3005",
        build: rar,
    },
    FormatEntry {
        name: "rgss",
        citation: "C093 UniExtract.au3:3009-3011",
        build: rgss,
    },
    FormatEntry {
        name: "rpa",
        citation: "C094 UniExtract.au3:3016-3017",
        build: rpa,
    },
    FormatEntry {
        name: "sfark",
        citation: "C095 UniExtract.au3:3019-3020",
        build: sfark,
    },
    FormatEntry {
        name: "sqlite",
        citation: "C097 UniExtract.au3:3032-3033",
        build: sqlite,
    },
    FormatEntry {
        name: "superdat",
        citation: "C118 UniExtract.au3:3038-3043",
        build: superdat,
    },
    FormatEntry {
        name: "ttarch",
        citation: "C100 UniExtract.au3:3147",
        build: ttarch,
    },
    FormatEntry {
        name: "uharc",
        citation: "C101 UniExtract.au3:3154",
        build: uharc,
    },
    FormatEntry {
        name: "uharc04",
        citation: "C101 UniExtract.au3:3156",
        build: uharc,
    },
    FormatEntry {
        name: "uharc02",
        citation: "C101 UniExtract.au3:3158",
        build: uharc,
    },
    FormatEntry {
        name: "uif",
        citation: "C102 UniExtract.au3:3161-3163",
        build: uif,
    },
    FormatEntry {
        name: "unreal",
        citation: "C103 UniExtract.au3:3211-3214",
        build: unreal,
    },
    FormatEntry {
        name: "unzip",
        citation: "C109 UniExtract.au3:3384-3388",
        build: unzip,
    },
    FormatEntry {
        name: "upx",
        citation: "C112 UniExtract.au3:3617-3623",
        build: upx,
    },
    FormatEntry {
        name: "wix",
        citation: "C107 UniExtract.au3:3373-3375",
        build: wix,
    },
    FormatEntry {
        name: "wolf",
        citation: "C108 UniExtract.au3:3377-3382",
        build: wolf,
    },
    FormatEntry {
        name: "xor",
        citation: "C070 UniExtract.au3:2598",
        build: xor,
    },
    FormatEntry {
        name: "zoo",
        citation: "C110 UniExtract.au3:3390-3394",
        build: zoo,
    },
    FormatEntry {
        name: "zpaq",
        citation: "C111 UniExtract.au3:3396-3399",
        build: zpaq,
    },
    FormatEntry {
        name: "mscf",
        citation: "C120 UniExtract.au3:2827",
        build: mscf,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The table has exactly one row per collapsed format, with unique
    /// names — a cheap sanity net for future edits to `FORMATS`.
    #[test]
    fn table_has_47_unique_formats() {
        assert_eq!(FORMATS.len(), 47);
        let mut names: Vec<&str> = FORMATS.iter().map(|e| e.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 47);
    }

    /// Parity test for capability C057: matches UniExtract.au3:2346-2349's
    /// `_Run($ace & ' -x -v -d "' & $outdir & '" "' & $file & '"',
    /// $outdir, @SW_HIDE, True, True, True, True)`.
    #[test]
    fn ace_matches_source_invocation() {
        let inv = ace(&Ctx {
            program: r"C:\UniExtract\bin\acefile.exe",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.ace",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\acefile.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                "-v".to_string(),
                "-d".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.ace".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C115: matches UniExtract.au3:2385-2390's
    /// effective `<file> /extract:<outdir>` call.
    #[test]
    fn ai_matches_source_invocation() {
        let inv = ai(&Ctx {
            file: r"C:\downloads\installer.exe",
            outdir: r"C:\downloads\installer_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![r"/extract:C:\downloads\installer_unpacked".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C113: matches UniExtract.au3:2394's
    /// effective `arc_conv.exe "<file>"` call.
    #[test]
    fn arc_conv_matches_source_invocation() {
        let inv = arc_conv(&Ctx {
            program: r"C:\UniExtract\bin\arc_conv.exe",
            file: r"C:\downloads\archive.arc",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\arc_conv.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.arc".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C058: matches UniExtract.au3:3624-3625's
    /// `_Run($aspack & ' "' & $file & '" "' & $sPath & '" /NO_PROMPT',
    /// $filedir)`.
    #[test]
    fn aspack_matches_source_invocation() {
        let inv = aspack(&Ctx {
            program: r"C:\UniExtract\bin\AspackDie.exe",
            file: r"C:\downloads\archive_unpacked\packed.exe",
            dest_path: r"C:\downloads\archive_unpacked\packed_unpacked.exe",
            file_dir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\AspackDie.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive_unpacked\packed.exe".to_string(),
                r"C:\downloads\archive_unpacked\packed_unpacked.exe".to_string(),
                "/NO_PROMPT".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C062: matches UniExtract.au3:2418-2419's
    /// `_Run($bcm & ' -d "' & $file & '" "' & $outdir & '\' & GetFileName()
    /// & '"', $filedir, @SW_HIDE, True, True, False)`.
    #[test]
    fn bcm_matches_source_invocation() {
        let inv = bcm(&Ctx {
            program: r"C:\UniExtract\bin\bcm.exe",
            file_dir: r"C:\downloads",
            file: r"C:\downloads\archive.bcm",
            outdir: r"C:\downloads\archive_unpacked",
            filename: "archive",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\bcm.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                r"C:\downloads\archive.bcm".to_string(),
                r"C:\downloads\archive_unpacked\archive".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C063: matches UniExtract.au3:2421-2429's
    /// effective `bootimg.exe --unpack-bootimg` call.
    #[test]
    fn bootimg_matches_source_invocation() {
        let inv = bootimg(&Ctx {
            program: r"C:\downloads\image_unpacked\bootimg.exe",
            outdir: r"C:\downloads\image_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\downloads\image_unpacked\bootimg.exe");
        assert_eq!(inv.args, vec!["--unpack-bootimg".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\image_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C065: matches UniExtract.au3:2441-2442's
    /// `_Run($chd & ' extracthd -i "' & $file & '" -o "' & $outdir & '\' &
    /// $filename & '.img"', $outdir)`.
    #[test]
    fn chdman_matches_source_invocation() {
        let inv = chdman(&Ctx {
            program: r"C:\UniExtract\bin\chdman.exe",
            outdir: r"C:\images\disk_unpacked",
            file: r"C:\images\disk.chd",
            filename: "disk",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\chdman.exe");
        assert_eq!(
            inv.args,
            vec![
                "extracthd".to_string(),
                "-i".to_string(),
                r"C:\images\disk.chd".to_string(),
                "-o".to_string(),
                r"C:\images\disk_unpacked\disk.img".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\images\disk_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C067: matches UniExtract.au3:2472-2475's
    /// `_Run($cic & ' -db "' & $file & '" "' & $outdir & '"', $filedir,
    /// @SW_HIDE)`.
    #[test]
    fn cic_matches_source_invocation() {
        let inv = cic(&Ctx {
            program: r"C:\UniExtract\bin\cicdec.exe",
            file: r"C:\downloads\installer.exe",
            outdir: r"C:\downloads\installer_unpacked",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\cicdec.exe");
        assert_eq!(
            inv.args,
            vec![
                "-db".to_string(),
                r"C:\downloads\installer.exe".to_string(),
                r"C:\downloads\installer_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C146: matches UniExtract.au3:2505-2508's
    /// `Local $sFile = $outdir & "\" & $filename & ".iso"` followed by
    /// `_Run($daa & ' "' & $file & '" "' & $sFile & '"', $outdir)`.
    #[test]
    fn daa_matches_source_invocation() {
        let inv = daa(&Ctx {
            program: r"C:\UniExtract\bin\daa2iso.exe",
            file: r"C:\downloads\image.daa",
            outdir: r"C:\downloads\image_unpacked",
            filename: "image",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\daa2iso.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\image.daa".to_string(),
                r"C:\downloads\image_unpacked\image.iso".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\image_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C116: matches UniExtract.au3:2514-2516's
    /// effective `<file> /batch /no-reg /no-postinstall /dest "<outdir>"`
    /// call.
    #[test]
    fn ei_matches_source_invocation() {
        let inv = ei(&Ctx {
            file: r"C:\downloads\installer.exe",
            outdir: r"C:\downloads\installer_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![
                "/batch".to_string(),
                "/no-reg".to_string(),
                "/no-postinstall".to_string(),
                "/dest".to_string(),
                r"C:\downloads\installer_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C096: matches UniExtract.au3:3026's
    /// `_Run($extsis & ' -x -xcsd "' & $file & '" -d "' & $tempoutdir &
    /// '"', $tempoutdir, @SW_MINIMIZE)`.
    #[test]
    fn extsis_matches_source_invocation() {
        let inv = extsis(&Ctx {
            program: r"C:\UniExtract\bin\extsis.exe",
            file: r"C:\downloads\app.sis",
            tempoutdir: r"C:\downloads\app_unpacked\tmp123456",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\extsis.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                "-xcsd".to_string(),
                r"C:\downloads\app.sis".to_string(),
                "-d".to_string(),
                r"C:\downloads\app_unpacked\tmp123456".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\app_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C117: matches UniExtract.au3:2530-2536's
    /// effective `<file> /s -nos_ne -nos_o<tempoutdir>\` call.
    #[test]
    fn fead_matches_source_invocation() {
        let inv = fead(&Ctx {
            file: r"C:\downloads\installer.exe",
            tempoutdir: r"C:\downloads\installer_temp",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\downloads\installer.exe");
        assert_eq!(
            inv.args,
            vec![
                "/s".to_string(),
                "-nos_ne".to_string(),
                r"-nos_oC:\downloads\installer_temp\".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C071: matches UniExtract.au3:2556-2557's
    /// `_Run($freearc & ' x -dp"' & $outdir & '" "' & $file & '"',
    /// $filedir, @SW_HIDE, True, True, False, False)`.
    #[test]
    fn freearc_matches_source_invocation() {
        let inv = freearc(&Ctx {
            program: r"C:\UniExtract\bin\unarc.exe",
            file_dir: r"C:\downloads",
            file: r"C:\downloads\archive.arc",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unarc.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r#"-dp"C:\downloads\archive_unpacked""#.to_string(),
                r"C:\downloads\archive.arc".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C072: matches UniExtract.au3:2559-2562's
    /// `_Run($fsb & ' -o -1 -A -d "' & $outdir & '" "' & $file & '"',
    /// $filedir, @SW_MINIMIZE, True, True, False)`.
    #[test]
    fn fsb_matches_source_invocation() {
        let inv = fsb(&Ctx {
            program: r"C:\UniExtract\bin\fsbext.exe",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.fsb",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\fsbext.exe");
        assert_eq!(
            inv.args,
            vec![
                "-o".to_string(),
                "-1".to_string(),
                "-A".to_string(),
                "-d".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.fsb".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C068: matches UniExtract.au3:2565-2566's
    /// `_Run($garbro & ' x -ocu -if png -o "' & $outdir & '" "' & $file &
    /// '"', $outdir, @SW_MINIMIZE)`.
    #[test]
    fn garbro_matches_source_invocation() {
        let inv = garbro(&Ctx {
            program: r"C:\UniExtract\bin\GARbro.Console.exe",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.arc",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\GARbro.Console.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                "-ocu".to_string(),
                "-if".to_string(),
                "png".to_string(),
                "-o".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.arc".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C076: matches UniExtract.au3:2711's
    /// `IsXunpack.exe "<outdir>\<filenamefull>"` call.
    #[test]
    fn isxunpack_matches_source_invocation() {
        let inv = isxunpack(&Ctx {
            program: r"C:\UniExtract\bin\IsXunpack.exe",
            outdir: r"C:\downloads\installer_unpacked",
            filename_full: "installer.exe",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\IsXunpack.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\installer_unpacked\installer.exe".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C078: matches UniExtract.au3:2775-2778's
    /// `_RunInTempOutdir($tempoutdir, $isz & ' "' & $file & '"',
    /// $tempoutdir, True, True)`.
    #[test]
    fn isz_matches_source_invocation() {
        let inv = isz(&Ctx {
            program: r"C:\UniExtract\bin\unisz.exe",
            file: r"C:\downloads\archive.isz",
            tempoutdir: r"C:\downloads\archive_unpacked\tmp123456",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unisz.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.isz".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C085: matches UniExtract.au3:2858's
    /// effective `jsMSIx.exe "<file>|<outdir>"` call.
    #[test]
    fn jsmsix_matches_source_invocation() {
        let inv = jsmsix(&Ctx {
            program: r"C:\UniExtract\bin\jsMSIx.exe",
            file: r"C:\downloads\installer.msi",
            outdir: r"C:\downloads\installer",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\jsMSIx.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\installer.msi|C:\downloads\installer".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C079: matches UniExtract.au3:2780-2781's
    /// `_Run($kgb & ' "' & $file & '"', $outdir, @SW_MINIMIZE, True, False,
    /// False)`.
    #[test]
    fn kgb_matches_source_invocation() {
        let inv = kgb(&Ctx {
            program: r"C:\UniExtract\bin\kgb2_console.exe",
            file: r"C:\downloads\archive.kgb",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\kgb2_console.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.kgb".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C084: matches UniExtract.au3:2843-2845's
    /// `lessmsi.exe x "<file>" "<outdir>\"` call.
    #[test]
    fn lessmsi_matches_source_invocation() {
        let inv = lessmsi(&Ctx {
            program: r"C:\UniExtract\bin\lessmsi.exe",
            file: r"C:\downloads\installer.msi",
            outdir: r"C:\downloads\installer",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\lessmsi.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\downloads\installer.msi".to_string(),
                r"C:\downloads\installer\".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\installer");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C080: matches UniExtract.au3:2783-2784's
    /// `_RunInTempOutdir($tempoutdir, $lz & ' -d -k -v -v "' & $file & '"',
    /// $tempoutdir, @SW_SHOW, True, True, False)`.
    #[test]
    fn lzip_matches_source_invocation() {
        let inv = lzip(&Ctx {
            program: r"C:\UniExtract\bin\lzip.exe",
            file: r"C:\downloads\archive.tar.lz",
            tempoutdir: r"C:\downloads\archive_unpacked\tmp123456",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\lzip.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                "-k".to_string(),
                "-v".to_string(),
                "-v".to_string(),
                r"C:\downloads\archive.tar.lz".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked\tmp123456");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C081: matches UniExtract.au3:2786-2787's
    /// `_Run($lzo & ' -d -p"' & $outdir & '" "' & $file & '"', $filedir)`.
    #[test]
    fn lzop_matches_source_invocation() {
        let inv = lzop(&Ctx {
            program: r"C:\UniExtract\bin\lzop.exe",
            file_dir: r"C:\downloads",
            outdir: r"C:\downloads\app_unpacked",
            file: r"C:\downloads\app.lzo",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\lzop.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                r#"-p"C:\downloads\app_unpacked""#.to_string(),
                r"C:\downloads\app.lzo".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C082: matches UniExtract.au3:2789-2790's
    /// `_Run($lzx & ' -x "' & $file & '"', $outdir)`.
    #[test]
    fn lzx_matches_source_invocation() {
        let inv = lzx(&Ctx {
            program: r"C:\UniExtract\bin\unlzx.exe",
            file: r"C:\downloads\archive.lzx",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unlzx.exe");
        assert_eq!(
            inv.args,
            vec!["-x".to_string(), r"C:\downloads\archive.lzx".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C083: matches UniExtract.au3:2792-2811's
    /// `_RunInTempOutdir($tempoutdir, $mole & ' /nogui "' & $file & '"',
    /// $outdir, @SW_HIDE, True, False, False)`.
    #[test]
    fn mole_matches_source_invocation() {
        let inv = mole(&Ctx {
            program: r"C:\UniExtract\bin\demoleition.exe",
            file: r"C:\downloads\archive.exe",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\demoleition.exe");
        assert_eq!(
            inv.args,
            vec![
                "/nogui".to_string(),
                r"C:\downloads\archive.exe".to_string()
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C087: matches UniExtract.au3:2882-2883's
    /// effective `msiexec.exe /a "<file>" /qb TARGETDIR="<outdir>"` call.
    #[test]
    fn msiexec_matches_source_invocation() {
        let inv = msiexec(&Ctx {
            file: r"C:\downloads\installer.msi",
            outdir: r"C:\downloads\installer",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, "msiexec.exe");
        assert_eq!(
            inv.args,
            vec![
                "/a".to_string(),
                r"C:\downloads\installer.msi".to_string(),
                "/qb".to_string(),
                r"TARGETDIR=C:\downloads\installer".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C086: `append_ext = false` omits the
    /// `/ext` argument.
    #[test]
    fn msix_matches_source_invocation_without_ext() {
        let inv = msix(&Ctx {
            program: r"C:\UniExtract\bin\MsiX.exe",
            file: r"C:\downloads\installer.msi",
            outdir: r"C:\downloads\installer",
            file_dir: r"C:\downloads",
            append_ext: false,
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\MsiX.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\installer.msi".to_string(),
                "/out".to_string(),
                r"C:\downloads\installer".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C086: `append_ext = true` appends
    /// `/ext`, matching `$TYPE_MSP`'s unconditional case.
    #[test]
    fn msix_matches_source_invocation_with_ext() {
        let inv = msix(&Ctx {
            program: r"C:\UniExtract\bin\MsiX.exe",
            file: r"C:\downloads\patch.msp",
            outdir: r"C:\downloads\patch",
            file_dir: r"C:\downloads",
            append_ext: true,
            ..Default::default()
        });
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\patch.msp".to_string(),
                "/out".to_string(),
                r"C:\downloads\patch".to_string(),
                "/ext".to_string(),
            ]
        );
    }

    /// Parity test for capability C088: matches UniExtract.au3:2952-2953's
    /// effective `NBHextract.exe "<file>"` call.
    #[test]
    fn nbh_matches_source_invocation() {
        let inv = nbh(&Ctx {
            program: r"C:\UniExtract\bin\NBHextract.exe",
            file: r"C:\downloads\ROM.nbh",
            outdir: r"C:\downloads\ROM_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\NBHextract.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\ROM.nbh".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\ROM_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C092: no password resolved builds
    /// `x -kb "<file>"`, matching `$sPassword == 0`.
    #[test]
    fn rar_matches_source_invocation_without_password() {
        let inv = rar(&Ctx {
            program: r"C:\UniExtract\bin\UnRAR.exe",
            file: r"C:\downloads\archive.rar",
            outdir: r"C:\downloads",
            password: None,
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\UnRAR.exe");
        assert_eq!(
            inv.args,
            vec!["x", "-kb", r"C:\downloads\archive.rar"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C092: a resolved password is folded
    /// into the `-p<password>` argument.
    #[test]
    fn rar_matches_source_invocation_with_password() {
        let inv = rar(&Ctx {
            program: r"C:\UniExtract\bin\UnRAR.exe",
            file: r"C:\downloads\archive.rar",
            outdir: r"C:\downloads",
            password: Some("hunter2"),
            ..Default::default()
        });
        assert_eq!(
            inv.args,
            vec!["x", "-kb", "-phunter2", r"C:\downloads\archive.rar"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C093: matches UniExtract.au3:3009-3011's
    /// `_Run($rgss & ' -p -o="' & $outdir & '" "' & $file & '"', $outdir,
    /// @SW_HIDE)`.
    #[test]
    fn rgss_matches_source_invocation() {
        let inv = rgss(&Ctx {
            program: r"C:\UniExtract\bin\RgssDecrypter.exe",
            file: r"C:\games\Game.rgss3a",
            outdir: r"C:\games\Game_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\RgssDecrypter.exe");
        assert_eq!(
            inv.args,
            vec![
                "-p".to_string(),
                r"-o=C:\games\Game_unpacked".to_string(),
                r"C:\games\Game.rgss3a".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\games\Game_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C094: matches UniExtract.au3:3016-3017's
    /// `_Run($rpa & ' -m -v --continue-on-error -p "' & $outdir & '" "' &
    /// $file & '"', @ScriptDir, True, True, True)`.
    #[test]
    fn rpa_matches_source_invocation() {
        let inv = rpa(&Ctx {
            program: r"C:\UniExtract\bin\unrpa.exe",
            script_dir: r"C:\UniExtract",
            file: r"C:\games\game.rpa",
            outdir: r"C:\games\game_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unrpa.exe");
        assert_eq!(
            inv.args,
            vec![
                "-m".to_string(),
                "-v".to_string(),
                "--continue-on-error".to_string(),
                "-p".to_string(),
                r"C:\games\game_unpacked".to_string(),
                r"C:\games\game.rpa".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\UniExtract");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C095: matches UniExtract.au3:3019-3020's
    /// `_Run($sfark & ' "' & $file & '" "' & $outdir & '\' & $filename &
    /// '.sf2"', $filedir, @SW_SHOW)`.
    #[test]
    fn sfark_matches_source_invocation() {
        let inv = sfark(&Ctx {
            program: r"C:\UniExtract\bin\sfarkxtc.exe",
            file_dir: r"C:\downloads",
            file: r"C:\downloads\MySoundFont.sfArk",
            outdir: r"C:\downloads\MySoundFont_unpacked",
            filename: "MySoundFont",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\sfarkxtc.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\MySoundFont.sfArk".to_string(),
                r"C:\downloads\MySoundFont_unpacked\MySoundFont.sf2".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C097: matches UniExtract.au3:3032-3033's
    /// effective `sqlite3.exe "<file>" .dump` call.
    #[test]
    fn sqlite_matches_source_invocation() {
        let inv = sqlite(&Ctx {
            program: r"C:\UniExtract\bin\sqlite3.exe",
            file: r"C:\downloads\data.db",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\sqlite3.exe");
        assert_eq!(
            inv.args,
            vec![r"C:\downloads\data.db".to_string(), ".dump".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C118: matches UniExtract.au3:3038-3043's
    /// effective `<file> /LOGFILE "<outdir>\SuperDAT.log" /e "<outdir>"`
    /// call.
    #[test]
    fn superdat_matches_source_invocation() {
        let inv = superdat(&Ctx {
            file: r"C:\downloads\updater.exe",
            outdir: r"C:\downloads\updater_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\downloads\updater.exe");
        assert_eq!(
            inv.args,
            vec![
                "/LOGFILE".to_string(),
                r"C:\downloads\updater_unpacked\SuperDAT.log".to_string(),
                "/e".to_string(),
                r"C:\downloads\updater_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\updater_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C100: matches UniExtract.au3:3147's
    /// `ttarchext.exe -m <index> "<file>" "<outdir>"` call — the game
    /// index is a bare numeric token, not quoted.
    #[test]
    fn ttarch_matches_source_invocation() {
        let inv = ttarch(&Ctx {
            program: r"C:\UniExtract\bin\ttarchext.exe",
            game_index: 7,
            file: r"C:\downloads\archive.ttarch",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\ttarchext.exe");
        assert_eq!(
            inv.args,
            vec![
                "-m".to_string(),
                "7".to_string(),
                r"C:\downloads\archive.ttarch".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C101: the first attempt matches
    /// UniExtract.au3:3154's `UNUHARC06.EXE x -t"<outdir>" "<file>"`.
    #[test]
    fn uharc_matches_source_invocation() {
        let inv = uharc(&Ctx {
            program: r"C:\UniExtract\bin\UNUHARC06.EXE",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.uha",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\UNUHARC06.EXE");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.uha".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C101: the second fallback attempt
    /// matches UniExtract.au3:3156 — same shape as the first, a different
    /// binary.
    #[test]
    fn uharc04_matches_source_invocation() {
        let inv = uharc(&Ctx {
            program: r"C:\UniExtract\bin\UHARC04.EXE",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.uha",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\UHARC04.EXE");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.uha".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C101: the third fallback attempt matches
    /// UniExtract.au3:3158 — 8.3 short-form paths, passed through the same
    /// `outdir`/`file` slots as the long-form attempts above.
    #[test]
    fn uharc02_matches_source_invocation() {
        let inv = uharc(&Ctx {
            program: r"C:\UniExtract\bin\UHARC02.EXE",
            outdir: r"C:\DOWNLO~1\ARCHIV~1",
            file: r"C:\DOWNLO~1\ARCHIV~2.UHA",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\UHARC02.EXE");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"-tC:\DOWNLO~1\ARCHIV~1".to_string(),
                r"C:\DOWNLO~1\ARCHIV~2.UHA".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\DOWNLO~1\ARCHIV~1");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C102: matches UniExtract.au3:3161-3163's
    /// `_Run($uif & ' "' & $file & '" "' & $outdir & "\" & $filename & '"',
    /// $filedir, True, True, True)`.
    #[test]
    fn uif_matches_source_invocation() {
        let inv = uif(&Ctx {
            program: r"C:\UniExtract\bin\uif2iso.exe",
            file: r"C:\downloads\image.uif",
            outdir: r"C:\downloads\image_unpacked",
            filename: "image.iso",
            file_dir: r"C:\downloads",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\uif2iso.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\image.uif".to_string(),
                r"C:\downloads\image_unpacked\image.iso".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C103: matches UniExtract.au3:3211-3214's
    /// `_Run($unreal & ' -export -all -sounds -3rdparty -path="' &
    /// $filedir & '" -out="' & $outdir & '" *', $outdir, @SW_MINIMIZE,
    /// True, True, False)`.
    #[test]
    fn unreal_matches_source_invocation() {
        let inv = unreal(&Ctx {
            program: r"C:\UniExtract\bin\umodel.exe",
            file_dir: r"C:\downloads",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\umodel.exe");
        assert_eq!(
            inv.args,
            vec![
                "-export".to_string(),
                "-all".to_string(),
                "-sounds".to_string(),
                "-3rdparty".to_string(),
                r#"-path="C:\downloads""#.to_string(),
                r#"-out="C:\downloads\archive_unpacked""#.to_string(),
                "*".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C109: matches UniExtract.au3:3384-3388's
    /// `_Run($zip & ' -x "' & $file & '"', $outdir, @SW_MINIMIZE, True,
    /// False)`.
    #[test]
    fn unzip_matches_source_invocation() {
        let inv = unzip(&Ctx {
            program: r"C:\UniExtract\bin\unzip.exe",
            file: r"C:\downloads\archive.zip",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unzip.exe");
        assert_eq!(
            inv.args,
            vec!["-x".to_string(), r"C:\downloads\archive.zip".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C112: matches UniExtract.au3:3617-3623's
    /// `_Run($upx & ' -d -k "' & $file & '"', $filedir)`.
    #[test]
    fn upx_matches_source_invocation() {
        let inv = upx(&Ctx {
            program: r"C:\UniExtract\bin\upx.exe",
            file: r"C:\downloads\archive_unpacked\packed.exe",
            file_dir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\upx.exe");
        assert_eq!(
            inv.args,
            vec![
                "-d".to_string(),
                "-k".to_string(),
                r"C:\downloads\archive_unpacked\packed.exe".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C107: matches UniExtract.au3:3373-3375's
    /// `_Run($wix & ' -x "' & $outdir & '" "' & $file & '"', $outdir,
    /// @SW_MINIMIZE, True, True, False)`.
    #[test]
    fn wix_matches_source_invocation() {
        let inv = wix(&Ctx {
            program: r"C:\UniExtract\bin\dark.exe",
            outdir: r"C:\downloads\archive_unpacked",
            file: r"C:\downloads\archive.msi",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\dark.exe");
        assert_eq!(
            inv.args,
            vec![
                "-x".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
                r"C:\downloads\archive.msi".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C108: matches UniExtract.au3:3377-3382's
    /// `_RunInTempOutdir($tempoutdir, $wolf & ' ' & Quote($file), $outdir,
    /// @SW_MINIMIZE, True, True, False)`.
    #[test]
    fn wolf_matches_source_invocation() {
        let inv = wolf(&Ctx {
            program: r"C:\UniExtract\bin\WolfDec.exe",
            file: r"C:\downloads\archive.wolf",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\WolfDec.exe");
        assert_eq!(inv.args, vec![r"C:\downloads\archive.wolf".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C070: matches UniExtract.au3:2598's
    /// `_Run($xor & ' "' & $ret2 & '" "' & $outdir & '\' & $filename &
    /// '.cab" 0x8D')`.
    #[test]
    fn xor_matches_source_invocation() {
        let inv = xor(&Ctx {
            program: r"C:\UniExtract\bin\xor.exe",
            file: r"C:\downloads\archive_unpacked\overlay.bin",
            outdir: r"C:\downloads\archive_unpacked",
            filename: "archive",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\xor.exe");
        assert_eq!(
            inv.args,
            vec![
                r"C:\downloads\archive_unpacked\overlay.bin".to_string(),
                r"C:\downloads\archive_unpacked\archive.cab".to_string(),
                "0x8D".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Minimized);
    }

    /// Parity test for capability C110: matches UniExtract.au3:3390-3394's
    /// `_Run($zoo & ' -x ' & $filenamefull, $tempoutdir, @SW_HIDE)`.
    #[test]
    fn zoo_matches_source_invocation() {
        let inv = zoo(&Ctx {
            program: r"C:\UniExtract\bin\unzoo.exe",
            filename_full: "archive.zoo",
            tempoutdir: r"C:\downloads\archive_temp",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\unzoo.exe");
        assert_eq!(inv.args, vec!["-x".to_string(), "archive.zoo".to_string()]);
        assert_eq!(inv.working_dir, r"C:\downloads\archive_temp");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C111: matches UniExtract.au3:3396-3399's
    /// `_Run($zpaq & ' x "' & $file & '" -to "' & $outdir & '"', $outdir,
    /// @SW_SHOW, True, True, False)`.
    #[test]
    fn zpaq_matches_source_invocation() {
        let inv = zpaq(&Ctx {
            program: r"C:\UniExtract\bin\zpaq.exe",
            file: r"C:\downloads\archive.zpaq",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\zpaq.exe");
        assert_eq!(
            inv.args,
            vec![
                "x".to_string(),
                r"C:\downloads\archive.zpaq".to_string(),
                "-to".to_string(),
                r"C:\downloads\archive_unpacked".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\archive_unpacked");
        assert_eq!(inv.window, WindowMode::Show);
    }

    /// Parity test for capability C120: matches UniExtract.au3:2827's
    /// effective `7z.exe x "<cab_file>"` call.
    #[test]
    fn mscf_matches_source_invocation() {
        let inv = mscf(&Ctx {
            program: r"C:\UniExtract\bin\7z.exe",
            file: r"C:\Temp\mscf_tmp\data1.cab",
            tempoutdir: r"C:\Temp\mscf_tmp",
            ..Default::default()
        });
        assert_eq!(inv.program, r"C:\UniExtract\bin\7z.exe");
        assert_eq!(
            inv.args,
            vec!["x".to_string(), r"C:\Temp\mscf_tmp\data1.cab".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\Temp\mscf_tmp");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// The table lookup fn (`build`) returns the same [`Invocation`] as
    /// calling the format's builder directly, and `None` for an unknown
    /// name — the main integration point other than direct-fn calls (see
    /// `main.rs`'s wiring for `rgss`/`ace`).
    #[test]
    fn build_looks_up_by_name() {
        let ctx = Ctx {
            program: r"C:\UniExtract\bin\kgb2_console.exe",
            file: r"C:\downloads\archive.kgb",
            outdir: r"C:\downloads\archive_unpacked",
            ..Default::default()
        };
        assert_eq!(build("kgb", &ctx), Some(kgb(&ctx)));
        assert_eq!(build("not-a-real-format", &ctx), None);
    }
}
