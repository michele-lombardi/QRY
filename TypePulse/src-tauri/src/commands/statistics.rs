//! Aggregate statistics and CSV commands.

use serde::Serialize;
use tauri::State;
use typepulse_core::{export_daily_csv, DailySummary};

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

    use super::DailySummaryDto;

    #[test]
    fn dto_contains_only_daily_aggregates() {
        let dto = DailySummaryDto::from(DailySummary::empty(LocalDate::new(2026, 8, 5).unwrap()));
        assert_eq!(dto.date, "2026-08-05");
        assert_eq!(dto.session_count, 0);
    }
}
