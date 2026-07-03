#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release --target-dir target/linux -p vietkb-gui
rm -rf dist/linux
mkdir -p dist/linux
cp target/linux/release/vietkb-gui dist/linux/
echo "Built: dist/linux"
