#!/usr/bin/env bash
# Package digital-protocell-phase1-v1 headless Linux research runtime.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/experiments/generated/d087/linux_runtime/digital-protocell-phase1-v1}"
mkdir -p "$OUT"
cd "$ROOT"
cargo build -p phase1-certifier --bin digital-protocell-phase1 --release
cp -f target/release/digital-protocell-phase1 "$OUT/"
cat > "$OUT/README.txt" <<'EOF'
digital-protocell-phase1-v1
Headless Phase 1 research runtime (no network, no GPU).

  ./digital-protocell-phase1 --steps 1000 --out run.json
  ./digital-protocell-phase1 --resume run.snapshot.json --steps 500 --out run2.json

Fails closed on incompatible snapshots.
EOF
echo "packaged -> $OUT"
