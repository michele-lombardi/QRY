//! Stable CSV export for daily aggregate summaries.

use std::fmt::Write;

use crate::DailySummary;

/// Exports summaries in chronological order using locale-independent decimals.
#[must_use]
pub fn export_daily_csv(summaries: &[DailySummary]) -> String {
    let mut ordered = summaries.to_vec();
    ordered.sort_by_key(|summary| summary.date);
    let mut output = String::from("date,estimated_words,average_wpm,peak_wpm,typing_minutes\n");
    for summary in ordered {
        let typing_minutes = summary.active_typing_duration.as_secs_f64() / 60.0;
        let _ = writeln!(
            output,
            "{},{:.2},{:.1},{:.1},{:.2}",
            summary.date,
            summary.estimated_word_count,
            summary.average_wpm,
            summary.peak_wpm,
            typing_minutes
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{export_daily_csv, DailySummary, LocalDate};

    #[test]
    fn export_has_stable_header_decimals_and_order() {
        let later = DailySummary {
            date: LocalDate::new(2026, 8, 5).unwrap(),
            estimated_character_count: 19_200,
            estimated_word_count: 3_840.0,
            average_wpm: 58.25,
            peak_wpm: 104.0,
            active_typing_duration: Duration::from_secs(3_120),
            session_count: 2,
        };
        let earlier = DailySummary::empty(LocalDate::new(2026, 8, 4).unwrap());
        let csv = export_daily_csv(&[later, earlier]);
        assert_eq!(
            csv,
            concat!(
                "date,estimated_words,average_wpm,peak_wpm,typing_minutes\n",
                "2026-08-04,0.00,0.0,0.0,0.00\n",
                "2026-08-05,3840.00,58.2,104.0,52.00\n"
            )
        );
    }
}
