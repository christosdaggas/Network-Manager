// Network Manager - Script Sandboxing
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Script execution with sandboxing support.
//!
//! Provides isolated script execution using bubblewrap or firejail.
//! When the user configures sandboxing, the required tool **must** be
//! present — falling back to unsandboxed execution is a security violation.

use std::process::{Command, Output};
use tracing::{debug, error, info};

use crate::models::SandboxMode;

/// Runner for executing scripts with optional sandboxing.
#[allow(dead_code)]
pub struct SandboxRunner {
    mode: SandboxMode,
}

impl Default for SandboxRunner {
    fn default() -> Self {
        Self::new(SandboxMode::None)
    }
}

#[allow(dead_code)]
impl SandboxRunner {
    /// Create a new sandbox runner with the specified mode.
    pub fn new(mode: SandboxMode) -> Self {
        Self { mode }
    }

    /// Update the sandbox mode.
    pub fn set_mode(&mut self, mode: SandboxMode) {
        self.mode = mode;
    }

    /// Get the current sandbox mode.
    pub fn mode(&self) -> SandboxMode {
        self.mode
    }

    /// Check if the required sandbox tool is available.
    pub fn is_available(&self) -> bool {
        match self.mode {
            SandboxMode::None => true,
            SandboxMode::Bubblewrap => Self::command_exists("bwrap"),
            SandboxMode::Firejail => Self::command_exists("firejail"),
        }
    }

    /// Check if a command exists in PATH.
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Execute a script with the configured sandboxing.
    pub fn execute(&self, script_path: &str, args: &[&str]) -> Result<Output, SandboxError> {
        if !std::path::Path::new(script_path).exists() {
            return Err(SandboxError::ScriptNotFound(script_path.to_string()));
        }

        match self.mode {
            SandboxMode::None => self.execute_direct(script_path, args),
            SandboxMode::Bubblewrap => self.execute_with_bubblewrap(script_path, args),
            SandboxMode::Firejail => self.execute_with_firejail(script_path, args),
        }
    }

    /// Execute script directly without sandboxing.
    fn execute_direct(&self, script_path: &str, args: &[&str]) -> Result<Output, SandboxError> {
        debug!("Executing script directly: {}", script_path);
        
        Command::new(script_path)
            .args(args)
            .output()
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))
    }

    /// Execute script with bubblewrap isolation.
    /// Returns an error if bwrap is not installed — never falls back silently.
    fn execute_with_bubblewrap(&self, script_path: &str, args: &[&str]) -> Result<Output, SandboxError> {
        if !Self::command_exists("bwrap") {
            error!("Bubblewrap (bwrap) is not installed but sandbox mode is set to Bubblewrap");
            return Err(SandboxError::SandboxNotAvailable(
                "bubblewrap (bwrap) is not installed. Install it or disable sandboxing.".to_string(),
            ));
        }

        info!("Executing script with bubblewrap: {}", script_path);

        // Build bubblewrap command with reasonable defaults.
        // Use --ro-bind-try for paths that may not exist on all distros.
        let mut cmd = Command::new("bwrap");
        cmd.args([
            // Bind essential directories read-only
            "--ro-bind", "/usr", "/usr",
            "--ro-bind", "/lib", "/lib",
        ]);
        // /lib64 may not exist on all distributions (e.g. Arch uses symlinks)
        if std::path::Path::new("/lib64").exists() {
            cmd.args(["--ro-bind", "/lib64", "/lib64"]);
        }
        cmd.args([
            "--ro-bind", "/bin", "/bin",
            "--ro-bind", "/sbin", "/sbin",
            // Optional system files: DNS, CA certificates (Fedora keeps them
            // in /etc/pki), the dynamic linker cache and alternatives symlinks.
            "--ro-bind-try", "/etc/resolv.conf", "/etc/resolv.conf",
            "--ro-bind-try", "/etc/pki", "/etc/pki",
            "--ro-bind-try", "/etc/ssl", "/etc/ssl",
            "--ro-bind-try", "/etc/ld.so.cache", "/etc/ld.so.cache",
            "--ro-bind-try", "/etc/alternatives", "/etc/alternatives",
            "--ro-bind", "/etc/hosts", "/etc/hosts",
            "--ro-bind", "/etc/passwd", "/etc/passwd",
            "--ro-bind", "/etc/group", "/etc/group",
            // Bind the script itself
            "--ro-bind", script_path, script_path,
            // Create necessary directories
            "--tmpfs", "/tmp",
            "--proc", "/proc",
            "--dev", "/dev",
            // Network access (needed for network scripts)
            "--share-net",
            // Disable new privileges
            "--new-session",
            // Unshare namespaces for isolation
            "--unshare-user",
            "--unshare-pid",
            "--unshare-ipc",
            // Die with parent
            "--die-with-parent",
            // The script to execute
            script_path,
        ]);
        
        // Add script arguments
        for arg in args {
            cmd.arg(arg);
        }

        cmd.output()
            .map_err(|e| SandboxError::ExecutionFailed(format!("bubblewrap: {}", e)))
    }

    /// Execute script with firejail isolation.
    /// Returns an error if firejail is not installed — never falls back silently.
    fn execute_with_firejail(&self, script_path: &str, args: &[&str]) -> Result<Output, SandboxError> {
        if !Self::command_exists("firejail") {
            error!("Firejail is not installed but sandbox mode is set to Firejail");
            return Err(SandboxError::SandboxNotAvailable(
                "firejail is not installed. Install it or disable sandboxing.".to_string(),
            ));
        }

        info!("Executing script with firejail: {}", script_path);

        let mut cmd = Command::new("firejail");
        cmd.args([
            // Use a restrictive profile
            "--quiet",
            "--noprofile",
            // Limit capabilities
            "--caps.drop=all",
            // No root access
            "--noroot",
            // Private /tmp
            "--private-tmp",
            // Read-only system
            "--read-only=/",
            // Allow network for network scripts
            // Disable dbus
            "--nodbus",
            // The script
            script_path,
        ]);
        
        // Add script arguments
        for arg in args {
            cmd.arg(arg);
        }

        cmd.output()
            .map_err(|e| SandboxError::ExecutionFailed(format!("firejail: {}", e)))
    }
}

/// Errors that can occur during sandboxed execution.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SandboxError {
    /// Script file not found.
    ScriptNotFound(String),
    /// Execution failed.
    ExecutionFailed(String),
    /// Sandbox tool not available.
    SandboxNotAvailable(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScriptNotFound(path) => write!(f, "Script not found: {}", path),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::SandboxNotAvailable(tool) => write!(f, "Sandbox tool not available: {}", tool),
        }
    }
}

impl std::error::Error for SandboxError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_runner_none() {
        let runner = SandboxRunner::new(SandboxMode::None);
        assert!(runner.is_available());
    }

    #[test]
    fn test_command_exists() {
        // 'sh' should exist on any Unix system
        assert!(SandboxRunner::command_exists("sh"));
        // Random non-existent command
        assert!(!SandboxRunner::command_exists("nonexistent_command_xyz"));
    }
}
