#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/QRY"

log_hits="$(
  rg -n 'print!|println!|eprint!|eprintln!|dbg!|tracing::|log::|console\.' \
    "$app_root/crates" "$app_root/src-tauri/src" "$app_root/src" || true
)"
unexpected_logs="$(
  printf '%s\n' "$log_hits" \
    | sed '/typing callback hot-path reference/d' \
    | sed '/^[[:space:]]*$/d'
)"
if [[ -n "$unexpected_logs" ]]; then
  printf 'Unexpected runtime logging found:\n%s\n' "$unexpected_logs" >&2
  exit 1
fi

node - "$app_root/src-tauri/capabilities/default.json" <<'NODE'
const fs = require("node:fs");
const capability = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const expected = ["core:default"];
if (JSON.stringify(capability.permissions) !== JSON.stringify(expected)) {
  throw new Error(`unexpected Tauri permissions: ${capability.permissions}`);
}
NODE

cd "$app_root"
cargo test -p typepulse-storage-sqlite schema_has_no_sensitive_input_columns
cargo test -p typepulse-app dto_contains_only_daily_aggregates
cargo test -p typepulse-app permission_dto_contains_only_the_status
cargo test -p typepulse-app menu_bar_preference_dto_contains_only_the_visibility_flag
cargo test -p typepulse-app preference_dto_contains_only_visual_configuration
cargo test -p typepulse-app overlay_event_dto_contains_only_aggregate_presentation_state

printf 'Privacy audit passed: no runtime event logs, narrow capability, aggregate-only schema and DTOs.\n'
