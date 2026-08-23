//! Light/dark theme and high-contrast detection, ported from
//! `_AppsUseLightTheme` and `_IsHighContrastMode` (UniExtract.au3:6139-
//! 6178), plus `_GuiSetColor`'s white-background decision
//! (UniExtract.au3:6186-6191) — part of capability C183.

use crate::automation::GuiAutomation;

const PERSONALIZE_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const APPS_USE_LIGHT_THEME_VALUE: &str = "AppsUseLightTheme";

/// Ports `_AppsUseLightTheme` (UniExtract.au3:6169-6175): reads the
/// `AppsUseLightTheme` DWORD from the Personalize key. **Fails open to
/// light** on a missing key or any read error — the opposite polarity
/// from [`is_high_contrast`]'s fail-closed-to-false, preserved exactly
/// rather than unified, since the source treats the two failure modes
/// differently on purpose (light is the safer default to assume for
/// rendering; "not high contrast" is the safer default for accessibility
/// detection).
///
/// Reuses the existing `GuiAutomation::reg_read_dword` primitive (C069)
/// rather than adding a new registry-read path — this is an ordinary
/// user-preference key, not a new capability of its own.
pub fn apps_use_light_theme<A: GuiAutomation>(automation: &mut A) -> bool {
    automation
        .reg_read_dword(PERSONALIZE_KEY, APPS_USE_LIGHT_THEME_VALUE)
        .map(|value| value != 0)
        .unwrap_or(true)
}

/// Ports the `HCF_HIGHCONTRASTON` bit test from `_IsHighContrastMode`
/// (UniExtract.au3:6139-6165). The real `SystemParametersInfo`
/// (`SPI_GETHIGHCONTRAST`) call that produces `flags` is real Win32 I/O,
/// out of this pure function's scope (see `gui::app` for the real
/// caller); this is the bit-test the source applies to its result,
/// including the **fail-closed-to-false** behavior on any API error
/// (modeled here as the caller passing `None`).
pub fn is_high_contrast(flags: Option<u32>) -> bool {
    const HCF_HIGHCONTRASTON: u32 = 0x0000_0001;
    flags.map(|f| f & HCF_HIGHCONTRASTON != 0).unwrap_or(false)
}

/// Ports `_GuiSetColor`'s white-background gate (UniExtract.au3:6186-
/// 6191): a three-way AND — Windows 10 (not 11) or newer, not
/// high-contrast, and light theme active. No corresponding "force a dark
/// background" branch exists in the source; dark mode simply falls
/// through to the OS default, an asymmetry preserved here rather than
/// "balanced" with a dark-mode branch of this port's own invention.
pub fn should_use_white_background(
    is_windows10_not_11: bool,
    high_contrast: bool,
    light_theme: bool,
) -> bool {
    is_windows10_not_11 && !high_contrast && light_theme
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::fake::FakeGuiAutomation;

    #[test]
    fn apps_use_light_theme_reads_the_registry_value() {
        let mut fake = FakeGuiAutomation::new();
        fake.set_reg_dword(PERSONALIZE_KEY, APPS_USE_LIGHT_THEME_VALUE, 0);
        assert!(!apps_use_light_theme(&mut fake));

        fake.set_reg_dword(PERSONALIZE_KEY, APPS_USE_LIGHT_THEME_VALUE, 1);
        assert!(apps_use_light_theme(&mut fake));
    }

    /// Parity test for capability C183: an unset/unreadable value fails
    /// open to light, not dark.
    #[test]
    fn apps_use_light_theme_fails_open_to_light_when_unset() {
        let mut fake = FakeGuiAutomation::new();
        assert!(apps_use_light_theme(&mut fake));
    }

    #[test]
    fn high_contrast_bit_test() {
        assert!(!is_high_contrast(None));
        assert!(!is_high_contrast(Some(0)));
        assert!(is_high_contrast(Some(0x0000_0001)));
        assert!(is_high_contrast(Some(0x0000_0001 | 0x0000_0002)));
    }

    /// Parity test for capability C183: all three conditions must hold;
    /// any one failing keeps the OS default background.
    #[test]
    fn white_background_requires_all_three_conditions() {
        assert!(should_use_white_background(true, false, true));
        assert!(!should_use_white_background(false, false, true));
        assert!(!should_use_white_background(true, true, true));
        assert!(!should_use_white_background(true, false, false));
    }
}
