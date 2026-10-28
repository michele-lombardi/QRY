ALTER TABLE app_preferences ADD COLUMN capsule_enabled INTEGER NOT NULL DEFAULT 0
    CHECK(capsule_enabled IN (0, 1));
ALTER TABLE app_preferences ADD COLUMN integrations_enabled INTEGER NOT NULL DEFAULT 0
    CHECK(integrations_enabled IN (0, 1));
ALTER TABLE app_preferences ADD COLUMN notch_pip_enabled INTEGER NOT NULL DEFAULT 0
    CHECK(notch_pip_enabled IN (0, 1));
