// Network Manager - Profile Scheduler
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Profile scheduling service.
//!
//! This module provides a scheduler that checks scheduled profile activations
//! and triggers them at the appropriate times.

use crate::models::ScheduleEntry;
use chrono::{DateTime, Datelike, TimeDelta, Timelike, Utc};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Longest gap between ticks that is caught up minute by minute.
///
/// Longer gaps (suspend/resume, large clock jumps) only evaluate the current
/// minute, so a backlog of activations is not replayed at once.
const MAX_CATCH_UP_MINUTES: i64 = 10;

/// Scheduler service for timed profile activations.
#[allow(dead_code)]
pub struct SchedulerService {
    running: Arc<AtomicBool>,
}

impl Default for SchedulerService {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl SchedulerService {
    /// Create a new scheduler service.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if a schedule should trigger at the given (local) time.
    /// 
    /// Cron format: minute hour day-of-month month day-of-week
    /// Supports: * for any, specific numbers, ranges (1-5), lists (1,3,5)
    pub fn should_trigger_at<T: Datelike + Timelike>(schedule: &ScheduleEntry, at: &T) -> bool {
        if !schedule.enabled {
            return false;
        }

        let parts: Vec<&str> = schedule.cron_expression.split_whitespace().collect();
        
        if parts.len() != 5 {
            return false;
        }

        let minute_match = Self::matches_field(parts[0], at.minute());
        let hour_match = Self::matches_field(parts[1], at.hour());
        let dom_match = Self::matches_field(parts[2], at.day());
        let month_match = Self::matches_field(parts[3], at.month());
        let dow_match = Self::matches_field(parts[4], at.weekday().num_days_from_sunday());

        minute_match && hour_match && dom_match && month_match && dow_match
    }

    /// Check if a cron field matches a value.
    fn matches_field(field: &str, value: u32) -> bool {
        if field == "*" {
            return true;
        }

        // Handle lists (e.g., "1,3,5")
        if field.contains(',') {
            return field.split(',')
                .any(|part| Self::matches_field(part.trim(), value));
        }

        // Handle ranges (e.g., "1-5")
        if field.contains('-') {
            let range: Vec<&str> = field.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<u32>(), range[1].parse::<u32>()) {
                    return value >= start && value <= end;
                }
            }
            return false;
        }

        // Handle step values (e.g., "*/5")
        if field.starts_with("*/") {
            if let Ok(step) = field[2..].parse::<u32>() {
                return step > 0 && value % step == 0;
            }
            return false;
        }

        // Simple number
        if let Ok(num) = field.parse::<u32>() {
            return value == num;
        }

        false
    }

    /// Work out which minutes a scheduler tick must evaluate.
    ///
    /// Returns the minutes after `last` up to and including `now`, and the new
    /// "last evaluated" minute. A delayed tick is caught up minute by minute;
    /// the first tick and long gaps only evaluate the current minute. A small
    /// backwards clock correction evaluates nothing, so no minute fires twice.
    pub fn minutes_to_evaluate(
        last: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> (Vec<DateTime<Utc>>, DateTime<Utc>) {
        let now = Self::minute_floor(now);
        let Some(last) = last else {
            return (vec![now], now);
        };

        let gap = (now - last).num_minutes();
        if gap == 0 || (-MAX_CATCH_UP_MINUTES..0).contains(&gap) {
            (Vec::new(), last)
        } else if (1..=MAX_CATCH_UP_MINUTES).contains(&gap) {
            let minutes = (1..=gap).map(|i| last + TimeDelta::minutes(i)).collect();
            (minutes, now)
        } else {
            (vec![now], now)
        }
    }

    /// Schedules due at any of the given (local) minutes, each listed once.
    pub fn due_schedules<'a, T: Datelike + Timelike>(
        schedules: &'a [ScheduleEntry],
        minutes: &[T],
    ) -> Vec<&'a ScheduleEntry> {
        schedules.iter()
            .filter(|s| minutes.iter().any(|m| Self::should_trigger_at(s, m)))
            .collect()
    }

    /// Disable the one-shot entries whose IDs are in `fired_ids`.
    /// Returns true if any entry changed.
    pub fn disable_one_shots(schedules: &mut [ScheduleEntry], fired_ids: &[String]) -> bool {
        let mut changed = false;
        for schedule in schedules.iter_mut() {
            if schedule.one_shot && schedule.enabled && fired_ids.contains(&schedule.id) {
                schedule.enabled = false;
                changed = true;
            }
        }
        changed
    }

    /// Truncate a time to the start of its minute.
    fn minute_floor(time: DateTime<Utc>) -> DateTime<Utc> {
        time.with_second(0)
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(time)
    }

    /// Stop the scheduler.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if the scheduler is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Parse a simple time string (HH:MM) into hour and minute.
#[allow(dead_code)]
pub fn parse_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
    }
    None
}

/// Create a cron expression for a specific time every day.
#[allow(dead_code)]
pub fn cron_daily_at(hour: u32, minute: u32) -> String {
    format!("{} {} * * *", minute, hour)
}

/// Create a cron expression for specific weekdays at a time.
/// days: comma-separated day numbers (0=Sunday, 1=Monday, etc.)
#[allow(dead_code)]
pub fn cron_weekdays_at(hour: u32, minute: u32, days: &str) -> String {
    format!("{} {} * * {}", minute, hour, days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_wildcard() {
        assert!(SchedulerService::matches_field("*", 5));
        assert!(SchedulerService::matches_field("*", 59));
    }

    #[test]
    fn test_matches_number() {
        assert!(SchedulerService::matches_field("5", 5));
        assert!(!SchedulerService::matches_field("5", 6));
    }

    #[test]
    fn test_matches_range() {
        assert!(SchedulerService::matches_field("1-5", 3));
        assert!(!SchedulerService::matches_field("1-5", 6));
    }

    #[test]
    fn test_matches_list() {
        assert!(SchedulerService::matches_field("1,3,5", 3));
        assert!(!SchedulerService::matches_field("1,3,5", 2));
    }

    #[test]
    fn test_matches_step() {
        assert!(SchedulerService::matches_field("*/5", 0));
        assert!(SchedulerService::matches_field("*/5", 15));
        assert!(!SchedulerService::matches_field("*/5", 7));
    }

    fn utc(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(2026, 10, 22, hour, minute, second).unwrap()
    }

    fn entry(id: &str, cron: &str, one_shot: bool) -> ScheduleEntry {
        ScheduleEntry {
            id: id.to_string(),
            profile_id: format!("profile-{}", id),
            cron_expression: cron.to_string(),
            enabled: true,
            one_shot,
            description: None,
        }
    }

    #[test]
    fn test_first_tick_evaluates_current_minute() {
        let (minutes, last) = SchedulerService::minutes_to_evaluate(None, utc(9, 0, 42));
        assert_eq!(minutes, vec![utc(9, 0, 0)]);
        assert_eq!(last, utc(9, 0, 0));
    }

    #[test]
    fn test_same_minute_is_not_evaluated_twice() {
        let (minutes, last) = SchedulerService::minutes_to_evaluate(Some(utc(9, 0, 0)), utc(9, 0, 50));
        assert!(minutes.is_empty());
        assert_eq!(last, utc(9, 0, 0));
    }

    #[test]
    fn test_delayed_tick_catches_up_skipped_minutes() {
        // Tick at 08:59:59, next one delayed to 09:01:00: 09:00 must not be lost.
        let (minutes, last) = SchedulerService::minutes_to_evaluate(Some(utc(8, 59, 0)), utc(9, 1, 0));
        assert_eq!(minutes, vec![utc(9, 0, 0), utc(9, 1, 0)]);
        assert_eq!(last, utc(9, 1, 0));
    }

    #[test]
    fn test_long_gap_only_evaluates_current_minute() {
        let (minutes, last) = SchedulerService::minutes_to_evaluate(Some(utc(1, 0, 0)), utc(9, 0, 5));
        assert_eq!(minutes, vec![utc(9, 0, 0)]);
        assert_eq!(last, utc(9, 0, 0));
    }

    #[test]
    fn test_small_backwards_correction_evaluates_nothing() {
        let (minutes, last) = SchedulerService::minutes_to_evaluate(Some(utc(9, 0, 0)), utc(8, 59, 58));
        assert!(minutes.is_empty());
        assert_eq!(last, utc(9, 0, 0));
    }

    #[test]
    fn test_due_schedules_lists_each_entry_once() {
        let schedules = vec![entry("every", "* * * * *", false), entry("nine", "0 9 * * *", false)];
        let minutes = [utc(8, 59, 0), utc(9, 0, 0)];
        let due = SchedulerService::due_schedules(&schedules, &minutes);
        let ids: Vec<&str> = due.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["every", "nine"]);
    }

    #[test]
    fn test_one_shot_is_disabled_after_firing() {
        let mut schedules = vec![entry("once", "0 9 * * *", true), entry("daily", "0 9 * * *", false)];
        let fired: Vec<String> = SchedulerService::due_schedules(&schedules, &[utc(9, 0, 0)])
            .iter()
            .filter(|s| s.one_shot)
            .map(|s| s.id.clone())
            .collect();
        assert_eq!(fired, vec!["once".to_string()]);

        assert!(SchedulerService::disable_one_shots(&mut schedules, &fired));
        assert!(!schedules[0].enabled);
        assert!(schedules[1].enabled);

        // Disabled one-shot entries are no longer due, and nothing changes again.
        let due = SchedulerService::due_schedules(&schedules, &[utc(9, 0, 0)]);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "daily");
        assert!(!SchedulerService::disable_one_shots(&mut schedules, &fired));
    }
}
