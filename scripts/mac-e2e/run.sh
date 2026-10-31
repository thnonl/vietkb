#!/bin/zsh
# VietKB macOS real-app E2E driver. Owns every process/config guardrail; vkbe2e.swift owns the input guardrails.
#
#   scripts/mac-e2e/run.sh [--app /path/VietKB.app] [--out DIR] [--phases telex,vni,perapp,app,static] [--targets a,b,c] [--dry]
#
# Phases: telex, vni, perapp  typing into real apps (need --app, EVKey is quit, input source -> ABC)
#         app                 CAP-1/2/3/4/14: dev-channel bundle via LaunchServices, Dock/windows/language/quit, AX only (--dev-app)
#         static              CAP-19/20: site-check.py + strings-audit.py, pure file reads, no gate, no process
# `app` and `static` post no keystrokes: EVKey and the input source are left alone, EVKey is never relaunched unless this run quit it.
#
# Guardrails (see README.md): idle >= 60 s gate; EVKey quit before / relaunched after (trap, always);
# input source ABC for the run and restored; VietKB launched DIRECTLY with HOME=<tmp> so the owner's
# ~/Library/Application Support/VietKB is never read or written (verified by manifest before/after);
# only pids launched here are killed; every KILL/LAUNCH/RELAUNCH is logged to guardrail.log.
set -u
HERE=${0:A:h}
REPO=${HERE:h:h}
APP=${APP:-/Users/nam/Workspace/vietkb/qa-build/VietKB.app}
STAMP=$(date +%Y%m%d-%H%M%S)
OUT=$REPO/docs/evkey-clone-work/qa/mac/run-$STAMP
PHASES=telex,vni,perapp
DEVAPP=${DEVAPP:-$REPO/target/dev-app/VietKB Dev.app}
APP_PHASES=main,badlang,hidden,single,update,quit   # `tap` (direct exec, inherits grants) is opt-in: an exec-launched Regular app leaves a stale Dock tile
IDLE_WAIT=300
TARGETS_TELEX=textedit-plain,textedit-rich,notes,stickies,chrome-textarea,chrome-omnibox,vscode,terminal
TARGETS_VNI=textedit-plain,textedit-rich,notes,stickies,chrome-textarea,vscode,terminal
DRY=0
ONLY=""
EXTRAS=1
while [[ $# -gt 0 ]]; do
  case $1 in
    --app) APP=$2; shift 2;;
    --out) OUT=$2; shift 2;;
    --phases) PHASES=$2; shift 2;;
    --targets-telex) TARGETS_TELEX=$2; shift 2;;
    --targets-vni) TARGETS_VNI=$2; shift 2;;
    --dry) DRY=1; shift;;
    --only) ONLY=$2; shift 2;;
    --no-extras) EXTRAS=0; shift;;
    --dev-app) DEVAPP=$2; shift 2;;
    --app-phases) APP_PHASES=$2; shift 2;;
    --idle-wait) IDLE_WAIT=$2; shift 2;;
    *) echo "unknown arg $1"; exit 2;;
  esac
done
BIN=$APP/Contents/MacOS/vietkb
TYPING=0; for ph in ${(s:,:)PHASES}; do case $ph in telex|vni|perapp) TYPING=1;; esac; done
mkdir -p "$OUT"
GLOG=$OUT/guardrail.log
TMPD=$(mktemp -d /tmp/vkbe2e.XXXXXX)   # scratch: Chrome/VS Code profiles, fake HOME, built harness
export E2E_OUT=$OUT E2E_TMP=$TMPD
glog() { print -r -- "$(date +%H:%M:%S) $*" | tee -a "$GLOG"; }

EV_WAS_RUNNING=0; EV_PATH=""; EV_PID=""
SRC_ORIG=""; VKPID=""
REAL_SUPPORT="$HOME/Library/Application Support"
manifest() {  # byte-level manifest of everything of the owner's that this run must not change
  { for d in "$REAL_SUPPORT/VietKB" "$REAL_SUPPORT/VietKB Dev" "$HOME/Library/LaunchAgents"; do
      if [[ -e $d ]]; then find "$d" -type f -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null; else echo "ABSENT $d"; fi
    done
    if [[ -e "$REAL_SUPPORT/EVKey" ]]; then find "$REAL_SUPPORT/EVKey" -type f -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null; fi
    defaults read com.apple.HIToolbox AppleSelectedInputSources 2>/dev/null | shasum -a 256
  }
}

cleanup() {
  trap - EXIT INT TERM HUP
  glog "CLEANUP start"
  if [[ -n $VKPID ]] && kill -0 $VKPID 2>/dev/null; then
    glog "KILL $(date +%H:%M:%S) pid=$VKPID path=$BIN reason=end of run (launched by this script)"; kill $VKPID; sleep 1; kill -9 $VKPID 2>/dev/null
  fi
  # dev-channel pids launched by the app suite (recorded by the harness): reap any survivor, matched by exact executable path
  if [[ -f $OUT/launched-pids.txt ]]; then
    for p in ${(f)"$(cat $OUT/launched-pids.txt)"}; do
      if kill -0 $p 2>/dev/null && [[ "$(ps -o comm= -p $p 2>/dev/null)" == "$DEVAPP/Contents/MacOS/vietkb" ]]; then
        glog "KILL $(date +%H:%M:%S) pid=$p path=$DEVAPP/Contents/MacOS/vietkb reason=survivor of the app suite (launched by it)"; kill $p; sleep 1; kill -9 $p 2>/dev/null
      fi
    done
  fi
  # TextEdit/Notes/Stickies the harness started are quit by the harness; belt and braces: nothing killed here.
  if [[ -n $SRC_ORIG ]]; then
    NOW=$($TMPD/vkbe2e inputsource 2>/dev/null | tail -1)
    if [[ $NOW != $SRC_ORIG ]]; then $TMPD/vkbe2e inputsource "$SRC_ORIG" >/dev/null 2>&1; glog "INPUT SOURCE restored $NOW -> $SRC_ORIG"; fi
  fi
  if [[ -f $OUT/manifest-before.txt ]]; then
  manifest > "$OUT/manifest-after.txt"
  if cmp -s "$OUT/manifest-before.txt" "$OUT/manifest-after.txt"; then glog "OWNER FILES manifest identical before/after (sha256 $(shasum -a 256 < $OUT/manifest-after.txt | cut -c1-16)...)"
  else glog "OWNER FILES manifest DIFFERS (see manifest-before/after.txt)"; fi
  fi
  if [[ -e "$REAL_SUPPORT/VietKB" ]]; then glog "NOTE real VietKB config dir exists: $REAL_SUPPORT/VietKB"; else glog "real VietKB config dir absent (never created): sha before=after=ABSENT"; fi
  if (( EV_WAS_RUNNING )); then
    if ! pgrep -x EVKey >/dev/null; then
      open -a EVKey 2>/dev/null; sleep 3
      if ! pgrep -x EVKey >/dev/null; then open "${EV_APP:-/Users/nam/Downloads/EVKeyMac/EVKey.app}"; sleep 3; fi
    fi
    glog "RELAUNCH $(date +%H:%M:%S) EVKey now pid=$(pgrep -x EVKey | head -1) (was pid=$EV_PID, path=$EV_PATH)"
  fi
  for a in TextEdit Notes Stickies "Google Chrome" Code; do pgrep -x "$a" >/dev/null && glog "NOTE still running after run: $a (pid $(pgrep -x "$a" | head -1)); check whether it was running before the run"; done
  rm -rf "$TMPD"
  glog "CLEANUP done"
}
trap cleanup EXIT INT TERM HUP

glog "RUN start $STAMP app=$APP out=$OUT"
glog "ENV $(sw_vers -productName) $(sw_vers -productVersion) build $(sw_vers -buildVersion) $(uname -m)"
for a in TextEdit Notes Stickies Terminal "Google Chrome" Code EVKey; do pgrep -x "$a" >/dev/null && glog "PRE running before the run: $a pid $(pgrep -x "$a" | head -1)"; done
if (( TYPING )); then
glog "BIN sha256=$(shasum -a 256 "$BIN" | cut -d' ' -f1) size=$(stat -f%z "$BIN") version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
codesign -dv "$APP" 2>&1 | grep -E "Identifier|Authority|Signature|TeamIdentifier" | head -4 | while read l; do glog "CODESIGN $l"; done
fi
if [[ ,$PHASES, == *,app,* ]]; then
  DEVBIN=$DEVAPP/Contents/MacOS/vietkb
  DEVID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$DEVAPP/Contents/Info.plist" 2>/dev/null)
  glog "DEVAPP path=$DEVAPP id=$DEVID sha256=$(shasum -a 256 "$DEVBIN" 2>/dev/null | cut -d' ' -f1) version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$DEVAPP/Contents/Info.plist" 2>/dev/null)"
  if [[ $DEVID != *.dev ]]; then glog "ABORT dev app identifier '$DEVID' is not a dev-channel build (guardrails section 6). Build it with scripts/mac-e2e/build-dev-app.sh"; trap - EXIT; rm -rf "$TMPD"; exit 2; fi
  for p in $(pgrep -f "$HOME/.*VietKB.app/Contents/MacOS/vietkb" 2>/dev/null); do glog "PRE owner/other VietKB running (observed only, never touched): pid $p $(ps -o comm= -p $p)"; done
fi

cp "$HERE/vkbe2e.swift" "$TMPD/main.swift"   # top-level code is only legal in main.swift when several files are compiled together
swiftc -O -swift-version 5 -o "$TMPD/vkbe2e" "$TMPD/main.swift" "$HERE/appsuite.swift" 2>&1 | grep -E "error" && { glog "harness build failed"; exit 1; }
H=$TMPD/vkbe2e
$H preflight | while read l; do glog "PREFLIGHT $l"; done
if [[ ,$PHASES, == *,static,* ]]; then
  glog "PHASE static (file reads only: site-check.py, strings-audit.py)"
  python3 -I "$HERE/site-check.py" --out "$OUT" 2>&1 | tee -a "$OUT/static.log" | grep -E "^\[site\] [0-9]+/|FAIL" | while read l; do glog "STATIC $l"; done
  python3 -I "$HERE/strings-audit.py" --out "$OUT" 2>&1 | tee -a "$OUT/static.log" | grep -E "^\[strings\] [0-9]+/|FAIL" | while read l; do glog "STATIC $l"; done
fi
if [[ $PHASES == static ]]; then $H report >/dev/null 2>&1; glog "RUN end (static only)"; exit 0; fi
SRC_ORIG=$($H inputsource | tail -1)
glog "INPUT SOURCE at start: $SRC_ORIG"
manifest > "$OUT/manifest-before.txt"; glog "OWNER FILES manifest saved (manifest-before.txt)"

idle_s() { ioreg -c IOHIDSystem | awk '/HIDIdleTime/ {printf "%d", $NF/1000000000; exit}'; }
IDLE=$(idle_s)
glog "HID idle ${IDLE}s"
# wait (up to 5 min) for the owner to be idle >= 60 s; nothing has been touched yet
for i in {1..$(( IDLE_WAIT / 5 ))}; do (( IDLE >= 60 )) && break; sleep 5; IDLE=$(idle_s); done
glog "HID idle ${IDLE}s at gate"
if (( IDLE < 60 )); then glog "ABORT owner not idle (< 60 s). Nothing touched."; trap - EXIT; rm -rf "$TMPD"; exit 4; fi
(( DRY )) && { glog "dry run: stopping after preflight"; exit 0; }

# --- input source -> ABC for the run
if (( TYPING )) && [[ $SRC_ORIG != com.apple.keylayout.ABC ]]; then $H inputsource com.apple.keylayout.ABC >/dev/null; glog "INPUT SOURCE switched to ABC (restored on exit)"; fi

# --- EVKey: quit (owner-approved in mac-handoff.md), relaunch in cleanup
EV_PID=$(pgrep -x EVKey | head -1)
if (( TYPING )) && [[ -n $EV_PID ]]; then
  EV_WAS_RUNNING=1
  EV_PATH=$(ps -o comm= -p $EV_PID)
  glog "KILL $(date +%H:%M:%S) pid=$EV_PID path=$EV_PATH reason=EVKey is a Vietnamese IME and would double-process keys; owner-approved in mac-handoff.md; relaunch follows"
  osascript -e 'with timeout of 5 seconds' -e 'quit app "EVKey"' -e 'end timeout' >/dev/null 2>&1
  for i in {1..10}; do pgrep -x EVKey >/dev/null || break; sleep 0.5; done
  if pgrep -x EVKey >/dev/null; then glog "KILL fallback pkill -x EVKey"; pkill -x EVKey; sleep 1; fi
  pgrep -x EVKey >/dev/null && { glog "EVKey still running, abort"; exit 1; }
fi

run_phase() {  # name method targets [extra harness args]
  local name=$1 method=$2 targets=$3; shift 3
  glog "PHASE $name method=$method"
  local HOMEDIR=$TMPD/home-$name
  local CFGDIR="$HOMEDIR/Library/Application Support/VietKB"
  mkdir -p "$CFGDIR"
  local EXTRA=""
  [[ $name == perapp ]] && EXTRA=$'\n[apps]\nremember_per_app = false\n[[apps.rules]]\nprocess = "com.apple.stickies"\nenabled = false\n'
  cat > "$CFGDIR/config.toml" <<EOF
version = 1
[input]
enabled = true
method = "$method"
[system]
start_with_windows = false
start_minimized = false
notification_sound = true
check_updates = false
language = "vi"
theme = "dark"
$EXTRA
EOF
  glog "CONFIG (test copy, $CFGDIR) sha256=$(shasum -a 256 "$CFGDIR/config.toml" | cut -d' ' -f1)"
  HOME=$HOMEDIR "$BIN" > "$OUT/vietkb-$name.log" 2>&1 &
  VKPID=$!
  glog "LAUNCH $(date +%H:%M:%S) pid=$VKPID path=$BIN HOME=$HOMEDIR"
  sleep 7
  if ! kill -0 $VKPID 2>/dev/null; then glog "VietKB exited early: $(tail -3 "$OUT/vietkb-$name.log")"; VKPID=""; return 1; fi
  [[ -e "$REAL_SUPPORT/VietKB" ]] && glog "WARNING VietKB wrote into the real Application Support dir"
  ls "$CFGDIR" | while read l; do glog "VIETKB config dir has: $l"; done
  local oa=(); [[ -n $ONLY ]] && oa=(--only $ONLY)
  [[ $name != telex ]] && export SKIP_IDLE_GATE=1   # later phases follow our own synthetic input; the watcher guards them
  $H run --method $method --targets "$targets" --vk-pid $VKPID "${oa[@]}" "$@"
  local rc=$?
  glog "HARNESS exit code $rc"
  glog "KILL $(date +%H:%M:%S) pid=$VKPID path=$BIN reason=phase $name finished (launched by this script)"
  kill $VKPID; sleep 1; kill -9 $VKPID 2>/dev/null; VKPID=""
  cp "$CFGDIR/config.toml" "$OUT/config-after-$name.toml" 2>/dev/null
  return $rc
}

RC=0
for ph in ${(s:,:)PHASES}; do
  case $ph in
    telex) run_phase telex telex "$( ((EXTRAS)) && echo gui,secure,gui-toggle,)$TARGETS_TELEX" || RC=$?;;
    vni)   run_phase vni vni "$( ((EXTRAS)) && echo secure,)$TARGETS_VNI" || RC=$?;;
    perapp) run_phase perapp telex "textedit-plain,stickies" --perapp-raw stickies || RC=$?;;
    app)    glog "PHASE app dev-app=$DEVAPP phases=$APP_PHASES"
            E2E_REPO=$REPO $H appsuite --app "$DEVAPP" --phases "$APP_PHASES" || RC=$?
            glog "HARNESS app exit code $RC";;
    static) ;;
  esac
  [[ $RC == 6 ]] && { glog "PHYSICAL INPUT abort; stopping all phases"; break; }
done
$H report > /dev/null 2>&1
glog "RUN end rc=$RC"
exit $RC
