// Network Manager - System Actions
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! System configuration actions.
//!
//! These actions modify system settings outside of NetworkManager.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// Hosts file entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsEntry {
    /// IP address.
    pub ip: IpAddr,
    /// Hostnames.
    pub hostnames: Vec<String>,
    /// Comment for identification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Proxy configuration type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    /// No proxy.
    #[default]
    None,
    /// Manual proxy configuration.
    Manual,
    /// Automatic (PAC URL).
    Auto,
}

/// Proxy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy mode.
    pub mode: ProxyMode,
    /// HTTP proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_proxy: Option<String>,
    /// HTTPS proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_proxy: Option<String>,
    /// FTP proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ftp_proxy: Option<String>,
    /// SOCKS proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socks_proxy: Option<String>,
    /// No-proxy hosts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub no_proxy: Vec<String>,
    /// PAC URL (for auto mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pac_url: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::None,
            http_proxy: None,
            https_proxy: None,
            ftp_proxy: None,
            socks_proxy: None,
            no_proxy: Vec::new(),
            pac_url: None,
        }
    }
}

/// Firewall zone/profile (firewalld).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    /// Default zone name.
    pub default_zone: String,
    /// Interface-to-zone mappings.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub interface_zones: HashMap<String, String>,
}

/// System configuration actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SystemAction {
    /// Set system hostname.
    SetHostname {
        /// Static hostname.
        hostname: String,
        /// Pretty hostname (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        pretty_hostname: Option<String>,
    },

    /// Modify /etc/hosts entries.
    HostsEntries {
        /// Entries to add.
        entries: Vec<HostsEntry>,
        /// Whether to replace existing managed entries.
        #[serde(default)]
        replace_managed: bool,
    },

    /// Configure system proxy.
    ProxyConfig(ProxyConfig),

    /// Set firewall configuration (firewalld).
    FirewallConfig(FirewallConfig),

    /// Set default printer (CUPS).
    DefaultPrinter {
        /// Printer name.
        printer_name: String,
    },

    /// Set system timezone.
    SetTimezone {
        /// Timezone identifier (e.g., "America/New_York").
        timezone: String,
    },

    /// Set environment variables (for user session).
    EnvironmentVariables {
        /// Variables to set.
        variables: HashMap<String, String>,
    },
}

impl SystemAction {
    /// Get a short name for the action.
    pub fn name(&self) -> String {
        match self {
            Self::SetHostname { .. } => "Set Hostname".to_string(),
            Self::HostsEntries { .. } => "Hosts Entries".to_string(),
            Self::ProxyConfig(_) => "Proxy Config".to_string(),
            Self::FirewallConfig(_) => "Firewall Config".to_string(),
            Self::DefaultPrinter { .. } => "Default Printer".to_string(),
            Self::SetTimezone { .. } => "Set Timezone".to_string(),
            Self::EnvironmentVariables { .. } => "Environment Variables".to_string(),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::SetHostname { hostname, .. } => {
                format!("Set hostname: {}", hostname)
            }
            Self::HostsEntries { entries, .. } => {
                format!("/etc/hosts: {} entries", entries.len())
            }
            Self::ProxyConfig(config) => {
                format!("Proxy: {:?}", config.mode)
            }
            Self::FirewallConfig(config) => {
                format!("Firewall zone: {}", config.default_zone)
            }
            Self::DefaultPrinter { printer_name } => {
                format!("Default printer: {}", printer_name)
            }
            Self::SetTimezone { timezone } => {
                format!("Timezone: {}", timezone)
            }
            Self::EnvironmentVariables { variables } => {
                format!("Environment: {} variables", variables.len())
            }
        }
    }

    /// Get the icon name for this action.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::SetHostname { .. } => "computer-symbolic",
            Self::HostsEntries { .. } => "document-properties-symbolic",
            Self::ProxyConfig(_) => "network-server-symbolic",
            Self::FirewallConfig(_) => "security-high-symbolic",
            Self::DefaultPrinter { .. } => "printer-symbolic",
            Self::SetTimezone { .. } => "preferences-system-time-symbolic",
            Self::EnvironmentVariables { .. } => "utilities-terminal-symbolic",
        }
    }

    /// Check if this action requires elevated privileges.
    pub fn requires_privilege(&self) -> bool {
        match self {
            // All system actions require privilege except env vars
            Self::EnvironmentVariables { .. } => false,
            _ => true,
        }
    }
}
