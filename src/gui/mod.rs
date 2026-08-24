//! `rusty_extract`'s own GUI shell (migration phase 2, capability rows
//! C183-C217) — not to be confused with `automation::GuiAutomation`,
//! which drives *other* programs' GUIs on this app's behalf. This module
//! is the app's own window/tray/dialogs, built with `egui`/`eframe`.
//!
//! Submodules split along the same "pure decision logic vs. real
//! rendering" line established throughout this crate: `layout`, `theme`,
//! and `window_state` are ordinary, cross-platform-buildable, fully
//! tested Rust; `app` is the real `#[cfg(windows)]` `eframe::App`
//! implementation, verified only by `cargo check`/`cargo clippy --target
//! x86_64-pc-windows-gnu` in this dev environment (see `app`'s own doc
//! comment for the honesty caveat this carries, same as
//! `automation::win32`/`dlllib::win32`).

pub mod file_input;
pub mod layout;
pub mod theme;
pub mod tray;
pub mod tray_status_box;
pub mod window_state;

#[cfg(windows)]
pub mod app;
#[cfg(windows)]
pub mod tray_icon_shell;
