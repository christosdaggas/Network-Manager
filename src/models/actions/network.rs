// Network Manager - Network Actions
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Network configuration actions.
//!
//! These actions are applied via NetworkManager D-Bus API.

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// IPv4 configuration method.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Ipv4Method {
    /// Obtain address via DHCP.
    #[default]
    Auto,
    /// Manual/static configuration.
    Manual,
    /// Link-local only.
    LinkLocal,
    /// Disabled.
    Disabled,
}

/// IPv6 configuration method.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Ipv6Method {
    /// Obtain address via SLAAC/DHCPv6.
    #[default]
    Auto,
    /// DHCPv6 only.
    Dhcp,
    /// Manual/static configuration.
    Manual,
    /// Link-local only.
    LinkLocal,
    /// Disabled.
    Disabled,
}

/// IPv4 address with prefix length.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv4Address {
    /// IP address.
    pub address: Ipv4Addr,
    /// Prefix length (e.g., 24 for /24).
    pub prefix: u8,
}

/// IPv6 address with prefix length.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Address {
    /// IP address.
    pub address: Ipv6Addr,
    /// Prefix length (e.g., 64 for /64).
    pub prefix: u8,
}

/// Static route definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticRoute {
    /// Destination network.
    pub destination: String,
    /// Prefix length.
    pub prefix: u8,
    /// Gateway address.
    pub gateway: IpAddr,
    /// Metric (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<u32>,
}

/// Network interface enable/disable action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceState {
    /// Interface name (e.g., "eth0", "wlan0").
    pub interface: String,
    /// Whether to enable or disable.
    pub enabled: bool,
}

/// Network configuration actions.
///
/// All actions are applied via NetworkManager D-Bus API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum NetworkAction {
    /// Configure IPv4 settings.
    Ipv4Config {
        /// Target interface (None = default/all).
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
        /// Configuration method.
        method: Ipv4Method,
        /// Static addresses (for manual method).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        addresses: Vec<Ipv4Address>,
        /// Default gateway.
        #[serde(skip_serializing_if = "Option::is_none")]
        gateway: Option<Ipv4Addr>,
    },

    /// Configure IPv6 settings.
    Ipv6Config {
        /// Target interface.
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
        /// Configuration method.
        method: Ipv6Method,
        /// Static addresses (for manual method).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        addresses: Vec<Ipv6Address>,
        /// Default gateway.
        #[serde(skip_serializing_if = "Option::is_none")]
        gateway: Option<Ipv6Addr>,
    },

    /// Configure DNS servers.
    DnsServers {
        /// Target interface (None = system-wide).
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
        /// DNS server addresses.
        servers: Vec<IpAddr>,
    },

    /// Configure DNS search domains.
    DnsSearchDomains {
        /// Target interface (None = system-wide).
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
        /// Search domains.
        domains: Vec<String>,
    },

    /// Add static routes.
    StaticRoutes {
        /// Target interface.
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
        /// Routes to add.
        routes: Vec<StaticRoute>,
    },

    /// Enable or disable a network interface.
    InterfaceEnable(InterfaceState),

    /// Set MTU for an interface.
    SetMtu {
        /// Target interface.
        interface: String,
        /// MTU value.
        mtu: u32,
    },

    /// Set MAC address (spoofing).
    SetMacAddress {
        /// Target interface.
        interface: String,
        /// MAC address (format: "AA:BB:CC:DD:EE:FF").
        mac_address: String,
    },

    /// Activate a Wi-Fi connection by SSID.
    WifiConnect {
        /// SSID to connect to.
        ssid: String,
        /// Specific interface (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        interface: Option<String>,
    },

    /// Activate a VPN connection.
    VpnConnect {
        /// VPN connection name (as configured in NetworkManager).
        connection_name: String,
    },

    /// Disconnect VPN.
    VpnDisconnect {
        /// VPN connection name.
        connection_name: String,
    },

    /// Configure VLAN.
    VlanConfig {
        /// Parent interface.
        parent_interface: String,
        /// VLAN ID.
        vlan_id: u16,
        /// VLAN interface name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

impl NetworkAction {
    /// Get a short name for the action.
    pub fn name(&self) -> String {
        match self {
            Self::Ipv4Config { .. } => "IPv4 Config".to_string(),
            Self::Ipv6Config { .. } => "IPv6 Config".to_string(),
            Self::DnsServers { .. } => "DNS Servers".to_string(),
            Self::DnsSearchDomains { .. } => "DNS Search Domains".to_string(),
            Self::StaticRoutes { .. } => "Static Routes".to_string(),
            Self::InterfaceEnable(state) => format!("{} Interface", if state.enabled { "Enable" } else { "Disable" }),
            Self::SetMtu { .. } => "Set MTU".to_string(),
            Self::SetMacAddress { .. } => "Set MAC Address".to_string(),
            Self::WifiConnect { ssid, .. } => format!("Connect WiFi: {}", ssid),
            Self::VpnConnect { connection_name } => format!("Connect VPN: {}", connection_name),
            Self::VpnDisconnect { connection_name } => format!("Disconnect VPN: {}", connection_name),
            Self::VlanConfig { vlan_id, .. } => format!("VLAN {}", vlan_id),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::Ipv4Config { interface, method, .. } => {
                format!(
                    "IPv4: {:?} on {}",
                    method,
                    interface.as_deref().unwrap_or("default")
                )
            }
            Self::Ipv6Config { interface, method, .. } => {
                format!(
                    "IPv6: {:?} on {}",
                    method,
                    interface.as_deref().unwrap_or("default")
                )
            }
            Self::DnsServers { servers, .. } => {
                format!("DNS: {}", servers.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "))
            }
            Self::DnsSearchDomains { domains, .. } => {
                format!("Search domains: {}", domains.join(", "))
            }
            Self::StaticRoutes { routes, .. } => {
                format!("Static routes: {} entries", routes.len())
            }
            Self::InterfaceEnable(state) => {
                format!(
                    "{} interface {}",
                    if state.enabled { "Enable" } else { "Disable" },
                    state.interface
                )
            }
            Self::SetMtu { interface, mtu } => {
                format!("MTU {} on {}", mtu, interface)
            }
            Self::SetMacAddress { interface, mac_address } => {
                format!("MAC {} on {}", mac_address, interface)
            }
            Self::WifiConnect { ssid, .. } => {
                format!("Connect to Wi-Fi: {}", ssid)
            }
            Self::VpnConnect { connection_name } => {
                format!("Connect VPN: {}", connection_name)
            }
            Self::VpnDisconnect { connection_name } => {
                format!("Disconnect VPN: {}", connection_name)
            }
            Self::VlanConfig { parent_interface, vlan_id, .. } => {
                format!("VLAN {} on {}", vlan_id, parent_interface)
            }
        }
    }

    /// Get the icon name for this action.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Ipv4Config { .. } | Self::Ipv6Config { .. } => "network-wired-symbolic",
            Self::DnsServers { .. } | Self::DnsSearchDomains { .. } => "network-server-symbolic",
            Self::StaticRoutes { .. } => "route-symbolic",
            Self::InterfaceEnable(_) => "network-wired-symbolic",
            Self::SetMtu { .. } => "preferences-system-network-symbolic",
            Self::SetMacAddress { .. } => "network-wired-symbolic",
            Self::WifiConnect { .. } => "network-wireless-symbolic",
            Self::VpnConnect { .. } | Self::VpnDisconnect { .. } => "network-vpn-symbolic",
            Self::VlanConfig { .. } => "network-wired-symbolic",
        }
    }
}
