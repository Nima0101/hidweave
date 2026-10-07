#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo run --release --locked --offline -- demo >/dev/null
cargo run --locked --offline --example consumer >/dev/null
python3 scripts/check_json.py
python3 scripts/check_consumer.py
if [ -x integrations/scripts/verify-local.sh ]; then integrations/scripts/verify-local.sh; fi
