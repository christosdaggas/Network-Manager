// Network Manager - Schema Versioning
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Schema versioning for profile migration support.

use semver::Version;
use serde::{Deserialize, Serialize};

/// Current schema version for profiles.
pub const CURRENT_SCHEMA_VERSION: &str = "1.0.0";

/// Schema version wrapper for serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Create a new schema version.
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Get the current schema version.
    pub fn current() -> Self {
        Self(CURRENT_SCHEMA_VERSION.to_string())
    }

    /// Parse the version string.
    pub fn parse(&self) -> Option<Version> {
        Version::parse(&self.0).ok()
    }

    /// Check if this version is compatible with the current version.
    pub fn is_compatible(&self) -> bool {
        if let (Some(this), Some(current)) = (self.parse(), Version::parse(CURRENT_SCHEMA_VERSION).ok()) {
            // Major version must match, minor can be lower or equal
            this.major == current.major && this.minor <= current.minor
        } else {
            false
        }
    }

    /// Check if migration is needed.
    pub fn needs_migration(&self) -> bool {
        self.0 != CURRENT_SCHEMA_VERSION && self.is_compatible()
    }

    /// Get the version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_current() {
        let v = SchemaVersion::current();
        assert_eq!(v.as_str(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_schema_version_compatible() {
        let current = SchemaVersion::current();
        assert!(current.is_compatible());

        let older = SchemaVersion::new("1.0.0");
        assert!(older.is_compatible());

        let incompatible = SchemaVersion::new("2.0.0");
        // Will be incompatible when we're at 1.x
        if CURRENT_SCHEMA_VERSION.starts_with("1.") {
            assert!(!incompatible.is_compatible());
        }
    }
}
