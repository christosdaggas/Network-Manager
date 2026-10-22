// Network Manager - Action Types
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Declarative action types for network and system configuration.
//!
//! Actions are designed to be:
//! - **Idempotent**: Safe to apply multiple times
//! - **Declarative**: Describe desired state, not steps
//! - **Reversible**: Prior state can be captured and restored
//!
//! Each action type maps to specific system operations performed by the application.

mod network;
mod system;
mod automation;

pub use network::*;
pub use system::*;
pub use system::{ProxyConfig, ProxyMode, HostsEntry};
pub use automation::{AutomationAction, ScriptMode, ProgramMode};

use serde::{Deserialize, Serialize};

/// Unified action enum covering all action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Network configuration action.
    Network(NetworkAction),
    /// System configuration action.
    System(SystemAction),
    /// Automation action (scripts, programs).
    Automation(AutomationAction),
}

impl Action {
    /// Get a short name for the action.
    pub fn name(&self) -> String {
        match self {
            Action::Network(a) => a.name(),
            Action::System(a) => a.name(),
            Action::Automation(a) => a.name(),
        }
    }

    /// Get a human-readable description of the action.
    pub fn description(&self) -> String {
        match self {
            Action::Network(a) => a.description(),
            Action::System(a) => a.description(),
            Action::Automation(a) => a.description(),
        }
    }

    /// Get the action category name.
    pub fn category(&self) -> &'static str {
        match self {
            Action::Network(_) => "Network",
            Action::System(_) => "System",
            Action::Automation(_) => "Automation",
        }
    }

    /// Get the icon name for this action.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Action::Network(a) => a.icon_name(),
            Action::System(a) => a.icon_name(),
            Action::Automation(a) => a.icon_name(),
        }
    }

    /// Check if this action requires elevated privileges.
    pub fn requires_privilege(&self) -> bool {
        match self {
            Action::Network(_) => true,
            Action::System(a) => a.requires_privilege(),
            Action::Automation(a) => a.requires_privilege(),
        }
    }
}
