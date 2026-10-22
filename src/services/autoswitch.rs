// Network Manager - Auto-Switch Service
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Auto-switch service for automatic profile activation based on rules.
//!
//! This service periodically evaluates rule conditions and triggers
//! profile switches when conditions match.

use crate::models::rules::{Condition, InterfaceStateMatch, RuleOperator, RuleSet};
use crate::models::Profile;
use chrono::{Datelike, Local, Timelike};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};

/// Maximum compiled regex size to prevent ReDoS (1 KiB).
const REGEX_SIZE_LIMIT: usize = 1 << 10;

/// Service for evaluating auto-switch rules and triggering profile changes.
pub struct AutoSwitchService {
    /// Current network SSID (cached).
    cached_ssid: Option<String>,
    /// Current gateway MAC (cached).
    cached_gateway_mac: Option<String>,
    /// Last evaluated profile ID.
    last_profile_id: Option<String>,
    /// Compiled regex cache keyed by pattern string.
    regex_cache: HashMap<String, regex::Regex>,
}

impl AutoSwitchService {
    /// Create a new auto-switch service.
    pub fn new() -> Self {
        Self {
            cached_ssid: None,
            cached_gateway_mac: None,
            last_profile_id: None,
            regex_cache: HashMap::new(),
        }
    }

    /// Get or compile a regex, using the cache and applying a size limit.
    fn get_or_compile_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        // Insert if not present
        if !self.regex_cache.contains_key(pattern) {
            match regex::RegexBuilder::new(pattern)
                .size_limit(REGEX_SIZE_LIMIT)
                .build()
            {
                Ok(re) => {
                    self.regex_cache.insert(pattern.to_string(), re);
                }
                Err(e) => {
                    warn!("Failed to compile regex '{}': {}", pattern, e);
                    return None;
                }
            }
        }
        self.regex_cache.get(pattern)
    }

    /// Evaluate all profiles and return the ID of the first matching profile.
    pub fn evaluate_profiles(&mut self, profiles: &[Profile]) -> Option<String> {
        // Sort profiles by priority (higher first)
        let mut sorted_profiles: Vec<_> = profiles
            .iter()
            .filter(|p| {
                p.auto_switch_rules
                    .as_ref()
                    .map(|r| r.enabled && !r.conditions.is_empty())
                    .unwrap_or(false)
            })
            .collect();
        
        sorted_profiles.sort_by(|a, b| {
            let a_priority = a.auto_switch_rules.as_ref().map(|r| r.priority).unwrap_or(0);
            let b_priority = b.auto_switch_rules.as_ref().map(|r| r.priority).unwrap_or(0);
            b_priority.cmp(&a_priority)
        });

        // Nothing to evaluate: skip the nmcli/ip queries entirely.
        if sorted_profiles.is_empty() {
            self.last_profile_id = None;
            return None;
        }

        // Update cached network state
        self.update_network_state();

        // Evaluate each profile's rules
        for profile in sorted_profiles {
            let Some(rules) = profile.auto_switch_rules.as_ref() else {
                continue;
            };
            
            if self.evaluate_ruleset(rules) {
                // Skip if already active
                if self.last_profile_id.as_ref() == Some(&profile.id().to_string()) {
                    debug!("Profile {} already active, skipping", profile.name());
                    return None;
                }

                info!("Auto-switch: Profile '{}' matches rules", profile.name());
                self.last_profile_id = Some(profile.id().to_string());
                return Some(profile.id().to_string());
            }
        }

        // No rules match any more: forget the last match so the profile is
        // applied again when its conditions return (e.g. back on the home Wi-Fi).
        self.last_profile_id = None;
        None
    }

    /// Evaluate a rule set.
    fn evaluate_ruleset(&mut self, rules: &RuleSet) -> bool {
        if rules.conditions.is_empty() {
            return false;
        }

        match rules.operator {
            RuleOperator::And => {
                for c in &rules.conditions {
                    if !self.evaluate_condition(c) {
                        return false;
                    }
                }
                true
            }
            RuleOperator::Or => {
                for c in &rules.conditions {
                    if self.evaluate_condition(c) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Evaluate a single condition.
    fn evaluate_condition(&mut self, condition: &Condition) -> bool {
        match condition {
            Condition::WifiSsid { ssid, regex } => {
                self.check_wifi_ssid(ssid, *regex)
            }
            Condition::GatewayMac { mac } => {
                self.check_gateway_mac(mac)
            }
            Condition::PingTarget { host, timeout_ms } => {
                self.check_ping(host, *timeout_ms)
            }
            Condition::InterfaceState { interface, state } => {
                self.check_interface_state(interface, state)
            }
            Condition::TimeWindow { window } => {
                let now = Local::now();
                let time = chrono::NaiveTime::from_hms_opt(
                    now.hour(), now.minute(), now.second()
                ).unwrap_or_default();
                let weekday = now.weekday();
                window.is_active(time, weekday)
            }
            Condition::NetworkAvailable => {
                self.check_network_available()
            }
            Condition::Not { condition } => {
                !self.evaluate_condition(condition)
            }
        }
    }

    /// Update cached network state.
    fn update_network_state(&mut self) {
        // Get current SSID
        self.cached_ssid = Self::get_current_ssid();
        
        // Get gateway MAC
        self.cached_gateway_mac = Self::get_gateway_mac();
    }

    /// Get current Wi-Fi SSID.
    fn get_current_ssid() -> Option<String> {
        // LC_ALL=C: nmcli translates "yes"/"no" even in terse mode.
        // --rescan no: listing must not trigger a Wi-Fi scan on every cycle.
        let output = Command::new("nmcli")
            .env("LC_ALL", "C")
            .args(["-t", "-f", "active,ssid", "dev", "wifi", "list", "--rescan", "no"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("yes:"))
            .map(unescape_nmcli_terse)
    }

    /// Get gateway MAC address.
    fn get_gateway_mac() -> Option<String> {
        // Get default gateway IP
        let output = Command::new("ip")
            .env("LC_ALL", "C")
            .args(["route", "show", "default"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let gateway_ip = stdout
            .lines()
            .next()?
            .split_whitespace()
            .nth(2)?;

        // Get MAC from ARP table
        let arp_output = Command::new("ip")
            .env("LC_ALL", "C")
            .args(["neigh", "show", gateway_ip])
            .output()
            .ok()?;

        let arp_stdout = String::from_utf8_lossy(&arp_output.stdout);
        arp_stdout
            .lines()
            .next()?
            .split_whitespace()
            .nth(4)
            .map(|s| s.to_lowercase())
    }

    /// Check Wi-Fi SSID condition.
    fn check_wifi_ssid(&mut self, ssid: &str, regex: bool) -> bool {
        let Some(current) = &self.cached_ssid.clone() else {
            return false;
        };

        if regex {
            // Use regex matching with size-limited cache
            self.get_or_compile_regex(ssid)
                .map(|re| re.is_match(current))
                .unwrap_or(false)
        } else {
            // Glob-style matching (simple * support)
            if ssid.contains('*') {
                let pattern = glob_to_regex(ssid);
                self.get_or_compile_regex(&pattern)
                    .map(|re| re.is_match(current))
                    .unwrap_or(false)
            } else {
                current == ssid
            }
        }
    }

    /// Check gateway MAC condition.
    fn check_gateway_mac(&self, mac: &str) -> bool {
        self.cached_gateway_mac
            .as_ref()
            .map(|m| m.to_lowercase() == mac.to_lowercase())
            .unwrap_or(false)
    }

    /// Check ping target reachability.
    fn check_ping(&self, host: &str, timeout_ms: u32) -> bool {
        let timeout_secs = (timeout_ms / 1000).max(1);
        
        Command::new("ping")
            .args(["-c", "1", "-W", &timeout_secs.to_string(), host])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Check interface state.
    fn check_interface_state(&self, interface: &str, expected: &InterfaceStateMatch) -> bool {
        let operstate_path = format!("/sys/class/net/{}/operstate", interface);
        let carrier_path = format!("/sys/class/net/{}/carrier", interface);

        match expected {
            InterfaceStateMatch::Up => {
                std::fs::read_to_string(&operstate_path)
                    .map(|s| s.trim() == "up")
                    .unwrap_or(false)
            }
            InterfaceStateMatch::Down => {
                std::fs::read_to_string(&operstate_path)
                    .map(|s| s.trim() == "down")
                    .unwrap_or(false)
            }
            InterfaceStateMatch::Carrier => {
                std::fs::read_to_string(&carrier_path)
                    .map(|s| s.trim() == "1")
                    .unwrap_or(false)
            }
            InterfaceStateMatch::NoCarrier => {
                std::fs::read_to_string(&carrier_path)
                    .map(|s| s.trim() == "0")
                    .unwrap_or(true)
            }
        }
    }

    /// Check if any network is available.
    fn check_network_available(&self) -> bool {
        // Check if connected to any network
        Command::new("nmcli")
            .env("LC_ALL", "C")
            .args(["-t", "-f", "STATE", "general"])
            .output()
            .map(|out| out.status.success() && nm_state_is_connected(&String::from_utf8_lossy(&out.stdout)))
            .unwrap_or(false)
    }

    /// Clear the last profile ID (used when profile is manually changed).
    #[allow(dead_code)]
    pub fn clear_last_profile(&mut self) {
        self.last_profile_id = None;
        self.regex_cache.clear();
    }
}

impl Default for AutoSwitchService {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a `*` glob into an anchored regex, escaping everything else.
fn glob_to_regex(glob: &str) -> String {
    let parts: Vec<String> = glob.split('*').map(regex::escape).collect();
    format!("^{}$", parts.join(".*"))
}

/// Undo nmcli terse-mode escaping (`\:` and `\\`) in a field value.
fn unescape_nmcli_terse(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// Whether `nmcli -t -f STATE general` output reports connectivity.
///
/// Only the `connected*` states count ("connected", "connected (local only)",
/// "connected (site only)"); "disconnected", "connecting" and "asleep" do not.
fn nm_state_is_connected(output: &str) -> bool {
    output.trim().starts_with("connected")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glob_matches(glob: &str, ssid: &str) -> bool {
        regex::Regex::new(&glob_to_regex(glob)).expect("valid regex").is_match(ssid)
    }

    #[test]
    fn test_glob_matches_wildcards() {
        assert!(glob_matches("Office*", "Office-5G"));
        assert!(glob_matches("*Guest*", "Hotel Guest WiFi"));
        assert!(!glob_matches("Office*", "My Office"));
    }

    #[test]
    fn test_glob_escapes_regex_characters() {
        assert!(glob_matches("Guest (5G)*", "Guest (5G) Lobby"));
        assert!(!glob_matches("Guest (5G)*", "Guest 5G Lobby"));
        assert!(glob_matches("Cafe+Bar*", "Cafe+Bar_EXT"));
        assert!(!glob_matches("Cafe+Bar*", "CafeeeBar"));
        assert!(!glob_matches("a.b*", "axb"));
    }

    #[test]
    fn test_unescape_nmcli_terse() {
        assert_eq!(unescape_nmcli_terse("Home"), "Home");
        assert_eq!(unescape_nmcli_terse("My\\:Net"), "My:Net");
        assert_eq!(unescape_nmcli_terse("back\\\\slash"), "back\\slash");
        assert_eq!(unescape_nmcli_terse("trailing\\"), "trailing\\");
    }

    #[test]
    fn test_nm_state_is_connected() {
        assert!(nm_state_is_connected("connected\n"));
        assert!(nm_state_is_connected("connected (local only)\n"));
        assert!(nm_state_is_connected("connected (site only)"));
        assert!(!nm_state_is_connected("disconnected\n"));
        assert!(!nm_state_is_connected("connecting\n"));
        assert!(!nm_state_is_connected("asleep"));
        assert!(!nm_state_is_connected(""));
    }

    #[test]
    fn test_no_rule_profiles_reset_last_match() {
        let mut service = AutoSwitchService::new();
        service.last_profile_id = Some("previous".to_string());
        let profiles = vec![Profile::new("No rules")];
        assert_eq!(service.evaluate_profiles(&profiles), None);
        assert_eq!(service.last_profile_id, None);
    }
}
