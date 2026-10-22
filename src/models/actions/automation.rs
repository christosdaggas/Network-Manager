// Network Manager - Automation Actions
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Automation actions (scripts and program execution).
//!
//! These actions run external scripts or programs as part of profile activation.
//! **Security note**: Script execution requires explicit user consent and logging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Script execution mode.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScriptMode {
    /// Wait for script to complete.
    #[default]
    Wait,
    /// Run in background, don't wait.
    Background,
    /// Run with timeout.
    Timeout {
        /// Timeout in seconds.
        seconds: u32,
    },
}

/// Program execution mode.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProgramMode {
    /// Run in foreground, wait for exit.
    #[default]
    Foreground,
    /// Run in background.
    Background,
    /// Run detached (survives profile manager).
    Detached,
}

/// Automation actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AutomationAction {
    /// Run a script before profile actions.
    PreScript {
        /// Path to the script.
        path: PathBuf,
        /// Arguments to pass.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// Environment variables.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        /// Execution mode.
        #[serde(default)]
        mode: ScriptMode,
        /// Working directory.
        #[serde(skip_serializing_if = "Option::is_none")]
        working_dir: Option<PathBuf>,
        /// Continue on script failure.
        #[serde(default)]
        continue_on_error: bool,
    },

    /// Run a script after profile actions.
    PostScript {
        /// Path to the script.
        path: PathBuf,
        /// Arguments to pass.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// Environment variables.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        /// Execution mode.
        #[serde(default)]
        mode: ScriptMode,
        /// Working directory.
        #[serde(skip_serializing_if = "Option::is_none")]
        working_dir: Option<PathBuf>,
        /// Continue on script failure.
        #[serde(default)]
        continue_on_error: bool,
    },

    /// Run an external program.
    RunProgram {
        /// Program path or command name.
        program: String,
        /// Arguments.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// Environment variables.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        /// Execution mode.
        #[serde(default)]
        mode: ProgramMode,
        /// Working directory.
        #[serde(skip_serializing_if = "Option::is_none")]
        working_dir: Option<PathBuf>,
    },

    /// Kill a running program (by name or PID file).
    KillProgram {
        /// Program name to kill.
        program_name: String,
        /// Signal to send (default: SIGTERM).
        #[serde(default)]
        signal: KillSignal,
    },

    /// Wait for a condition before proceeding.
    WaitFor {
        /// What to wait for.
        condition: WaitCondition,
        /// Timeout in seconds.
        timeout_seconds: u32,
    },

    /// Send a desktop notification.
    Notification {
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
        /// Icon name.
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },
}

/// Kill signal type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum KillSignal {
    #[default]
    Sigterm,
    Sigkill,
    Sighup,
    Sigint,
}

/// Wait condition for automation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WaitCondition {
    /// Wait for network connectivity.
    NetworkUp,
    /// Wait for specific host to be reachable.
    HostReachable { host: String },
    /// Wait for a file to exist.
    FileExists { path: PathBuf },
    /// Wait for a fixed duration.
    Duration { seconds: u32 },
}

impl AutomationAction {
    /// Get a short name for the action.
    pub fn name(&self) -> String {
        match self {
            Self::PreScript { path, .. } => format!("Pre-script: {}", path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()),
            Self::PostScript { path, .. } => format!("Post-script: {}", path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()),
            Self::RunProgram { program, .. } => format!("Run: {}", program),
            Self::KillProgram { program_name, .. } => format!("Kill: {}", program_name),
            Self::WaitFor { .. } => "Wait Condition".to_string(),
            Self::Notification { title, .. } => format!("Notify: {}", title),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::PreScript { path, .. } => {
                format!("Pre-script: {}", path.display())
            }
            Self::PostScript { path, .. } => {
                format!("Post-script: {}", path.display())
            }
            Self::RunProgram { program, .. } => {
                format!("Run: {}", program)
            }
            Self::KillProgram { program_name, .. } => {
                format!("Kill: {}", program_name)
            }
            Self::WaitFor { condition, timeout_seconds } => {
                format!("Wait for {:?} ({}s timeout)", condition, timeout_seconds)
            }
            Self::Notification { title, .. } => {
                format!("Notify: {}", title)
            }
        }
    }

    /// Get the icon name for this action.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::PreScript { .. } | Self::PostScript { .. } => "utilities-terminal-symbolic",
            Self::RunProgram { .. } => "system-run-symbolic",
            Self::KillProgram { .. } => "process-stop-symbolic",
            Self::WaitFor { .. } => "hourglass-symbolic",
            Self::Notification { .. } => "dialog-information-symbolic",
        }
    }

    /// Check if this action requires elevated privileges.
    pub fn requires_privilege(&self) -> bool {
        match self {
            // Scripts run with daemon privileges
            Self::PreScript { .. } | Self::PostScript { .. } => true,
            // Kill may require privilege depending on target
            Self::KillProgram { .. } => true,
            // Run program, wait, and notifications don't require privilege
            Self::RunProgram { .. } | Self::WaitFor { .. } | Self::Notification { .. } => false,
        }
    }

    /// Check if this is a script action.
    pub fn is_script(&self) -> bool {
        matches!(self, Self::PreScript { .. } | Self::PostScript { .. })
    }
}
