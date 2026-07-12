#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source "${HOME}/.cargo/env" 2>/dev/null || true
cargo build --release -p godot-bridge
mkdir -p godot/bin
cp target/release/libgodot_bridge.so godot/bin/ 2>/dev/null || \
  cp target/release/libgodot_bridge.dylib godot/bin/ 2>/dev/null || true
echo "Godot extension built. Open godot/project.godot in Godot 4.x"
