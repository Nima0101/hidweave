#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --locked --offline
python3 integrations/tests/test_gate.py
