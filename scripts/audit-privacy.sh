#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/QRY"

node "$script_dir/audit-privacy.mjs"

cd "$app_root"
cargo test -p typepulse-storage-sqlite schema_has_no_sensitive_input_columns
cargo test -p typepulse-app dto_contains_only_daily_aggregates
cargo test -p typepulse-app permission_dto_contains_only_status_and_platform_capabilities
cargo test -p typepulse-app gate_dto_exposes_only_permission_lifecycle_state
cargo test -p typepulse-app menu_bar_preference_dto_contains_only_the_visibility_flag
cargo test -p typepulse-app preference_dto_contains_only_visual_configuration
cargo test -p typepulse-app overlay_event_dto_contains_only_aggregate_presentation_state

printf 'Privacy audit passed: no runtime event logs, narrow capability, aggregate-only schema and DTOs.\n'
