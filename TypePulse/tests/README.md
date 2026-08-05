# Tests

Rust unit tests live beside the code they verify. This directory contains cross-crate
fixtures and manual platform checklists that cannot run reliably in CI.

Automated tests must not require Input Monitoring permission or a live global keyboard
listener.
