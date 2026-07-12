#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source "${HOME}/.cargo/env" 2>/dev/null || true
cargo run --release -p experiment-runner -- all
