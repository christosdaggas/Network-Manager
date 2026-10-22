// Network Manager - Dialog Windows
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Separate, movable windows for the app's editor and list dialogs.
//!
//! An `adw::Dialog` is drawn inside its parent window, so it cannot be moved
//! on its own. These windows stay above the main window but are not modal:
//! GNOME attaches modal dialogs to their parent, which would pin them in
//! place. They are deliberately not added to the application, so
//! `Application::active_window()` keeps returning the main window while one
//! of them has focus.

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;

/// Create a dialog window with a title and a default size.
///
/// Pass `-1` as the height to size the window to its content.
pub fn new(title: &str, width: i32, height: i32) -> adw::Window {
    let window = adw::Window::builder()
        .title(title)
        .default_width(width)
        .default_height(height)
        .modal(false)
        .destroy_with_parent(true)
        .build();

    // Escape closes the window, as it does for adw::Dialog.
    let shortcuts = gtk::ShortcutController::new();
    shortcuts.add_shortcut(gtk::Shortcut::new(
        gtk::ShortcutTrigger::parse_string("Escape"),
        Some(gtk::NamedAction::new("window.close")),
    ));
    window.add_controller(shortcuts);

    window
}

/// Show `window` above the window that contains `parent`.
pub fn present(window: &adw::Window, parent: Option<&impl IsA<gtk::Widget>>) {
    if let Some(root) = parent.and_then(|p| p.root()).and_downcast::<gtk::Window>() {
        window.set_transient_for(Some(&root));
    }
    window.present();
}
