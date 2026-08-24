//! About dialog and website menu actions — capability C197. Ports the
//! one real decision inside `GUI_About` (UniExtract.au3:8084-8104): the
//! high-contrast logo asset swap.
//!
//! Everything else in this row is either static string composition or
//! fixed real I/O with no decision logic to port:
//! - The version/timestamp/credits/GUID display fields (`t('ABOUT_VERSION',
//!   ...)`, `t('ABOUT_INFO_LABEL', ...)`, `$sOptGuid`) are plain string
//!   composition and a direct display of an already-resolved value (the
//!   per-install ID, capability C215) — nothing to decide.
//! - `GUI_Website_Original`/`GUI_Website`/`GUI_Website_Github`
//!   (UniExtract.au3:8128-8140) are `OpenURL(...)` one-liners against
//!   fixed URLs — real I/O, not decision logic.
//! - `GUI_Close`'s multi-window "which GUI is active" guesswork
//!   (`$aGUIs`/`WinActive`, UniExtract.au3:8116-8125) exists only because
//!   AutoIt's `GUIOnEventMode` binds one event handler across potentially
//!   several simultaneously-open windows without its own per-window
//!   context. Each `egui` viewport's own `show_viewport_immediate`
//!   closure already knows exactly which window it's closing — no
//!   tracked-handles array or active-window guess needed — the same
//!   class of "old workaround made moot by the new toolkit" as C183's
//!   DPI-scaling note and C190's window-recreate note. `GUI_Close` also
//!   isn't in this row's own function list (`GUI_About`,
//!   `GUI_Website_Original`, `GUI_Website`, `GUI_Website_Github`) — it's
//!   shared infrastructure other dialogs use too, not this capability's
//!   own concern.
//!
//! **Not wired to a real window** — same treatment as every other
//! dialog this migration phase has ported so far, and doubly so here:
//! the per-install ID this dialog displays (C215) doesn't exist as GUI
//! state yet either.

/// Ports the logo asset filename swap
/// (UniExtract.au3:8096: `$iconsdir & "Bioruebe" & ($bHighContrastMode?
/// "White": "") & ".png"`).
pub fn resolve_about_logo_filename(high_contrast_mode: bool) -> &'static str {
    if high_contrast_mode {
        "BioruebeWhite.png"
    } else {
        "Bioruebe.png"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_filename_swaps_for_high_contrast() {
        assert_eq!(resolve_about_logo_filename(true), "BioruebeWhite.png");
        assert_eq!(resolve_about_logo_filename(false), "Bioruebe.png");
    }
}
