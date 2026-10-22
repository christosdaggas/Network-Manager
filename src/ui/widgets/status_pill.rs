// Network Manager - Status Pill Widget
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! A small pill-shaped status indicator widget.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;

/// Status variants for the pill.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PillStatus {
    #[default]
    Inactive,
    Active,
    Pending,
    Error,
    Warning,
}

impl PillStatus {
    fn css_class(&self) -> &'static str {
        match self {
            PillStatus::Inactive => "dim-label",
            PillStatus::Active => "success",
            PillStatus::Pending => "accent",
            PillStatus::Error => "error",
            PillStatus::Warning => "warning",
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct StatusPill {
        pub label: RefCell<gtk::Label>,
        pub status: RefCell<PillStatus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StatusPill {
        const NAME: &'static str = "CdNetworkManagerStatusPill";
        type Type = super::StatusPill;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for StatusPill {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for StatusPill {}
    impl BoxImpl for StatusPill {}
}

glib::wrapper! {
    pub struct StatusPill(ObjectSubclass<imp::StatusPill>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible;
}

impl StatusPill {
    pub fn new(text: &str, status: PillStatus) -> Self {
        let this: Self = glib::Object::builder()
            .property("orientation", gtk::Orientation::Horizontal)
            .build();

        this.set_text(text);
        this.set_status(status);
        this
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.add_css_class("status-pill");
        self.add_css_class("pill");
        self.set_halign(gtk::Align::Start);
        self.set_valign(gtk::Align::Center);
        self.set_accessible_role(gtk::AccessibleRole::Status);

        let label = gtk::Label::new(None);
        label.add_css_class("caption");
        self.append(&label);

        *imp.label.borrow_mut() = label;
    }

    /// Set the pill text.
    pub fn set_text(&self, text: &str) {
        let imp = self.imp();
        imp.label.borrow().set_text(text);
        self.update_property(&[gtk::accessible::Property::Label(text)]);
    }

    /// Get the pill text.
    pub fn text(&self) -> String {
        let imp = self.imp();
        imp.label.borrow().text().to_string()
    }

    /// Set the status (changes styling).
    pub fn set_status(&self, status: PillStatus) {
        let imp = self.imp();

        // Remove old class
        let old_status = *imp.status.borrow();
        self.remove_css_class(old_status.css_class());

        // Add new class
        self.add_css_class(status.css_class());
        *imp.status.borrow_mut() = status;
    }

    /// Get the current status.
    pub fn status(&self) -> PillStatus {
        let imp = self.imp();
        *imp.status.borrow()
    }
}

impl Default for StatusPill {
    fn default() -> Self {
        Self::new("", PillStatus::Inactive)
    }
}
