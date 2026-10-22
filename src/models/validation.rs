// Network Manager - Validation Utilities
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Input validation utilities for profiles and actions.

#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use super::error::{Error, Result};

/// Validate an IPv4 address string.
pub fn validate_ipv4(s: &str) -> Result<Ipv4Addr> {
    Ipv4Addr::from_str(s).map_err(|_| Error::InvalidIpAddress(s.to_string()))
}

/// Validate an IPv6 address string.
pub fn validate_ipv6(s: &str) -> Result<Ipv6Addr> {
    Ipv6Addr::from_str(s).map_err(|_| Error::InvalidIpAddress(s.to_string()))
}

/// Validate an IP address string (v4 or v6).
pub fn validate_ip(s: &str) -> Result<IpAddr> {
    IpAddr::from_str(s).map_err(|_| Error::InvalidIpAddress(s.to_string()))
}

/// Validate a CIDR notation (e.g., "192.168.1.0/24").
pub fn validate_cidr(s: &str) -> Result<(IpAddr, u8)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidIpAddress(format!(
            "Invalid CIDR notation: {}",
            s
        )));
    }

    let ip = validate_ip(parts[0])?;
    let prefix: u8 = parts[1]
        .parse()
        .map_err(|_| Error::InvalidIpAddress(format!("Invalid prefix: {}", parts[1])))?;

    let max_prefix = if ip.is_ipv4() { 32 } else { 128 };
    if prefix > max_prefix {
        return Err(Error::InvalidIpAddress(format!(
            "Prefix {} exceeds maximum {} for address type",
            prefix, max_prefix
        )));
    }

    Ok((ip, prefix))
}

/// Validate a MAC address string.
pub fn validate_mac_address(s: &str) -> Result<String> {
    // Accept formats: AA:BB:CC:DD:EE:FF or AA-BB-CC-DD-EE-FF
    let normalized = s.replace('-', ":").to_uppercase();
    let parts: Vec<&str> = normalized.split(':').collect();

    if parts.len() != 6 {
        return Err(Error::InvalidMacAddress(s.to_string()));
    }

    for part in &parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidMacAddress(s.to_string()));
        }
    }

    Ok(normalized)
}

/// Validate a hostname.
pub fn validate_hostname(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 253 {
        return Err(Error::InvalidHostname(format!(
            "Hostname must be 1-253 characters: {}",
            s
        )));
    }

    // Check each label
    for label in s.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(Error::InvalidHostname(format!(
                "Label must be 1-63 characters: {}",
                label
            )));
        }

        if !label
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(Error::InvalidHostname(format!(
                "Invalid characters in label: {}",
                label
            )));
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(Error::InvalidHostname(format!(
                "Label cannot start or end with hyphen: {}",
                label
            )));
        }
    }

    Ok(s.to_lowercase())
}

/// Validate a DNS server address.
pub fn validate_dns_server(s: &str) -> Result<IpAddr> {
    validate_ip(s).map_err(|_| Error::InvalidDnsServer(s.to_string()))
}

/// Validate a search domain.
pub fn validate_search_domain(s: &str) -> Result<String> {
    // Search domains are essentially hostnames
    validate_hostname(s)
}

/// Validate an MTU value.
pub fn validate_mtu(mtu: u32) -> Result<u32> {
    // Standard Ethernet MTU range
    if !(68..=65535).contains(&mtu) {
        return Err(Error::ValidationFailed(format!(
            "MTU must be between 68 and 65535: {}",
            mtu
        )));
    }
    Ok(mtu)
}

/// Validate a VLAN ID.
pub fn validate_vlan_id(id: u16) -> Result<u16> {
    if id == 0 || id > 4094 {
        return Err(Error::ValidationFailed(format!(
            "VLAN ID must be 1-4094: {}",
            id
        )));
    }
    Ok(id)
}

/// Validate a profile name.
pub fn validate_profile_name(s: &str) -> Result<String> {
    let s = s.trim();
    if s.is_empty() {
        return Err(Error::ValidationFailed(
            "Profile name cannot be empty".to_string(),
        ));
    }
    if s.len() > 100 {
        return Err(Error::ValidationFailed(
            "Profile name must be 100 characters or less".to_string(),
        ));
    }
    Ok(s.to_string())
}

/// Validate a timezone string.
pub fn validate_timezone(s: &str) -> Result<String> {
    // Basic validation - check format like "America/New_York"
    if s.is_empty() || !s.contains('/') {
        return Err(Error::ValidationFailed(format!(
            "Invalid timezone format: {}",
            s
        )));
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("256.1.1.1").is_err());
        assert!(validate_ipv4("not-an-ip").is_err());
    }

    #[test]
    fn test_validate_mac() {
        assert!(validate_mac_address("AA:BB:CC:DD:EE:FF").is_ok());
        assert!(validate_mac_address("aa:bb:cc:dd:ee:ff").is_ok());
        assert!(validate_mac_address("AA-BB-CC-DD-EE-FF").is_ok());
        assert!(validate_mac_address("invalid").is_err());
    }

    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("example.com").is_ok());
        assert!(validate_hostname("my-host").is_ok());
        assert!(validate_hostname("-invalid").is_err());
        assert!(validate_hostname("").is_err());
    }

    #[test]
    fn test_validate_cidr() {
        assert!(validate_cidr("192.168.1.0/24").is_ok());
        assert!(validate_cidr("::1/128").is_ok());
        assert!(validate_cidr("192.168.1.0/33").is_err());
        assert!(validate_cidr("192.168.1.0").is_err());
    }
}
