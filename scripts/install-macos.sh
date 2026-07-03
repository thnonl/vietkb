#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
"$ROOT/scripts/build-macos.sh"
PREFIX="${PREFIX:-$HOME/.local}"
mkdir -p "$PREFIX/bin"
cp "$ROOT/dist/macos/vietkb-gui" "$PREFIX/bin/"
echo "Installed VietKB GUI to $PREFIX/bin"
