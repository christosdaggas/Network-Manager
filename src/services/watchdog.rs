// Network Manager - Connection Watchdog Service
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Connection watchdog that monitors network connectivity.
//!
//! Periodically pings a target and takes action when connectivity is lost.

use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::models::{WatchdogAction, WatchdogConfig};

/// Watchdog service for monitoring connectivity.
#[allow(dead_code)]
pub struct WatchdogService {
    running: Arc<AtomicBool>,
    failure_count: Arc<AtomicU32>,
    config: WatchdogConfig,
}

#[allow(dead_code)]
impl WatchdogService {
    /// Create a new watchdog service.
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            failure_count: Arc::new(AtomicU32::new(0)),
            config,
        }
    }

    /// Update the configuration.
    pub fn update_config(&mut self, config: WatchdogConfig) {
        self.config = config;
    }

    /// Check connectivity by pinging the target.
    pub fn check_connectivity(&self) -> bool {
        let target = &self.config.ping_target;
        
        // Use system ping command with timeout
        let result = Command::new("ping")
            .args(["-c", "1", "-W", "3", target])
            .output();

        match result {
            Ok(output) => output.status.success(),
            Err(e) => {
                debug!("Ping command failed: {}", e);
                false
            }
        }
    }

    /// Perform a single watchdog check.
    /// Returns the action to take, if any.
    pub fn check(&self) -> Option<WatchdogAction> {
        if !self.config.enabled {
            return None;
        }

        let connected = self.check_connectivity();
        
        if connected {
            // Reset failure count on success
            self.failure_count.store(0, Ordering::SeqCst);
            debug!("Watchdog: connectivity OK");
            None
        } else {
            // Increment failure count
            let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
            warn!("Watchdog: connectivity check failed ({}/{})", count, self.config.failure_threshold);
            
            if count >= self.config.failure_threshold {
                // Reset counter and return action
                self.failure_count.store(0, Ordering::SeqCst);
                info!("Watchdog: threshold reached, taking action: {:?}", self.config.failure_action);
                Some(self.config.failure_action)
            } else {
                None
            }
        }
    }

    /// Execute the specified watchdog action.
    pub fn execute_action(&self, action: WatchdogAction) -> Result<(), String> {
        match action {
            WatchdogAction::Notify => {
                // Notification is handled by the UI layer
                info!("Watchdog: sending notification");
                Ok(())
            }
            WatchdogAction::Reconnect => {
                info!("Watchdog: attempting to reconnect");
                // Try to restart NetworkManager connection
                let result = Command::new("nmcli")
                    .env("LC_ALL", "C")
                    .args(["networking", "off"])
                    .status();
                
                std::thread::sleep(Duration::from_secs(2));
                
                let result2 = Command::new("nmcli")
                    .env("LC_ALL", "C")
                    .args(["networking", "on"])
                    .status();
                
                if result.is_ok() && result2.is_ok() {
                    Ok(())
                } else {
                    Err("Failed to restart networking".to_string())
                }
            }
            WatchdogAction::SwitchProfile => {
                // Profile switch is handled by the caller (needs profile ID)
                info!("Watchdog: requesting profile switch");
                Ok(())
            }
            WatchdogAction::RestartNetworkManager => {
                info!("Watchdog: restarting NetworkManager");
                let result = Command::new("systemctl")
                    .args(["restart", "NetworkManager"])
                    .status();
                
                match result {
                    Ok(status) if status.success() => Ok(()),
                    Ok(_) => Err("NetworkManager restart failed".to_string()),
                    Err(e) => Err(format!("Failed to restart NetworkManager: {}", e)),
                }
            }
        }
    }

    /// Get the current failure count.
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::SeqCst)
    }

    /// Check if the service is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stop the service.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Get the check interval.
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.config.check_interval_secs as u64)
    }

    /// Get the fallback profile ID if configured.
    pub fn fallback_profile_id(&self) -> Option<&str> {
        self.config.fallback_profile_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_creation() {
        let config = WatchdogConfig::default();
        let watchdog = WatchdogService::new(config);
        assert!(!watchdog.is_running());
        assert_eq!(watchdog.failure_count(), 0);
    }
}
