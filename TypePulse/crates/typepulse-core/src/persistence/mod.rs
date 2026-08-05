//! Aggregate-only persistence boundary and in-memory reference implementation.

mod csv;
mod model;
mod repository;

pub use csv::export_daily_csv;
pub use model::{AppPreferences, CompletedSessionRecord, DailySummary, MetricBucketRecord};
pub use repository::{
    InMemoryStatisticsRepository, RepositoryError, RepositoryErrorKind, StatisticsRepository,
};
