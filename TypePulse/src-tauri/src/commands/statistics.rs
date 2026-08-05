//! Aggregate statistics and CSV commands.

use serde::Serialize;
use tauri::State;
use typepulse_core::{export_daily_csv, DailySummary, MetricBucketRecord};

use crate::app_state::DiagnosticState;

/// Serializable daily aggregate with no individual event data.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DailySummaryDto {
    date: String,
    estimated_character_count: u64,
    estimated_word_count: f64,
    average_wpm: f64,
    peak_wpm: f64,
    active_typing_seconds: f64,
    session_count: u64,
}

impl From<DailySummary> for DailySummaryDto {
    fn from(summary: DailySummary) -> Self {
        Self {
            date: summary.date.to_string(),
            estimated_character_count: summary.estimated_character_count,
            estimated_word_count: summary.estimated_word_count,
            average_wpm: summary.average_wpm,
            peak_wpm: summary.peak_wpm,
            active_typing_seconds: summary.active_typing_duration.as_secs_f64(),
            session_count: summary.session_count,
        }
    }
}

/// One aggregate chart point for the detailed daily view.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetricBucketDto {
    interval_start_unix_ms: i64,
    interval_duration_seconds: f64,
    estimated_character_count: u64,
    average_wpm: f64,
    peak_wpm: f64,
}

impl From<MetricBucketRecord> for MetricBucketDto {
    fn from(bucket: MetricBucketRecord) -> Self {
        Self {
            interval_start_unix_ms: bucket.interval_start_unix_ms,
            interval_duration_seconds: bucket.interval_duration.as_secs_f64(),
            estimated_character_count: bucket.estimated_character_count,
            average_wpm: bucket.average_wpm,
            peak_wpm: bucket.peak_wpm,
        }
    }
}

#[tauri::command]
pub(crate) fn today_summary(state: State<'_, DiagnosticState>) -> Result<DailySummaryDto, String> {
    state.today_summary().map(Into::into)
}

#[tauri::command]
pub(crate) fn recent_daily_summaries(
    days: usize,
    state: State<'_, DiagnosticState>,
) -> Result<Vec<DailySummaryDto>, String> {
    if !(1..=366).contains(&days) {
        return Err("days must be between 1 and 366".into());
    }
    state
        .recent_summaries(days)
        .map(|summaries| summaries.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub(crate) fn today_metric_buckets(
    state: State<'_, DiagnosticState>,
) -> Result<Vec<MetricBucketDto>, String> {
    state
        .today_buckets()
        .map(|buckets| buckets.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub(crate) fn export_daily_statistics_csv(
    days: usize,
    state: State<'_, DiagnosticState>,
) -> Result<String, String> {
    if !(1..=366).contains(&days) {
        return Err("days must be between 1 and 366".into());
    }
    state
        .recent_summaries(days)
        .map(|summaries| export_daily_csv(&summaries))
}

#[tauri::command]
pub(crate) fn reset_today_statistics(state: State<'_, DiagnosticState>) -> Result<(), String> {
    state.reset_today()
}

#[cfg(test)]
mod tests {
    use typepulse_core::{DailySummary, LocalDate};

    use super::{DailySummaryDto, MetricBucketDto};

    #[test]
    fn dto_contains_only_daily_aggregates() {
        let dto = DailySummaryDto::from(DailySummary::empty(LocalDate::new(2026, 8, 5).unwrap()));
        assert_eq!(dto.date, "2026-08-05");
        assert_eq!(dto.session_count, 0);
    }

    #[test]
    fn bucket_dto_contains_only_aggregate_chart_data() {
        let dto = MetricBucketDto::from(typepulse_core::MetricBucketRecord {
            local_date: LocalDate::new(2026, 8, 5).unwrap(),
            interval_start_unix_ms: 1_000,
            interval_duration: std::time::Duration::from_secs(60),
            estimated_character_count: 10,
            average_wpm: 42.0,
            peak_wpm: 51.0,
        });
        assert_eq!(dto.interval_start_unix_ms, 1_000);
        assert_eq!(dto.estimated_character_count, 10);
    }
}
