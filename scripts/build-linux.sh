#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release --target-dir target/linux -p vietkb-cli -p vietkb-linux-ibus -p vietkb-linux-fcitx5
mkdir -p dist/linux
cp target/linux/release/vietkb-cli dist/linux/
echo "Built: dist/linux"
