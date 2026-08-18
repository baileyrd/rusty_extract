//! ffmpeg (`ffmpeg.exe`) — audio conversion to WAV, video/audio track
//! extraction from containers.
//!
//! **Scope — partial.** This module covers the two simple,
//! single-invocation cases (`Case $TYPE_AUDIO`, `Case
//! $TYPE_VIDEO_CONVERT`) and the stream-info probe `Case $TYPE_VIDEO`
//! opens with. `$TYPE_VIDEO`'s actual per-stream extraction — parsing
//! ffmpeg's raw `-i` stdout via a regex-based pattern match
//! (UniExtract.au3:3235-3236) to discover each stream's codec/type/index,
//! then dynamically building one extraction command per stream via
//! `_MakeFFmpegCommand` (UniExtract.au3:5118-5126) — is **not yet
//! ported**: a real, documented gap, not silently dropped. Capability
//! C104's own manifest description covers all three call sites, so this
//! row stays `REQUIRED` until that parsing lands too (matching the
//! precedent set by C140, ported across two PRs before being marked
//! `DONE`).

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
/// dispatching one extraction command per stream — is the not-yet-ported
/// gap documented in this module's own doc comment.
pub fn probe_invocation(program: &str, file: &str, outdir: &str) -> Invocation {
    Invocation {
        program: program.to_string(),
        args: vec!["-i".to_string(), file.to_string()],
        working_dir: outdir.to_string(),
        window: WindowMode::Hidden,
    }
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
}
