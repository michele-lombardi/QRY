ALTER TABLE app_preferences
    ADD COLUMN menu_bar_wpm_enabled INTEGER NOT NULL DEFAULT 1
    CHECK(menu_bar_wpm_enabled IN (0, 1));
