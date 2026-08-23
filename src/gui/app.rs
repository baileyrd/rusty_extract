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
use crate::gui::tray_icon_shell::{TrayCommand, TrayHandle};
use crate::gui::tray_status_box::{self, ScreenRect};
use eframe::egui;
use std::time::Instant;

/// The popup's fixed size (UniExtract.au3:4274, `$iWidth`/`$iHeight`).
const STATUS_BOX_SIZE: (i32, i32) = (225, 100);
/// How long the fade-in takes. The source fades in 23 discrete steps at
/// ~1ms `Sleep` each (UniExtract.au3:4317-4320) -- effectively near-
/// instant on modern hardware; this reproduces the same "fades in over a
/// perceptible fraction of a second" visual effect via frame-timed alpha
/// interpolation instead of a blocking step loop, since `eframe`'s
/// viewport transparency is continuous rather than stepped. Fade-out
/// (UniExtract.au3:4335-4338) is not animated in this pass -- the popup
/// is simply removed -- a documented simplification, not an oversight.
const FADE_IN_MS: f32 = 230.0;

/// The tray status popup's own state (capability C185), separate from
/// C166's `teelog`, which only ports the *text-update* call into an
/// already-existing popup.
struct StatusBoxState {
    filename: String,
    message: String,
    extended: String,
    shown_at: Instant,
}

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
    pub no_status_box: bool,
    pub hide_status_box_if_fullscreen: bool,
    use_white_background: bool,
    tray: TrayHandle,
    status_box: Option<StatusBoxState>,
}

impl MainWindow {
    /// Constructs the window's initial state, running the same
    /// theme/high-contrast detection `CreateGUI`'s own `_GuiSetColor`
    /// call performs (UniExtract.au3:5823-ish, via `_GuiSetColor`) so the
    /// very first frame already reflects the OS theme rather than
    /// flashing the wrong background before a later detection pass, and
    /// building the tray icon (C184) the same way `Tray_Create` does at
    /// startup.
    pub fn new() -> Self {
        let mut automation = Win32GuiAutomation::default();
        let light_theme = theme::apps_use_light_theme(&mut automation);
        let high_contrast = theme::is_high_contrast(query_high_contrast_flags());
        let is_windows10_not_11 = is_windows10_not_11();
        let no_status_box = false;
        Self {
            extraction_mode: ExtractionMode::Extract,
            file_path: String::new(),
            output_dir: String::new(),
            no_status_box,
            hide_status_box_if_fullscreen: false,
            use_white_background: theme::should_use_white_background(
                is_windows10_not_11,
                high_contrast,
                light_theme,
            ),
            tray: TrayHandle::new(no_status_box, false),
            status_box: None,
        }
    }

    /// Ports `_CreateTrayMessageBox` (UniExtract.au3:4261-4321). Always
    /// clears any existing popup first (matching the source's own
    /// unconditional `_DeleteTrayMessageBox()` call), then shows the new
    /// one only if neither gate suppresses it.
    pub fn show_status(&mut self, filename: &str, message: &str) {
        self.status_box = None;

        if !tray_status_box::should_show_status_box(self.no_status_box) {
            return;
        }
        if tray_status_box::should_suppress_for_fullscreen(
            self.hide_status_box_if_fullscreen,
            active_window_size(),
            desktop_size(),
        ) {
            return;
        }

        self.status_box = Some(StatusBoxState {
            filename: tray_status_box::truncate_with_ellipsis(filename, 28),
            message: tray_status_box::truncate_with_ellipsis(message, 56),
            extended: String::new(),
            shown_at: Instant::now(),
        });
    }

    /// Ports `_SetTrayMessageBoxText` (UniExtract.au3:4324-4327),
    /// already covered decision-logic-wise by C166's `teelog` -- this is
    /// the real widget-side setter that logic drives, once wired.
    pub fn set_status_extended(&mut self, text: &str) {
        if let Some(state) = &mut self.status_box {
            state.extended = text.to_string();
        }
    }

    /// Ports `_DeleteTrayMessageBox` (UniExtract.au3:4331-4342), minus
    /// the fade-out animation (see [`FADE_IN_MS`]'s doc comment).
    pub fn clear_status(&mut self) {
        self.status_box = None;
    }

    fn render_status_box(&mut self, ctx: &egui::Context) {
        let Some(state) = &self.status_box else {
            return;
        };
        let alpha = (state.shown_at.elapsed().as_millis() as f32 / FADE_IN_MS).min(1.0);
        let (x, y) = tray_status_box::resolve_position(
            None,
            STATUS_BOX_SIZE,
            taskbar_rect(),
            desktop_size(),
        );

        let filename = state.filename.clone();
        let message = state.message.clone();
        let extended = state.extended.clone();

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("tray_status_box"),
            egui::ViewportBuilder::default()
                .with_title("")
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_taskbar(false)
                .with_inner_size(egui::vec2(
                    STATUS_BOX_SIZE.0 as f32,
                    STATUS_BOX_SIZE.1 as f32,
                ))
                .with_position(egui::pos2(x as f32, y as f32)),
            move |ctx, _class| {
                let background =
                    egui::Color32::from_rgba_unmultiplied(0x2D, 0x2D, 0x2D, (alpha * 255.0) as u8);
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(background).corner_radius(5))
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::WHITE, &filename);
                        ui.colored_label(egui::Color32::WHITE, &message);
                        ui.colored_label(egui::Color32::WHITE, &extended);
                    });
            },
        );
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_status_box(ctx);

        match self.tray.poll_command() {
            Some(TrayCommand::ToggleHideStatus(checked)) => self.no_status_box = checked,
            Some(TrayCommand::Exit) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            None => {}
        }

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

        // Keeps polling the tray-menu event channel promptly even while
        // the window is idle/unfocused -- egui otherwise only redraws
        // on input, and a tray click is exactly the kind of event that
        // arrives with none.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
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

/// Real `GetSystemMetrics(SM_CXSCREEN/SM_CYSCREEN)` call backing
/// [`tray_status_box::should_suppress_for_fullscreen`]'s and
/// [`tray_status_box::resolve_position`]'s desktop-size inputs
/// (`@DesktopWidth`/`@DesktopHeight`, UniExtract.au3:4269,4309,4311).
fn desktop_size() -> (i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

/// Real `WinGetPos("[ACTIVE]")` equivalent backing
/// [`tray_status_box::should_suppress_for_fullscreen`]'s fullscreen
/// check (UniExtract.au3:4268). Returns `(0, 0)` on any API failure,
/// which never equals a real desktop size and so never falsely
/// suppresses the popup.
fn active_window_size() -> (i32, i32) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return (0, 0);
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return (0, 0);
        }
        (rect.right - rect.left, rect.bottom - rect.top)
    }
}

/// Real `WinGetPos("[CLASS:Shell_TrayWnd]")` equivalent backing
/// [`tray_status_box::resolve_position`]'s taskbar-relative placement
/// (UniExtract.au3:4305-4306). Returns `None` on any API failure,
/// matching the source's own `@error` fallback path.
fn taskbar_rect() -> Option<ScreenRect> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowRect};
    unsafe {
        let class_name: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        let hwnd = FindWindowW(
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR::null(),
        )
        .ok()?;
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        Some(ScreenRect {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        })
    }
}
