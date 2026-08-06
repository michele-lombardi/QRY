ALTER TABLE app_preferences
    ADD COLUMN overlay_hide_delay_seconds INTEGER NOT NULL DEFAULT 5
    CHECK(overlay_hide_delay_seconds BETWEEN 1 AND 15);

CREATE TABLE typing_records (
    singleton_id INTEGER PRIMARY KEY CHECK(singleton_id = 1),
    peak_wpm REAL CHECK(peak_wpm IS NULL OR peak_wpm >= 0),
    sustained_30_wpm REAL CHECK(sustained_30_wpm IS NULL OR sustained_30_wpm >= 0),
    sustained_60_wpm REAL CHECK(sustained_60_wpm IS NULL OR sustained_60_wpm >= 0)
);

INSERT INTO typing_records(
    singleton_id, peak_wpm, sustained_30_wpm, sustained_60_wpm
)
SELECT 1, MAX(peak_wpm), NULL, NULL FROM completed_sessions;
