// Network Manager - Profiles Page
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Profiles listing page with search and grouping.

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;
use adw::subclass::prelude::*;
use std::cell::RefCell;

use crate::models::profile::ProfileStatus;
use crate::tr;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ProfilesPage {
        pub search_entry: RefCell<Option<gtk::SearchEntry>>,
        pub profiles_list: RefCell<Option<gtk::ListBox>>,
        pub empty_state: RefCell<Option<adw::StatusPage>>,
        pub stack: RefCell<Option<gtk::Stack>>,
        pub active_profile_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProfilesPage {
        const NAME: &'static str = "CdNetworkManagerProfilesPage";
        type Type = super::ProfilesPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ProfilesPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ProfilesPage {}
    impl BoxImpl for ProfilesPage {}
}

glib::wrapper! {
    pub struct ProfilesPage(ObjectSubclass<imp::ProfilesPage>)
        @extends gtk::Widget, gtk::Box;
}

impl ProfilesPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 0)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Toolbar with search and create button
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        toolbar.set_margin_top(12);
        toolbar.set_margin_bottom(12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some(&tr("Search profiles...")));
        search_entry.set_hexpand(true);
        toolbar.append(&search_entry);

        let create_btn = gtk::Button::with_label(&tr("Create Profile"));
        create_btn.add_css_class("suggested-action");
        create_btn.set_icon_name("list-add-symbolic");
        create_btn.set_action_name(Some("win.new-profile"));
        toolbar.append(&create_btn);
        
        // Import/Export buttons
        let import_btn = gtk::Button::new();
        import_btn.set_icon_name("document-open-symbolic");
        import_btn.set_tooltip_text(Some(&tr("Import profiles from file")));
        import_btn.set_action_name(Some("win.import-profiles"));
        toolbar.append(&import_btn);
        
        let export_btn = gtk::Button::new();
        export_btn.set_icon_name("document-save-symbolic");
        export_btn.set_tooltip_text(Some(&tr("Export profiles to file")));
        export_btn.set_action_name(Some("win.export-profiles"));
        toolbar.append(&export_btn);

        self.append(&toolbar);

        // Stack for list vs empty state
        let stack = gtk::Stack::new();
        stack.set_vexpand(true);

        // Profiles list
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let profiles_list = gtk::ListBox::new();
        profiles_list.set_selection_mode(gtk::SelectionMode::Single);
        profiles_list.add_css_class("profiles-list");
        profiles_list.set_margin_start(12);
        profiles_list.set_margin_end(12);
        profiles_list.set_margin_bottom(12);
        
        // Placeholder: would be populated from daemon
        profiles_list.set_placeholder(Some(&self.create_loading_placeholder()));

        // Filter rows by the search text (name or group)
        let search_weak = search_entry.downgrade();
        profiles_list.set_filter_func(move |row| {
            let Some(search) = search_weak.upgrade() else { return true };
            let query = search.text().to_lowercase();
            if query.is_empty() {
                return true;
            }
            row.downcast_ref::<adw::ActionRow>()
                .map(|r| {
                    r.title().to_lowercase().contains(&query)
                        || r.subtitle().is_some_and(|s| s.to_lowercase().contains(&query))
                })
                .unwrap_or(true)
        });
        let list_weak = profiles_list.downgrade();
        search_entry.connect_search_changed(move |_| {
            if let Some(list) = list_weak.upgrade() {
                list.invalidate_filter();
            }
        });

        scroll.set_child(Some(&profiles_list));
        stack.add_named(&scroll, Some("list"));

        // Empty state
        let empty_state = adw::StatusPage::new();
        empty_state.set_icon_name(Some("contact-new-symbolic"));
        empty_state.set_title(&tr("No Profiles Yet"));
        empty_state.set_description(Some(&tr("Create your first network profile to get started")));
        empty_state.add_css_class("compact");

        let empty_create_btn = gtk::Button::with_label(&tr("Create Profile"));
        empty_create_btn.add_css_class("suggested-action");
        empty_create_btn.add_css_class("pill");
        empty_create_btn.set_halign(gtk::Align::Center);
        empty_create_btn.set_action_name(Some("win.new-profile"));
        empty_state.set_child(Some(&empty_create_btn));

        stack.add_named(&empty_state, Some("empty"));

        // Show empty state by default (would switch based on data)
        stack.set_visible_child_name("empty");

        self.append(&stack);

        // Store references
        *imp.search_entry.borrow_mut() = Some(search_entry);
        *imp.profiles_list.borrow_mut() = Some(profiles_list);
        *imp.empty_state.borrow_mut() = Some(empty_state);
        *imp.stack.borrow_mut() = Some(stack);
    }

    fn create_loading_placeholder(&self) -> gtk::Box {
        let placeholder = gtk::Box::new(gtk::Orientation::Vertical, 12);
        placeholder.set_valign(gtk::Align::Center);
        placeholder.set_margin_top(48);
        placeholder.set_margin_bottom(48);

        let spinner = gtk::Spinner::new();
        spinner.set_spinning(true);
        spinner.set_size_request(32, 32);
        placeholder.append(&spinner);

        let label = gtk::Label::new(Some(&tr("Loading profiles...")));
        label.add_css_class("dim-label");
        placeholder.append(&label);

        placeholder
    }

    /// Create a profile row widget.
    pub fn create_profile_row(
        &self,
        profile_id: &str,
        name: &str,
        group: Option<&str>,
        status: ProfileStatus,
        _last_applied: Option<&str>,
        is_active: bool,
    ) -> adw::ActionRow {
        let row = adw::ActionRow::new();
        // Names and groups are user text, not Pango markup
        row.set_use_markup(false);
        row.set_title(name);
        
        if let Some(g) = group {
            row.set_subtitle(g);
        }
        
        row.add_css_class("profile-row");
        
        // Store profile ID in row for later retrieval
        row.set_widget_name(profile_id);

        // Status pill: translated text, CSS class from the untranslated status
        let status_text = if is_active {
            tr("Active")
        } else {
            match status {
                ProfileStatus::Active => tr("Active"),
                ProfileStatus::Inactive => tr("Inactive"),
                ProfileStatus::Applying => tr("Applying"),
                ProfileStatus::Error => tr("Error"),
            }
        };
        let status_pill = gtk::Label::new(Some(&status_text));
        status_pill.add_css_class("status-pill");
        if is_active {
            status_pill.add_css_class("status-active");
        } else {
            status_pill.add_css_class(&format!("status-{}", status.as_str()));
        }
        status_pill.set_valign(gtk::Align::Center);
        row.add_suffix(&status_pill);

        // Button box for actions
        let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        btn_box.set_valign(gtk::Align::Center);

        // Apply/Activate button
        let apply_btn = gtk::Button::from_icon_name("media-playback-start-symbolic");
        apply_btn.set_tooltip_text(Some(&tr("Apply Profile (Enter)")));
        apply_btn.add_css_class("flat");
        if is_active {
            apply_btn.set_sensitive(false);
            apply_btn.set_tooltip_text(Some(&tr("Already active")));
        }
        
        // Connect apply button
        let profile_id_clone = profile_id.to_string();
        let page_weak = self.downgrade();
        // The window confirms and applies; the list is rebuilt once it is done,
        // so a cancelled confirmation leaves this button usable
        apply_btn.connect_clicked(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.activate_profile(&profile_id_clone);
            }
        });
        btn_box.append(&apply_btn);

        // Edit button
        let edit_btn = gtk::Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some(&tr("Edit Profile")));
        edit_btn.add_css_class("flat");
        
        let profile_id_clone = profile_id.to_string();
        let page_weak = self.downgrade();
        edit_btn.connect_clicked(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.emit_edit_profile(&profile_id_clone);
            }
        });
        btn_box.append(&edit_btn);
        
        // Duplicate button
        let duplicate_btn = gtk::Button::from_icon_name("edit-copy-symbolic");
        duplicate_btn.set_tooltip_text(Some(&tr("Duplicate Profile")));
        duplicate_btn.add_css_class("flat");
        
        let profile_id_clone = profile_id.to_string();
        duplicate_btn.connect_clicked(move |btn| {
            if let Some(root) = btn.root() {
                let _ = root.activate_action("win.duplicate-profile", Some(&profile_id_clone.to_variant()));
            }
        });
        btn_box.append(&duplicate_btn);

        // Delete button
        let delete_btn = gtk::Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_tooltip_text(Some(&tr("Delete Profile")));
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("destructive-action");
        
        let profile_id_clone = profile_id.to_string();
        let page_weak = self.downgrade();
        delete_btn.connect_clicked(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.request_delete_profile(&profile_id_clone);
            }
        });
        btn_box.append(&delete_btn);

        row.add_suffix(&btn_box);

        // Make row activatable (clicking anywhere applies the profile)
        row.set_activatable(true);
        let profile_id_clone = profile_id.to_string();
        let page_weak = self.downgrade();
        row.connect_activated(move |_| {
            if let Some(page) = page_weak.upgrade() {
                page.activate_profile(&profile_id_clone);
            }
        });

        row
    }
    
    /// Activate a profile
    fn activate_profile(&self, profile_id: &str) {
        // Trigger the window action to apply the profile (which does the actual work)
        if let Some(root) = self.root() {
            let _ = root.activate_action("win.apply-profile", Some(&profile_id.to_variant()));
        }
    }
    
    /// Refresh active status display
    #[allow(dead_code)]
    fn refresh_active_status(&self, active_id: &str) {
        let imp = self.imp();
        
        if let Some(list) = imp.profiles_list.borrow().as_ref() {
            let mut child = list.first_child();
            while let Some(row_widget) = child {
                if let Ok(row) = row_widget.clone().downcast::<adw::ActionRow>() {
                    let row_id = row.widget_name().to_string();
                    let is_active = row_id == active_id;
                    
                    // Update styling - find and update status pill
                    if let Some(_suffix) = row.first_child() {
                        // Navigate through suffixes to find status pill
                        Self::update_row_active_status(&row, is_active);
                    }
                }
                child = row_widget.next_sibling();
            }
        }
    }
    
    #[allow(dead_code)]
    fn update_row_active_status(row: &adw::ActionRow, is_active: bool) {
        // Add/remove active class from the row itself
        if is_active {
            row.add_css_class("active-profile");
        } else {
            row.remove_css_class("active-profile");
        }
    }
    
    /// Emit signal to edit profile (handled by main window)
    fn emit_edit_profile(&self, profile_id: &str) {
        // Activate the window action for editing
        if let Some(root) = self.root() {
            let _ = root.activate_action("win.edit-profile", Some(&profile_id.to_variant()));
        }
    }
    
    /// Request profile deletion with confirmation
    fn request_delete_profile(&self, profile_id: &str) {
        // Activate the window action for deletion
        if let Some(root) = self.root() {
            let _ = root.activate_action("win.delete-profile", Some(&profile_id.to_variant()));
        }
    }
    
    /// Set the currently active profile ID
    pub fn set_active_profile(&self, profile_id: Option<&str>) {
        let imp = self.imp();
        *imp.active_profile_id.borrow_mut() = profile_id.map(|s| s.to_string());
    }
    
    /// Get the currently active profile ID
    pub fn active_profile_id(&self) -> Option<String> {
        self.imp().active_profile_id.borrow().clone()
    }

    /// Update the profiles list.
    pub fn update_profiles(&self, profiles: Vec<crate::models::Profile>) {
        let imp = self.imp();
        let active_id = imp.active_profile_id.borrow().clone();

        if let Some(list) = imp.profiles_list.borrow().as_ref() {
            // Clear existing rows
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }

            if profiles.is_empty() {
                if let Some(stack) = imp.stack.borrow().as_ref() {
                    stack.set_visible_child_name("empty");
                }
            } else {
                for profile in &profiles {
                    let profile_id_str = profile.id().to_string();
                    let is_active = active_id.as_ref().map(|id| id == &profile_id_str).unwrap_or(false);
                    let row = self.create_profile_row(
                        &profile_id_str,
                        profile.name(),
                        profile.metadata.group.as_ref().map(|g| g.name.as_str()),
                        profile.status,
                        profile.metadata.last_applied_at.map(|_| "Recently").as_deref(),
                        is_active,
                    );
                    list.append(&row);
                }

                if let Some(stack) = imp.stack.borrow().as_ref() {
                    stack.set_visible_child_name("list");
                }
            }
        }
    }
}

impl Default for ProfilesPage {
    fn default() -> Self {
        Self::new()
    }
}
