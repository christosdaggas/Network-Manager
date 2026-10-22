// Network Manager - Shared Library
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! # Network Manager Shared Library
//!
//! This crate contains shared types, models, and logic used by the GUI
//! application:
//!
//! - **Profile**: Network/system configuration profiles
//! - **Actions**: Declarative actions for network and system changes
//! - **Adapter**: Per-adapter network configuration
//! - **Rules**: Auto-switch condition rules (rule engine)
//! - **Execution**: Result types for profile application
//! - **Error**: Shared error types
//!
//! ## Design Principles
//!
//! 1. **Idempotent**: Actions can be applied multiple times safely
//! 2. **Declarative**: Actions describe desired state, not steps
//! 3. **Reversible**: Prior state is captured for rollback where possible
//! 4. **Serializable**: All types serialize to TOML for persistence

pub mod actions;
pub mod adapter;
pub mod config;
pub mod error;
pub mod profile;
pub mod result;
pub mod rules;
pub mod schema;
pub mod templates;
pub mod validation;

// Re-export main types for convenience
pub use actions::{AutomationAction, NetworkAction, SystemAction};
pub use actions::{InterfaceState, Ipv4Address, Ipv4Method};
pub use actions::{ProgramMode, ScriptMode};
pub use actions::{ProxyConfig, ProxyMode};
pub use adapter::{AdapterInfo, AdapterType};
// Adapter config types available via adapter:: when needed
#[allow(unused_imports)]
pub use config::{
    AppConfig, HotkeyEntry, SandboxMode, ScheduleEntry, ThemePreference, WatchdogAction,
    WatchdogConfig,
};
#[allow(unused_imports)]
pub use error::{Error, Result};
#[allow(unused_imports)]
pub use profile::{Profile, ProfileGroup};
#[allow(unused_imports)]
pub use result::ExecutionResult;
#[allow(unused_imports)]
pub use templates::ProfileTemplate;
// Rules types available via rules:: when needed
// SchemaVersion available via schema:: when needed

/// Crate version for schema compatibility checking.
#[allow(dead_code)]
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application ID (matches desktop/D-Bus identifiers).
pub const APP_ID: &str = "com.chrisdaggas.network-manager";

/// Configuration directory name (under XDG_CONFIG_HOME).
pub const CONFIG_DIR_NAME: &str = "network-manager";

/// Data directory name (under XDG_DATA_HOME).
#[allow(dead_code)]
pub const DATA_DIR_NAME: &str = "network-manager";
