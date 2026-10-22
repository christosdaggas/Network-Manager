// Network Manager - Application Configuration
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Application configuration model.

use serde::{Deserialize, Serialize};

/// Theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Follow system theme.
    #[default]
    System,
    /// Force light theme.
    Light,
    /// Force dark theme.
    Dark,
}

impl ThemePreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// Script sandboxing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SandboxMode {
    /// No sandboxing (scripts run with full permissions).
    #[default]
    None,
    /// Use bubblewrap for sandboxing.
    Bubblewrap,
    /// Use firejail for sandboxing.
    Firejail,
}

impl SandboxMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bubblewrap => "bubblewrap",
            Self::Firejail => "firejail",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "No Sandboxing",
            Self::Bubblewrap => "Bubblewrap",
            Self::Firejail => "Firejail",
        }
    }
}

/// Connection watchdog configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchdogConfig {
    /// Enable connection watchdog.
    #[serde(default)]
    pub enabled: bool,

    /// Check interval in seconds.
    #[serde(default = "default_watchdog_interval")]
    pub check_interval_secs: u32,

    /// Target to ping for connectivity check.
    #[serde(default = "default_watchdog_target")]
    pub ping_target: String,

    /// Number of failed checks before taking action.
    #[serde(default = "default_watchdog_threshold")]
    pub failure_threshold: u32,

    /// Action to take on connection failure.
    #[serde(default)]
    pub failure_action: WatchdogAction,

    /// Profile to switch to on failure (if action is SwitchProfile).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_profile_id: Option<String>,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            check_interval_secs: default_watchdog_interval(),
            ping_target: default_watchdog_target(),
            failure_threshold: default_watchdog_threshold(),
            failure_action: WatchdogAction::default(),
            fallback_profile_id: None,
        }
    }
}

/// Action to take when watchdog detects connection failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatchdogAction {
    /// Just notify the user.
    #[default]
    Notify,
    /// Attempt to reconnect.
    Reconnect,
    /// Switch to a fallback profile.
    SwitchProfile,
    /// Restart NetworkManager.
    RestartNetworkManager,
}

fn default_watchdog_interval() -> u32 {
    30
}

fn default_watchdog_target() -> String {
    "8.8.8.8".to_string()
}

fn default_watchdog_threshold() -> u32 {
    3
}

/// Profile scheduling entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Unique ID for this schedule.
    pub id: String,

    /// Profile ID to activate.
    pub profile_id: String,

    /// Cron-like expression (minute hour day-of-month month day-of-week).
    pub cron_expression: String,

    /// Whether this schedule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// One-shot (run once then disable).
    #[serde(default)]
    pub one_shot: bool,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Hotkey entry for profile activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyEntry {
    /// Unique ID for this hotkey.
    pub id: String,

    /// Profile ID to activate.
    pub profile_id: String,

    /// Profile name (for display).
    pub profile_name: String,

    /// Key modifiers (Ctrl, Alt, Shift, Super).
    pub modifiers: Vec<String>,

    /// The key (e.g., "1", "F1", "a").
    pub key: String,

    /// Whether this hotkey is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl HotkeyEntry {
    /// Get the shortcut string (e.g., "Ctrl+Alt+1").
    pub fn shortcut_string(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", self.modifiers.join("+"), self.key)
        }
    }
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Theme preference.
    #[serde(default)]
    pub theme: ThemePreference,

    /// Auto-switch check interval in seconds.
    #[serde(default = "default_auto_switch_interval")]
    pub auto_switch_interval_secs: u32,

    /// Enable auto-switch globally.
    #[serde(default = "default_true")]
    pub auto_switch_enabled: bool,

    /// Allow scripts to run during auto-switch.
    #[serde(default)]
    pub auto_switch_allow_scripts: bool,

    /// Show desktop notifications.
    #[serde(default = "default_true")]
    pub show_notifications: bool,

    /// Minimize to system tray.
    #[serde(default)]
    pub minimize_to_tray: bool,

    /// Start on login (autostart).
    #[serde(default)]
    pub autostart_on_login: bool,

    /// Show system tray icon.
    #[serde(default = "default_true")]
    pub show_tray_icon: bool,

    /// Start minimized.
    #[serde(default)]
    pub start_minimized: bool,

    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Maximum log entries to keep.
    #[serde(default = "default_max_log_entries")]
    pub max_log_entries: usize,

    /// Window width.
    #[serde(default = "default_window_width")]
    pub window_width: i32,

    /// Window height.
    #[serde(default = "default_window_height")]
    pub window_height: i32,

    /// Window X position.
    #[serde(default)]
    pub window_x: Option<i32>,

    /// Window Y position.
    #[serde(default)]
    pub window_y: Option<i32>,

    /// Window maximized state.
    #[serde(default)]
    pub window_maximized: bool,

    /// Last active profile ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_profile_id: Option<String>,

    /// Confirm before switching profiles.
    #[serde(default = "default_true")]
    pub confirm_profile_switch: bool,

    /// Connection watchdog configuration.
    #[serde(default)]
    pub watchdog: WatchdogConfig,

    /// Scheduled profile activations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedules: Vec<ScheduleEntry>,

    /// Script sandboxing mode.
    #[serde(default)]
    pub sandbox_mode: SandboxMode,

    /// Encrypt sensitive profile data.
    #[serde(default)]
    pub encrypt_profiles: bool,

    /// Encryption passphrase (never persisted to disk — runtime only).
    #[serde(skip)]
    pub encryption_key: Option<String>,

    /// Ciphertext of a known string, used to verify an entered passphrase
    /// without touching the profile cache. Safe to persist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_check: Option<String>,

    /// Enable scheduled profile activation.
    #[serde(default)]
    pub scheduling_enabled: bool,

    /// Keyboard hotkeys for profiles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hotkeys: Vec<HotkeyEntry>,

    /// Enable profile hotkeys globally.
    #[serde(default)]
    pub hotkeys_enabled: bool,
}

fn default_auto_switch_interval() -> u32 {
    30
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_entries() -> usize {
    10000
}

fn default_window_width() -> i32 {
    1060
}

fn default_window_height() -> i32 {
    680
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            auto_switch_interval_secs: default_auto_switch_interval(),
            auto_switch_enabled: true,
            auto_switch_allow_scripts: false,
            show_notifications: true,
            minimize_to_tray: false,
            autostart_on_login: false,
            show_tray_icon: true,
            start_minimized: false,
            log_level: default_log_level(),
            max_log_entries: default_max_log_entries(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            window_x: None,
            window_y: None,
            window_maximized: false,
            last_active_profile_id: None,
            confirm_profile_switch: true,
            watchdog: WatchdogConfig::default(),
            schedules: Vec::new(),
            sandbox_mode: SandboxMode::None,
            encrypt_profiles: false,
            encryption_key: None,
            encryption_check: None,
            scheduling_enabled: false,
            hotkeys: Vec::new(),
            hotkeys_enabled: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from TOML file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, super::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to TOML file with restrictive permissions (0600).
    ///
    /// The file is replaced atomically, so a crash mid-write cannot leave a
    /// truncated config behind.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), super::Error> {
        let content = toml::to_string_pretty(self)?;
        crate::storage::write_private_atomic(path, content.as_bytes())?;
        Ok(())
    }
}
