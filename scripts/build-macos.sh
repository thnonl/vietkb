#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release --target-dir target/macos -p vietkb-gui
rm -rf dist/macos
mkdir -p dist/macos
cp target/macos/release/vietkb-gui dist/macos/
echo "Built: dist/macos"
