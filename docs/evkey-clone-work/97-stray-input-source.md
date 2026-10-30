# 97 — Where does the stray `ư` come from?

**Asked:** owner reports an occasional `ư` appearing while typing, after "we killed every
VietKB process". Read-only diagnosis on the owner's Windows 11 machine, 2026-10-30 ~11:45.
Nothing was killed, closed, launched, or typed into. `99-agent-guardrails.md` read first.

## Verdict

**A — a VietKB instance is alive and is the keystroke transformer.**

| | |
|---|---|
| pid / path | **19224** / `C:\Users\namth\AppData\Local\VietKB\vietkb.exe` (installed 0.2.3, the daily driver) |
| Started | **2026-10-30 11:41:08** — after the kill-switch |
| Parent | **11344 = explorer.exe** (interactive ShellExecute: shortcut / Start-menu click / `Desktop\VietKB.lnk`), **not** a Run key, task, or service |
| Hook | `mechanism = "hook"`, `enabled = true`, `method = "vni"`, `auto_horn_uo = true`, `spell_check = true` |
| Why `ư` | VNI mode: `method.rs:226` `'7' => mark_vni_horn(...)` — a `u` followed by the digit **7** becomes `ư`; `method.rs:124` `auto_horn_uo` also promotes `uơ` → `ươ` once a legal coda follows. Both are *correct VNI behaviour*, so the output looks like a random `ư` to someone who believes no IME is running. |

Not "restarting": exactly **one** relaunch at 11:41:08, no respawn loop. The kill-switch
(`log.md`, 11:38:53) stopped **pid 26028** of the same path; 2m15s later the app was started
again from the shell, and the guardrail file written at 11:42:25 already records that the
installed instance must be left alone.

## Findings

| # | Check | Raw result |
|---|---|---|
| 1 | Suspect processes (`vietkb|vkb|unikey|evkey|openskey|gotv|ime`) | one hit: `19224 vietkb` `C:\Users\namth\AppData\Local\VietKB\vietkb.exe`, StartTime 11:41:08, PPid 11344. Chain: `19224:vietkb.exe <- 11344:explorer.exe`. No `vkb_windows-*.exe`, no EVKey, no UniKey, no OpenKey. |
| 2a | `HKCU\...\CurrentVersion\Run` | `VietKB = "E:\vietkb\target\debug\deps\vkb_windows-591f7a65901d2457.exe" --autostart` — **an agent-built debug binary**, not the installed app. Target exists, mtime 11:31:57, 2 089 984 B. `StartupApproved\Run\VietKB = 02-00-00-…` → **enabled**. |
| 2b | Startup folders | user: `9router.vbs`, `claude-remote-control.vbs`, **`EVKey64.lnk` → `D:\Software\EVKey\EVKey64.exe`**, `Send to OneNote.lnk`. ProgramData: `Tailscale.lnk` only. `StartupApproved\StartupFolder\EVKey64.lnk = 03-…52-67-DD-01` → enabled. |
| 2c | Scheduled tasks with a `vietkb/vkb/unikey/evkey` action | **none** |
| 2d | Services matching | **none** |
| 2e | WMI permanent subscriptions | only the stock `SCM Event Log Filter` / `SCM Event Log Consumer` — no CommandLineEventConsumer |
| 3 | Process creation audit | 4688 audit policy is **off**; no `Application`/`System` event mentions vietkb in the last 3 h (no crash/WerFault record). Newest-process list shows only normal 11:45 build/Chrome activity. Explorer's child list: `19224 vietkb.exe 11:41:08` sits between `11:06:10 Code.exe` and nothing else — consistent with a single shell launch, not a respawner. |
| 4 | Other IMEs | `Get-WinUserLanguageList` = **en-US only**, single IMT `0409:00000409`. `HKCU\Keyboard Layout\Preload` empty; no `0000042a` (Vietnamese) layout; `Substitutes`/`Toggle` empty. Installed programs: **VietKB 0.2.3** only (no UniKey/EVKey/VietKey/GoTiengViet/OpenKey/Skey installed). **EVKey 2021 build does exist unpackaged at `D:\Software\EVKey\`** (`EVKey64.exe` 2022-03-15, `setting.ini` mtime **10:53:27 today**, `AutoStart=1`); `Application` log shows its SideBySide activation failures at **10:51:57** (`D:\Software\EVKey\EVKey32.exe`, `evkau.exe`). It is **not running now**, and those events predate this boot (LastBootUpTime **11:03:50**). |
| 5 | Hook/module footprint | `Get-Process vietkb` → `19224`, and the only loaded module matching `vkb|vietkb|evkey|unikey` anywhere on the box is `19224: C:\Users\namth\AppData\Local\VietKB\vietkb.exe`. `MainWindowTitle = vn.vietkb.app-siw`, Responding=True, 237 handles, 24 MB WS. (A `WH_KEYBOARD_LL` hook needs no DLL in the target, so this only rules out an injected module.) |
| 6 | App data / logs | `%LOCALAPPDATA%\VietKB\` = `vietkb.exe` + `uninstall.exe` only, **no log files**. `%APPDATA%\VietKB\keys.log` (12.5 MB) last line **10:23:09** — the old instance's hook log, stops before the kill; the 11:41 instance writes nothing there (no `VKB_*` debug env), so there is **no log evidence of a crash or restart** and none of the last `ư` events. Also present: `config.toml` (mtime 09:51:39), `config.toml.testresidue-20261029`, `config.backup-20261029-034136.toml`, two `perf-*.tsv`. |
| 7 | Windows Telex | **none**: no Vietnamese keyboard layout, no `Keyboard Layout\Toggle`, no `EnableHexNumpad`, `HKCU\Control Panel\Input Method` holds only `Show Status = 1`, `HKCU\Software\Microsoft\InputMethod\Settings\CHS` absent. Windows itself cannot turn `uw`/`u7` into `ư` here. |

## Two separate defects this turned up

1. **D-GUARD-01 — autostart hijack.** `HKCU\...\Run\VietKB` points at an agent build,
   `E:\vietkb\target\debug\deps\vkb_windows-591f7a65901d2457.exe --autostart` (enabled in
   `StartupApproved`). The app's own "start with Windows" (tauri-plugin-autostart,
   `config.toml [system] start_with_windows = true`) rewrote the value while a *debug* binary
   was the running exe. At the **next logon** Windows will start that debug test binary — a
   second hook with whatever config it finds — while the installed app may also be started.
   Owner action (they must do it, agents must not): Settings → Apps → Startup, or Task Manager
   → Startup apps, turn **VietKB** off, then let the installed app re-register its own path.
2. **D-GUARD-02 — a second Vietnamese IME autostarts.** `EVKey64.lnk` in the user Startup
   folder, enabled, `D:\Software\EVKey\setting.ini` with `AutoStart=1`, EVKey last touched
   today 10:51–10:53. If it ever starts successfully beside VietKB, two hooks compose Telex
   over VNI and *that* is the classic "one extra `ư`". Its 2021 build currently only manages
   SideBySide activation failures on this OS build.

## Next non-invasive check for the owner (no agent involvement)

1. Type `u7` (then `uw`) in Notepad — expect `ư` / `uw`, because VietKB is in **VNI** mode.
2. Press **F2** (config `[hotkeys] disable = "F2"`) — VietKB's title icon should show disabled —
   and type `u7` / `uw` again. `ư` gone ⇒ confirmed: the live VietKB is the source.
3. Still there with VietKB disabled ⇒ quit VietKB from the tray (*Thoát hẳn*) and repeat. If it
   persists, the next suspect is whatever the Startup entries launch, so switch **VietKB** and
   **EVKey64** off in Task Manager → Startup apps and reboot.
4. If it persists even with both off and no `vietkb`/`EVKey64` process in Task Manager, the
   last candidate is an IME inside a specific app (browser extension, Zalo, VS Code extension) —
   note which app shows the `ư`.

**Do not** run `cargo build` over `target\debug\deps` while that Run value still points there;
the value is a landmine for the owner's next boot.
