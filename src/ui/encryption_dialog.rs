// Network Manager - Encryption Passphrase Dialog
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Reusable dialog for setting or unlocking the profile encryption passphrase.

use std::rc::Rc;

use adw::prelude::*;
use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::ui::dialog_window;
use crate::{ntr, tr};

/// Minimum accepted passphrase length.
pub const MIN_PASSPHRASE_LEN: usize = 8;

/// Show a passphrase dialog.
///
/// `heading`, `body` and `accept_label` are shown as given, so callers pass
/// translated text. `on_submit` runs when the user accepts the dialog.
/// Returning `Err` keeps the dialog open and displays the message (also
/// translated), so the user can correct it.
pub fn show_passphrase_dialog<F>(
    parent: Option<&gtk::Window>,
    heading: &str,
    body: &str,
    confirm: bool,
    accept_label: &str,
    on_submit: F,
) where
    F: Fn(&str) -> Result<(), String> + 'static,
{
    let dialog = dialog_window::new(heading, 400, -1);

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let cancel_button = gtk::Button::with_label(&tr("Cancel"));
    let accept_button = gtk::Button::with_label(accept_label);
    accept_button.add_css_class("suggested-action");
    header.pack_start(&cancel_button);
    header.pack_end(&accept_button);
    toolbar.add_top_bar(&header);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(24)
        .margin_start(18)
        .margin_end(18)
        .build();

    let description = gtk::Label::builder()
        .label(body)
        .wrap(true)
        .xalign(0.0)
        .build();
    content.append(&description);

    let entry = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .placeholder_text(tr("Passphrase"))
        .build();
    content.append(&entry);

    let confirm_entry = confirm.then(|| {
        let entry = gtk::PasswordEntry::builder()
            .show_peek_icon(true)
            .placeholder_text(tr("Confirm passphrase"))
            .build();
        content.append(&entry);
        entry
    });

    let error_label = gtk::Label::builder()
        .wrap(true)
        .xalign(0.0)
        .visible(false)
        .build();
    error_label.add_css_class("error");
    content.append(&error_label);

    toolbar.set_content(Some(&content));
    dialog.set_content(Some(&toolbar));

    cancel_button.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
        }
    });

    let submit: Rc<dyn Fn()> = Rc::new({
        let dialog = dialog.clone();
        let entry = entry.clone();
        let confirm_entry = confirm_entry.clone();
        let error_label = error_label.clone();
        move || {
            let passphrase = entry.text().to_string();
            let show_error = |message: &str| {
                error_label.set_text(message);
                error_label.set_visible(true);
            };

            if passphrase.chars().count() < MIN_PASSPHRASE_LEN {
                let min = MIN_PASSPHRASE_LEN as u32;
                show_error(
                    &ntr(
                        "Use at least {n} character.",
                        "Use at least {n} characters.",
                        min,
                    )
                    .replace("{n}", &min.to_string()),
                );
                return;
            }

            if let Some(ref confirm_entry) = confirm_entry {
                if passphrase != confirm_entry.text() {
                    show_error(&tr("The passphrases do not match."));
                    return;
                }
            }

            match on_submit(&passphrase) {
                Ok(()) => {
                    dialog.close();
                }
                Err(message) => show_error(&message),
            }
        }
    });

    accept_button.connect_clicked({
        let submit = submit.clone();
        move |_| submit()
    });
    entry.connect_activate({
        let submit = submit.clone();
        move |_| submit()
    });
    if let Some(ref confirm_entry) = confirm_entry {
        confirm_entry.connect_activate(move |_| submit());
    }

    match parent {
        Some(parent) => dialog_window::present(&dialog, Some(parent)),
        None => dialog_window::present(&dialog, None::<&gtk::Widget>),
    }
}
