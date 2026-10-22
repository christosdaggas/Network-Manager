// Network Manager - Adapter Configuration
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Per-adapter network configuration types.
//!
//! This module provides structures for configuring individual network adapters
//! within a single profile. Each adapter can have its own IP, DNS, and state settings.

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::actions::{Ipv4Method, Ipv6Method, Ipv4Address, Ipv6Address};
use crate::tr;

/// Type of network adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterType {
    /// Wired Ethernet adapter.
    Ethernet,
    /// Wireless (WiFi) adapter.
    Wifi,
    /// Virtual adapter (bridges, VLANs, etc.).
    Virtual,
    /// Loopback interface.
    Loopback,
    /// Unknown or other type.
    Other,
}

impl AdapterType {
    /// Get icon name for this adapter type.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Ethernet => "network-wired-symbolic",
            Self::Wifi => "network-wireless-symbolic",
            Self::Virtual => "network-vpn-symbolic",
            Self::Loopback => "network-workgroup-symbolic",
            Self::Other => "network-wired-symbolic",
        }
    }

    /// Get the translated, human-readable name for this adapter type.
    /// Only for display; serialization uses the serde names.
    pub fn display_name(&self) -> String {
        match self {
            Self::Ethernet => tr("Ethernet"),
            Self::Wifi => tr("Wi-Fi"),
            // TRANSLATORS: Virtual network adapter type (bridges, VLANs)
            Self::Virtual => tr("Virtual"),
            // TRANSLATORS: Loopback network interface type
            Self::Loopback => tr("Loopback"),
            // TRANSLATORS: Generic network interface type
            Self::Other => tr("Network"),
        }
    }
}

/// Information about a detected network adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    /// Interface name (e.g., "eth0", "wlan0", "enp3s0").
    pub name: String,
    /// Type of adapter.
    pub adapter_type: AdapterType,
    /// Hardware MAC address (if available).
    pub mac_address: Option<String>,
    /// Human-readable description or driver name.
    pub description: Option<String>,
    /// Whether the adapter is currently connected/up.
    pub is_connected: bool,
    /// Current speed in Mbps (if available).
    pub speed_mbps: Option<u32>,
}

impl AdapterInfo {
    /// Create a new AdapterInfo.
    pub fn new(name: impl Into<String>, adapter_type: AdapterType) -> Self {
        Self {
            name: name.into(),
            adapter_type,
            mac_address: None,
            description: None,
            is_connected: false,
            speed_mbps: None,
        }
    }

    /// Get a display label for the adapter.
    #[allow(dead_code)]
    pub fn display_label(&self) -> String {
        if let Some(desc) = &self.description {
            format!("{} ({})", self.name, desc)
        } else {
            format!("{} - {}", self.name, self.adapter_type.display_name())
        }
    }
}

/// Configuration for a single network adapter within a profile.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Interface name to apply this configuration to.
    pub interface: String,
    
    /// Whether this adapter should be enabled/disabled.
    /// If None, the adapter state is not changed.
    pub enabled: Option<bool>,
    
    /// IPv4 configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<Ipv4Config>,
    
    /// IPv6 configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<Ipv6Config>,
    
    /// DNS servers for this adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns_servers: Vec<IpAddr>,
    
    /// DNS search domains for this adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns_search_domains: Vec<String>,
    
    /// WiFi SSID to connect to (only for WiFi adapters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_ssid: Option<String>,
    
    /// MTU setting (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    
    /// MAC address spoofing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
}

#[allow(dead_code)]
impl AdapterConfig {
    /// Create a new adapter configuration with default (DHCP) settings.
    pub fn new(interface: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            enabled: Some(true),
            ipv4: Some(Ipv4Config::default()),
            ipv6: Some(Ipv6Config::default()),
            dns_servers: Vec::new(),
            dns_search_domains: Vec::new(),
            wifi_ssid: None,
            mtu: None,
            mac_address: None,
        }
    }

    /// Create a disabled adapter configuration.
    pub fn disabled(interface: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            enabled: Some(false),
            ipv4: None,
            ipv6: None,
            dns_servers: Vec::new(),
            dns_search_domains: Vec::new(),
            wifi_ssid: None,
            mtu: None,
            mac_address: None,
        }
    }
}

/// IPv4 configuration for an adapter.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv4Config {
    /// Configuration method.
    pub method: Ipv4Method,
    
    /// Static addresses (for manual method).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<Ipv4Address>,
    
    /// Default gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<Ipv4Addr>,
}

impl Default for Ipv4Config {
    fn default() -> Self {
        Self {
            method: Ipv4Method::Auto,
            addresses: Vec::new(),
            gateway: None,
        }
    }
}

/// IPv6 configuration for an adapter.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Config {
    /// Configuration method.
    pub method: Ipv6Method,
    
    /// Static addresses (for manual method).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<Ipv6Address>,
    
    /// Default gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<Ipv6Addr>,
}

impl Default for Ipv6Config {
    fn default() -> Self {
        Self {
            method: Ipv6Method::Auto,
            addresses: Vec::new(),
            gateway: None,
        }
    }
}
