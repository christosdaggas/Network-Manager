// Network Manager - Profile Row Widget
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! A list row widget for displaying a profile.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;
use adw::subclass::prelude::*;
use std::cell::RefCell;

use crate::models::profile::{Profile, ProfileStatus};
use super::status_pill::{StatusPill, PillStatus};
use crate::tr;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ProfileRow {
        pub profile_id: RefCell<Option<String>>,
        pub activate_btn: RefCell<Option<gtk::Button>>,
        pub status_pill: RefCell<Option<StatusPill>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProfileRow {
        const NAME: &'static str = "CdNetworkManagerProfileRow";
        type Type = super::ProfileRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for ProfileRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ProfileRow {}
    impl ListBoxRowImpl for ProfileRow {}
    impl adw::subclass::prelude::PreferencesRowImpl for ProfileRow {}
    impl adw::subclass::prelude::ActionRowImpl for ProfileRow {}
}

glib::wrapper! {
    pub struct ProfileRow(ObjectSubclass<imp::ProfileRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible;
}

impl ProfileRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.add_css_class("profile-row");
        self.set_accessible_role(gtk::AccessibleRole::ListItem);

        // Activate button
        let activate_btn = gtk::Button::from_icon_name("media-playback-start-symbolic");
        activate_btn.set_valign(gtk::Align::Center);
        activate_btn.set_tooltip_text(Some(&tr("Activate Profile")));
        activate_btn.add_css_class("flat");
        self.add_suffix(&activate_btn);
        *imp.activate_btn.borrow_mut() = Some(activate_btn);

        // Status pill
        let status_pill = StatusPill::new(&tr("Inactive"), PillStatus::Inactive);
        self.add_suffix(&status_pill);
        *imp.status_pill.borrow_mut() = Some(status_pill);

        // Chevron for navigation
        let chevron = gtk::Image::from_icon_name("go-next-symbolic");
        chevron.add_css_class("dim-label");
        self.add_suffix(&chevron);
        self.set_activatable(true);
    }

    /// Configure the row from a Profile.
    pub fn set_profile(&self, profile: &Profile) {
        let imp = self.imp();

        *imp.profile_id.borrow_mut() = Some(profile.metadata.id.to_string());

        self.set_title(&profile.metadata.name);

        if let Some(ref desc) = profile.metadata.description {
            self.set_subtitle(desc);
        } else {
            self.set_subtitle("");
        }

        // Screen-reader label; the status words sit inside a sentence
        let status_text = match profile.status {
            // TRANSLATORS: Profile status inside a screen-reader sentence
            ProfileStatus::Active => tr("active"),
            // TRANSLATORS: Profile status inside a screen-reader sentence
            ProfileStatus::Inactive => tr("inactive"),
            // TRANSLATORS: Profile status inside a screen-reader sentence
            ProfileStatus::Applying => tr("applying"),
            // TRANSLATORS: Profile status inside a screen-reader sentence
            ProfileStatus::Error => tr("error"),
        };
        // TRANSLATORS: Screen-reader label for a profile row, e.g.
        // "Profile Home, status: active"
        let label = tr("Profile {name}, status: {status}")
            .replace("{status}", &status_text)
            .replace("{name}", &profile.metadata.name);
        self.update_property(&[gtk::accessible::Property::Label(&label)]);

        // Update status pill
        if let Some(pill) = imp.status_pill.borrow().as_ref() {
            let (text, status) = match profile.status {
                ProfileStatus::Active => (tr("Active"), PillStatus::Active),
                ProfileStatus::Inactive => (tr("Inactive"), PillStatus::Inactive),
                ProfileStatus::Applying => (tr("Applying"), PillStatus::Pending),
                ProfileStatus::Error => (tr("Error"), PillStatus::Error),
            };
            pill.set_text(&text);
            pill.set_status(status);
        }

        // Update activate button visibility
        if let Some(btn) = imp.activate_btn.borrow().as_ref() {
            btn.set_visible(profile.status != ProfileStatus::Active);
        }
    }

    /// Get the profile ID.
    pub fn profile_id(&self) -> Option<String> {
        self.imp().profile_id.borrow().clone()
    }

    /// Connect to the activate button click.
    pub fn connect_activate_clicked<F>(&self, f: F)
    where
        F: Fn(&Self) + 'static,
    {
        let imp = self.imp();
        if let Some(btn) = imp.activate_btn.borrow().as_ref() {
            let this = self.clone();
            btn.connect_clicked(move |_| {
                f(&this);
            });
        }
    }
}

impl Default for ProfileRow {
    fn default() -> Self {
        Self::new()
    }
}
