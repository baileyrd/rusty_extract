//! The real main-window shell (capability C183), built on `eframe`/`egui`.
//! Windows-only (`#[cfg(windows)]`), same as `automation::win32`/
//! `dlllib::win32` — this dev environment cannot visually render or
//! interact with a real window, so this module is verified by
//! `cargo check`/`cargo clippy --target x86_64-pc-windows-gnu` only; real
//! interactive verification needs CI's `windows-latest` runner and,
//! eventually, a real Windows machine. The layout/theme *decisions* this
//! window makes (`gui::layout`, `gui::theme`, `gui::window_state`) are
//! fully real, cross-platform-buildable Rust, tested for real via
//! `cargo test` regardless of host OS.
//!
//! Ports `CreateGUI` (UniExtract.au3:5806-5980): base window, File/Edit/
//! Help menus, an Extract-vs-Scan-only radio pair, file/dir input fields
//! with browse buttons, and OK/Cancel/Batch buttons. The individual
//! fields' own behavior (validation, browse dialogs, batch queuing) is
//! separate capabilities (C186-C190) — this shell wires the layout only;
//! the field actions below are placeholders those capabilities fill in.

use crate::automation::win32::Win32GuiAutomation;
use crate::gui::theme;
use eframe::egui;

/// Ports the Extract-vs-Scan-only radio pair (UniExtract.au3:5850
/// region). The real behavior each mode drives is capability C190
/// (`GUI_ScanOnly`); this enum only names the two states the main
/// window's own radio buttons present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMode {
    Extract,
    ScanOnly,
}

/// The main window's own state, scoped to what C183 owns: the two input
/// fields' current text and which extraction mode is selected. Field
/// validation/auto-fill (C186), drag-and-drop (C187), and the batch
/// queue (C188) are separate capabilities that read/mutate this same
/// state once implemented.
pub struct MainWindow {
    pub extraction_mode: ExtractionMode,
    pub file_path: String,
    pub output_dir: String,
    use_white_background: bool,
}

impl MainWindow {
    /// Constructs the window's initial state, running the same
    /// theme/high-contrast detection `CreateGUI`'s own `_GuiSetColor`
    /// call performs (UniExtract.au3:5823-ish, via `_GuiSetColor`) so the
    /// very first frame already reflects the OS theme rather than
    /// flashing the wrong background before a later detection pass.
    pub fn new() -> Self {
        let mut automation = Win32GuiAutomation::default();
        let light_theme = theme::apps_use_light_theme(&mut automation);
        let high_contrast = theme::is_high_contrast(query_high_contrast_flags());
        let is_windows10_not_11 = is_windows10_not_11();
        Self {
            extraction_mode: ExtractionMode::Extract,
            file_path: String::new(),
            output_dir: String::new(),
            use_white_background: theme::should_use_white_background(
                is_windows10_not_11,
                high_contrast,
                light_theme,
            ),
        }
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.use_white_background {
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = egui::Color32::WHITE;
            ctx.set_visuals(visuals);
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let _ = ui.button("Open...");
                    let _ = ui.button("Keep window open");
                    let _ = ui.button("Always on top");
                    ui.separator();
                    let _ = ui.button("Show");
                    let _ = ui.button("Clear");
                    ui.separator();
                    let _ = ui.button("Open log");
                    let _ = ui.button("Open log folder");
                });
                ui.menu_button("Edit", |ui| {
                    let _ = ui.button("Preferences");
                    let _ = ui.button("Context menu entries");
                });
                ui.menu_button("Help", |ui| {
                    let _ = ui.button("Command line help");
                    let _ = ui.button("About");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.extraction_mode,
                    ExtractionMode::Extract,
                    "Extract",
                );
                ui.selectable_value(
                    &mut self.extraction_mode,
                    ExtractionMode::ScanOnly,
                    "Scan only",
                );
            });

            ui.horizontal(|ui| {
                ui.label("File:");
                ui.text_edit_singleline(&mut self.file_path);
                let _ = ui.button("...");
            });

            let output_dir_enabled = self.extraction_mode == ExtractionMode::Extract;
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                ui.add_enabled(
                    output_dir_enabled,
                    egui::TextEdit::singleline(&mut self.output_dir),
                );
                let _ = ui.add_enabled(output_dir_enabled, egui::Button::new("..."));
            });

            ui.horizontal(|ui| {
                let _ = ui.button("OK");
                let _ = ui.button("Cancel");
                let _ = ui.button("Batch");
            });
        });
    }
}

/// Real `SystemParametersInfo(SPI_GETHIGHCONTRAST)` call
/// (UniExtract.au3:6139-6165's own Win32 call). Returns `None` on any API
/// failure, matching the source's fail-closed-to-not-high-contrast
/// behavior once passed through [`theme::is_high_contrast`].
fn query_high_contrast_flags() -> Option<u32> {
    use windows::Win32::UI::Accessibility::HIGHCONTRASTW;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };
    let mut info = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    let result = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            info.cbSize,
            Some(&mut info as *mut _ as *mut std::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    result.ok().map(|()| info.dwFlags.0)
}

/// Real Windows-version check backing [`theme::should_use_white_background`]'s
/// "Windows 10, not 11" condition (`_IsWin10OrNewer()` combined with the
/// source's implicit "not 11" framing). Windows 11 reports a build number
/// of 22000 or higher while still identifying as major version 10 via the
/// classic `GetVersionEx`-style APIs, so the build-number check below is
/// what actually distinguishes the two.
fn is_windows10_not_11() -> bool {
    use windows::Wdk::System::SystemServices::RtlGetVersion;
    use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    let status = unsafe { RtlGetVersion(&mut info) };
    status.is_ok() && info.dwMajorVersion == 10 && info.dwBuildNumber < 22000
}
