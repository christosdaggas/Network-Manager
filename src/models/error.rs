// Network Manager - Error Types
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Shared error types for the Network Manager application.

use thiserror::Error;

/// Result type alias for Network Manager operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Network Manager operations.
#[derive(Debug, Error)]
pub enum Error {
    // ========================================
    // Profile Errors
    // ========================================
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileAlreadyExists(String),

    #[error("Invalid profile: {0}")]
    InvalidProfile(String),

    #[error("Profile schema version mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    // ========================================
    // Action Errors
    // ========================================
    #[error("Action failed: {action} - {reason}")]
    ActionFailed { action: String, reason: String },

    #[error("Action not supported on this system: {0}")]
    ActionNotSupported(String),

    #[error("Action requires privilege: {0}")]
    PrivilegeRequired(String),

    #[error("Action timed out: {0}")]
    ActionTimeout(String),

    // ========================================
    // Validation Errors
    // ========================================
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid DNS server: {0}")]
    InvalidDnsServer(String),

    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("Invalid MAC address: {0}")]
    InvalidMacAddress(String),

    #[error("Invalid route: {0}")]
    InvalidRoute(String),

    // ========================================
    // D-Bus Errors
    // ========================================
    #[error("D-Bus error: {0}")]
    Dbus(String),

    #[error("NetworkManager D-Bus error: {0}")]
    NetworkManagerDbus(String),

    #[error("Daemon not running")]
    DaemonNotRunning,

    #[error("D-Bus connection failed: {0}")]
    DbusConnectionFailed(String),

    // ========================================
    // Polkit Errors
    // ========================================
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    // ========================================
    // Storage Errors
    // ========================================
    #[error("Failed to read configuration: {0}")]
    ConfigReadFailed(String),

    #[error("Failed to write configuration: {0}")]
    ConfigWriteFailed(String),

    #[error("Failed to parse configuration: {0}")]
    ConfigParseFailed(String),

    // ========================================
    // Script Errors
    // ========================================
    #[error("Script execution failed: {script} - {reason}")]
    ScriptFailed { script: String, reason: String },

    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    #[error("Script not executable: {0}")]
    ScriptNotExecutable(String),

    // ========================================
    // System Errors
    // ========================================
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("System error: {0}")]
    System(String),

    // ========================================
    // Rule Engine Errors
    // ========================================
    #[error("Rule evaluation failed: {0}")]
    RuleEvaluationFailed(String),

    #[error("Invalid time window: {0}")]
    InvalidTimeWindow(String),

    // ========================================
    // Generic Errors
    // ========================================
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a new action failed error.
    pub fn action_failed(action: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ActionFailed {
            action: action.into(),
            reason: reason.into(),
        }
    }

    /// Create a new script failed error.
    pub fn script_failed(script: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ScriptFailed {
            script: script.into(),
            reason: reason.into(),
        }
    }

    /// Check if this error indicates the daemon is not running.
    pub fn is_daemon_not_running(&self) -> bool {
        matches!(self, Self::DaemonNotRunning | Self::DbusConnectionFailed(_))
    }

    /// Check if this error is an authorization error.
    pub fn is_authorization_error(&self) -> bool {
        matches!(
            self,
            Self::AuthorizationDenied(_) | Self::AuthorizationFailed(_) | Self::PrivilegeRequired(_)
        )
    }
}

// Convert from zbus errors
impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::Dbus(err.to_string())
    }
}

// Convert from toml parse errors
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::ConfigParseFailed(err.to_string())
    }
}

// Convert from toml serialize errors
impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::ConfigWriteFailed(err.to_string())
    }
}

// Convert from serde_json errors
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ConfigParseFailed(err.to_string())
    }
}
