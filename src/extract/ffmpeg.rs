//! ffmpeg (`ffmpeg.exe`) — audio conversion to WAV, video/audio track
//! extraction from containers.
//!
//! **`$TYPE_VIDEO`'s per-stream extraction** (UniExtract.au3:3216-3283),
//! completing the gap this module's earlier partial coverage left open:
//! parsing ffmpeg's raw `-i` stderr for a `Not enough...`/`Stream`
//! marker, splitting on the literal `"Stream"` token, regex-parsing each
//! segment's `#<idx>(<lang>): <Category>: <codec>` header, classifying
//! each stream into an action (image-sequence split, h264-specific
//! extraction, ordinary video/audio extraction, or an unrecognized
//! category), and building each extraction's `_MakeFFmpegCommand`-shaped
//! output filename and full invocation.
//!
//! ```autoit
//! Local $command = $ffmpeg & ' -i "' & $file & '"'
//! Local $return = FetchStdout($command, $outdir, @SW_HIDE)
//! If StringInStr($return, "Invalid data found when processing input") Or Not StringInStr($return, "Stream") Then terminate(...)
//!
//! Local $aStreams = StringSplit($return, "Stream", 1)
//! Local $iStreams = $aStreams[0] - 2
//! If $fileext == "wma" And $iStreams < 2 Then extract($TYPE_AUDIO, ...)
//!
//! For $i = 2 To $aStreams[0]
//!     $aStreams[$i] = StringRegExpReplace($aStreams[$i], "(?i)(?s).*?#(\d:\d)(.*?): (\w+): (\w+).*", "$3,$4,$1,$2")
//!     $aStreamType = StringSplit($aStreams[$i], ",")
//!     If $aStreamType[1] == "Video" Then
//!         If $aStreamType[2] == "gif" Or $aStreamType[2] == "apng" Or $aStreamType[2] == "webp" Then
//!             _Run($command & ' "' & GetFileName() & '%05d.png"', ...)
//!         ElseIf Not $bOptExtractVideo Then
//!             ContinueLoop
//!         ElseIf $aStreamType[2] == "h264" Then
//!             _Run(_MakeFFmpegCommand($command & ' -vcodec copy -an -bsf:v h264_mp4toannexb -map ', $aStreamType, t('TERM_VIDEO'), $iVideo), ...)
//!         Else
//!             ; wmv/mpeg/vp8/flv codec-name remapping, then plain -vcodec copy extraction
//!         EndIf
//!     ElseIf $aStreamType[1] == "Audio" Then
//!         ; wma/vorbis/pcm codec-name remapping, then -acodec copy extraction
//!     EndIf
//! Next
//! If $iVideo + $iAudio < 1 Then terminate($STATUS_NOTPACKED, ...)
//! ```
//!
//! **A genuine off-by-one, preserved exactly**: `$iStreams` (computed as
//! `$aStreams[0] - 2`) under-counts by one relative to the number of
//! segments the loop actually processes (`For $i = 2 To $aStreams[0]`
//! processes `$aStreams[0] - 1` segments). [`undercounted_stream_count`]
//! reproduces the exact (wrong) arithmetic, not a "corrected" count —
//! it feeds directly into the WMA shortcut check (`$fileext == "wma"
//! And $iStreams < 2`), which as a result actually fires for up to
//! *two* real streams, not fewer than two, despite its own name.
//!
//! **Two different case-sensitivity rules in the same block, easy to
//! conflate**: the category check (`$aStreamType[1] == "Video"`/
//! `"Audio"`) and the gif/apng/webp/h264 exact-codec checks all use `==`
//! (case-sensitive); the wmv/mpeg/vp8/flv/wma/vorbis/pcm remapping uses
//! bare `StringInStr` (case-insensitive substring). [`classify_stream`]
//! and the two `normalize_*_codec` helpers preserve this split exactly.
//!
//! **A real asymmetry in dash-stripping**: `_MakeFFmpegCommand` strips
//! every leading `-` from the output base name before building a
//! filename (UniExtract.au3:5121-5123) — but the gif/apng/webp
//! image-sequence branch calls `GetFileName()` *directly*, bypassing
//! `_MakeFFmpegCommand` (and its dash-stripping) entirely. Modeled via
//! [`build_stream_output_filename`]'s own `strip_leading_dashes`
//! parameter, `false` for the image-sequence case.
//!
//! **A genuine, accepted limitation**: the header regex's stream-index
//! group is exactly `\d:\d` (one digit each side of the colon, not
//! `\d+:\d+`). For a real stream index of 10 or higher (e.g. `#0:10`),
//! PCRE backtracking would still match by letting the *next* lazy group
//! absorb the extra digit, silently truncating the captured index to
//! `"0:1"`. [`parse_stream_segment`] doesn't reproduce this backtracking
//! edge case — it requires exactly one digit each side and returns
//! `None` otherwise, which differs from the source only for
//! double-digit-or-higher stream indices, a case rare enough (and messy
//! enough to hand-replicate byte-for-byte) that this is a documented,
//! accepted limitation rather than a silent behavior gap.
//!
//! **`StringFormat("_%02s", $iIndex)`'s zero-padding** is applied here
//! via ordinary numeric zero-padding (`{index:02}`), on the reading that
//! `$iIndex` is always a plain counter and the evident intent is
//! `"01"`/`"02"`-style padding — Windows CRT `sprintf` does honor the
//! `0` flag for `%s` conversions the same as for numeric ones, so this
//! isn't a guess, but it's flagged since AutoIt's own docs don't spell
//! out `%0Ns`'s behavior explicitly.
//!
//! **Not modeled**: `HasFFMPEG()` (real plugin-existence check, possibly
//! GUI download-prompt), `FetchStdout` itself, and the actual `_Run`
//! process spawn — the same boundaries documented throughout this port.

use super::{Invocation, WindowMode};

/// Builds the invocation `Case $TYPE_AUDIO` (UniExtract.au3:2414-2416)
/// makes: `<program> -i "<file>" "<filename_stem>.wav"`, run in `outdir`
/// with the window hidden.
///
/// **Scope note — shell wrapping not modeled as a literal string:** the
/// source builds this via a literal `cmd.exe /d /c ` prefix concatenated
/// directly onto the command string (the same idiom already documented
/// for `extract::expand`/`extract::bootimg`) — functionally still `<ffmpeg>
/// -i "<file>" "<filename_stem>.wav"`, so this port's `Invocation` targets
/// the exe directly.
pub fn audio_invocation(
    program: &str,
    file: &str,
    filename_stem: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            format!("{filename_stem}.wav"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the invocation `Case $TYPE_VIDEO_CONVERT`
/// (UniExtract.au3:3288) makes: `<program> -i "<file>"
/// "<filename_stem>.mp4"`, run in `outdir` with the window hidden.
pub fn video_convert_invocation(
    program: &str,
    file: &str,
    filename_stem: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            format!("{filename_stem}.mp4"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds `Case $TYPE_VIDEO`'s stream-info probe (UniExtract.au3:3220-3221):
/// `<program> -i "<file>"`, run in `outdir` with the window hidden. ffmpeg
/// reports container stream info on stderr when given no output file —
/// captured via `FetchStdout` in the source, the same probe-then-classify
/// shape as `detection::sevenzip_probe`/`detection::alz_probe`/
/// `detection::arj_probe`.
///
/// The classification this probe's captured output feeds — splitting on
/// `"Stream"`, regex-matching each stream's codec/type/index, and
/// dispatching one extraction command per stream — is
/// [`probe_output_unreadable`]/[`undercounted_stream_count`]/
/// [`parse_stream_segment`]/[`classify_stream`] and the invocation
/// builders below.
pub fn probe_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-i".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports the terminate check (UniExtract.au3:3224): the probe output
/// either explicitly says ffmpeg couldn't read the file, or never
/// mentions a stream at all.
pub fn probe_output_unreadable(probe_output: &str) -> bool {
    probe_output.contains("Invalid data found when processing input")
        || !probe_output.contains("Stream")
}

/// Ports `$aStreams[0] - 2` (UniExtract.au3:3227) — see the module doc
/// comment's off-by-one note. `StringSplit($return, "Stream", 1)`'s
/// count is `1 +` the number of `"Stream"` occurrences.
pub fn undercounted_stream_count(probe_output: &str) -> i64 {
    (1 + probe_output.matches("Stream").count()) as i64 - 2
}

/// Ports the actual per-stream segments the loop processes
/// (UniExtract.au3:3236, `For $i = 2 To $aStreams[0]`): every piece of
/// text after each `"Stream"` occurrence, in order — one more segment
/// than [`undercounted_stream_count`] reports (that's the point of the
/// off-by-one).
pub fn stream_segments(probe_output: &str) -> Vec<&str> {
    probe_output.split("Stream").skip(1).collect()
}

/// Ports `$fileext == "wma" And $iStreams < 2` (UniExtract.au3:3232).
/// `undercounted_count` is [`undercounted_stream_count`]'s result,
/// deliberately not a corrected count — see the module doc comment.
pub fn should_shortcut_to_plain_audio(fileext: &str, undercounted_count: i64) -> bool {
    fileext == "wma" && undercounted_count < 2
}

/// One stream segment's parsed header — `$aStreamType` after
/// `StringRegExpReplace(..., "$3,$4,$1,$2")` and `StringSplit(..., ",")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStream {
    /// `$aStreamType[1]` — `"Video"`/`"Audio"`/anything else ffmpeg
    /// might report, exact case as ffmpeg wrote it.
    pub category: String,
    /// `$aStreamType[2]` — the codec name, exact case.
    pub codec: String,
    /// `$aStreamType[3]` — the ffmpeg stream index, e.g. `"0:0"`, used
    /// for `-map`.
    pub stream_index: String,
    /// `$aStreamType[4]` — the language-tag suffix as ffmpeg wrote it
    /// (e.g. `"(und)"`), appended verbatim into output filenames.
    pub language_tag: String,
}

/// Ports the header regex (UniExtract.au3:3237): `.*?#(\d:\d)(.*?): (\w+): (\w+).*`.
/// Returns `None` when no `#` is found, or the text right after it
/// isn't exactly one digit, `:`, one digit, or either `": "`-delimited
/// word group is missing — see the module doc comment for the one
/// documented divergence from the source's own PCRE backtracking
/// (multi-digit stream indices).
pub fn parse_stream_segment(segment: &str) -> Option<ParsedStream> {
    let hash_pos = segment.find('#')?;
    let after_hash = &segment[hash_pos + 1..];
    let bytes = after_hash.as_bytes();
    if bytes.len() < 3
        || !bytes[0].is_ascii_digit()
        || bytes[1] != b':'
        || !bytes[2].is_ascii_digit()
    {
        return None;
    }
    let stream_index = after_hash[..3].to_string();
    let rest = &after_hash[3..];

    let first_colon_space = rest.find(": ")?;
    let language_tag = rest[..first_colon_space].to_string();
    let after_lang = &rest[first_colon_space + 2..];

    let (category, after_category) = take_word(after_lang)?;
    let after_category = after_category.strip_prefix(": ")?;
    let (codec, _rest) = take_word(after_category)?;

    Some(ParsedStream {
        category,
        codec,
        stream_index,
        language_tag,
    })
}

/// `\w+` — one or more ASCII word characters (letters, digits,
/// underscore; PCRE `\w` without `UCP`).
fn take_word(s: &str) -> Option<(String, &str)> {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    if end == 0 {
        None
    } else {
        Some((s[..end].to_string(), &s[end..]))
    }
}

/// What a parsed stream header decides (UniExtract.au3:3241-3281).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamAction {
    /// `$aStreamType[2] == "gif"/"apng"/"webp"` (case-sensitive exact
    /// match): split into individual PNG frames instead of the usual
    /// extraction.
    SplitToImageSequence,
    /// A video stream, but `$bOptExtractVideo` is off: skip.
    SkipVideoDisabled,
    /// `$aStreamType[2] == "h264"` (case-sensitive exact match): the
    /// bitstream-filter extraction. The codec name is used unchanged
    /// as the output extension (`"h264"`), no remapping.
    ExtractH264Video,
    /// Any other video codec: plain copy extraction, with the codec
    /// name possibly remapped by [`normalize_video_codec`] first.
    ExtractOtherVideo { output_extension: String },
    /// An audio stream, with the codec name possibly remapped by
    /// [`normalize_audio_codec`] first.
    ExtractAudio { output_extension: String },
    /// Neither `"Video"` nor `"Audio"` (case-sensitive exact match):
    /// logged, no action.
    UnknownCategory,
}

/// Ports the per-stream classification (UniExtract.au3:3241-3281).
pub fn classify_stream(parsed: &ParsedStream, extract_video_enabled: bool) -> StreamAction {
    if parsed.category == "Video" {
        if parsed.codec == "gif" || parsed.codec == "apng" || parsed.codec == "webp" {
            StreamAction::SplitToImageSequence
        } else if !extract_video_enabled {
            StreamAction::SkipVideoDisabled
        } else if parsed.codec == "h264" {
            StreamAction::ExtractH264Video
        } else {
            StreamAction::ExtractOtherVideo {
                output_extension: normalize_video_codec(&parsed.codec),
            }
        }
    } else if parsed.category == "Audio" {
        StreamAction::ExtractAudio {
            output_extension: normalize_audio_codec(&parsed.codec),
        }
    } else {
        StreamAction::UnknownCategory
    }
}

/// Ports the video codec-name remapping (UniExtract.au3:3255-3263): all
/// four `StringInStr` checks are bare (case-insensitive substring).
pub fn normalize_video_codec(codec: &str) -> String {
    let lower = codec.to_lowercase();
    if lower.contains("wmv") {
        "wmv".to_string()
    } else if lower.contains("mpeg") {
        "mpeg".to_string()
    } else if lower.contains("vp8") {
        "webm".to_string()
    } else if lower.contains("flv") {
        "flv".to_string()
    } else {
        codec.to_string()
    }
}

/// Ports the audio codec-name remapping (UniExtract.au3:3270-3276): all
/// three `StringInStr` checks are bare (case-insensitive substring).
pub fn normalize_audio_codec(codec: &str) -> String {
    let lower = codec.to_lowercase();
    if lower.contains("wma") {
        "wma".to_string()
    } else if lower.contains("vorbis") {
        "ogg".to_string()
    } else if lower.contains("pcm") {
        "wav".to_string()
    } else {
        codec.to_string()
    }
}

/// Ports `_MakeFFmpegCommand`'s output-filename construction
/// (UniExtract.au3:5119-5125). `strip_leading_dashes` is `false` only
/// for the image-sequence branch, which bypasses `_MakeFFmpegCommand`
/// entirely — see the module doc comment.
pub fn build_stream_output_filename(
    base_name: &str,
    strip_leading_dashes: bool,
    type_label: &str,
    index: u32,
    language_tag: &str,
    extension: &str,
) -> String {
    let name = if strip_leading_dashes {
        base_name.trim_start_matches('-')
    } else {
        base_name
    };
    format!("{name}_{type_label}_{index:02}{language_tag}.{extension}")
}

/// Builds the gif/apng/webp image-sequence invocation
/// (UniExtract.au3:3244): `<ffmpeg> -i "<file>" "<base_name>%05d.png"`
/// — no codec flags, no `-map`, `base_name` used unstripped (see the
/// module doc comment's dash-stripping asymmetry note).
pub fn image_sequence_invocation(
    program: &str,
    file: &str,
    base_name: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            format!("{base_name}%05d.png"),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the h264 bitstream-filter video extraction invocation
/// (UniExtract.au3:3252).
pub fn h264_video_extraction_invocation(
    program: &str,
    file: &str,
    stream_index: &str,
    output_filename: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            "-vcodec".to_string(),
            "copy".to_string(),
            "-an".to_string(),
            "-bsf:v".to_string(),
            "h264_mp4toannexb".to_string(),
            "-map".to_string(),
            stream_index.to_string(),
            output_filename.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the ordinary (non-h264) video extraction invocation
/// (UniExtract.au3:3264).
pub fn video_extraction_invocation(
    program: &str,
    file: &str,
    stream_index: &str,
    output_filename: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            "-vcodec".to_string(),
            "copy".to_string(),
            "-an".to_string(),
            "-map".to_string(),
            stream_index.to_string(),
            output_filename.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Builds the audio extraction invocation (UniExtract.au3:3278).
pub fn audio_extraction_invocation(
    program: &str,
    file: &str,
    stream_index: &str,
    output_filename: &str,
    outdir: &str,
) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec![
            "-i".to_string(),
            file.to_string(),
            "-acodec".to_string(),
            "copy".to_string(),
            "-vn".to_string(),
            "-map".to_string(),
            stream_index.to_string(),
            output_filename.to_string(),
        ],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
}

/// Ports `If $iVideo + $iAudio < 1 Then terminate($STATUS_NOTPACKED, ...)`
/// (UniExtract.au3:3283).
pub fn no_streams_extracted(video_count: u32, audio_count: u32) -> bool {
    video_count + audio_count < 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parity test for capability C104: the audio-conversion invocation
    /// matches UniExtract.au3:2414-2416's effective `ffmpeg.exe -i
    /// "<file>" "<stem>.wav"` call.
    #[test]
    fn audio_invocation_matches_source() {
        let inv = audio_invocation(
            r"C:\UniExtract\bin\ffmpeg.exe",
            r"C:\downloads\track.wma",
            "track",
            r"C:\downloads\unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\ffmpeg.exe");
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\track.wma".to_string(),
                "track.wav".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C104: the video-convert invocation
    /// matches UniExtract.au3:3288's effective `ffmpeg.exe -i "<file>"
    /// "<stem>.mp4"` call.
    #[test]
    fn video_convert_invocation_matches_source() {
        let inv = video_convert_invocation(
            r"C:\UniExtract\bin\ffmpeg.exe",
            r"C:\downloads\clip.avi",
            "clip",
            r"C:\downloads\unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\ffmpeg.exe");
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\clip.avi".to_string(),
                "clip.mp4".to_string(),
            ]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    /// Parity test for capability C104: the stream-info probe matches
    /// UniExtract.au3:3220-3221's effective `ffmpeg.exe -i "<file>"` call.
    #[test]
    fn probe_invocation_matches_source() {
        let inv = probe_invocation(
            r"C:\UniExtract\bin\ffmpeg.exe",
            r"C:\downloads\container.mkv",
            r"C:\downloads\unpacked",
        );
        assert_eq!(inv.program, r"C:\UniExtract\bin\ffmpeg.exe");
        assert_eq!(
            inv.args,
            vec!["-i".to_string(), r"C:\downloads\container.mkv".to_string()]
        );
        assert_eq!(inv.working_dir, r"C:\downloads\unpacked");
        assert_eq!(inv.window, WindowMode::Hidden);
    }

    #[test]
    fn probe_output_unreadable_matches_either_signal() {
        assert!(probe_output_unreadable(
            "file.bin: Invalid data found when processing input"
        ));
        assert!(probe_output_unreadable("no stream keyword anywhere here"));
        assert!(!probe_output_unreadable(
            "Input #0, wav\n    Stream #0:0: Audio: pcm_s16le"
        ));
    }

    /// Parity test for capability C104: the source's own count
    /// (`$aStreams[0] - 2`) is one less than the number of segments the
    /// loop actually processes -- a real off-by-one, preserved exactly.
    #[test]
    fn undercounted_stream_count_is_one_less_than_processed_segments() {
        let one_stream = "Input #0, asf\n    Stream #0:0(eng): Audio: wmav2\nAt least one output file must be specified";
        assert_eq!(undercounted_stream_count(one_stream), 0);
        assert_eq!(stream_segments(one_stream).len(), 1);

        let two_streams =
            "Input #0, matroska\n    Stream #0:0: Video: h264\n    Stream #0:1: Audio: aac";
        assert_eq!(undercounted_stream_count(two_streams), 1);
        assert_eq!(stream_segments(two_streams).len(), 2);
    }

    /// Parity test for capability C104: the WMA shortcut's own `< 2`
    /// check, fed the undercounted value, actually fires for up to two
    /// real streams -- not "fewer than two" as its name suggests.
    #[test]
    fn wma_shortcut_fires_for_up_to_two_real_streams() {
        // One real stream -> undercounted 0 -> fires.
        assert!(should_shortcut_to_plain_audio("wma", 0));
        // Two real streams -> undercounted 1 -> still fires.
        assert!(should_shortcut_to_plain_audio("wma", 1));
        // Three real streams -> undercounted 2 -> no longer fires.
        assert!(!should_shortcut_to_plain_audio("wma", 2));
        assert!(!should_shortcut_to_plain_audio("mp4", 0));
    }

    #[test]
    fn parse_stream_segment_extracts_all_four_groups() {
        let segment = " #0:0(eng): Audio: wmav2, 44100 Hz, stereo, fltp, 128 kb/s";
        assert_eq!(
            parse_stream_segment(segment),
            Some(ParsedStream {
                category: "Audio".to_string(),
                codec: "wmav2".to_string(),
                stream_index: "0:0".to_string(),
                language_tag: "(eng)".to_string(),
            })
        );
    }

    #[test]
    fn parse_stream_segment_handles_video_with_empty_language_tag() {
        let segment = " #0:1: Video: h264 (High), yuv420p, 1920x1080";
        assert_eq!(
            parse_stream_segment(segment),
            Some(ParsedStream {
                category: "Video".to_string(),
                codec: "h264".to_string(),
                stream_index: "0:1".to_string(),
                language_tag: "".to_string(),
            })
        );
    }

    #[test]
    fn parse_stream_segment_returns_none_without_a_hash() {
        assert_eq!(parse_stream_segment("no stream marker here"), None);
    }

    #[test]
    fn gif_apng_webp_route_to_image_sequence_split() {
        for codec in ["gif", "apng", "webp"] {
            let parsed = ParsedStream {
                category: "Video".to_string(),
                codec: codec.to_string(),
                stream_index: "0:0".to_string(),
                language_tag: "".to_string(),
            };
            assert_eq!(
                classify_stream(&parsed, true),
                StreamAction::SplitToImageSequence
            );
        }
    }

    /// Parity test for capability C104: the image-sequence codec check
    /// is case-sensitive `==`, unlike the wmv/mpeg/vp8/flv substring
    /// checks below it.
    #[test]
    fn gif_codec_check_is_case_sensitive() {
        let parsed = ParsedStream {
            category: "Video".to_string(),
            codec: "GIF".to_string(),
            stream_index: "0:0".to_string(),
            language_tag: "".to_string(),
        };
        assert_eq!(
            classify_stream(&parsed, true),
            StreamAction::ExtractOtherVideo {
                output_extension: "GIF".to_string()
            }
        );
    }

    #[test]
    fn video_extraction_disabled_skips_non_image_video() {
        let parsed = ParsedStream {
            category: "Video".to_string(),
            codec: "h264".to_string(),
            stream_index: "0:0".to_string(),
            language_tag: "".to_string(),
        };
        assert_eq!(
            classify_stream(&parsed, false),
            StreamAction::SkipVideoDisabled
        );
    }

    #[test]
    fn h264_gets_its_own_action_with_unchanged_extension() {
        let parsed = ParsedStream {
            category: "Video".to_string(),
            codec: "h264".to_string(),
            stream_index: "0:0".to_string(),
            language_tag: "".to_string(),
        };
        assert_eq!(
            classify_stream(&parsed, true),
            StreamAction::ExtractH264Video
        );
    }

    #[test]
    fn other_video_codecs_get_remapped_extensions() {
        for (codec, expected) in [
            ("wmv3", "wmv"),
            ("mpeg1video", "mpeg"),
            ("vp8", "webm"),
            ("flv1", "flv"),
            ("mjpeg", "mjpeg"),
        ] {
            let parsed = ParsedStream {
                category: "Video".to_string(),
                codec: codec.to_string(),
                stream_index: "0:0".to_string(),
                language_tag: "".to_string(),
            };
            assert_eq!(
                classify_stream(&parsed, true),
                StreamAction::ExtractOtherVideo {
                    output_extension: expected.to_string()
                }
            );
        }
    }

    #[test]
    fn audio_codecs_get_remapped_extensions() {
        for (codec, expected) in [
            ("wmav2", "wma"),
            ("vorbis", "ogg"),
            ("pcm_s16le", "wav"),
            ("aac", "aac"),
        ] {
            let parsed = ParsedStream {
                category: "Audio".to_string(),
                codec: codec.to_string(),
                stream_index: "0:0".to_string(),
                language_tag: "".to_string(),
            };
            assert_eq!(
                classify_stream(&parsed, true),
                StreamAction::ExtractAudio {
                    output_extension: expected.to_string()
                }
            );
        }
    }

    #[test]
    fn unrecognized_category_is_logged_only() {
        let parsed = ParsedStream {
            category: "Subtitle".to_string(),
            codec: "srt".to_string(),
            stream_index: "0:2".to_string(),
            language_tag: "".to_string(),
        };
        assert_eq!(
            classify_stream(&parsed, true),
            StreamAction::UnknownCategory
        );
    }

    #[test]
    fn output_filename_strips_leading_dashes_when_requested() {
        assert_eq!(
            build_stream_output_filename("--archive", true, "Video", 1, "(und)", "h264"),
            "archive_Video_01(und).h264"
        );
        assert_eq!(
            build_stream_output_filename("--archive", false, "Video", 1, "(und)", "h264"),
            "--archive_Video_01(und).h264"
        );
    }

    #[test]
    fn output_filename_zero_pads_index_to_two_digits() {
        assert_eq!(
            build_stream_output_filename("clip", true, "Audio", 3, "", "wav"),
            "clip_Audio_03.wav"
        );
        assert_eq!(
            build_stream_output_filename("clip", true, "Audio", 12, "", "wav"),
            "clip_Audio_12.wav"
        );
    }

    #[test]
    fn image_sequence_invocation_has_no_codec_flags() {
        let inv = image_sequence_invocation(
            r"C:\bin\ffmpeg.exe",
            r"C:\downloads\anim.gif",
            "anim",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\anim.gif".to_string(),
                "anim%05d.png".to_string(),
            ]
        );
    }

    #[test]
    fn h264_invocation_includes_bitstream_filter() {
        let inv = h264_video_extraction_invocation(
            r"C:\bin\ffmpeg.exe",
            r"C:\downloads\clip.mkv",
            "0:0",
            "clip_Video_01.h264",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\clip.mkv".to_string(),
                "-vcodec".to_string(),
                "copy".to_string(),
                "-an".to_string(),
                "-bsf:v".to_string(),
                "h264_mp4toannexb".to_string(),
                "-map".to_string(),
                "0:0".to_string(),
                "clip_Video_01.h264".to_string(),
            ]
        );
    }

    #[test]
    fn video_invocation_omits_bitstream_filter() {
        let inv = video_extraction_invocation(
            r"C:\bin\ffmpeg.exe",
            r"C:\downloads\clip.mkv",
            "0:0",
            "clip_Video_01.wmv",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\clip.mkv".to_string(),
                "-vcodec".to_string(),
                "copy".to_string(),
                "-an".to_string(),
                "-map".to_string(),
                "0:0".to_string(),
                "clip_Video_01.wmv".to_string(),
            ]
        );
    }

    #[test]
    fn audio_invocation_uses_acodec_and_vn() {
        let inv = audio_extraction_invocation(
            r"C:\bin\ffmpeg.exe",
            r"C:\downloads\clip.mkv",
            "0:1",
            "clip_Audio_01.wma",
            r"C:\downloads\unpacked",
        );
        assert_eq!(
            inv.args,
            vec![
                "-i".to_string(),
                r"C:\downloads\clip.mkv".to_string(),
                "-acodec".to_string(),
                "copy".to_string(),
                "-vn".to_string(),
                "-map".to_string(),
                "0:1".to_string(),
                "clip_Audio_01.wma".to_string(),
            ]
        );
    }

    #[test]
    fn no_streams_extracted_requires_zero_video_and_audio() {
        assert!(no_streams_extracted(0, 0));
        assert!(!no_streams_extracted(1, 0));
        assert!(!no_streams_extracted(0, 1));
    }
}
