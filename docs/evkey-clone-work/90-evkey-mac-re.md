# 90 - EVKey for Mac: how it delivers keystrokes (reverse engineering) and a design for VietKB

Target: `/Applications/EVKeyMac.app/Contents/MacOS/EVKey` (v3.3.10, x86_64 Mach-O, symbols not stripped) and
`Contents/Resources/EVKeyHelper.app`. This was static analysis only. EVKey was not run and nothing in the app was modified.

Method: the existing `otool -tV` listing, plus Ghidra 12.2 headless decompilation (`analyzeHeadless` with a small
`DecompAll.java` post-script; outputs in `/private/tmp/claude-501/rea-evkey/out/*.c`). REA 6.8.0 refused Ghidra 12.2
(`unsupported_version`, it accepts 12.1.x only), so Ghidra was driven directly. Constants were read from the file
(the `__TEXT` vmaddr is 0x100000000 and the file offset is 0).

Owner prefs (`defaults read com.lamquangminh.evkey`): `Type=0` (Vietnamese off at the time), `CSuggestion=1`,
`CSuggestion2=0`, `SSKey=0`, `ForceEx=0`, `Codec=0` (Unicode), and no `UseIME` key (so 0).

## 0. Config map (`g_ccfg` @ 0x100054ee0)

`GSaveConfig`/`GLoadConfig` walk a `vector<key_value_type>` of (name, offset). The offsets below were recovered from the table builder at 0x100008f..:

| offset | key | meaning used by the key path |
|---|---|---|
| 0x08 | `Type` | Vietnamese on/off (`SetVietnamOn` writes it) |
| 0x0c | `Codec` | output charset; 0 = Unicode, 4 = Composite Unicode |
| 0x2c | `EApp` | app blacklist enabled |
| 0x30 | `SSKey` | **1 = synchronous mode** (process in the tap callback, no worker); owner = 0 |
| 0x50 / 0x70 | `EMacro` / `AMarcro` | macros (still active when VN is off) |
| 0x64 | `CSuggestion` | browser suggestion fix (variant 1) |
| 0x68 | `CSuggestion2` | browser suggestion fix (variant 2) |
| 0x74 | `UseIME` | use the EVKeyIM input method instead of the tap (the tap then passes everything) |
| 0x78 | `NSound` | beep on toggle |
| 0x7c | `ForceEx` | force slow push for every app |

## 1. The tap

`_InitHookManager` @ 0x1000177c1:

```c
kbd   = CGEventTapCreate(kCGSessionEventTap /*1*/, kCGHeadInsertEventTap /*0*/,
                         kCGEventTapOptionDefault /*0*/, 0x1c00 /*KeyDown|KeyUp|FlagsChanged*/,
                         KeyboardHookCallback, ...);
mouse = CGEventTapCreate(1, 0, 0, 0x200000a /*LeftDown|RightDown|OtherDown*/, MouseHookCallback, ...);
CFRunLoopAddSource(CFRunLoopGetCurrent() /*main thread*/, ..., kCFRunLoopCommonModes);
NSTimer every 1.5 s -> StartWatchdog block   // re-enable either tap if !CGEventTapIsEnabled;
                                              // IsSecureEventInputEnabled for 20 ticks (30 s) -> log the owning app once
```

- Both taps run on the **main thread**, the same run loop as the AppKit UI.
- Event source (`_InputHookInit` @ 0x10001842f): `CGEventSourceCreate(-1 /*private state*/)`. Its state ID is saved in
  `g_evkEndSourceID`. Four events are created once and reused: char down and char up (keycode 0), and Backspace down and up (keycode 0x33).

### `_KeyboardHookCallback` @ 0x100018d56 (paraphrased)

```c
if (type >= 0xFFFFFFFE) { CGEventTapEnable(tap, true); return ev; }          // disabled by timeout or user input
if (GetIntegerValueField(ev, 45 /*kCGEventSourceStateID*/) == g_evkEndSourceID) return ev;  // own event
if (cfg.UseIME || IsOurIMKInputSourceActive()) return ev;
flags = GetFlags(ev); key = field 9;
CaptureKey(...);                                    // global pressed map (all keys)
now = GetAppMicroseconds();                         // steady_clock
if (type==KeyDown) { if (pressed[key]) double_down[key]=1; else if (key<0x80) down_time[key]=now; }
if (type==KeyUp && double_down[key]) { double_down[key]=0; SendKeyAction(key,0,0,/*up*/0); pressed[key]=0; return ev; }
if (type==FlagsChanged && key is Opt/Ctrl/Cmd) special_mark_time = now;
switch (CheckShortKey(...)) { 2: SwitchEVKey(); return ev;   3: return NULL;
                               4,5: ProcessKey(0xFF01) /*restore word, BLOCKING wait <= 1000 ms*/; swallow if handled }
if (!cfg.Type && !(cfg.AMarcro && cfg.EMacro)) return ev;   // VN off and no macros
if (ProcessCurrentAppActive(0)) ProcessKey(0xFF02 /*reset*/);  // NSWorkspace.frontmostApplication on every key
if (cfg.EApp==1 && IsCurrentAppInBlackList()) return ev;
if (type==KeyUp) {
   if (key in g_resetAfterKey) ProcessKey(0xFF02);   // Esc, modifiers, F-keys, Home/End/PgUp/PgDn/FwdDel, arrows
   if (key<0x80 && down_time[key] < special_mark_time) { SendKeyAction(key,0,0,0); pressed[key]=0; return ev; }
} else if (type==FlagsChanged) {
   if (Ctrl or Cmd released) ProcessKey(0xFF02);
   return ev;                                       // modifiers are NEVER queued
}
if (IsExistDeniedFlagkey(flags)) { ProcessKey(0xFF02); return ev; }   // Ctrl|Opt|Cmd|Help|Fn (0x40000,0x80000,0x100000,0x400000,0x800000)
if (cfg.SSKey) return ProcessKey(type, ev) ? NULL : ev;               // synchronous mode
if (type==KeyUp)  { if (!pressed[key]) return ev; pressed[key]=0; ProcessKey(type, ev); return NULL; }
if (type==KeyDown){ if (pressed[key]) { double_down[key]=1; ProcessKey(0xFF02); return ev; }  // autorepeat: pass + reset
                    pressed[key]=1; ProcessKey(type, ev); return NULL; }
```

`_ProcessKey` @ 0x1000187f6: in async mode it **copies the event** with `CGEventCreateCopy`, pushes `{type, copy}`
and returns true (swallow). The pseudo-keys 0xFF01 (restore) and 0xFF02 (reset) are queued as items too, so resets stay in order with keys.

Answers to Q1:

- It swallows **every** plain key-down while Vietnamese (or macros) is on, plus the matching key-up (only if its down was swallowed). Both are re-posted later.
- **Passed straight through** (not queued): flagsChanged (all modifiers); any key that has Ctrl, Opt, Cmd, Help or Fn in its flags (this includes arrows and the nav keys, which carry the Fn flag); the second and later downs of a held key (autorepeat, found through its own `pressed[]` map, not the autorepeat field); blacklisted apps; everything when VN is off.
- Autorepeat: the first down is queued and the repeats pass straight through with an engine reset. On key-up, EVKey posts its **own extra key-up** (`SendKeyAction(key,..,up)`) and then passes the real one too.
- A held modifier is never re-posted. The queued copy of a letter carries the flags it had when it was captured.

### `_MouseHookCallback` @ 0x1000188cf

```c
if (type >= 0xFFFFFFFE) CGEventTapEnable(tap, true);
else if (type in {1,3,25}) ProcessKey(0xFF02);   // queued reset in async mode, evkResetBuff(0) in sync mode
return ev;                                       // never swallows
```

Q4: any mouse button down resets the word buffer, and nothing else. In async mode the reset is queued, so it is ordered after the keys already pending, but the **click itself is not queued**.

## 2. The worker loop

`_PushKeyDataToQueue` @ 0x100019b28 locks `g_mutex_process_key`, calls `deque<HOOKKEY_DATA{int type; CGEventRef ev}>::push_back`, unlocks, and calls `c_event::set_event` (a condvar `notify_all`).

`_ProcessKeyFunc` @ 0x100019c45 (std::thread started by `_InitProcessThread`):

```c
for (;;) {
  g_even_process_key.wait(INFINITE);
  lock; local = std::move(g_key_data); unlock;            // swap the WHOLE deque out
  while (!local.empty()) {
    {type, ev} = local.pop_front();
    if (type==0xFF01) { evkRestoreKeys(); g_restore_ret = PushBuffer(); g_even_restore_key.set_event(); continue; }
    if (type==0xFF02) { evkResetBuff(0); continue; }
    if (type==KeyUp || !ProcessKeyDown(type, ev)) {        // not consumed: re-post the original
        CGEventSetSource(ev, g_evkEventSource);            // re-source to the private source -> own tap skips it
        CGEventPost(kCGSessionEventTap /*1*/, ev);
    }
    CFRelease(ev);
  }
}
```

Q2:

- **Ordering**: one FIFO and one consumer. Keys, key-ups, resets and restores are all strictly ordered among themselves. They are not ordered against anything that passed straight through (modifiers, Cmd/Ctrl/Opt chords, arrows, autorepeats, mouse clicks).
- **Self-recognition**: `kCGEventSourceStateID` (field 45) compared with the private source's ID. The user-data field (42) is not used.
- **Original event**: re-sourced with `CGEventSetSource` (flags, keycode, Unicode string and timestamp are kept, only the source changes), then posted at the **session** tap. It re-enters the head of the session tap chain, so it goes through EVKey's own main-thread callback a second time and through every other session tap.
- **Waits**: the loop itself does not sleep. The delays are inside the push functions (section 3).
- **Queue bound**: none. It is an unbounded `std::deque`. If the worker is slow (clipboard path, slow mode), keys pile up and are all re-posted late. None are dropped.

`_ProcessKeyDown` @ 0x10001914f maps the key through a keypad map, then Return/Enter (0x24/0x4c) -> `evkProcessChar(0xd)`,
Backspace (0x33) -> `evkPutBackspace`, and otherwise `GetCharFromKeyCode` (`UCKeyTranslate` on the current layout, applying only Shift and Caps). It then calls
`PushBuffer()`. If PushBuffer emitted something and the key is Return/Enter, it posts a **synthetic Return down** (`SendKeyAction(key,0,0,1)`) and resets.
It returns true (consumed) only when PushBuffer emitted something.

## 3. Injection

### Primitives

| function @ addr | what it posts |
|---|---|
| `_SendKeyAction` @ 0x10001869c | `CGEventCreateKeyboardEvent(private src, key, down)`; when shift or cmd is set, the flags become `0x20000`/`0x100000` OR `0x100`; posted at **session** (1) |
| `_SendBackspace` @ 0x10001876a | prebuilt BS down + BS up at session |
| `_SendChar` / `_SendString` @ 0x100018791 / 0x1000187c5 | prebuilt keycode-0 event + `CGEventKeyboardSetUnicodeString`, **key-down only**, posted at **HID** (0) |
| `_SendKeyByAction` @ 0x100018674 | re-source + post at session (same as the worker) |

### `_PushBuffer` @ 0x100019316 (a replace)

```c
if (!evkGetBackspaceLength() && !evkGetBuffLen()) return 0;
evkConvertUnicode1ByteTo2Byte();
if (IsSpotlightAXTarget() /*bundle == "com.apple.spotlight"*/ && 1 <= bs && bs <= 31) {
    if (unicode-ish output && newLen == bs && SpotlightAXReplace(bs, buf, newLen)) return 1;
    repeat bs: SendKeyAction(kLeft, shift, down); SendKeyAction(kLeft, shift, up);  // select, then type over the selection
    PushChars(); return 1;
}
if (bs) PushBackSpace();
PushChars(); return 1;
```

### `_PushBackSpace` @ 0x100019976 (the exact sequence)

```c
bs = low32(evkGetBackspaceLength());
if (cfg.Codec==4 /*Composite*/ && IsAppFixCompositeUnicode()) bs -= high32(...);   // count clusters, not code points
if ((cfg.CSuggestion || cfg.CSuggestion2) && IsCurrentAppNeedCorrectSuggesion()
    && (evkGetBuffLen()!=0 || evkIsAbleMarkTone())) {
    if (IsCorrect_Like_Chrome()) {
        if (cfg.CSuggestion) { post U+202F (key-down, HID); bs += 1; }                 // Chrome-like, variant 1 (owner)
        else { Shift+Left down; usleep(1000); Shift+Left up; /* bs unchanged */ }    // Chrome-like, CSuggestion2 only
    } else if (IsCorrect_1_Byte()) { key 0x27 (') down; usleep(1000); up; usleep(1000); bs += 1; }  // Excel
    else { post U+200C ZWNJ (key-down, HID); bs += 1; }                               // Safari, Firefox, Opera, Rider
}
if (evkGetBuffLen() && (IsSlowPushKey() || cfg.ForceEx))
    usleep(IsCurrentAppNeedCorrectSuggesion() ? 10000 : 50000);
repeat bs: CGEventPost(session, BS down); CGEventPost(session, BS up);                // no gap between them
if (IsMustUseClipboard()) usleep(30000);
```

### `_PushChars` @ 0x100019941

`slow || ForceEx` -> `PushEachChar`, `must-use-clipboard` -> `PushByClipboard`, else `PushCharString`.

- `_PushCharString` @ 0x100019658: the whole word in **one or more key-down events of at most 16 UTF-16 units** (`_PushSegmentString` @ 0x10001951d), posted at **HID**, with no key-up. Then `usleep(1000)` always, plus 50 ms if the app is slow and not a suggestion app.
- `_PushEachChar` @ 0x1000193e5: one key-down per unit at HID, `usleep(1000)` between them, then 50 ms at the end.
- `_PushByClipboard` @ 0x100019731: NSLogs the text (`PUSHPASTEBOARDCONTENT %s`), `clearContents`, `setString`, waits 100 ms, Cmd down, V down (cmd flag), waits 5 ms, V up, Cmd up, waits 80 ms. It **never restores the user's clipboard**.

So a normal replace is **N x (BS down, BS up) at session**, then **1 key-down carrying the string at HID**, then 1 ms. Backspace is sent as down and up; text as down only.

### Per-app classes (static init `__GLOBAL__sub_I_sp_special_apps.mm`; flags are recomputed in `_InitValue` @ 0x1000071a4 when the app changes)

Bundle IDs are lowercased and matched exactly (`std::set::find`), except the composite list, which uses substring `find`.

| flag | apps |
|---|---|
| `need_correct_suggestion` (14) | chrome, safari, safaritechnologypreview, firefox, jetbrains.rider, microsoft.excel, edge.dev, edge, brave.browser, operasoftware.opera, edgemac, operagx, yandex-browser, chrome.canary |
| `correct_like_chrome` (9) | chrome, edge, edge.dev, brave, edgemac, operagx, yandex, chrome.canary, apple.spotlight |
| `correct_1_byte` (1) | microsoft.excel |
| `app_fix_composite_unicode` (4, substring) | `com.apple.`, chrome, brave, edge |
| `slow_push_key` (1) | adobe.illustrator |
| `must_use_clipboard` (2) | parallels.desktop.console, vmware.fusion |

Q3, the suggestion fix: it fires on **every replace in the whole app** (not only in the omnibox) when the app is in the 14-app list,
one of `CSuggestion`/`CSuggestion2` is on, and the engine buffer is non-empty or the engine can still place a tone mark (`evkIsAbleMarkTone`). Spotlight is in the
Chrome-like list but not in the 14-app list, so the fix never triggers there; Spotlight goes through the AX path.

### Spotlight AX replace (`_SpotlightAXReplace` @ 0x100007c6e)

`AXUIElementCreateSystemWide` with a 50 ms messaging timeout, then `AXFocusedUIElement`. It reads `AXValue` and `AXSelectedTextRange`, and checks that the
`bs` characters before the caret, ASCII-folded (`AxAsciiFold`), match the new text folded. If they match, it replaces the range in the string, sets the
whole `AXValue` and moves the caret with `AXSelectedTextRange`. It retries up to 20 times with 300 us sleeps. It is only used when the replacement has the same length.
Spotlight is detected two ways: through `CGWindowListCopyWindowInfo(OnScreenOnly|ExcludeDesktop)`, looking for an owner named "Spotlight" with a window of at least 80x20, throttled to every 0.1 s
(`_IsSpotlightActive` @ 0x10000759f), and through the frontmost bundle ID.

## 5. Other instances and other IMEs

- `_CheckFirstInstance` @ 0x100015255 counts `runningApplications` whose bundle ID is `com.lamquangminh.evkey` or
  `com.lamquangminh.evkeynsb`. If there is more than one, it calls `[NSApp terminate:]`. **It does not detect any other IME.**
- Distributed notifications `com.lamquangminh.evkey.configChanged` and `.vietnameseChanged` (posted by `EVKPostNotification`
  with `userInfo{sender: bundleID}`) sync the app with its own EVKeyIM input method. They are not used for instance detection.
- EVKeyHelper.app (`com.lamquangminh.evkeyhelper`, LSBackgroundOnly) is only a login-item launcher. If `com.lamquangminh.evkey` is not running,
  it calls `launchApplication:` on the path 4 components up from itself, then terminates.
- Because EVKey's filter is the **source state ID only**, it processes other injectors' events. VietKB's text events use keycode 0 with a
  Unicode string. EVKey ignores the string and translates **keycode 0 as `a`**. Running both at once corrupts text.

## 6. Input-source handling

- `_UpdateIMKActiveState` @ 0x100017dd2: `TISCopyCurrentKeyboardInputSource` and its bundle ID == `com.lamquangminh.evkey.inputmethod`. The result is cached in
  `g_imkInputSourceActive` and refreshed only at launch and in `ApplyInputMechanism`.
- `_ApplyInputMechanism` @ 0x100017eaa: with `UseIME` on, it enables and selects the EVKeyIM source (`TISCreateInputSourceList`, `TISEnableInputSource`,
  `TISSelectInputSource`; `_InstallAndRegisterIMK` copies EVKeyIM.app to ~/Library/Input Methods, but that app is **not bundled** in this
  install). Otherwise it selects `com.apple.keylayout.ABC` (or `TISCopyCurrentASCIICapableKeyboardInputSource`).
- `kTISNotifySelectedKeyboardInputSourceChanged` (a distributed observer) leads to `InitKeyboardLayout` on the main queue, which refreshes the `UCKeyTranslate` layout.

## 7. Weaknesses and costs in EVKey's design

1. **Every key gets slower while VN is on.** Each key goes through the main-thread callback, `CGEventCreateCopy`, a mutex and condvar wake, the worker, `CGEventPost(session)`, and then the **main-thread callback again** (the re-posted copy goes through EVKey's own tap) and every other session tap. That is two passes through the callback, one thread hop and one extra WindowServer round trip on every key, including keys that change nothing.
2. **The main thread is both tap and UI.** A busy AppKit (config window, alert, menu tracking) stalls typing. The callback calls ObjC (`NSWorkspace.frontmostApplication`) on every key and `CGWindowListCopyWindowInfo` up to 10 times a second.
3. **Tap timeout risk.** The restore-word hotkey path blocks the callback for up to **1000 ms** waiting on the worker (`c_event::wait(1000)`). If the worker is stuck in the clipboard path (>200 ms) or a slow app, the tap can be disabled. The 1.5 s watchdog plus re-enable on `TapDisabledByTimeout` hides this.
4. **Order holes.** Modifiers, chords (Cmd/Ctrl/Opt), arrows and nav keys (Fn flag), autorepeats and mouse clicks pass straight through while earlier keys may still be queued. Typing `abc` and then pressing Left or Cmd+Z quickly can act before `abc` is re-posted.
5. **Unbounded queue.** In Parallels or VMware each replace costs about 215 ms of sleeps, so lag builds up without limit.
6. **Fixed sleeps and fixed app lists.** The suggestion trick runs on every replace in every field of a browser and costs an extra char and an extra Backspace each time. The lists are hard-coded, the key for Excel is layout-dependent, and opera vs operagx are inconsistent.
7. **Mixed tap levels.** Text is posted at HID and Backspaces at session, so cross-level ordering relies on behaviour that is not documented.
8. **Clipboard path** destroys the user's clipboard and logs the typed text through NSLog.
9. **Duplicate key-up.** On autorepeat, and when a modifier was pressed after a key's down, EVKey posts its own key-up and also passes the real one.

## Comparison with VietKB today (`crates/vkb-macos/src/sys.rs`, `state.rs`, `inject.rs`)

| | EVKey (async, default) | VietKB |
|---|---|---|
| tap | session, head, default; **main thread** | session, head, default; **dedicated `vkb-event-tap` thread** |
| mask | KeyDown/Up, FlagsChanged; separate mouse tap | the same events plus mouse-downs, all in one tap |
| unchanged key | swallowed, copied, re-posted by the worker | **passed through** (zero added latency) |
| changed key | worker: BS... then text, `CGEventPost` | in the callback: swallow, then BS... and text via `CGEventTapPostEvent(proxy)` |
| self-recognition | source state ID (field 45) | user data (field 42) == `MARKER` |
| text event | key-down only, up to 16 units, HID | down **and** up with the string, up to 16 units, proxy |
| BS / text tap level | session / HID | proxy for both (they land right after VietKB's tap) |
| pacing | 1 ms after text; 10/50 ms in slow apps; clipboard 100+5+80 ms | none |
| browser autocomplete | U+202F / U+200C / `'` + one extra BS, fixed list | none |
| autorepeat | own pressed map | `kCGKeyboardEventAutorepeat` field |
| app detection | `frontmostApplication` per key in the callback | `Host` queries per event |
| tap disabled | re-enable in callback plus a 1.5 s watchdog | re-enable in callback and engine reset |

VietKB is already better on 1, 2, 3 and 7. What it lacks is EVKey's **back-pressure**: in EVKey the next key cannot reach the app until the
previous injection has been posted and given 1 ms. In VietKB a fast next key, rollover, or a burst can arrive right behind a large BS-and-text
injection. It also has no browser suggestion fix.

## Proposed VietKB design: "serialize only while output is pending"

Goal: keep EVKey's ordering and pacing guarantees, but pay for them only when there is output in flight, and fix holes 3, 4, 5 and 6.

### Core

1. **Keep the tap on its own thread and keep the engine in the callback.** It is pure Rust and runs in microseconds, so decisions stay in logical key order. Never call AX, `CGWindowList` or blocking ObjC in the callback. Cache the frontmost app from `NSWorkspaceDidActivateApplicationNotification` instead of querying it per key.
2. **Two modes, chosen per event.**
   - *Idle* (output queue empty, nothing posting): unchanged keys **pass straight through**, exactly as today.
   - *Pending* (the queue is non-empty or the worker is posting): **every** keyboard event is swallowed and enqueued as a re-post item, so nothing can overtake. That covers key-downs, key-ups, flagsChanged, chords, arrows and autorepeats, which closes hole 4. A key-up is enqueued exactly when its key-down was enqueued, using a per-keycode bit as EVKey does with `pressed[]`.
   - A changed key always enqueues `Inject{bs, text}` and enters Pending.
   Evaluation: serializing "while a word is being composed" is not needed. Ordering only matters while output is in flight, and pending-only keeps idle latency at zero for most keystrokes, including most keys inside a word.
3. **Worker (one consumer, FIFO).** Re-post the copy with `CGEventSetIntegerValueField(42, MARKER_REPOST)` and `CGEventPost(kCGSessionEventTap)`. The worker cannot use the proxy, which is valid only inside the callback. Use a distinct marker so the tap can tell a re-post from an injection in logs. Keep the original flags and timestamp, and do not re-source unless tests show it helps.
4. **Bounded queue.** Cap it at about 64 items or 150 ms age. On overflow, flush everything **raw** in order (no further transforms), reset the engine and log it. Never drop or reorder keys.
5. **Mouse-down while Pending.** Reset the engine at the callback (logically correct). Optionally enqueue a "fence": the click cannot be queued without feeling laggy, so accept the hole but keep Pending windows short (a few ms) so it rarely matters.

### Pacing: adaptive instead of fixed sleeps

- Add a **listen-only tail tap** (`kCGTailAppendEventTap`, `kCGEventTapOptionListenOnly`, own thread; Input Monitoring is already granted). When the last event of an injection (marked) is seen at the tail, it has passed all taps. The worker waits for that **delivery fence** (timeout about 20 ms) before posting the next item. This replaces EVKey's blind 1 ms with a measured, usually shorter wait, and it adapts under load.
- On top of the fence, use a small **per-app-class settle** (default 0, browsers about 1 ms, "slow" apps 5-10 ms) from a table the owner can override, not a hard-coded 50 ms. Also learn it: if the tail fence keeps hitting its timeout for an app, raise that app's settle.
- Experiment worth one A/B run: **in-callback pacing**. Post via the proxy, then sleep at most 2-3 ms inside the callback. The tap thread is not the UI thread, so this holds back later physical keys without reposting them. Whether WindowServer delivers proxy-posted events while that callback is still out must be measured with the tail tap before relying on it. If it works, it is simpler than a worker and adds no latency to idle keys.

### Injection details to adopt or test

- **Text as key-down only** (EVKey) as an option against the current down+up. It may be the cause of drop/dup letters in some apps (see `qa/mac/run-drop-letter*`), so test it per app class.
- Keep everything on **one tap level** (session or proxy). Do not copy EVKey's HID/session mix.
- **Composite (NFD) charset:** in `com.apple.*` and Chromium apps, count Backspaces in **grapheme clusters**, not code points. EVKey subtracts the combining-mark count there. VietKB's NFD output needs the same per-app rule.
- **Clipboard fallback** (only if ever needed for VMs): save and restore the pasteboard, never log the content, and use the fence instead of 185 ms of sleeps.

### Browser inline autocomplete

- Phase 1 (cheap and proven): port EVKey's trick behind a setting, applied only when the replace has `bs > 0` or a non-empty word.
  Chromium list: U+202F then bs+1. Safari/Firefox: U+200C then bs+1. Excel: one ASCII char then bs+1, but post it as a Unicode string, not keycode 0x27, so it does not depend on the layout.
- Phase 2 (better): **AX selection probe**. On the worker, never in the callback, read the focused element's `AXSelectedTextRange`. If its length is > 0 just before a replace, an autocomplete suffix is selected, so add exactly one Backspace (or Fwd-Delete) and skip the dummy char. Use `AXUIElementSetMessagingTimeout` of 20-50 ms, cache the focused element per focus change, and fall back to Phase 1 when AX errors. Cost to measure: querying Chromium's AX switches on its renderer accessibility, which can slow the browser. So probe only in browser apps and only when a field is plausibly an omnibox (role `AXTextField` outside an `AXWebArea`), or make it opt-in.
- Spotlight: EVKey's AX same-length replace with a folded-text check is a sound pattern to reuse as an opt-in for Spotlight-like fields.

### Detect a second VietKB or another IME

- Startup and periodic check (e.g. on app activation): `runningApplications` for known Vietnamese IMEs. EVKey `com.lamquangminh.evkey` and `com.lamquangminh.evkeynsb` are confirmed here. Others such as OpenKey and GoTiengViet need their bundle IDs checked before they are hard-coded. Warn once in the UI and do not auto-quit others.
- Runtime heuristic in the tap: a keyboard event with user data != MARKER, keycode 0, a non-empty Unicode string and a source PID that is neither ours nor 0 means another injector is active. Count these and warn once. (VietKB's own singleton per identifier already covers a second VietKB.)
- Note: VietKB's keycode-0 text events are read as `a` by EVKey. This cannot be fixed on VietKB's side beyond detecting and warning.

### Measure added latency (do this first, before and after)

- Tail listen-only tap: for every keyboard event that reaches the tail, compute `now - CGEventGetTimestamp(ev)`. A re-posted copy keeps the original timestamp, so this is the end-to-end tap-chain delay for pass-through, re-posted and injected events. Log p50/p95/p99 per mode into `VKB_DEBUG_KEYS` (`debuglog::EvIn.lag_us` already exists for the head side).
- Also log the queue depth and time-in-queue per item, plus fence waits and timeouts per app. Target: in Idle, p99 added latency about 0. In Pending, p99 under 3 ms outside slow apps.

### Avoid

- A blocking wait in the callback on the worker (EVKey's 1 s restore wait). The restore hotkey should enqueue like any other output.
- Per-key AX, window-list or workspace queries in the callback.

## What carries over to Windows (`crates/vkb-windows`: WH_KEYBOARD_LL + SendInput)

- The **pending-only serialization** applies directly. A low-level hook cannot delay an event either, so it must swallow and re-inject (`SendInput` with `dwExtraInfo` = tag) while output is pending. The same bounded queue and raw-flush on overflow apply. Windows' `LowLevelHooksTimeout` (and silent hook removal) makes "never block in the hook" even more important than on macOS.
- Windows already sends BS and text in **one `SendInput` batch**, which is atomic against physical input. That covers most of what EVKey's queue gives on Mac. A queue is only needed for pacing in slow apps.
- **Adaptive pacing via a fence:** a second LL hook cannot see "delivered to app". The closest equivalents are `WaitForInputIdle`-style checks or a short per-app settle. Learn it per app from failures rather than using fixed sleeps.
- The **browser suggestion trick** (dummy char, then bs+1) works the same in Chromium on Windows. On Windows the AX probe would be UI Automation `TextPattern` selection, with the same cost caveat for Chromium accessibility.
- **Grapheme-cluster Backspace counting** for composite output in Chromium is platform-independent. The Notepad/WinUI behaviour is already handled in `input.rs`.
- **Clipboard fallback:** save and restore, never log (already opt-in in `clipboard.rs`).
- Detecting other IMEs (EVKey for Windows, UniKey): look for foreign `LLKHF_INJECTED` events with `VK_PACKET` and a `dwExtraInfo` that is not VietKB's tag, then warn.

## Evidence files

- Decompiled C: `/private/tmp/claude-501/rea-evkey/out/` (`_KeyboardHookCallback.c`, `_ProcessKeyFunc.c`, `_PushBackSpace.c`,
  `_PushBuffer.c`, `_PushChars.c`, `_PushCharString.c`, `_PushEachChar.c`, `_PushByClipboard.c`, `_SpotlightAXReplace.c`,
  `_IsSpotlightActive.c`, `_ProcessCurrentAppActive.c`, `_InitHookManager.c`, `_CheckFirstInstance.c`, `_ApplyInputMechanism.c`, ...).
- Ghidra project: `/private/tmp/claude-501/rea-evkey/proj/evk.gpr`; script: `/private/tmp/claude-501/rea-evkey/scripts/DecompAll.java`.
- otool listing: the scratchpad `evk/dis.txt` (line refs: callback 26864, worker 27955, PushBackSpace 27741, app sets 8280-8520).
