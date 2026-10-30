# 96 — Windows Sandbox feasibility for the typing E2E

Read-only measurement. No app launched, no sandbox started, no feature changed, no setting touched.
Shell was **not elevated** (`nampc\namth`, `IsInRole(Administrator) = False`), so DISM/`Get-WindowsOptionalFeature`
were unavailable; feature state was read non-elevated through `Win32_OptionalFeature`.

## Measured facts

| # | Fact | Command | Raw output |
|---|---|---|---|
| 1 | Edition / build | `Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'` | `ProductName=Windows 10 Pro` (stale reg string), `Caption=Microsoft Windows 11 Pro`, `Version=10.0.26300`, `DisplayVersion=26H2`, `UBR=9711`, `EditionID=Professional`, `InstallationType=Client` |
| 2 | Host is bare metal, hypervisor already on | `Get-CimInstance Win32_ComputerSystem` | `Manufacturer=Gigabyte Technology Co., Ltd.`, `Model=B760M DS3H DDR4`, `HypervisorPresent=True`, 24 logical CPUs, 34183061504 B RAM |
| 3 | CPU virt flags are masked | `Get-CimInstance Win32_Processor` | `i7-13700F`, `VirtualizationFirmwareEnabled=False`, `VMMonitorModeExtensions=False`, `SecondLevelAddressTranslationExtensions=False` (masked because a hypervisor already owns VT-x, not because VT-x is off) |
| 4 | **Windows Sandbox feature** | `Get-CimInstance Win32_OptionalFeature -Filter "Name='Containers-DisposableClientVM'"` | `InstallState = 1` → **Enabled** |
| 5 | Containers / WSL / Hyper-V features | same query, 6 names | `Containers=2` (Disabled), `Microsoft-Hyper-V-All=2` (Disabled), `HypervisorPlatform=2` (Disabled), `Microsoft-Windows-Subsystem-Linux=2` (Disabled), `VirtualMachinePlatform=1` (Enabled) — legend 1=Enabled 2=Disabled 3=Absent |
| 6 | Elevation needed for the DISM path | `Get-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM`; `dism /online /get-featureinfo /featurename:Containers-DisposableClientVM` | `The requested operation requires elevation.` / `Error: 740 Elevated permissions are required to run DISM.` |
| 7 | Sandbox binaries shipped | `Test-Path C:\Windows\System32\WindowsSandbox*.exe` | `WindowsSandbox.exe=True`, `WindowsSandboxClient.exe=True` (v10.0.26100.8875), `WindowsSandboxServer.exe=False`, `WindowsSandboxProxy=False`; `wsb.exe` on PATH; Appx `MicrosoftWindows.WindowsSandbox 0.8.107.0` |
| 8 | VM/container services | `Get-Service vmcompute,vmms,HvHost` | `vmcompute=Running/Manual`, `HvHost=Running/Manual`, `vmms=ABSENT` (no Hyper-V management stack) |
| 9 | VBS is on | `Get-CimInstance Win32_DeviceGuard -Namespace root\Microsoft\Windows\DeviceGuard` | `VirtualizationBasedSecurityStatus=2`, `SecurityServicesRunning={2}` (HVCI) |
| 10 | Live hypervisor process | `Get-Process vmwp` | `vmwp pid=38192 SessionId=0` (one VM worker alive, SYSTEM-owned) |
| 11 | Host audio endpoints | `Get-CimInstance Win32_SoundDevice` | Realtek HD Audio, NVIDIA HD Audio, NVIDIA Virtual Audio Device, USB Audio Device — all `Status=OK` |
| 12 | No hypervisor product installed | `Get-Command VBoxManage,vmrun,qemu-system-x86_64`; scans of `C:\Program Files*`, Downloads, Documents, `E:\` | all `not-in-PATH`; **no** VMware/VirtualBox/QEMU install, **no** `.vhdx/.vhd/.vmdk/.ova/.iso` found, **no** Hyper-V VM store |
| 13 | WSL2 present but headless | `wsl.exe -l -v` | `docker-desktop Running v2`, `podman-machine-default Stopped v2`, `podman-net-usermode Stopped v2` |
| 14 | Spare local accounts | `Get-LocalUser` | `CodexSandboxOffline` Enabled (last logon 10/24), `CodexSandboxOnline` Enabled, `WDAGUtilityAccount` Disabled, `Administrator`/`Guest` Disabled |

### Sandbox behaviour already measured in this repo (not re-run today)

Artifacts: `sandbox-share/e2e/{probe-unicode.txt,results3.txt,results4.txt,install-diag.txt,keys.log}`,
`docs/evkey-clone-work/qa/e2e/sandbox-e2e.log` (175 KB), `docs/evkey-clone-work/{88-e2e-report.md,89-e2e-paragraphs.md,log.md}`.

| Step | Result inside Sandbox |
|---|---|
| Tauri app starts, WebView2 renders | **Works**: `sandbox-e2e.log` → `os=Microsoft Windows NT 10.0.26100.0`, `exe=True`, `app_ready telex pid=9392` |
| Low-level hook loads, keys arrive, engine decides | **Works**: `keys.log` → `stage=engine-swallow action=Replace{delete:3,insert:"tiê"} … lasterror=0 attempts=1`, `stage=own-injection` |
| `SendKeys` / VK keystrokes reach the focused TextBox | **Works**: `results4.txt` → `probe_app text=[a]`, `probe_noapp text=[a]`, `focus_check fg=131164 … focused=True` |
| `SendInput` with `KEYEVENTF_UNICODE` reaches the target | **Dropped**: `probe-unicode.txt` → `viet_probe send=[down=1 up=1] text=[]` (packet accepted, nothing typed) |
| Accented output observable | **No**: `results4.txt` → `CASE T1-tieengs expected=[tiếng] actual=[tieng] FAIL`, `T4-horn expected=[thương] actual=[huowng]`; ASCII-only `T6` PASS. 7 ASCII PASS / 18 accented FAIL in the paragraph run |
| Installer, uninstall, autostart, profile isolation | **Works**: `install-diag.txt` → `installer_exit=0`, install dir + `uninstall.exe` + `vietkb.exe` present, `app_alive=True`, uninstall clean |
| Results come back to the host | **Works**: `results4.txt`, `keys.log`, `install-diag.txt`, `sandbox-e2e.log` all written from inside the sandbox into the read/write mapped folder |
| `.wsb` logon command | **Works with `powershell -File`** (5 `.wsb` files in repo); a `.cmd` logon command has *not* been tried — needs `cmd /c C:\share\x.cmd` |
| UIA read-back / screenshot from inside sandbox | **Not proven**: no `.png` exists in `sandbox-share`; all read-back so far was in-process (WinForms `$tb.Text`), never UIA |
| Audio | **Unknown**: sandbox is an RDP-style session, so a render endpoint is normally present, but audibility of `SystemAsterisk`/`SystemHand` and WASAPI loopback capture were never measured |
| 30-minute soak | **Unmeasured**: longest recorded run is a few minutes; known ops hazard from `log.md` — a stale SYSTEM-owned `vmwp` blocks every later run with *"Only one running instance of Windows Sandbox is allowed"* and needs an **elevated** kill |

## Verdict — **B: sandbox not viable for the typing E2E**

The blocking fact is not the GUI, the hook, or Tauri: **Windows Sandbox does not deliver
`KEYEVENTF_UNICODE` input to a target window.** Both a direct `SendInput` unicode probe and VietKB's own
`Replace{insert:"tiê"}` injections land in the session, are reported successful, and are then dropped by
the sandbox input path. Only the ASCII skeleton of a word survives, so every accented case fails with
undiacritized text. A typing test there would grade the transport, not the app.

What the sandbox *is* good for on this box: installer/uninstall/autostart/registry, profile isolation,
config parsing, app launch + WebView2 start, and hook-decision assertions read from `keys.log`.

### Cheapest isolated alternative actually present: none for typing

| Option | State on this machine | Cost | Blocking limit |
|---|---|---|---|
| Hyper-V VM (`Microsoft-Hyper-V-All`) | **Disabled**, `vmms` absent, needs elevation + reboot | ~1 h (feature + reboot + Windows install + VM) | Owner forbade feature changes; also the hypervisor is already busy with HVCI/VBS |
| VirtualBox / VMware / QEMU | **Not installed**, no VM images on disk | 10–20 min download + guest install | Needs admin install + a Windows ISO; no ISO present |
| WSL2 (`docker-desktop`) | Running | already paid | No interactive desktop, no low-level hook target, no GUI audio — useless for this harness |
| Second local session (`CodexSandboxOffline`/`Online`) | Accounts exist and are enabled | ~2 min | **Not isolated.** `WH_KEYBOARD_LL` is per-session so it would not see the owner's typing, but the session shares one physical console, GPU and audio; a fast-user-switch removes the owner's desktop for the whole run, and capture in a disconnected console session is unreliable. Hooks and injection there are a courtesy boundary at best |
| Windows Sandbox | **Enabled and working** | 0 | Drops unicode input → no accented typing oracle |

So: the only isolation capable of grading accented typing would be a real VM, and that requires an
elevated feature enable + reboot, i.e. out of scope for this task today.

### Recommended split (what to run where)

1. **Now, headless, no desktop** — engine oracle:
   `powershell -NoProfile -File docs\evkey-clone-work\qa\e2e\check-corpus.ps1` (20/20 paragraphs) and
   `check-edge-cases.ps1` (95/95), plus `vkb simulate --no-jfwz-consonant` for the `w` rule.
2. **Off the owner's desktop, in Sandbox** — installer/uninstall/autostart/profile, app launch, and
   hook-decision assertions from `keys.log` (unicode delivery *not* asserted):
   `wsb.exe E:\vietkb\sandbox-share\e2e\e2e4.wsb`
3. **On the host, one short window** — the 34-case typing E2E (`host-e2e-paragraphs.ps1`, Telex + VNI)
   and the 20-case config matrix. Measured: ~10 min of typing at 45 ms/key, single foreground window,
   plus ~5 min warm-up/focus settling → **ask the owner for a 20–25 minute window**, 30 min if the soak
   case is added. Nothing else in this document needs the desktop.

### `.wsb` skeleton for the off-desktop slice (B variant, unicode not asserted)

```xml
<Configuration>
  <MappedFolders>
    <MappedFolder>
      <HostFolder>E:\vietkb\sandbox-share</HostFolder>
      <SandboxFolder>C:\share</SandboxFolder>
      <ReadOnly>false</ReadOnly>
    </MappedFolder>
  </MappedFolders>
  <Networking>Disable</Networking>
  <MemoryInMB>8192</MemoryInMB>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <LogonCommand>
    <Command>cmd /c C:\share\e2e\run-e2e4.cmd</Command>
  </LogonCommand>
</Configuration>
```

`run-e2e4.cmd` should launch PowerShell with `-File`, write `*.txt`/`*.log`/`*.png` back into `C:\share`,
and end with `shutdown /s /t 5` — that is the pattern every earlier `.wsb` here used (measured working).

## Not covered by any of the above

- **Audio audibility**: whether `SystemAsterisk` (E/V on) and `SystemHand` (off) are audible, and whether
  a WASAPI loopback capture inside a sandbox session sees the sound scheme, is **unknown** — never
  measured. That check needs the host or a real VM with an audio device.
- **Installed-build behaviours** that need the production channel: update flow, autostart at logon,
  Start-menu shortcut, taskbar/tray on a real signed-in desktop. Sandbox proves the *mechanism*
  (`install-diag.txt`), not the owner's build.
- **UIA read-back and screenshots inside a sandbox**: no artifact exists; the current harness reads the
  WinForms `TextBox` in-process and never used UIA. Untested, not assumed to work.
- **30-minute soak** inside a sandbox: unmeasured, and a stale `vmwp` can only be cleared with
  administrator rights, so an unattended soak has a known failure mode this account cannot recover from.
