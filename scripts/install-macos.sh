#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
"$ROOT/scripts/build-macos.sh"
PREFIX="${PREFIX:-$HOME/.local}"
mkdir -p "$PREFIX/bin"
cp "$ROOT/dist/macos/vietkb-cli" "$PREFIX/bin/"
echo "Installed VietKB CLI to $PREFIX/bin"
