// Network Manager - Logs Page
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Application logs page showing profile activation history and diagnostics.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::subclass::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;
use tracing::error;

use crate::storage::DataStore;
use crate::tr;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct LogsPage {
        pub log_view: RefCell<Option<gtk::TextView>>,
        pub data_store: RefCell<Option<Arc<DataStore>>>,
        pub filter_dropdown: RefCell<Option<gtk::DropDown>>,
        pub search_entry: RefCell<Option<gtk::SearchEntry>>,
        pub current_filter: RefCell<String>,
        pub search_text: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogsPage {
        const NAME: &'static str = "CdNetworkManagerLogsPage";
        type Type = super::LogsPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for LogsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for LogsPage {}
    impl BoxImpl for LogsPage {}
}

glib::wrapper! {
    pub struct LogsPage(ObjectSubclass<imp::LogsPage>)
        @extends gtk::Widget, gtk::Box;
}

impl LogsPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 0)
            .build()
    }

    /// Initialize with data store.
    pub fn init_with_store(&self, store: Arc<DataStore>) {
        let imp = self.imp();
        *imp.data_store.borrow_mut() = Some(store.clone());
        *imp.current_filter.borrow_mut() = "All".to_string();
        *imp.search_text.borrow_mut() = String::new();
        
        // Load existing logs from disk
        self.load_logs_from_store();
    }

    /// Refresh logs from the data store (public method for external refresh).
    pub fn refresh_logs(&self) {
        self.load_logs_from_store();
    }

    /// Load logs from the data store into the UI, applying current filter.
    fn load_logs_from_store(&self) {
        let imp = self.imp();
        if let Some(store) = imp.data_store.borrow().as_ref() {
            let entries = store.logs();
            let filter = imp.current_filter.borrow().clone();
            let search = imp.search_text.borrow().to_lowercase();
            
            if let Some(log_view) = imp.log_view.borrow().as_ref() {
                let buffer = log_view.buffer();
                let mut text = String::new();
                
                for entry in entries {
                    // Apply level filter
                    let level_match = match filter.as_str() {
                        "All" => true,
                        "Info" => entry.level.to_uppercase() == "INFO",
                        "Warning" => entry.level.to_uppercase() == "WARNING" || entry.level.to_uppercase() == "WARN",
                        "Error" => entry.level.to_uppercase() == "ERROR",
                        _ => true,
                    };
                    
                    // Apply text search filter
                    let text_match = search.is_empty() || 
                        entry.message.to_lowercase().contains(&search) ||
                        entry.level.to_lowercase().contains(&search);
                    
                    if level_match && text_match {
                        text.push_str(&format!(
                            "[{}] {}: {}\n",
                            entry.timestamp, entry.level, entry.message
                        ));
                    }
                }
                buffer.set_text(&text);
            }
        }
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Toolbar
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);

        let title = gtk::Label::new(Some(&tr("Application Logs")));
        title.add_css_class("heading");
        title.set_halign(gtk::Align::Start);
        title.set_hexpand(true);
        toolbar.append(&title);

        // Search entry for filtering logs
        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some(&tr("Search logs...")));
        search_entry.set_hexpand(false);
        search_entry.set_width_chars(20);
        toolbar.append(&search_entry);

        // Filter dropdown. The selected position picks the filter, so the
        // translated labels are never compared with log levels.
        let filter_labels = [
            // TRANSLATORS: Log filter: show entries of every level
            tr("All"),
            // TRANSLATORS: Log level filter
            tr("Info"),
            // TRANSLATORS: Log level filter
            tr("Warning"),
            // TRANSLATORS: Log level filter
            tr("Error"),
        ];
        let filter_label_refs: Vec<&str> = filter_labels.iter().map(String::as_str).collect();
        let filter_dropdown = gtk::DropDown::from_strings(&filter_label_refs);
        filter_dropdown.set_selected(0);
        toolbar.append(&filter_dropdown);

        // Actions
        let clear_btn = gtk::Button::from_icon_name("edit-clear-all-symbolic");
        clear_btn.set_tooltip_text(Some(&tr("Clear Logs")));
        toolbar.append(&clear_btn);

        let export_btn = gtk::Button::from_icon_name("document-save-symbolic");
        export_btn.set_tooltip_text(Some(&tr("Export Logs")));
        toolbar.append(&export_btn);

        self.append(&toolbar);

        // Log view
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scroll.set_margin_start(12);
        scroll.set_margin_end(12);
        scroll.set_margin_bottom(12);

        let log_view = gtk::TextView::new();
        log_view.set_editable(false);
        log_view.set_cursor_visible(false);
        log_view.set_monospace(true);
        log_view.set_wrap_mode(gtk::WrapMode::WordChar);
        log_view.add_css_class("log-view");

        // Logs will be loaded from storage via init_with_store()

        scroll.set_child(Some(&log_view));
        self.append(&scroll);

        // Store references
        *imp.log_view.borrow_mut() = Some(log_view.clone());
        *imp.filter_dropdown.borrow_mut() = Some(filter_dropdown.clone());
        *imp.search_entry.borrow_mut() = Some(search_entry.clone());

        // Connect search entry
        let this_search = self.downgrade();
        search_entry.connect_search_changed(move |entry| {
            if let Some(page) = this_search.upgrade() {
                let text = entry.text().to_string();
                *page.imp().search_text.borrow_mut() = text;
                page.load_logs_from_store();
            }
        });

        // Connect filter dropdown
        let this_filter = self.downgrade();
        filter_dropdown.connect_selected_notify(move |dropdown| {
            if let Some(page) = this_filter.upgrade() {
                let selected = dropdown.selected();
                let filter = match selected {
                    0 => "All",
                    1 => "Info",
                    2 => "Warning",
                    3 => "Error",
                    _ => "All",
                };
                *page.imp().current_filter.borrow_mut() = filter.to_string();
                page.load_logs_from_store();
            }
        });

        // Connect clear button
        let this = self.downgrade();
        clear_btn.connect_clicked(move |_| {
            if let Some(page) = this.upgrade() {
                page.clear_logs();
            }
        });

        // Connect export button
        let this_export = self.downgrade();
        export_btn.connect_clicked(move |_| {
            if let Some(page) = this_export.upgrade() {
                page.export_logs();
            }
        });
    }

    /// Append a log entry and persist to storage.
    pub fn append_log(&self, level: &str, message: &str) {
        let imp = self.imp();
        
        // Persist to storage first
        if let Some(store) = imp.data_store.borrow().as_ref() {
            store.append_log(level, message);
        }
        
        // Update UI
        if let Some(log_view) = imp.log_view.borrow().as_ref() {
            let buffer = log_view.buffer();
            let mut end = buffer.end_iter();
            
            let now = chrono::Local::now();
            let entry = format!(
                "[{}] {}: {}\n",
                now.format("%Y-%m-%d %H:%M:%S"),
                level.to_uppercase(),
                message
            );
            
            buffer.insert(&mut end, &entry);
        }
    }

    /// Clear all logs from storage and UI.
    pub fn clear_logs(&self) {
        let imp = self.imp();
        
        // Clear from storage
        if let Some(store) = imp.data_store.borrow().as_ref() {
            store.clear_logs();
        }
        
        // Clear UI
        if let Some(log_view) = imp.log_view.borrow().as_ref() {
            let buffer = log_view.buffer();
            buffer.set_text("");
        }
    }

    /// Export logs to a file.
    fn export_logs(&self) {
        let imp = self.imp();
        let window = self.root().and_downcast::<gtk::Window>();
        
        // Get all logs from store
        let logs_text = if let Some(store) = imp.data_store.borrow().as_ref() {
            let entries = store.logs();
            let mut text = String::new();
            for entry in entries {
                text.push_str(&format!(
                    "[{}] {}: {}\n",
                    entry.timestamp, entry.level, entry.message
                ));
            }
            text
        } else {
            String::new()
        };
        
        if logs_text.is_empty() {
            return;
        }
        
        let dialog = gtk::FileDialog::builder()
            .title(tr("Export Logs"))
            .initial_name("network-manager-logs.txt")
            .build();
        
        let logs_for_save = logs_text.clone();
        dialog.save(
            window.as_ref(),
            None::<&gtk::gio::Cancellable>,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        if let Err(e) = std::fs::write(&path, &logs_for_save) {
                            error!("Failed to export logs: {}", e);
                        }
                    }
                }
            },
        );
    }
}

impl Default for LogsPage {
    fn default() -> Self {
        Self::new()
    }
}
