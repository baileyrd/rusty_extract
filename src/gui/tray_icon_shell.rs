//! Real system tray icon and menu (capability C184), built on the
//! `tray-icon` crate — the closest real-API equivalent of AutoIt's own
//! `TrayCreateItem`/`TraySetOnEvent` primitives, chosen because
//! `eframe`/`winit` owns the event loop and a hand-rolled
//! `Shell_NotifyIconW` integration would have to cooperate with that
//! loop rather than run its own. Windows-only (`#[cfg(windows)]`), same
//! honesty caveat as the rest of `gui`: this dev environment can't
//! visually verify a real tray icon, only that this compiles for the
//! target.
//!
//! Ports `Tray_Create` (UniExtract.au3:8149-8161): a two-item menu
//! ("Hide status" checkbox, "Exit"), built once before the `eframe`
//! event loop starts (per this crate's own documented winit-integration
//! pattern) and polled for clicks from [`MainWindow::update`]
//! (`crate::gui::app`) every frame via [`TrayHandle::poll_command`].

use crate::gui::tray;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

/// A command the tray menu produced this frame, for
/// [`crate::gui::app::MainWindow::update`] to act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    /// The "Hide status" item was clicked; carries its new checked
    /// state (`tray-icon` toggles a `CheckMenuItem`'s own checked state
    /// on click, so this just reports the result).
    ToggleHideStatus(bool),
    Exit,
}

pub struct TrayHandle {
    _icon: Option<TrayIcon>,
    hide_status_item: CheckMenuItem,
    exit_item: MenuItem,
}

impl TrayHandle {
    /// Ports `Tray_Create`. `no_status_box`/`no_tray_icon` are the
    /// `$bOptNoStatusBox`/`$bOptNoTrayIcon` preferences at startup.
    pub fn new(no_status_box: bool, no_tray_icon: bool) -> Self {
        let hide_status_item = CheckMenuItem::new(
            "Hide status",
            true,
            tray::hide_status_item_checked(no_status_box),
            None,
        );
        let exit_item = MenuItem::new("Exit", true, None);
        let menu = Menu::new();
        let _ = menu.append(&hide_status_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&exit_item);

        let icon = if tray::should_hide_icon(no_tray_icon) {
            None
        } else {
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("UniExtract")
                .build()
                .ok()
        };

        Self {
            _icon: icon,
            hide_status_item,
            exit_item,
        }
    }

    /// Polls this frame's tray-menu click, if any. Matches by menu-item
    /// identity (`tray-icon` hands back the clicked item's ID, not the
    /// item itself) rather than assuming ordering.
    pub fn poll_command(&self) -> Option<TrayCommand> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        if event.id == self.hide_status_item.id() {
            Some(TrayCommand::ToggleHideStatus(
                self.hide_status_item.is_checked(),
            ))
        } else if event.id == self.exit_item.id() {
            Some(TrayCommand::Exit)
        } else {
            None
        }
    }
}
