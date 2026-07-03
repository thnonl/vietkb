#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
"$ROOT/scripts/build-linux.sh"
PREFIX="${PREFIX:-$HOME/.local}"
mkdir -p "$PREFIX/bin"
cp "$ROOT/dist/linux/vietkb-cli" "$PREFIX/bin/"
echo "Installed VietKB CLI to $PREFIX/bin"
echo "For IME integration, install files from packaging/linux for your distro/input method."
