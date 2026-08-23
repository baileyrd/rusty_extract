//! Window position/size persistence gating, ported from
//! `GUI_SavePosition` (UniExtract.au3:6988-6998) — part of capability
//! C183. Folds in capability-manifest.md row D010's
//! `storeguiposition`/`posx`/`posy`/`GuiWidth`/`GuiHeight` preferences.

/// The main window's persisted position/size, as written to the four
/// separate preference keys the source uses (`posx`/`posy`/`GuiWidth`/
/// `GuiHeight`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Ports `GUI_SavePosition`'s guard (UniExtract.au3:6990): saving only
/// happens when the main window actually exists **and** the "remember
/// window size/position" preference is on. Both this function's inputs
/// collapse a real `WinGetPos` failure (a destroyed/nonexistent window)
/// into the same "don't save" outcome the source's own silent-return
/// produces — no error is surfaced either way.
pub fn should_save_position(main_window_exists: bool, remember_position: bool) -> bool {
    main_window_exists && remember_position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_only_when_window_exists_and_preference_is_on() {
        assert!(should_save_position(true, true));
    }

    #[test]
    fn skips_when_window_does_not_exist() {
        assert!(!should_save_position(false, true));
    }

    /// Parity test for capability C183: even with a live window, the
    /// preference gates whether position gets persisted at all.
    #[test]
    fn skips_when_remember_preference_is_off() {
        assert!(!should_save_position(true, false));
    }
}
