#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/TypePulse"

if [[ ! -d "$app_root/node_modules" ]]; then
  echo "Missing TypePulse/node_modules. Run: cd TypePulse && npm install" >&2
  exit 1
fi

cd "$app_root"

npm run format:check
npm run lint
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo check --workspace --all-targets

"$project_root/scripts/audit-privacy.sh"
