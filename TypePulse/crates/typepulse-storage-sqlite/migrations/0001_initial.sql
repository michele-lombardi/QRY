CREATE TABLE completed_sessions (
    id INTEGER PRIMARY KEY,
    local_date TEXT NOT NULL CHECK(length(local_date) = 10),
    started_at_unix_ms INTEGER NOT NULL,
    ended_at_unix_ms INTEGER NOT NULL,
    estimated_character_count INTEGER NOT NULL CHECK(estimated_character_count >= 0),
    estimated_word_count REAL NOT NULL CHECK(estimated_word_count >= 0),
    average_wpm REAL NOT NULL CHECK(average_wpm >= 0),
    peak_wpm REAL NOT NULL CHECK(peak_wpm >= 0),
    active_typing_ms INTEGER NOT NULL CHECK(active_typing_ms >= 0),
    UNIQUE(started_at_unix_ms, ended_at_unix_ms)
);

CREATE INDEX completed_sessions_by_date
    ON completed_sessions(local_date, started_at_unix_ms);

CREATE TABLE metric_buckets (
    id INTEGER PRIMARY KEY,
    local_date TEXT NOT NULL CHECK(length(local_date) = 10),
    interval_start_unix_ms INTEGER NOT NULL UNIQUE,
    interval_duration_ms INTEGER NOT NULL CHECK(interval_duration_ms > 0),
    estimated_character_count INTEGER NOT NULL CHECK(estimated_character_count >= 0),
    average_wpm REAL NOT NULL CHECK(average_wpm >= 0),
    peak_wpm REAL NOT NULL CHECK(peak_wpm >= 0)
);

CREATE INDEX metric_buckets_by_date
    ON metric_buckets(local_date, interval_start_unix_ms);

CREATE TABLE app_preferences (
    singleton_id INTEGER PRIMARY KEY CHECK(singleton_id = 1),
    auto_start_enabled INTEGER NOT NULL CHECK(auto_start_enabled IN (0, 1))
);

INSERT INTO app_preferences(singleton_id, auto_start_enabled) VALUES (1, 0);
