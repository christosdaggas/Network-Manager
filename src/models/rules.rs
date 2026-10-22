// Network Manager - Rule Engine
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Auto-switch rule engine for automatic profile activation.
//!
//! The rule engine evaluates conditions to determine if a profile
//! should be automatically activated based on:
//! - Wi-Fi SSID
//! - Gateway MAC address
//! - Network interface state
//! - Ping target reachability
//! - Time windows
//!
//! ## Design
//!
//! - Rules are evaluated non-blocking (async)
//! - Evaluation is efficient and cached where possible
//! - Multiple conditions can be combined with AND/OR logic

use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

/// Time window definition for time-based rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time.
    pub start: NaiveTime,
    /// End time.
    pub end: NaiveTime,
    /// Days of week (empty = all days).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub days: Vec<Weekday>,
}

impl TimeWindow {
    /// Create a new time window.
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self {
            start,
            end,
            days: Vec::new(),
        }
    }

    /// Check if the current time is within this window.
    pub fn is_active(&self, now: NaiveTime, today: Weekday) -> bool {
        // Check day constraint
        if !self.days.is_empty() && !self.days.contains(&today) {
            return false;
        }

        // Handle overnight windows
        if self.start <= self.end {
            now >= self.start && now <= self.end
        } else {
            // Overnight window (e.g., 22:00 - 06:00)
            now >= self.start || now <= self.end
        }
    }
}

/// Network interface state for condition matching.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceStateMatch {
    /// Interface is up.
    #[default]
    Up,
    /// Interface is down.
    Down,
    /// Interface has carrier.
    Carrier,
    /// Interface has no carrier.
    NoCarrier,
}

/// Auto-switch condition types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Match Wi-Fi SSID.
    WifiSsid {
        /// SSID to match (supports glob patterns).
        ssid: String,
        /// Use regex matching.
        #[serde(default)]
        regex: bool,
    },

    /// Match gateway MAC address.
    GatewayMac {
        /// MAC address to match.
        mac: String,
    },

    /// Ping target is reachable.
    PingTarget {
        /// Host to ping.
        host: String,
        /// Timeout in milliseconds.
        #[serde(default = "default_ping_timeout")]
        timeout_ms: u32,
    },

    /// Network interface state.
    InterfaceState {
        /// Interface name.
        interface: String,
        /// Expected state.
        state: InterfaceStateMatch,
    },

    /// Time window.
    TimeWindow {
        /// Time window definition.
        window: TimeWindow,
    },

    /// Network is available (any connectivity).
    NetworkAvailable,

    /// Not condition (negation).
    Not {
        /// Condition to negate.
        condition: Box<Condition>,
    },
}

fn default_ping_timeout() -> u32 {
    1000
}

impl Condition {
    /// Get a human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::WifiSsid { ssid, regex } => {
                if *regex {
                    format!("Wi-Fi SSID matches: {}", ssid)
                } else {
                    format!("Wi-Fi SSID: {}", ssid)
                }
            }
            Self::GatewayMac { mac } => {
                format!("Gateway MAC: {}", mac)
            }
            Self::PingTarget { host, .. } => {
                format!("Ping: {}", host)
            }
            Self::InterfaceState { interface, state } => {
                format!("{} is {:?}", interface, state)
            }
            Self::TimeWindow { window } => {
                format!("Time: {} - {}", window.start, window.end)
            }
            Self::NetworkAvailable => "Network available".to_string(),
            Self::Not { condition } => {
                format!("NOT ({})", condition.description())
            }
        }
    }

    /// Get the icon name for this condition.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::WifiSsid { .. } => "network-wireless-symbolic",
            Self::GatewayMac { .. } => "network-wired-symbolic",
            Self::PingTarget { .. } => "network-server-symbolic",
            Self::InterfaceState { .. } => "network-wired-symbolic",
            Self::TimeWindow { .. } => "preferences-system-time-symbolic",
            Self::NetworkAvailable => "network-transmit-receive-symbolic",
            Self::Not { .. } => "dialog-error-symbolic",
        }
    }
}

/// How multiple conditions are combined.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleOperator {
    /// All conditions must match.
    #[default]
    And,
    /// Any condition must match.
    Or,
}

/// A set of conditions with an operator.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleSet {
    /// How conditions are combined.
    #[serde(default)]
    pub operator: RuleOperator,
    /// The conditions.
    pub conditions: Vec<Condition>,
    /// Enable auto-switch for this profile.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Priority (higher = evaluated first).
    #[serde(default)]
    pub priority: i32,
}

fn default_true() -> bool {
    true
}

impl RuleSet {
    /// Create a new empty rule set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a condition.
    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    /// Check if the rule set is empty.
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// Get the number of conditions.
    pub fn len(&self) -> usize {
        self.conditions.len()
    }
}

/// Result of evaluating a rule set.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    /// Whether all/any conditions matched (depending on operator).
    pub matched: bool,
    /// Individual condition results.
    pub condition_results: Vec<ConditionResult>,
    /// Evaluation duration.
    pub duration_ms: u64,
}

/// Result of evaluating a single condition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// The condition that was evaluated.
    pub condition: Condition,
    /// Whether the condition matched.
    pub matched: bool,
    /// Optional detail message.
    pub detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window_simple() {
        let window = TimeWindow::new(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );

        // 10:00 should be in window
        assert!(window.is_active(
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            Weekday::Mon
        ));

        // 20:00 should be outside window
        assert!(!window.is_active(
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            Weekday::Mon
        ));
    }

    #[test]
    fn test_time_window_overnight() {
        let window = TimeWindow::new(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        );

        // 23:00 should be in window
        assert!(window.is_active(
            NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
            Weekday::Mon
        ));

        // 02:00 should be in window
        assert!(window.is_active(
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
            Weekday::Mon
        ));

        // 12:00 should be outside window
        assert!(!window.is_active(
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            Weekday::Mon
        ));
    }
}
