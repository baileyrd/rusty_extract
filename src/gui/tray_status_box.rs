//! Tray status/progress popup window decisions, ported from
//! `_CreateTrayMessageBox`/`_DeleteTrayMessageBox` (UniExtract.au3:4261-
//! 4342) — capability C185. Distinct from C166's `teelog`, which only
//! ported the *text-update* call (`_SetTrayMessageBoxText`) into an
//! already-existing popup; this row covers the popup's own window
//! lifecycle: whether to show it at all, where to place it, and what
//! text it displays.

/// Ports the `$bOptNoStatusBox` gate (UniExtract.au3:4264). Note the
/// source calls `_DeleteTrayMessageBox()` **unconditionally** before
/// this check runs (UniExtract.au3:4262) — any existing popup is always
/// closed when a new status message arrives, even if the new one won't
/// end up shown. Callers must replicate that ordering: delete-then-
/// maybe-create, not maybe-create-then-delete-if-not.
pub fn should_show_status_box(no_status_box: bool) -> bool {
    !no_status_box
}

/// Ports the fullscreen-suppression check (UniExtract.au3:4267-4270):
/// when enabled, the popup is suppressed if the currently active
/// window's size exactly matches the desktop size (a simple, exact-match
/// fullscreen heuristic — not a "mostly covers the screen" fuzzy check).
pub fn should_suppress_for_fullscreen(
    hide_if_fullscreen: bool,
    active_window_size: (i32, i32),
    desktop_size: (i32, i32),
) -> bool {
    hide_if_fullscreen && active_window_size == desktop_size
}

/// Ports the two truncation rules (UniExtract.au3:4282-4283): the
/// filename label is capped at 28 characters, the status message at 56
/// (`28 * 2`) — both append `" [...]"` when truncated. Truncates by
/// character count, not byte length, matching AutoIt's own
/// `StringLen`/`StringLeft` (character-counting) semantics rather than a
/// byte-slice that could split a multi-byte character.
pub fn truncate_with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{truncated} [...]")
    } else {
        text.to_string()
    }
}

/// A screen rectangle, in the same `(x, y, width, height)` shape
/// `WinGetPos` returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Ports the popup's position resolution (UniExtract.au3:4272-4313):
/// a persisted `statusposx`/`statusposy` override (any value `> -1`)
/// wins outright; otherwise the position is computed relative to the
/// taskbar.
///
/// **Real quirk, preserved exactly**: the source decides whether the
/// taskbar is top-docked by testing `$pos[0] = $pos[1]` — the taskbar
/// rectangle's X and Y coordinates being *equal* — rather than a direct
/// "is Y zero" check. This only actually holds when the taskbar sits at
/// `(0, 0)` (a full-width top dock); any other dock position (bottom,
/// left, right) has X and Y coordinates that differ, so the equality
/// test happens to work as a top-dock detector without saying so
/// directly. Ported as the same equality comparison, not "clarified"
/// into an explicit Y-is-zero check, in case some taskbar configuration
/// makes the two diverge.
///
/// After computing a candidate vertical offset, an out-of-range result
/// (negative, or taller than the desktop) falls back to anchoring the
/// popup to the bottom-right corner of the screen, ignoring the taskbar
/// entirely — a sane default when the taskbar geometry looks unusable
/// rather than a hard failure.
pub fn resolve_position(
    stored_position: Option<(i32, i32)>,
    popup_size: (i32, i32),
    taskbar_rect: Option<ScreenRect>,
    desktop_size: (i32, i32),
) -> (i32, i32) {
    const BETWEEN: i32 = 5;
    let (popup_width, popup_height) = popup_size;
    let (desktop_width, desktop_height) = desktop_size;

    if let Some((x, y)) = stored_position {
        if x > -1 && y > -1 {
            return (x, y);
        }
    }

    // `WinGetPos("[CLASS:Shell_TrayWnd]")` failing (`@error`) defaults
    // to a synthetic top-docked rect spanning the desktop width.
    let taskbar = taskbar_rect.unwrap_or(ScreenRect {
        x: 0,
        y: 0,
        width: desktop_width,
        height: 30,
    });

    let is_top_docked = taskbar.x == taskbar.y;
    let mut space = if is_top_docked {
        taskbar.height + BETWEEN
    } else {
        taskbar.y - popup_height - BETWEEN
    };

    if space < 0 || space > desktop_height {
        space = desktop_height - popup_height - BETWEEN;
    }

    let x = desktop_width - (popup_width + BETWEEN);
    (x, space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_unless_no_status_box_is_set() {
        assert!(should_show_status_box(false));
        assert!(!should_show_status_box(true));
    }

    #[test]
    fn fullscreen_suppression_requires_exact_size_match() {
        assert!(should_suppress_for_fullscreen(
            true,
            (1920, 1080),
            (1920, 1080)
        ));
        assert!(!should_suppress_for_fullscreen(
            true,
            (1918, 1080),
            (1920, 1080)
        ));
        assert!(!should_suppress_for_fullscreen(
            false,
            (1920, 1080),
            (1920, 1080)
        ));
    }

    #[test]
    fn truncates_by_character_count_with_ellipsis_suffix() {
        assert_eq!(truncate_with_ellipsis("short", 28), "short");
        let exactly_28 = "a".repeat(28);
        assert_eq!(truncate_with_ellipsis(&exactly_28, 28), exactly_28);
        let long = "a".repeat(30);
        assert_eq!(
            truncate_with_ellipsis(&long, 28),
            format!("{} [...]", "a".repeat(28))
        );
    }

    /// Parity test for capability C185: truncation counts characters,
    /// not bytes -- a multi-byte character must not be split.
    #[test]
    fn truncation_counts_characters_not_bytes() {
        let text = "é".repeat(30);
        let result = truncate_with_ellipsis(&text, 28);
        assert_eq!(result.chars().count(), 28 + " [...]".chars().count());
        assert!(result.starts_with(&"é".repeat(28)));
    }

    #[test]
    fn stored_position_wins_outright() {
        assert_eq!(
            resolve_position(Some((100, 200)), (225, 100), None, (1920, 1080)),
            (100, 200)
        );
    }

    /// Parity test for capability C185: a top-docked taskbar (x == y ==
    /// 0) places the popup just below it.
    #[test]
    fn top_docked_taskbar_places_popup_below_it() {
        let taskbar = ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 40,
        };
        let (x, y) = resolve_position(None, (225, 100), Some(taskbar), (1920, 1080));
        assert_eq!(x, 1920 - (225 + 5));
        assert_eq!(y, 40 + 5);
    }

    /// Parity test for capability C185: a bottom-docked taskbar (x != y)
    /// places the popup just above it.
    #[test]
    fn bottom_docked_taskbar_places_popup_above_it() {
        let taskbar = ScreenRect {
            x: 0,
            y: 1040,
            width: 1920,
            height: 40,
        };
        let (_, y) = resolve_position(None, (225, 100), Some(taskbar), (1920, 1080));
        assert_eq!(y, 1040 - 100 - 5);
    }

    /// Parity test for capability C185: an out-of-range computed space
    /// falls back to the bottom-right corner of the screen.
    #[test]
    fn out_of_range_space_falls_back_to_bottom_right() {
        let bogus_taskbar = ScreenRect {
            x: 0,
            y: 2000,
            width: 1920,
            height: 40,
        };
        let (_, y) = resolve_position(None, (225, 100), Some(bogus_taskbar), (1920, 1080));
        assert_eq!(y, 1080 - 100 - 5);
    }

    #[test]
    fn missing_taskbar_defaults_to_a_synthetic_top_docked_rect() {
        let (_, y) = resolve_position(None, (225, 100), None, (1920, 1080));
        assert_eq!(y, 30 + 5);
    }
}
