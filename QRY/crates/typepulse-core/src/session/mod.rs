//! Typing session state machine and aggregate models.

mod engine;
mod model;

pub use engine::{EngineError, TypingEngine};
pub use model::{
    ActiveSessionMetrics, EngineSnapshot, EngineUpdate, NewRecord, RecordKind, SessionPhase,
    SessionSummary, TypingRecords,
};
