#!/bin/sh
# VietKB in-app update helper (macOS). Started detached by the app, which then
# quits. Everything arrives as an argument; nothing is interpolated into this
# text, so paths with spaces or quotes are safe.
#
# usage: apply-update.sh PID DMG BUNDLE VERSION BUNDLE_ID LOG SHA256
#   PID        process to wait for (the running VietKB)
#   DMG        the downloaded VietKB_<version>_universal.dmg
#   BUNDLE     the installed .app to replace
#   VERSION    the CFBundleShortVersionString the new app must report
#   BUNDLE_ID  the CFBundleIdentifier the new app must have
#   LOG        file the steps are appended to
#   SHA256     the hash the dmg must have (re-checked right before mounting)
PID=$1
DMG=$2
BUNDLE=$3
WANT_VERSION=$4
WANT_ID=$5
LOG=$6
WANT_SHA=$7

MNT=""
NEW=""
OLD=""

log() {
  printf '%s %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*" >>"$LOG" 2>/dev/null
}

detach() {
  if [ -n "$MNT" ]; then
    hdiutil detach -quiet "$MNT" >>"$LOG" 2>&1 || hdiutil detach -quiet -force "$MNT" >>"$LOG" 2>&1
    rmdir "$MNT" 2>/dev/null
    MNT=""
  fi
}

# Put things back as found, then bring the old app back up.
fail() {
  log "FAIL: $*"
  if [ -n "$OLD" ] && [ -e "$OLD" ] && [ ! -e "$BUNDLE" ]; then
    mv "$OLD" "$BUNDLE" && log "restored the old bundle"
  fi
  [ -n "$NEW" ] && rm -rf "$NEW"
  detach
  if [ -e "$BUNDLE" ]; then
    open "$BUNDLE" >>"$LOG" 2>&1 && log "reopened the old app"
  fi
  exit 1
}

log "start: pid=$PID dmg=$DMG bundle=$BUNDLE want=$WANT_VERSION"

# Whatever happens below, never leave the volume mounted.
trap detach EXIT
trap 'detach; exit 1' INT TERM HUP

# 1. Wait for the app to exit (30 s). Never replace a bundle that is running.
i=0
while kill -0 "$PID" 2>/dev/null; do
  i=$((i + 1))
  if [ "$i" -gt 60 ]; then
    sleep 1
    if kill -0 "$PID" 2>/dev/null; then
      log "FAIL: pid $PID still running after 30 s; nothing was changed"
    else
      log "FAIL: pid $PID exited only after the 30 s wait; nothing was changed, reopening the old app"
      [ -e "$BUNDLE" ] && open "$BUNDLE" >>"$LOG" 2>&1
    fi
    exit 1
  fi
  sleep 0.5
done
log "app exited"

[ -f "$DMG" ] || fail "dmg is missing"
[ -d "$BUNDLE" ] || fail "bundle is missing"
PARENT=$(dirname "$BUNDLE")
NAME=$(basename "$BUNDLE")

# The dmg must still be the bytes the app verified (it sat in a temp folder).
[ -n "$WANT_SHA" ] || fail "no expected SHA-256 was given"
GOT_SHA=$(shasum -a 256 "$DMG" | cut -d' ' -f1)
[ "$GOT_SHA" = "$WANT_SHA" ] || fail "dmg SHA-256 is $GOT_SHA, expected $WANT_SHA"
log "dmg SHA-256 re-checked"

# 2. Mount read-only, not shown in Finder.
MNT=$(mktemp -d "${TMPDIR:-/tmp}/vietkb-mount.XXXXXX") || fail "no temp mount point"
hdiutil attach -nobrowse -readonly -noautoopen -mountpoint "$MNT" "$DMG" >>"$LOG" 2>&1 \
  || fail "hdiutil attach failed"
log "mounted at $MNT"

# 3. The app inside must be ours and the version we were told.
SRC="$MNT/VietKB.app"
PLIST="$SRC/Contents/Info.plist"
[ -f "$PLIST" ] || fail "no VietKB.app in the dmg"
GOT_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$PLIST" 2>/dev/null)
GOT_VERSION=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$PLIST" 2>/dev/null)
EXE=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$PLIST" 2>/dev/null)
log "dmg app: id=$GOT_ID version=$GOT_VERSION exe=$EXE"
[ -n "$EXE" ] || fail "no CFBundleExecutable in the dmg app"
[ "$GOT_ID" = "$WANT_ID" ] || fail "bundle id '$GOT_ID' is not '$WANT_ID'"
[ "$GOT_VERSION" = "$WANT_VERSION" ] || fail "version '$GOT_VERSION' is not '$WANT_VERSION'"

# 4. Copy next to the target (same volume, so the final mv is a rename).
NEW="$PARENT/.$NAME.new.$$"
OLD="$PARENT/.$NAME.old.$$"
rm -rf "$NEW" "$OLD"
ditto "$SRC" "$NEW" >>"$LOG" 2>&1 || fail "ditto failed"
[ -x "$NEW/Contents/MacOS/$EXE" ] || fail "copied bundle has no executable"
# Warning only, never a failure: release builds carry a self-signed (before
# 0.2.9: ad-hoc) signature, which can fail strict verification on good copies.
if ! codesign --verify --deep --strict "$NEW" >>"$LOG" 2>&1; then
  log "warning: codesign --verify failed on the copy (not fatal)"
fi
log "copied to $NEW"

# 5. Swap: old aside, new in place; on any failure the old one comes back.
mv "$BUNDLE" "$OLD" || fail "could not move the old bundle aside"
if ! mv "$NEW" "$BUNDLE"; then
  # $NEW stays set so fail removes the half-moved copy and restores the old one.
  fail "could not move the new bundle into place"
fi
NEW=""
log "swapped"

# Our own download has no quarantine mark; clear one if anything added it.
xattr -dr com.apple.quarantine "$BUNDLE" >>"$LOG" 2>&1
rm -rf "$OLD"
log "removed the old bundle (rc=$?)"
OLD=""
detach
rm -f "$DMG"
log "cleaned up; opening the new app"

open "$BUNDLE" >>"$LOG" 2>&1 || { log "FAIL: open failed"; exit 1; }
log "done"
exit 0
