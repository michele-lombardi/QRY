ALTER TABLE app_preferences
    ADD COLUMN overlay_background_enabled INTEGER NOT NULL DEFAULT 1
    CHECK(overlay_background_enabled IN (0, 1));
