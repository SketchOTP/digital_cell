#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source "${HOME}/.cargo/env" 2>/dev/null || true
cargo build --release -p chemistry-core -p experiment-runner
