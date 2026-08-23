//! Pure layout-geometry decisions ported from `CreateGUI`'s control-chaining
//! helper (`GetPos`, UniExtract.au3:6036-6046) and the RTL ex-style
//! sentinel resolution used when building the main window
//! (UniExtract.au3:5806-5823) — capability C183.

/// A control's live on-screen rectangle, as `ControlGetPos`/`WinGetPos`
/// would report it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Ports `GetPos($hGui, $hControl, $iOffset, $bX)` (UniExtract.au3:6036-
/// 6046): chains the next control's position off `control`'s own live
/// rectangle. `along_x` selects the axis (`$bX`): the right edge plus
/// offset when true, the bottom edge plus offset when false.
///
/// **RTL quirk, preserved exactly**: in right-to-left layout mode the X
/// offset is multiplied by an undocumented `0.4` factor — not `1.0`, and
/// not mirrored to the left edge. This only affects the X-axis chain;
/// `along_x = false` (vertical stacking) is unaffected by `is_rtl`.
pub fn next_control_position(control: Rect, offset: f64, along_x: bool, is_rtl: bool) -> f64 {
    if along_x {
        let effective_offset = if is_rtl { offset * 0.4 } else { offset };
        control.x + control.width + effective_offset
    } else {
        control.y + control.height + offset
    }
}

/// Ports the ex-style sentinel resolution used when building the main
/// window (`$exStyle < 0? 0: $exStyle`, UniExtract.au3:5823): an ex-style
/// of exactly `-1` (this crate's "unset" sentinel, matching the source's
/// own default parameter convention) resolves to no extra style bits
/// rather than being OR'd in literally.
pub fn resolve_ex_style(ex_style: i32) -> i32 {
    if ex_style < 0 {
        0
    } else {
        ex_style
    }
}

/// The main window's minimum size, as enforced by
/// `GUI_WM_GETMINMAXINFO_Main` (UniExtract.au3:7001-7008).
///
/// **Preserve exactly**: this is the size **measured from the real window
/// after `GUICreate` returns** (`WinGetPos` on the just-created window),
/// not the nominal size passed into `GUICreate` — client-vs-full-window
/// size differs by theme/Windows version, so the two can disagree. A port
/// that enforces the nominal requested size instead of the measured one
/// would silently pick the wrong minimum on any theme where they differ.
/// This function is the identity on the measured size; its only purpose
/// is to name and test that contract so a future edit can't quietly start
/// passing the nominal size in by mistake.
pub fn min_window_size(measured_after_creation: (f64, f64)) -> (f64, f64) {
    measured_after_creation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains_right_edge_for_x_axis() {
        let control = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 25.0,
        };
        assert_eq!(next_control_position(control, 8.0, true, false), 118.0);
    }

    #[test]
    fn chains_bottom_edge_for_y_axis_regardless_of_rtl() {
        let control = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 25.0,
        };
        assert_eq!(next_control_position(control, 6.0, false, false), 51.0);
        assert_eq!(next_control_position(control, 6.0, false, true), 51.0);
    }

    /// Parity test for capability C183: RTL mode shrinks the X offset by
    /// the source's own undocumented 0.4 factor -- not a 1:1 mirror, and
    /// not zeroed out.
    #[test]
    fn rtl_mode_multiplies_x_offset_by_point_four() {
        let control = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 25.0,
        };
        assert_eq!(next_control_position(control, 10.0, true, true), 114.0);
    }

    #[test]
    fn ex_style_sentinel_negative_one_resolves_to_zero() {
        assert_eq!(resolve_ex_style(-1), 0);
    }

    #[test]
    fn ex_style_non_negative_passes_through_unchanged() {
        assert_eq!(resolve_ex_style(0), 0);
        assert_eq!(resolve_ex_style(0x00080000), 0x00080000);
    }

    /// Parity test for capability C183: the enforced minimum is whatever
    /// was actually measured post-creation, not a hardcoded nominal size
    /// -- this test would still pass a naive "return the 344x136 nominal
    /// constant" bug, which is exactly why the doc comment calls this out
    /// explicitly; this test pins the *contract* (identity on the measured
    /// value) so that bug is visible in review, not just in the source.
    #[test]
    fn min_window_size_is_the_measured_post_creation_size() {
        assert_eq!(min_window_size((344.0, 136.0)), (344.0, 136.0));
        assert_eq!(min_window_size((360.0, 150.0)), (360.0, 150.0));
    }
}
