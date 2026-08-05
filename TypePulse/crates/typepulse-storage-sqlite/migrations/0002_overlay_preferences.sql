ALTER TABLE app_preferences
    ADD COLUMN overlay_enabled INTEGER NOT NULL DEFAULT 1
    CHECK(overlay_enabled IN (0, 1));

ALTER TABLE app_preferences
    ADD COLUMN overlay_position TEXT NOT NULL DEFAULT 'top-right'
    CHECK(overlay_position IN ('top-left', 'top-right', 'bottom-left', 'bottom-right'));

ALTER TABLE app_preferences
    ADD COLUMN overlay_size TEXT NOT NULL DEFAULT 'medium'
    CHECK(overlay_size IN ('small', 'medium', 'large'));

ALTER TABLE app_preferences
    ADD COLUMN overlay_content TEXT NOT NULL DEFAULT 'both'
    CHECK(overlay_content IN ('wpm', 'animation', 'both'));
