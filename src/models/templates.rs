// Network Manager - Profile Templates
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Predefined profile templates for common network scenarios.
//!
//! Templates provide a quick way to create profiles with sensible defaults
//! for typical use cases like home networks, office environments, or public WiFi.

use super::actions::{NetworkAction, Ipv4Method, AutomationAction, SystemAction};
use super::actions::{ProxyConfig, ProxyMode, HostsEntry};
use super::profile::Profile;
use crate::tr;

/// Available profile templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProfileTemplate {
    /// Home network with DHCP
    HomeNetwork,
    /// Office/Corporate network with static IP
    OfficeNetwork,
    /// Public WiFi with security measures
    PublicWifi,
    /// VPN-only profile
    VpnOnly,
    /// Development environment
    Development,
    /// Minimal/blank template
    Blank,
}

#[allow(dead_code)]
impl ProfileTemplate {
    /// Get all available templates.
    pub fn all() -> &'static [ProfileTemplate] {
        &[
            Self::HomeNetwork,
            Self::OfficeNetwork,
            Self::PublicWifi,
            Self::VpnOnly,
            Self::Development,
            Self::Blank,
        ]
    }
    
    /// Get the template name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::HomeNetwork => "Home Network",
            Self::OfficeNetwork => "Office/Corporate",
            Self::PublicWifi => "Public WiFi (Secure)",
            Self::VpnOnly => "VPN Only",
            Self::Development => "Development",
            Self::Blank => "Blank Profile",
        }
    }
    
    /// Get the translated template description. It becomes the new
    /// profile's description, which is shown in the UI.
    pub fn description(&self) -> String {
        match self {
            Self::HomeNetwork => tr("Basic home network setup with DHCP and default DNS"),
            Self::OfficeNetwork => tr("Corporate network with static IP configuration"),
            Self::PublicWifi => tr("Secure configuration for untrusted public networks"),
            Self::VpnOnly => tr("Route all traffic through VPN connection"),
            Self::Development => tr("Local development with custom hosts and DNS"),
            Self::Blank => tr("Start with an empty profile"),
        }
    }
    
    /// Get the icon name for this template.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::HomeNetwork => "user-home-symbolic",
            Self::OfficeNetwork => "x-office-address-book-symbolic",
            Self::PublicWifi => "network-wireless-symbolic",
            Self::VpnOnly => "network-vpn-symbolic",
            Self::Development => "utilities-terminal-symbolic",
            Self::Blank => "document-new-symbolic",
        }
    }
    
    /// Create a profile from this template.
    pub fn create_profile(&self, name: &str) -> Profile {
        let mut profile = Profile::new(name);
        profile.metadata.description = Some(self.description());
        
        match self {
            Self::HomeNetwork => {
                profile.metadata.icon = Some("user-home-symbolic".to_string());
                profile.metadata.group = Some(super::profile::ProfileGroup::new(
                    // TRANSLATORS: Profile group name for the home network template
                    tr("Home"),
                ));
                
                // DHCP configuration
                profile.network_actions.push(NetworkAction::Ipv4Config {
                    interface: None,
                    method: Ipv4Method::Auto,
                    addresses: vec![],
                    gateway: None,
                });
                
                // Use standard DNS (Google/Cloudflare)
                profile.network_actions.push(NetworkAction::DnsServers {
                    interface: None,
                    servers: vec![
                        "8.8.8.8".parse().unwrap(),
                        "1.1.1.1".parse().unwrap(),
                    ],
                });
            }
            
            Self::OfficeNetwork => {
                profile.metadata.icon = Some("x-office-address-book-symbolic".to_string());
                profile.metadata.group = Some(super::profile::ProfileGroup::new(
                    // TRANSLATORS: Profile group name for the office network template
                    tr("Work"),
                ));
                
                // Static IP placeholder configuration
                profile.network_actions.push(NetworkAction::Ipv4Config {
                    interface: None,
                    method: Ipv4Method::Manual,
                    addresses: vec![
                        super::actions::Ipv4Address {
                            address: "192.168.1.100".parse().unwrap(),
                            prefix: 24,
                        }
                    ],
                    gateway: Some("192.168.1.1".parse().unwrap()),
                });
                
                // Corporate DNS placeholders
                profile.network_actions.push(NetworkAction::DnsServers {
                    interface: None,
                    servers: vec![
                        "192.168.1.1".parse().unwrap(),
                    ],
                });
                
                // Add corporate proxy placeholder
                profile.system_actions.push(SystemAction::ProxyConfig(ProxyConfig {
                    mode: ProxyMode::Manual,
                    http_proxy: Some("http://proxy.company.com:8080".to_string()),
                    https_proxy: Some("http://proxy.company.com:8080".to_string()),
                    ftp_proxy: None,
                    socks_proxy: None,
                    no_proxy: vec!["localhost".to_string(), "127.0.0.1".to_string(), ".company.com".to_string()],
                    pac_url: None,
                }));
            }
            
            Self::PublicWifi => {
                profile.metadata.icon = Some("network-wireless-symbolic".to_string());
                profile.metadata.group = Some(super::profile::ProfileGroup::new(
                    // TRANSLATORS: Profile group name for the public Wi-Fi template
                    tr("Travel"),
                ));
                
                // DHCP but with secure DNS
                profile.network_actions.push(NetworkAction::Ipv4Config {
                    interface: None,
                    method: Ipv4Method::Auto,
                    addresses: vec![],
                    gateway: None,
                });
                
                // Use DNS over HTTPS capable resolvers
                profile.network_actions.push(NetworkAction::DnsServers {
                    interface: None,
                    servers: vec![
                        "1.1.1.1".parse().unwrap(),      // Cloudflare
                        "9.9.9.9".parse().unwrap(),      // Quad9 (blocks malware)
                    ],
                });
                
                // Add a notification about public WiFi security
                profile.automation_actions.push(AutomationAction::Notification {
                    title: tr("Public WiFi Connected"),
                    body: tr("Remember: Use HTTPS and consider enabling VPN"),
                    icon: Some("dialog-warning-symbolic".to_string()),
                });
            }
            
            Self::VpnOnly => {
                profile.metadata.icon = Some("network-vpn-symbolic".to_string());
                profile.metadata.group = Some(super::profile::ProfileGroup::new(
                    // TRANSLATORS: Profile group name for the VPN template
                    tr("Security"),
                ));
                
                // VPN connection (placeholder - user must configure)
                profile.network_actions.push(NetworkAction::VpnConnect {
                    connection_name: "My VPN".to_string(),
                });
                
                // Notification when VPN connects
                profile.automation_actions.push(AutomationAction::Notification {
                    title: tr("VPN Connected"),
                    body: tr("All traffic is now routed through VPN"),
                    icon: Some("network-vpn-symbolic".to_string()),
                });
            }
            
            Self::Development => {
                profile.metadata.icon = Some("utilities-terminal-symbolic".to_string());
                profile.metadata.group = Some(super::profile::ProfileGroup::new(
                    // TRANSLATORS: Profile group name for the development template
                    tr("Development"),
                ));
                
                // DHCP for basic connectivity
                profile.network_actions.push(NetworkAction::Ipv4Config {
                    interface: None,
                    method: Ipv4Method::Auto,
                    addresses: vec![],
                    gateway: None,
                });
                
                // Local DNS for development
                profile.network_actions.push(NetworkAction::DnsServers {
                    interface: None,
                    servers: vec![
                        "127.0.0.1".parse().unwrap(),  // Local DNS (e.g., dnsmasq)
                        "8.8.8.8".parse().unwrap(),    // Fallback
                    ],
                });
                
                // Add common development host entries
                profile.system_actions.push(SystemAction::HostsEntries {
                    entries: vec![
                        HostsEntry {
                            ip: "127.0.0.1".parse().unwrap(),
                            hostnames: vec![
                                "local.dev".to_string(),
                                "api.local.dev".to_string(),
                                "app.local.dev".to_string(),
                            ],
                            comment: Some("Development hosts".to_string()),
                        },
                    ],
                    replace_managed: true,
                });
            }
            
            Self::Blank => {
                // Empty profile - no pre-configured actions
                profile.metadata.icon = Some("document-new-symbolic".to_string());
            }
        }
        
        profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_creation() {
        for template in ProfileTemplate::all() {
            let profile = template.create_profile("Test Profile");
            assert_eq!(profile.name(), "Test Profile");
            assert!(profile.metadata.description.is_some());
        }
    }
    
    #[test]
    fn test_home_network_template() {
        let profile = ProfileTemplate::HomeNetwork.create_profile("My Home");
        assert!(!profile.network_actions.is_empty());
        // Should have DHCP config
        assert!(profile.network_actions.iter().any(|a| matches!(a, NetworkAction::Ipv4Config { method: Ipv4Method::Auto, .. })));
    }
    
    #[test]
    fn test_blank_template() {
        let profile = ProfileTemplate::Blank.create_profile("Empty");
        assert!(profile.network_actions.is_empty());
        assert!(profile.system_actions.is_empty());
        assert!(profile.automation_actions.is_empty());
    }
}
