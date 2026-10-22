// Network Manager - Profile Data Model
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Profile data model and serialization.
//!
//! A Profile represents a complete configuration state that can be applied
//! to the system. It contains:
//! - Metadata (name, group, timestamps)
//! - Network actions (IP, DNS, routes, VPN, etc.)
//! - System actions (hostname, hosts, proxy, firewall, etc.)
//! - Automation (pre/post scripts, program execution)
//! - Auto-switch conditions (rules for automatic activation)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::actions::{Action, AutomationAction, NetworkAction, SystemAction};
use super::rules::RuleSet;
use super::schema::SchemaVersion;

/// Profile status indicating the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// Profile is not active.
    #[default]
    Inactive,
    /// Profile is currently active.
    Active,
    /// Profile is being applied.
    Applying,
    /// Profile application failed.
    Error,
}

impl ProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Applying => "applying",
            Self::Error => "error",
        }
    }
}

/// Profile group for organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProfileGroup {
    /// Group name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional icon name (from GNOME icon set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl ProfileGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            icon: None,
        }
    }
}

/// Profile metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// Unique profile identifier.
    pub id: Uuid,
    /// Profile name (user-visible).
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Profile group (for organization).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<ProfileGroup>,
    /// Optional icon name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
    /// Last applied timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_applied_at: Option<DateTime<Utc>>,
    /// Schema version for migration support.
    #[serde(default)]
    pub schema_version: SchemaVersion,
}

impl ProfileMetadata {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            group: None,
            icon: None,
            created_at: now,
            updated_at: now,
            last_applied_at: None,
            schema_version: SchemaVersion::current(),
        }
    }
}

/// A complete network/system profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile metadata.
    pub metadata: ProfileMetadata,

    /// Network-related actions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network_actions: Vec<NetworkAction>,

    /// System-related actions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub system_actions: Vec<SystemAction>,

    /// Automation actions (scripts, programs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub automation_actions: Vec<AutomationAction>,

    /// Auto-switch conditions (rule engine).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_switch_rules: Option<RuleSet>,

    /// Whether this profile is approved for non-admin activation.
    #[serde(default)]
    pub approved_for_users: bool,

    /// Current status (not persisted, set at runtime).
    #[serde(skip)]
    pub status: ProfileStatus,
}

impl Profile {
    /// Create a new empty profile with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: ProfileMetadata::new(name),
            network_actions: Vec::new(),
            system_actions: Vec::new(),
            automation_actions: Vec::new(),
            auto_switch_rules: None,
            approved_for_users: false,
            status: ProfileStatus::Inactive,
        }
    }

    /// Get the profile ID.
    pub fn id(&self) -> Uuid {
        self.metadata.id
    }

    /// Get the profile name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get all actions as a unified iterator.
    pub fn all_actions(&self) -> impl Iterator<Item = Action> + '_ {
        self.network_actions
            .iter()
            .cloned()
            .map(Action::Network)
            .chain(self.system_actions.iter().cloned().map(Action::System))
            .chain(
                self.automation_actions
                    .iter()
                    .cloned()
                    .map(Action::Automation),
            )
    }

    /// Count total number of actions.
    pub fn action_count(&self) -> usize {
        self.network_actions.len()
            + self.system_actions.len()
            + self.automation_actions.len()
    }

    /// Check if the profile has any actions.
    pub fn has_actions(&self) -> bool {
        self.action_count() > 0
    }

    /// Check if the profile has auto-switch rules.
    pub fn has_auto_switch(&self) -> bool {
        self.auto_switch_rules
            .as_ref()
            .map(|r| !r.conditions.is_empty())
            .unwrap_or(false)
    }

    /// Check if this profile requires privileged operations.
    pub fn requires_privilege(&self) -> bool {
        // Most actions require privilege
        !self.network_actions.is_empty()
            || !self.system_actions.is_empty()
            || self.automation_actions.iter().any(|a| matches!(a, AutomationAction::PreScript { .. } | AutomationAction::PostScript { .. }))
    }

    /// Update the last applied timestamp.
    pub fn mark_applied(&mut self) {
        self.metadata.last_applied_at = Some(Utc::now());
        self.status = ProfileStatus::Active;
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new("New Profile")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("Test Profile");
        assert_eq!(profile.name(), "Test Profile");
        assert_eq!(profile.action_count(), 0);
        assert!(!profile.has_actions());
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile::new("Test Profile");
        let toml = profile.to_toml().expect("Profile should serialize to TOML");
        let restored: Profile = Profile::from_toml(&toml).expect("Profile should deserialize from TOML");
        assert_eq!(restored.name(), profile.name());
    }
    
    #[test]
    fn test_profile_with_description() {
        let mut profile = Profile::new("Work Profile");
        profile.metadata.description = Some("Configuration for office network".to_string());
        assert_eq!(profile.metadata.description.as_deref(), Some("Configuration for office network"));
    }
    
    #[test]
    fn test_profile_clone_preserves_data() {
        let profile = Profile::new("Original");
        let cloned = profile.clone();
        assert_eq!(cloned.name(), profile.name());
        assert_eq!(cloned.id(), profile.id());
    }
}
