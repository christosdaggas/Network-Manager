// Network Manager - Network Card Widget
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! A card widget for displaying network interface status.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;

use crate::tr;

/// Network interface type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InterfaceType {
    #[default]
    Ethernet,
    Wifi,
    Vpn,
    Bridge,
    Other,
}

impl InterfaceType {
    fn icon_name(&self) -> &'static str {
        match self {
            InterfaceType::Ethernet => "network-wired-symbolic",
            InterfaceType::Wifi => "network-wireless-symbolic",
            InterfaceType::Vpn => "network-vpn-symbolic",
            InterfaceType::Bridge => "network-transmit-receive-symbolic",
            InterfaceType::Other => "network-idle-symbolic",
        }
    }
}

/// Connection state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct NetworkCard {
        pub icon: RefCell<Option<gtk::Image>>,
        pub name_label: RefCell<Option<gtk::Label>>,
        pub status_label: RefCell<Option<gtk::Label>>,
        pub ip_label: RefCell<Option<gtk::Label>>,
        pub interface_type: RefCell<InterfaceType>,
        pub state: RefCell<ConnectionState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NetworkCard {
        const NAME: &'static str = "CdNetworkManagerNetworkCard";
        type Type = super::NetworkCard;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for NetworkCard {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for NetworkCard {}
    impl BoxImpl for NetworkCard {}
}

glib::wrapper! {
    pub struct NetworkCard(ObjectSubclass<imp::NetworkCard>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible;
}

impl NetworkCard {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Horizontal)
            .property("spacing", 12)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.add_css_class("card");
        self.add_css_class("network-card");
        self.set_margin_top(6);
        self.set_margin_bottom(6);
        self.set_margin_start(6);
        self.set_margin_end(6);
        self.set_accessible_role(gtk::AccessibleRole::Group);

        // Icon
        let icon = gtk::Image::from_icon_name("network-wired-symbolic");
        icon.set_pixel_size(32);
        icon.add_css_class("dim-label");
        self.append(&icon);
        *imp.icon.borrow_mut() = Some(icon);

        // Info box
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_hexpand(true);
        info_box.set_valign(gtk::Align::Center);

        // TRANSLATORS: Placeholder for an interface whose name is not known yet
        let name_label = gtk::Label::new(Some(&tr("Unknown")));
        name_label.add_css_class("heading");
        name_label.set_halign(gtk::Align::Start);
        info_box.append(&name_label);
        *imp.name_label.borrow_mut() = Some(name_label);

        let details_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let status_label = gtk::Label::new(Some(&tr("Disconnected")));
        status_label.add_css_class("dim-label");
        status_label.add_css_class("caption");
        details_box.append(&status_label);
        *imp.status_label.borrow_mut() = Some(status_label);

        let ip_label = gtk::Label::new(Some(""));
        ip_label.add_css_class("dim-label");
        ip_label.add_css_class("caption");
        ip_label.add_css_class("monospace");
        details_box.append(&ip_label);
        *imp.ip_label.borrow_mut() = Some(ip_label);

        info_box.append(&details_box);
        self.append(&info_box);

        // Status indicator
        let status_dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        status_dot.set_size_request(12, 12);
        status_dot.add_css_class("status-indicator");
        status_dot.add_css_class("disconnected");
        status_dot.set_valign(gtk::Align::Center);
        self.append(&status_dot);
    }

    /// Set the interface name.
    pub fn set_name(&self, name: &str) {
        let imp = self.imp();
        if let Some(label) = imp.name_label.borrow().as_ref() {
            label.set_text(name);
        }
    }

    /// Set the interface type.
    pub fn set_interface_type(&self, iface_type: InterfaceType) {
        let imp = self.imp();
        *imp.interface_type.borrow_mut() = iface_type;

        if let Some(icon) = imp.icon.borrow().as_ref() {
            icon.set_icon_name(Some(iface_type.icon_name()));
        }
    }

    /// Set the connection state.
    pub fn set_state(&self, state: ConnectionState) {
        let imp = self.imp();
        *imp.state.borrow_mut() = state;

        if let Some(label) = imp.status_label.borrow().as_ref() {
            let text = match state {
                ConnectionState::Disconnected => tr("Disconnected"),
                ConnectionState::Connecting => tr("Connecting..."),
                ConnectionState::Connected => tr("Connected"),
                ConnectionState::Error => tr("Error"),
            };
            label.set_text(&text);
        }
    }

    /// Set the IP address.
    pub fn set_ip(&self, ip: Option<&str>) {
        let imp = self.imp();
        if let Some(label) = imp.ip_label.borrow().as_ref() {
            match ip {
                Some(addr) => {
                    label.set_text(addr);
                    label.set_visible(true);
                }
                None => {
                    label.set_visible(false);
                }
            }
        }
    }

    /// Update the card with interface info.
    pub fn update(&self, name: &str, iface_type: InterfaceType, state: ConnectionState, ip: Option<&str>) {
        self.set_name(name);
        self.set_interface_type(iface_type);
        self.set_state(state);
        self.set_ip(ip);

        // Screen-reader label; the words are lowercase because they sit
        // inside a sentence.
        let state_text = match state {
            // TRANSLATORS: Connection state inside a screen-reader sentence
            ConnectionState::Disconnected => tr("disconnected"),
            // TRANSLATORS: Connection state inside a screen-reader sentence
            ConnectionState::Connecting => tr("connecting"),
            // TRANSLATORS: Connection state inside a screen-reader sentence
            ConnectionState::Connected => tr("connected"),
            // TRANSLATORS: Connection state inside a screen-reader sentence
            ConnectionState::Error => tr("error"),
        };
        let ip_text = match ip {
            Some(addr) => addr.to_string(),
            None => tr("no IP address"),
        };
        let type_text = match iface_type {
            InterfaceType::Ethernet => tr("Ethernet"),
            InterfaceType::Wifi => tr("Wi-Fi"),
            InterfaceType::Vpn => tr("VPN"),
            // TRANSLATORS: Network bridge interface type
            InterfaceType::Bridge => tr("Bridge"),
            // TRANSLATORS: Generic network interface type
            InterfaceType::Other => tr("Network"),
        };
        // TRANSLATORS: Screen-reader label for a network interface card, e.g.
        // "wlan0 Wi-Fi interface, connected, 192.168.1.5"
        let label = tr("{name} {type} interface, {state}, {ip}")
            .replace("{type}", &type_text)
            .replace("{state}", &state_text)
            .replace("{ip}", &ip_text)
            .replace("{name}", name);
        self.update_property(&[gtk::accessible::Property::Label(&label)]);
    }
}

impl Default for NetworkCard {
    fn default() -> Self {
        Self::new()
    }
}
