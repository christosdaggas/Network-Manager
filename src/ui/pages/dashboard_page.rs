// Network Manager - Dashboard Page
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Dashboard page showing current network status and active profile.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use adw::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::fs;

use crate::tr;

mod imp {
    use super::*;
    use once_cell::sync::OnceCell;

    #[derive(Default)]
    pub struct DashboardPage {
        // Network info labels
        pub ip_label: OnceCell<gtk::Label>,
        pub gw_label: OnceCell<gtk::Label>,
        pub dns_label: OnceCell<gtk::Label>,
        pub conn_label: OnceCell<gtk::Label>,
        pub network_icon: OnceCell<gtk::Image>,
        // Active profile labels
        pub profile_name_label: OnceCell<gtk::Label>,
        pub profile_status_pill: OnceCell<gtk::Label>,
        pub profile_last_applied: OnceCell<gtk::Label>,
        // Daemon status
        pub daemon_status_icon: OnceCell<gtk::Image>,
        pub daemon_status_label: OnceCell<gtk::Label>,
        pub daemon_restart_button: OnceCell<gtk::Button>,
        /// Increments per network refresh so a slow, older result is ignored.
        pub network_refresh_seq: Cell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DashboardPage {
        const NAME: &'static str = "CdNetworkManagerDashboardPage";
        type Type = super::DashboardPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for DashboardPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            // Initial update of network info
            self.obj().update_network_info();
        }
    }

    impl WidgetImpl for DashboardPage {}
    impl BoxImpl for DashboardPage {}
}

glib::wrapper! {
    pub struct DashboardPage(ObjectSubclass<imp::DashboardPage>)
        @extends gtk::Widget, gtk::Box;
}

impl DashboardPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 24)
            .build()
    }

    fn setup_ui(&self) {
        self.set_margin_top(12);
        self.set_margin_bottom(24);
        self.set_margin_start(32);
        self.set_margin_end(32);

        // Create scrollable content
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_valign(gtk::Align::Start);

        // Welcome Header
        let header = self.create_welcome_header();
        content.append(&header);
        content.append(&Self::create_divider());

        // Quick Actions, directly under the top panel
        let quick_actions = self.create_quick_actions_section();
        content.append(&quick_actions);
        content.append(&Self::create_divider());

        // Active Profile and Daemon Status Section (50/50 split)
        let status_row = gtk::Box::new(gtk::Orientation::Horizontal, 24);
        status_row.set_homogeneous(true);
        
        let active_profile_card = self.create_active_profile_section();
        status_row.append(&active_profile_card);
        
        let daemon_status_card = self.create_daemon_status_section();
        daemon_status_card.add_css_class("dashboard-column-end");
        status_row.append(&daemon_status_card);
        
        content.append(&status_row);
        content.append(&Self::create_divider());

        // Network Status Section (50/50 split with activity graph)
        let network_row = gtk::Box::new(gtk::Orientation::Horizontal, 24);
        network_row.set_homogeneous(true);
        
        let network_status_card = self.create_network_status_section();
        network_row.append(&network_status_card);
        
        let network_activity_card = self.create_network_activity_section();
        network_activity_card.add_css_class("dashboard-column-end");
        network_row.append(&network_activity_card);
        
        content.append(&network_row);

        scroll.set_child(Some(&content));
        self.append(&scroll);
    }

    /// Thin rule that separates the flat dashboard sections.
    fn create_divider() -> gtk::Separator {
        let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
        divider.add_css_class("dashboard-divider");
        divider
    }

    fn create_welcome_header(&self) -> gtk::Widget {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.add_css_class("dashboard-card");
        container.add_css_class("dashboard-header");
        container.set_overflow(gtk::Overflow::Hidden);

        let overlay = gtk::Overlay::new();
        
        // Network graphic drawn with accent color
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_content_width(220);
        // Matches the header text height so the graphic is never clipped
        drawing_area.set_content_height(80);
        drawing_area.set_halign(gtk::Align::End);
        drawing_area.set_valign(gtk::Align::Start);
        drawing_area.set_accessible_role(gtk::AccessibleRole::Img);
        drawing_area.update_property(&[gtk::accessible::Property::Label(&tr(
            "Decorative network topology graphic",
        ))]);
        
        drawing_area.set_draw_func(|_area, cr, width, height| {
            let width = width as f64;
            let height = height as f64;
            
            // Get accent color from the style manager
            let style_manager = adw::StyleManager::default();
            let accent_color = style_manager.accent_color_rgba();
            
            let r = accent_color.red() as f64;
            let g = accent_color.green() as f64;
            let b = accent_color.blue() as f64;
            
            // Scale factor for the drawing (original was 400x200, we draw at 300x150)
            let scale_x = width / 400.0;
            let scale_y = height / 200.0;
            
            // Draw connecting lines
            cr.set_source_rgba(r, g, b, 0.4);
            cr.set_line_width(2.0);
            
            // Path 1: M 350,50 L 300,100 L 250,60 L 200,120 L 150,80
            cr.move_to(350.0 * scale_x, 50.0 * scale_y);
            cr.line_to(300.0 * scale_x, 100.0 * scale_y);
            cr.line_to(250.0 * scale_x, 60.0 * scale_y);
            cr.line_to(200.0 * scale_x, 120.0 * scale_y);
            cr.line_to(150.0 * scale_x, 80.0 * scale_y);
            cr.stroke().ok();
            
            // Path 2: M 300,100 L 320,150 L 380,120
            cr.move_to(300.0 * scale_x, 100.0 * scale_y);
            cr.line_to(320.0 * scale_x, 150.0 * scale_y);
            cr.line_to(380.0 * scale_x, 120.0 * scale_y);
            cr.stroke().ok();
            
            // Path 3: M 350,50 L 380,120
            cr.move_to(350.0 * scale_x, 50.0 * scale_y);
            cr.line_to(380.0 * scale_x, 120.0 * scale_y);
            cr.stroke().ok();
            
            // Path 4: M 250,60 L 280,30 L 350,50
            cr.move_to(250.0 * scale_x, 60.0 * scale_y);
            cr.line_to(280.0 * scale_x, 30.0 * scale_y);
            cr.line_to(350.0 * scale_x, 50.0 * scale_y);
            cr.stroke().ok();
            
            // Path 5: M 200,120 L 220,170
            cr.move_to(200.0 * scale_x, 120.0 * scale_y);
            cr.line_to(220.0 * scale_x, 170.0 * scale_y);
            cr.stroke().ok();
            
            // Draw nodes
            cr.set_source_rgba(r, g, b, 0.6);
            
            let nodes = [
                (350.0, 50.0, 4.0),
                (300.0, 100.0, 4.0),
                (250.0, 60.0, 4.0),
                (200.0, 120.0, 4.0),
                (150.0, 80.0, 3.0),
                (320.0, 150.0, 3.0),
                (380.0, 120.0, 4.0),
                (280.0, 30.0, 3.0),
                (220.0, 170.0, 2.0),
            ];
            
            for (x, y, radius) in nodes {
                cr.arc(x * scale_x, y * scale_y, radius, 0.0, 2.0 * std::f64::consts::PI);
                cr.fill().ok();
            }
            
            // Pulse ring around main node (static representation)
            cr.set_source_rgba(r, g, b, 0.3);
            cr.set_line_width(2.0);
            cr.arc(350.0 * scale_x, 50.0 * scale_y, 12.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().ok();
            
            cr.set_source_rgba(r, g, b, 0.15);
            cr.arc(200.0 * scale_x, 120.0 * scale_y, 10.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().ok();
        });
        
        // Listen for accent color changes and redraw
        let drawing_area_weak = drawing_area.downgrade();
        let style_manager = adw::StyleManager::default();
        style_manager.connect_accent_color_rgba_notify(move |_| {
            if let Some(area) = drawing_area_weak.upgrade() {
                area.queue_draw();
            }
        });
        
        // Content
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content_box.set_margin_top(16);
        content_box.set_margin_start(0);
        content_box.set_margin_end(150); // Make space for image

        let title = gtk::Label::new(Some(&tr("Network Manager")));
        title.add_css_class("dashboard-title");
        title.set_halign(gtk::Align::Start);
        
        let subtitle = gtk::Label::new(Some(&tr("Manage your system connectivity")));
        subtitle.add_css_class("title-4");
        subtitle.add_css_class("dim-label");
        subtitle.set_halign(gtk::Align::Start);

        content_box.append(&title);
        content_box.append(&subtitle);

        // Put content as the child (bottom layer) so it dictates size
        overlay.set_child(Some(&content_box));
        // Add drawing as overlay (top layer)
        overlay.add_overlay(&drawing_area);
        
        container.append(&overlay);
        container.upcast()
    }

    fn create_active_profile_section(&self) -> gtk::Box {
        let imp = self.imp();
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("dashboard-card");
        card.add_css_class("dashboard-card-compact");

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let icon = gtk::Image::from_icon_name("contact-new-symbolic");
        icon.add_css_class("accent");
        header.append(&icon);
        let title = gtk::Label::new(Some(&tr("Active Profile")));
        title.add_css_class("heading");
        header.append(&title);
        card.append(&header);

        // Profile info
        let profile_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let profile_name = gtk::Label::new(Some(&tr("No profile active")));
        profile_name.add_css_class("title-4");
        profile_name.set_halign(gtk::Align::Start);
        profile_box.append(&profile_name);
        let _ = imp.profile_name_label.set(profile_name);

        // Status pill
        let status_pill = gtk::Label::new(Some(&tr("Inactive")));
        status_pill.add_css_class("status-pill");
        status_pill.add_css_class("inactive");
        status_pill.set_halign(gtk::Align::End);
        status_pill.set_hexpand(true);
        profile_box.append(&status_pill);
        let _ = imp.profile_status_pill.set(status_pill);

        card.append(&profile_box);

        // Last applied info
        let last_applied = gtk::Label::new(Some(&tr("Never applied")));
        last_applied.add_css_class("dim-label");
        last_applied.add_css_class("caption");
        last_applied.set_halign(gtk::Align::Start);
        card.append(&last_applied);
        let _ = imp.profile_last_applied.set(last_applied);

        card
    }
    
    fn create_daemon_status_section(&self) -> gtk::Box {
        let imp = self.imp();
        let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        card.add_css_class("dashboard-card");
        card.add_css_class("dashboard-card-compact");

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let icon = gtk::Image::from_icon_name("emblem-system-symbolic");
        icon.add_css_class("accent");
        header.append(&icon);
        let title = gtk::Label::new(Some(&tr("Service Status")));
        title.add_css_class("heading");
        header.append(&title);
        card.append(&header);

        // Status row
        let status_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        
        let status_icon = gtk::Image::from_icon_name("content-loading-symbolic");
        status_icon.set_icon_size(gtk::IconSize::Large);
        status_row.append(&status_icon);
        let _ = imp.daemon_status_icon.set(status_icon);
        
        let status_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        status_box.set_hexpand(true);
        
        let status_label = gtk::Label::new(Some(&tr("Checking...")));
        status_label.add_css_class("title-4");
        status_label.set_halign(gtk::Align::Start);
        status_box.append(&status_label);
        let _ = imp.daemon_status_label.set(status_label);
        
        let status_desc = gtk::Label::new(Some(&tr("NetworkManager handles network configuration")));
        status_desc.add_css_class("dim-label");
        status_desc.add_css_class("caption");
        status_desc.set_halign(gtk::Align::Start);
        status_box.append(&status_desc);
        
        status_row.append(&status_box);
        
        // Start button (hidden by default, shown when NetworkManager is not running)
        let restart_button = gtk::Button::with_label(&tr("Start Service"));
        restart_button.add_css_class("suggested-action");
        restart_button.set_valign(gtk::Align::Center);
        restart_button.set_visible(false);
        let page_weak = self.downgrade();
        restart_button.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            btn.set_label(&tr("Starting…"));
            let page_weak = page_weak.clone();
            glib::spawn_future_local(async move {
                let running = gio::spawn_blocking(|| {
                    // Start NetworkManager via pkexec (prompts for root password)
                    let started = std::process::Command::new("pkexec")
                        .args(["systemctl", "start", "NetworkManager.service"])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if !started {
                        // Failed or cancelled: report whatever state it is in now
                        return networkmanager_state() == "active";
                    }
                    // Wait for systemd to settle on a final state (about 15 s at most)
                    for _ in 0..15 {
                        match networkmanager_state().as_str() {
                            "activating" | "reloading" | "deactivating" => {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                            }
                            state => return state == "active",
                        }
                    }
                    false
                }).await.unwrap_or(false);

                if let Some(page) = page_weak.upgrade() {
                    page.update_daemon_status(running);
                }
            });
        });
        status_row.append(&restart_button);
        let _ = imp.daemon_restart_button.set(restart_button);
        
        card.append(&status_row);
        
        // Initial check
        self.check_daemon_status();
        
        card
    }

    fn create_network_status_section(&self) -> gtk::Box {
        let imp = self.imp();
        let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
        card.add_css_class("dashboard-card");

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let icon = gtk::Image::from_icon_name("network-wired-symbolic");
        icon.add_css_class("accent");
        header.append(&icon);
        let title = gtk::Label::new(Some(&tr("Current Network")));
        title.add_css_class("heading");
        header.append(&title);
        card.append(&header);
        
        // Store icon reference to update based on connection type
        let _ = imp.network_icon.set(icon);

        // Network info grid
        let grid = gtk::Grid::new();
        grid.set_row_spacing(8);
        grid.set_column_spacing(16);

        // IP Address
        let ip_label_title = gtk::Label::new(Some(&tr("IP Address")));
        ip_label_title.add_css_class("dim-label");
        ip_label_title.set_halign(gtk::Align::Start);
        grid.attach(&ip_label_title, 0, 0, 1, 1);

        let ip_value = gtk::Label::new(Some("—"));
        ip_value.set_halign(gtk::Align::Start);
        ip_value.set_selectable(true);
        grid.attach(&ip_value, 1, 0, 1, 1);
        let _ = imp.ip_label.set(ip_value);

        // Gateway
        let gw_label_title = gtk::Label::new(Some(&tr("Gateway")));
        gw_label_title.add_css_class("dim-label");
        gw_label_title.set_halign(gtk::Align::Start);
        grid.attach(&gw_label_title, 0, 1, 1, 1);

        let gw_value = gtk::Label::new(Some("—"));
        gw_value.set_halign(gtk::Align::Start);
        gw_value.set_selectable(true);
        grid.attach(&gw_value, 1, 1, 1, 1);
        let _ = imp.gw_label.set(gw_value);

        // DNS
        let dns_label_title = gtk::Label::new(Some(&tr("DNS Servers")));
        dns_label_title.add_css_class("dim-label");
        dns_label_title.set_halign(gtk::Align::Start);
        grid.attach(&dns_label_title, 0, 2, 1, 1);

        let dns_value = gtk::Label::new(Some("—"));
        dns_value.set_halign(gtk::Align::Start);
        dns_value.set_selectable(true);
        dns_value.set_wrap(true);
        dns_value.set_max_width_chars(30);
        grid.attach(&dns_value, 1, 2, 1, 1);
        let _ = imp.dns_label.set(dns_value);

        // Connection Type
        // TRANSLATORS: Label for the type and name of the active connection
        let conn_label_title = gtk::Label::new(Some(&tr("Connection")));
        conn_label_title.add_css_class("dim-label");
        conn_label_title.set_halign(gtk::Align::Start);
        grid.attach(&conn_label_title, 0, 3, 1, 1);

        let conn_value = gtk::Label::new(Some("—"));
        conn_value.set_halign(gtk::Align::Start);
        grid.attach(&conn_value, 1, 3, 1, 1);
        let _ = imp.conn_label.set(conn_value);

        card.append(&grid);

        card
    }

    fn create_network_activity_section(&self) -> gtk::Box {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
        card.add_css_class("dashboard-card");

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let icon = gtk::Image::from_icon_name("network-transmit-receive-symbolic");
        icon.add_css_class("accent");
        header.append(&icon);
        let title = gtk::Label::new(Some(&tr("Network Activity")));
        title.add_css_class("heading");
        header.append(&title);
        
        // Current speeds on the trailing edge; the swatches double as the
        // graph legend.
        let speeds = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        speeds.set_halign(gtk::Align::End);
        speeds.set_valign(gtk::Align::Center);
        speeds.set_hexpand(true);

        let dl_legend = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let dl_color = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        dl_color.set_size_request(8, 8);
        dl_color.set_valign(gtk::Align::Center);
        dl_color.add_css_class("legend-download");
        let dl_label = gtk::Label::new(Some(&format!("↓ {}", format_speed(0.0))));
        dl_label.add_css_class("caption");
        dl_label.add_css_class("numeric");
        dl_label.set_width_chars(10);
        dl_label.set_xalign(0.0);
        dl_legend.append(&dl_color);
        dl_legend.append(&dl_label);

        let ul_legend = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let ul_color = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        ul_color.set_size_request(8, 8);
        ul_color.set_valign(gtk::Align::Center);
        ul_color.add_css_class("legend-upload");
        let ul_label = gtk::Label::new(Some(&format!("↑ {}", format_speed(0.0))));
        ul_label.add_css_class("caption");
        ul_label.add_css_class("numeric");
        ul_label.set_width_chars(10);
        ul_label.set_xalign(0.0);
        ul_legend.append(&ul_color);
        ul_legend.append(&ul_label);

        speeds.append(&dl_legend);
        speeds.append(&ul_legend);
        header.append(&speeds);

        card.append(&header);

        // Drawing area for the graph
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_content_height(96);
        drawing_area.set_vexpand(true);
        drawing_area.add_css_class("network-graph");
        drawing_area.set_accessible_role(gtk::AccessibleRole::Img);
        drawing_area.update_property(&[gtk::accessible::Property::Label(&tr(
            "Network activity graph showing download and upload rates over time",
        ))]);

        // Store data points for the graph (real network data)
        let download_data = Rc::new(RefCell::new(vec![0.0f64; 60]));
        let upload_data = Rc::new(RefCell::new(vec![0.0f64; 60]));
        
        // Store previous byte counts to calculate rate
        let prev_stats: Rc<RefCell<Option<(u64, u64)>>> = Rc::new(RefCell::new(None));

        // Set up drawing function
        let dl_data = download_data.clone();
        let ul_data = upload_data.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let width = width as f64;
            let height = height as f64;
            
            // Get current color scheme
            let is_dark = adw::StyleManager::default().is_dark();
            
            // Use transparent background - let the card handle the background
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().ok();
            
            // Grid lines - subtle, theme-aware
            if is_dark {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
            } else {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
            }
            cr.set_line_width(0.5);
            for i in 1..4 {
                let y = height * (i as f64) / 4.0;
                cr.move_to(0.0, y);
                cr.line_to(width, y);
            }
            cr.stroke().ok();

            let dl = dl_data.borrow();
            let ul = ul_data.borrow();
            
            let max_val = dl.iter().chain(ul.iter())
                .cloned()
                .fold(1.0f64, f64::max);
            
            let step = width / (dl.len() as f64 - 1.0);
            
            // Draw download line (blue)
            cr.set_source_rgba(0.21, 0.52, 0.89, 0.8); // Accent blue
            cr.set_line_width(2.0);
            for (i, &val) in dl.iter().enumerate() {
                let x = i as f64 * step;
                let y = height - (val / max_val * height * 0.9) - 5.0;
                if i == 0 {
                    cr.move_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }
            cr.stroke().ok();
            
            // Fill under download line
            cr.set_source_rgba(0.21, 0.52, 0.89, 0.15);
            for (i, &val) in dl.iter().enumerate() {
                let x = i as f64 * step;
                let y = height - (val / max_val * height * 0.9) - 5.0;
                if i == 0 {
                    cr.move_to(x, height);
                    cr.line_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }
            cr.line_to(width, height);
            cr.close_path();
            cr.fill().ok();
            
            // Draw upload line (green)
            cr.set_source_rgba(0.18, 0.76, 0.49, 0.8); // Success green
            cr.set_line_width(2.0);
            for (i, &val) in ul.iter().enumerate() {
                let x = i as f64 * step;
                let y = height - (val / max_val * height * 0.9) - 5.0;
                if i == 0 {
                    cr.move_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }
            cr.stroke().ok();
        });

        card.append(&drawing_area);

        // Real network data update
        let drawing_area_weak = drawing_area.downgrade();
        let dl_label_weak = dl_label.downgrade();
        let ul_label_weak = ul_label.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
            let Some(area) = drawing_area_weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            
            // Read real network stats from /proc/net/dev
            let (rx_bytes, tx_bytes) = read_network_stats();
            
            let mut prev = prev_stats.borrow_mut();
            let (dl_rate, ul_rate) = if let Some((prev_rx, prev_tx)) = *prev {
                // Calculate bytes per second
                let dl = rx_bytes.saturating_sub(prev_rx) as f64;
                let ul = tx_bytes.saturating_sub(prev_tx) as f64;
                (dl, ul)
            } else {
                (0.0, 0.0)
            };
            *prev = Some((rx_bytes, tx_bytes));
            drop(prev);
            
            // Update data arrays
            let mut dl = download_data.borrow_mut();
            let mut ul = upload_data.borrow_mut();
            dl.remove(0);
            ul.remove(0);
            // Scale to KB/s for graph (divide by 1024)
            dl.push(dl_rate / 1024.0);
            ul.push(ul_rate / 1024.0);
            drop(dl);
            drop(ul);
            
            // Update legend labels with human-readable speeds
            if let Some(label) = dl_label_weak.upgrade() {
                label.set_text(&format!("↓ {}", format_speed(dl_rate)));
            }
            if let Some(label) = ul_label_weak.upgrade() {
                label.set_text(&format!("↑ {}", format_speed(ul_rate)));
            }
            
            area.queue_draw();
            glib::ControlFlow::Continue
        });

        card
    }

    fn create_quick_actions_section(&self) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        row.add_css_class("dashboard-card");
        row.add_css_class("dashboard-card-compact");
        row.add_css_class("quick-actions");
        row.set_valign(gtk::Align::Center);

        // Section title and short description on the leading edge
        let text_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        text_box.set_hexpand(true);
        text_box.set_valign(gtk::Align::Center);
        text_box.set_margin_start(4);

        let icon = gtk::Image::from_icon_name("system-run-symbolic");
        icon.add_css_class("accent");
        text_box.append(&icon);

        let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let title = gtk::Label::new(Some(&tr("Quick Actions")));
        title.add_css_class("heading");
        title.set_halign(gtk::Align::Start);
        labels.append(&title);

        let subtitle = gtk::Label::new(Some(&tr("Refresh status or switch to another profile")));
        subtitle.add_css_class("caption");
        subtitle.add_css_class("dim-label");
        subtitle.set_halign(gtk::Align::Start);
        labels.append(&subtitle);

        text_box.append(&labels);
        row.append(&text_box);

        // Compact action buttons on the trailing edge
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        button_box.set_halign(gtk::Align::End);
        button_box.set_valign(gtk::Align::Center);

        let refresh_btn = gtk::Button::with_label(&tr("Refresh"));
        refresh_btn.add_css_class("suggested-action");
        // Connect refresh button to update network info
        let page_weak = self.downgrade();
        refresh_btn.connect_clicked(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.update_network_info();
            }
        });
        button_box.append(&refresh_btn);

        let switch_btn = gtk::Button::with_label(&tr("Switch Profile"));
        switch_btn.set_action_name(Some("win.show-profile-switcher"));
        button_box.append(&switch_btn);

        let new_profile_btn = gtk::Button::with_label(&tr("New Profile"));
        new_profile_btn.set_action_name(Some("win.new-profile"));
        button_box.append(&new_profile_btn);

        row.append(&button_box);

        row
    }
    
    /// Refresh the network info labels; the lookup runs off the GTK thread.
    pub fn update_network_info(&self) {
        let imp = self.imp();
        let seq = imp.network_refresh_seq.get().wrapping_add(1);
        imp.network_refresh_seq.set(seq);
        
        let page_weak = self.downgrade();
        glib::spawn_future_local(async move {
            // Get primary network interface and its info
            let Ok(net_info) = gio::spawn_blocking(get_primary_network_info).await else {
                return;
            };
            if let Some(page) = page_weak.upgrade() {
                // A newer refresh has started; its result will replace this one
                if page.imp().network_refresh_seq.get() == seq {
                    page.show_network_info(&net_info);
                }
            }
        });
    }
    
    /// Update network info labels with current network status
    fn show_network_info(&self, net_info: &NetworkInfo) {
        let imp = self.imp();
        
        if let Some(ip_label) = imp.ip_label.get() {
            ip_label.set_text(&net_info.ip_address);
        }
        
        if let Some(gw_label) = imp.gw_label.get() {
            gw_label.set_text(&net_info.gateway);
        }
        
        if let Some(dns_label) = imp.dns_label.get() {
            dns_label.set_text(&net_info.dns_servers);
        }
        
        if let Some(conn_label) = imp.conn_label.get() {
            conn_label.set_text(&net_info.connection_type);
        }
        
        // Update icon based on connection type
        if let Some(icon) = imp.network_icon.get() {
            icon.set_icon_name(Some(net_info.icon_name));
        }
    }
    
    /// Update the active profile display
    pub fn update_active_profile(&self, profile_name: Option<&str>, applied_time: Option<&str>) {
        let imp = self.imp();
        
        if let Some(name_label) = imp.profile_name_label.get() {
            if let Some(name) = profile_name {
                name_label.set_text(name);
            } else {
                name_label.set_text(&tr("No profile active"));
            }
        }
        
        if let Some(status_pill) = imp.profile_status_pill.get() {
            if profile_name.is_some() {
                status_pill.set_text(&tr("Active"));
                status_pill.remove_css_class("inactive");
                status_pill.add_css_class("active");
            } else {
                status_pill.set_text(&tr("Inactive"));
                status_pill.remove_css_class("active");
                status_pill.add_css_class("inactive");
            }
        }
        
        if let Some(last_applied_label) = imp.profile_last_applied.get() {
            if let Some(time) = applied_time {
                last_applied_label.set_text(&tr("Applied: {time}").replace("{time}", time));
            } else {
                last_applied_label.set_text(&tr("Never applied"));
            }
        }
    }
    
    /// Check daemon status asynchronously
    pub fn check_daemon_status(&self) {
        let page_weak = self.downgrade();
        glib::spawn_future_local(async move {
            // Check if NetworkManager is running (the real service this app depends on);
            // the future completes once systemctl has answered, however long that takes
            let is_running = gio::spawn_blocking(|| networkmanager_state() == "active")
                .await
                .unwrap_or(false);
            if let Some(page) = page_weak.upgrade() {
                page.update_daemon_status(is_running);
            }
        });
    }
    
    /// Update daemon status UI
    pub fn update_daemon_status(&self, is_running: bool) {
        let imp = self.imp();
        
        if let Some(icon) = imp.daemon_status_icon.get() {
            if is_running {
                icon.set_icon_name(Some("object-select-symbolic"));
                icon.remove_css_class("error");
                icon.add_css_class("success");
            } else {
                icon.set_icon_name(Some("dialog-error-symbolic"));
                icon.remove_css_class("success");
                icon.add_css_class("error");
            }
        }
        
        if let Some(label) = imp.daemon_status_label.get() {
            if is_running {
                // TRANSLATORS: State of the NetworkManager system service
                label.set_text(&tr("Running"));
            } else {
                // TRANSLATORS: State of the NetworkManager system service
                label.set_text(&tr("Not Running"));
            }
        }
        
        if let Some(button) = imp.daemon_restart_button.get() {
            // Clear any "Starting…" state left by a previous attempt
            button.set_label(&tr("Start Service"));
            button.set_sensitive(true);
            button.set_visible(!is_running);
        }
    }
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new()
    }
}

/// NetworkManager's systemd state ("active", "activating", "failed", ...),
/// or an empty string when systemctl cannot be run.
fn networkmanager_state() -> String {
    std::process::Command::new("systemctl")
        .env("LC_ALL", "C")
        .args(["is-active", "NetworkManager.service"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Read network statistics from /proc/net/dev
/// Returns (total_rx_bytes, total_tx_bytes) across all non-loopback interfaces
fn read_network_stats() -> (u64, u64) {
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;
    
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            // Each line format: interface: rx_bytes rx_packets ... tx_bytes tx_packets ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let iface = parts[0].trim_end_matches(':');
                // Skip loopback
                if iface == "lo" {
                    continue;
                }
                // rx_bytes is at index 0 (after interface name), tx_bytes at index 8
                if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[9].parse::<u64>()) {
                    total_rx = total_rx.saturating_add(rx);
                    total_tx = total_tx.saturating_add(tx);
                }
            }
        }
    }
    
    (total_rx, total_tx)
}

/// Format bytes per second to human-readable string
fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    
    if bytes_per_sec >= GB {
        // TRANSLATORS: Transfer rate in gigabytes per second
        tr("{speed} GB/s").replace("{speed}", &format!("{:.1}", bytes_per_sec / GB))
    } else if bytes_per_sec >= MB {
        // TRANSLATORS: Transfer rate in megabytes per second
        tr("{speed} MB/s").replace("{speed}", &format!("{:.1}", bytes_per_sec / MB))
    } else if bytes_per_sec >= KB {
        // TRANSLATORS: Transfer rate in kilobytes per second
        tr("{speed} KB/s").replace("{speed}", &format!("{:.1}", bytes_per_sec / KB))
    } else {
        // TRANSLATORS: Transfer rate in bytes per second
        tr("{speed} B/s").replace("{speed}", &format!("{:.0}", bytes_per_sec))
    }
}

/// Network info structure
struct NetworkInfo {
    ip_address: String,
    gateway: String,
    dns_servers: String,
    connection_type: String,
    /// Chosen from the nmcli connection type, not from the display text,
    /// which also contains the user's connection name
    icon_name: &'static str,
}

/// Get primary network interface information using nmcli and ip commands
fn get_primary_network_info() -> NetworkInfo {
    use std::process::Command;
    
    let mut info = NetworkInfo {
        ip_address: "—".to_string(),
        gateway: "—".to_string(),
        dns_servers: "—".to_string(),
        connection_type: tr("Disconnected"),
        icon_name: "network-offline-symbolic",
    };
    
    // Get active connection info using nmcli
    if let Ok(output) = Command::new("nmcli")
        .env("LC_ALL", "C")
        .args(["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show", "--active"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let conn_name = parts[0];
                    let conn_type = parts[1];
                    let device = parts[2];
                    
                    // Skip loopback and virtual interfaces
                    if device == "lo" || device.starts_with("virbr") || device.starts_with("docker") {
                        continue;
                    }
                    
                    // Determine connection type display name
                    info.connection_type = match conn_type {
                        "802-11-wireless" => tr("Wi-Fi ({name})").replace("{name}", conn_name),
                        "802-3-ethernet" => tr("Ethernet ({name})").replace("{name}", conn_name),
                        "vpn" => tr("VPN ({name})").replace("{name}", conn_name),
                        // TRANSLATORS: A network bridge connection and its name
                        "bridge" => tr("Bridge ({name})").replace("{name}", conn_name),
                        _ => format!("{} ({})", conn_type, conn_name),
                    };
                    info.icon_name = match conn_type {
                        "802-11-wireless" => "network-wireless-symbolic",
                        "vpn" | "wireguard" => "network-vpn-symbolic",
                        _ => "network-wired-symbolic",
                    };
                    
                    // Get IP address for this device
                    if let Ok(ip_output) = Command::new("ip")
                        .env("LC_ALL", "C")
                        .args(["-4", "-o", "addr", "show", device])
                        .output()
                    {
                        if ip_output.status.success() {
                            let ip_stdout = String::from_utf8_lossy(&ip_output.stdout);
                            for ip_line in ip_stdout.lines() {
                                // Format: 2: eth0    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0
                                if let Some(inet_pos) = ip_line.find("inet ") {
                                    let after_inet = &ip_line[inet_pos + 5..];
                                    if let Some(space_pos) = after_inet.find(' ') {
                                        info.ip_address = after_inet[..space_pos].to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Found a valid connection, break
                    break;
                }
            }
        }
    }
    
    // Get default gateway
    if let Ok(output) = Command::new("ip")
        .env("LC_ALL", "C")
        .args(["route", "show", "default"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Format: default via 192.168.1.1 dev eth0 proto dhcp metric 100
            for line in stdout.lines() {
                if line.starts_with("default via ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        info.gateway = parts[2].to_string();
                        break;
                    }
                }
            }
        }
    }
    
    // Get DNS servers from resolv.conf or nmcli
    if let Ok(output) = Command::new("nmcli")
        .env("LC_ALL", "C")
        .args(["-t", "-f", "IP4.DNS", "device", "show"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut dns_servers: Vec<String> = Vec::new();
            for line in stdout.lines() {
                if line.starts_with("IP4.DNS") {
                    if let Some(dns) = line.split(':').nth(1) {
                        if !dns.is_empty() && !dns_servers.contains(&dns.to_string()) {
                            dns_servers.push(dns.to_string());
                        }
                    }
                }
            }
            if !dns_servers.is_empty() {
                info.dns_servers = dns_servers.join(", ");
            }
        }
    }
    
    // Fallback to resolv.conf if nmcli didn't provide DNS
    if info.dns_servers == "—" {
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            let mut dns_servers: Vec<String> = Vec::new();
            for line in content.lines() {
                if line.starts_with("nameserver ") {
                    if let Some(dns) = line.split_whitespace().nth(1) {
                        if !dns_servers.contains(&dns.to_string()) {
                            dns_servers.push(dns.to_string());
                        }
                    }
                }
            }
            if !dns_servers.is_empty() {
                info.dns_servers = dns_servers.join(", ");
            }
        }
    }
    
    info
}
