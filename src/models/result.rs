// Network Manager - Execution Results
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Execution result types for profile application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::actions::Action;

/// Status of a single execution step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Step is pending.
    Pending,
    /// Step is currently running.
    Running,
    /// Step completed successfully.
    Success,
    /// Step completed with warnings.
    Warning,
    /// Step partially succeeded (some actions failed).
    PartialSuccess,
    /// Step failed.
    Error,
    /// Step was skipped.
    Skipped,
}

#[allow(dead_code)]
impl StepStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::Warning | Self::PartialSuccess)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::PartialSuccess => "partial_success",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }
}

/// Result of a single action execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// The action that was executed.
    pub action: Action,
    /// Execution status.
    pub status: StepStatus,
    /// Human-readable message.
    pub message: String,
    /// Detailed error message (if error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Timestamp when execution started.
    pub started_at: DateTime<Utc>,
    /// Prior state captured for rollback (if reversible).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_state: Option<String>,
}

#[allow(dead_code)]
impl ActionResult {
    /// Create a success result.
    pub fn success(action: Action, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            action,
            status: StepStatus::Success,
            message: message.into(),
            error_detail: None,
            duration_ms,
            started_at: Utc::now(),
            prior_state: None,
        }
    }

    /// Create an error result.
    pub fn error(action: Action, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            action,
            status: StepStatus::Error,
            message: message.into(),
            error_detail: detail,
            duration_ms: 0,
            started_at: Utc::now(),
            prior_state: None,
        }
    }

    /// Create a warning result.
    pub fn warning(action: Action, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            action,
            status: StepStatus::Warning,
            message: message.into(),
            error_detail: None,
            duration_ms,
            started_at: Utc::now(),
            prior_state: None,
        }
    }

    /// Create a skipped result.
    pub fn skipped(action: Action, reason: impl Into<String>) -> Self {
        Self {
            action,
            status: StepStatus::Skipped,
            message: reason.into(),
            error_detail: None,
            duration_ms: 0,
            started_at: Utc::now(),
            prior_state: None,
        }
    }
}

/// Overall result of profile execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Profile ID that was executed.
    pub profile_id: String,
    /// Profile name.
    pub profile_name: String,
    /// Overall status.
    pub status: StepStatus,
    /// Summary message.
    pub message: String,
    /// Individual action results.
    pub actions: Vec<ActionResult>,
    /// Total execution duration in milliseconds.
    pub total_duration_ms: u64,
    /// Execution start timestamp.
    pub started_at: DateTime<Utc>,
    /// Execution end timestamp.
    pub completed_at: DateTime<Utc>,
    /// User who initiated the execution (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiated_by: Option<String>,
}

#[allow(dead_code)]
impl ExecutionResult {
    /// Create a new execution result.
    pub fn new(profile_id: impl Into<String>, profile_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            profile_id: profile_id.into(),
            profile_name: profile_name.into(),
            status: StepStatus::Pending,
            message: String::new(),
            actions: Vec::new(),
            total_duration_ms: 0,
            started_at: now,
            completed_at: now,
            initiated_by: None,
        }
    }

    /// Add an action result.
    pub fn add_action(&mut self, result: ActionResult) {
        self.actions.push(result);
    }

    /// Finalize the result, calculating overall status.
    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
        self.total_duration_ms = (self.completed_at - self.started_at).num_milliseconds() as u64;

        // Determine overall status
        let has_errors = self.actions.iter().any(|a| a.status.is_error());
        let has_warnings = self.actions.iter().any(|a| a.status == StepStatus::Warning);

        if has_errors {
            self.status = StepStatus::Error;
            let error_count = self.actions.iter().filter(|a| a.status.is_error()).count();
            self.message = format!("{} action(s) failed", error_count);
        } else if has_warnings {
            self.status = StepStatus::Warning;
            self.message = "Completed with warnings".to_string();
        } else {
            self.status = StepStatus::Success;
            self.message = format!(
                "{} action(s) completed successfully",
                self.actions.iter().filter(|a| a.status == StepStatus::Success).count()
            );
        }
    }

    /// Count successful actions.
    pub fn success_count(&self) -> usize {
        self.actions.iter().filter(|a| a.status.is_success()).count()
    }

    /// Count failed actions.
    pub fn error_count(&self) -> usize {
        self.actions.iter().filter(|a| a.status.is_error()).count()
    }

    /// Check if execution was successful overall.
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}
