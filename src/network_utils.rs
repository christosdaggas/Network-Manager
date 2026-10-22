// Network Manager - Network Utilities
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Network interface detection and utilities.
//!
//! This module provides functions to detect and query network interfaces
//! on the system using the Linux sysfs interface.

use std::fs;
use std::path::Path;
use crate::models::{AdapterInfo, AdapterType};

/// Detect all network adapters on the system.
///
/// Reads from /sys/class/net to find all network interfaces and determines
/// their type (Ethernet, WiFi, etc.) and current state.
pub fn detect_network_adapters() -> Vec<AdapterInfo> {
    let mut adapters = Vec::new();
    let net_path = Path::new("/sys/class/net");

    if let Ok(entries) = fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip loopback for the primary list
            if name == "lo" {
                continue;
            }

            // Determine adapter type
            let adapter_type = determine_adapter_type(&entry.path(), &name);
            
            // Skip virtual/tunnel interfaces for now
            if matches!(adapter_type, AdapterType::Virtual | AdapterType::Loopback) {
                continue;
            }

            let mut info = AdapterInfo::new(&name, adapter_type);
            
            // Read MAC address
            let address_path = entry.path().join("address");
            if let Ok(mac) = fs::read_to_string(&address_path) {
                let mac = mac.trim().to_uppercase();
                if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                    info.mac_address = Some(mac);
                }
            }

            // Read operational state (up/down)
            let operstate_path = entry.path().join("operstate");
            if let Ok(state) = fs::read_to_string(&operstate_path) {
                info.is_connected = state.trim() == "up";
            }

            // Read speed (for Ethernet)
            let speed_path = entry.path().join("speed");
            if let Ok(speed_str) = fs::read_to_string(&speed_path) {
                if let Ok(speed) = speed_str.trim().parse::<i32>() {
                    if speed > 0 {
                        info.speed_mbps = Some(speed as u32);
                    }
                }
            }

            // Try to get a human-readable description from device driver
            let device_path = entry.path().join("device");
            if device_path.exists() {
                // Read driver name
                let driver_path = device_path.join("driver");
                if let Ok(driver_link) = fs::read_link(&driver_path) {
                    if let Some(driver_name) = driver_link.file_name() {
                        info.description = Some(driver_name.to_string_lossy().to_string());
                    }
                }
            }

            adapters.push(info);
        }
    }

    // Sort by name for consistent ordering
    adapters.sort_by(|a, b| natural_sort_key(&a.name).cmp(&natural_sort_key(&b.name)));

    adapters
}

/// Determine the type of network adapter.
fn determine_adapter_type(path: &Path, name: &str) -> AdapterType {
    // Check for wireless by looking for wireless directory
    let wireless_path = path.join("wireless");
    if wireless_path.exists() {
        return AdapterType::Wifi;
    }

    // Check uevent file for device type
    let uevent_path = path.join("uevent");
    if let Ok(uevent) = fs::read_to_string(&uevent_path) {
        if uevent.contains("DEVTYPE=wlan") {
            return AdapterType::Wifi;
        }
    }

    // Check type file (1 = Ethernet/ARPHRD_ETHER)
    let type_path = path.join("type");
    if let Ok(type_str) = fs::read_to_string(&type_path) {
        let type_num: u32 = type_str.trim().parse().unwrap_or(0);
        match type_num {
            1 => {
                // ARPHRD_ETHER - could be Ethernet or WiFi
                // If we haven't detected WiFi above, assume Ethernet
                // But check for common virtual interface patterns
                if is_virtual_interface(name) {
                    return AdapterType::Virtual;
                }
                return AdapterType::Ethernet;
            }
            772 => return AdapterType::Loopback, // ARPHRD_LOOPBACK
            _ => {}
        }
    }

    // Check name patterns for WiFi
    if name.starts_with("wl") || name.starts_with("wlan") || name.starts_with("wifi") {
        return AdapterType::Wifi;
    }

    // Check name patterns for virtual interfaces
    if is_virtual_interface(name) {
        return AdapterType::Virtual;
    }

    // Default to Ethernet for physical-looking names
    if name.starts_with("en") || name.starts_with("eth") {
        return AdapterType::Ethernet;
    }

    AdapterType::Other
}

/// Check if interface name suggests a virtual/tunnel interface.
fn is_virtual_interface(name: &str) -> bool {
    name.starts_with("veth") ||
    name.starts_with("br") ||
    name.starts_with("virbr") ||
    name.starts_with("docker") ||
    name.starts_with("vnet") ||
    name.starts_with("tun") ||
    name.starts_with("tap") ||
    name.starts_with("bond") ||
    name.starts_with("team") ||
    name.starts_with("vlan") ||
    name.contains("podman")
}

/// Generate a sort key that sorts numbers naturally.
fn natural_sort_key(s: &str) -> (String, u32) {
    let mut prefix = String::new();
    let mut num_str = String::new();
    
    for c in s.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else {
            if num_str.is_empty() {
                prefix.push(c);
            }
        }
    }

    let num: u32 = num_str.parse().unwrap_or(0);
    (prefix, num)
}

/// Detect available WiFi networks (SSIDs).
/// 
/// Uses nmcli to scan for available networks.
#[allow(dead_code)]
pub fn detect_wifi_networks() -> Vec<String> {
    let output = std::process::Command::new("nmcli")
        .env("LC_ALL", "C")
        .args(["-t", "-f", "SSID", "device", "wifi", "list"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(|s| s.to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Detect configured VPN connections.
///
/// Uses nmcli to list VPN connections.
#[allow(dead_code)]
pub fn detect_vpn_connections() -> Vec<String> {
    let output = std::process::Command::new("nmcli")
        .env("LC_ALL", "C")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 && parts[1].contains("vpn") {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort_key() {
        assert_eq!(natural_sort_key("eth0"), ("eth".to_string(), 0));
        assert_eq!(natural_sort_key("eth10"), ("eth".to_string(), 10));
        assert_eq!(natural_sort_key("enp3s0"), ("enp".to_string(), 30));
    }

    #[test]
    fn test_detect_adapters() {
        // This test will vary by system, just ensure it doesn't panic
        let adapters = detect_network_adapters();
        println!("Detected {} adapters", adapters.len());
        for adapter in &adapters {
            println!("  {} ({:?})", adapter.name, adapter.adapter_type);
        }
    }
}
