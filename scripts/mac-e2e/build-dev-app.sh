#!/bin/zsh
# Build the dev-channel VietKB bundle used by the app suite (identifier vn.vietkb.app.dev, config dir "VietKB Dev").
#
#   scripts/mac-e2e/build-dev-app.sh [--skip-build]     -> prints the .app path on the last line
#
# Same result as `cargo tauri build --features custom-protocol,dev-channel --config crates/vkb-app/tauri.dev.conf.json`
# (scripts/build-dev.ps1 on Windows) for hosts without `cargo tauri`: the UI is built with vite, the binary with cargo
# (TAURI_CONFIG carries the dev identity into the compiled config) and the bundle skeleton is assembled by hand, merging
# crates/vkb-app/Info.plist (LSUIElement) and ad-hoc signed. Output: target/dev-app/VietKB Dev.app (git-ignored).
# This NEVER produces or touches the owner's production bundle (identifier vn.vietkb.app).
set -eu
HERE=${0:A:h}; REPO=${HERE:h:h}
APPDIR=$REPO/target/dev-app/"VietKB Dev.app"
if [[ ${1:-} != --skip-build ]]; then
  [[ -d $REPO/ui/node_modules ]] || (cd $REPO/ui && npm ci >&2)
  (cd $REPO/ui && npm run build >&2)
  TAURI_CONFIG='{"identifier":"vn.vietkb.app.dev","productName":"VietKB Dev"}' \
    cargo build --release -p vkb-app --features custom-protocol,dev-channel --manifest-path $REPO/Cargo.toml >&2
fi
BIN=$REPO/target/release/vietkb
[[ -x $BIN ]] || { echo "missing $BIN" >&2; exit 1; }
VER=$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' $REPO/crates/vkb-app/tauri.conf.json | head -1)
rm -rf "$APPDIR"; mkdir -p "$APPDIR/Contents/MacOS" "$APPDIR/Contents/Resources"
cp $BIN "$APPDIR/Contents/MacOS/vietkb"
cp $REPO/crates/vkb-app/icons/icon.icns "$APPDIR/Contents/Resources/icon.icns"
cat > "$APPDIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>vietkb</string>
  <key>CFBundleIdentifier</key><string>vn.vietkb.app.dev</string>
  <key>CFBundleName</key><string>VietKB Dev</string>
  <key>CFBundleDisplayName</key><string>VietKB Dev</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VER</string>
  <key>CFBundleVersion</key><string>$VER</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSUIElement</key><true/>
</dict></plist>
EOF
codesign --force --deep -s - "$APPDIR" >&2
plutil -lint "$APPDIR/Contents/Info.plist" >&2
echo "$APPDIR"
