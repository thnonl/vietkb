. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
$dir = 'E:\vietkb\docs\evkey-clone-work\qa\e2e95'
$out = Join-Path $dir 'tray-results.txt'
Add-Content -Encoding UTF8 $out 'TRAY (overflow flyout)'
function W($s) { Add-Content -Encoding UTF8 $out $s; Write-Host $s }
Add-Type -Namespace W -Name K2 -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool win) {
  INPUT[] a = new INPUT[4]; int i = 0;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (win) { a[i].type = 1; a[i].u.ki.wVk = 0x5B; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
[W.K2]::Tap(0x42, $true)   # Win+B
Start-Sleep -Milliseconds 700
# Right until "Show Hidden Icons" focused, then open it
for ($i = 0; $i -lt 8; $i++) {
  $n = ''
  try { $n = [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name } catch {}
  if ($n -match 'Hidden Icons|biểu tượng ẩn') { W "focused=$n"; break }
  [QAInput]::Combo(0x27, $false, $false, $false); Start-Sleep -Milliseconds 250
}
[QAInput]::Combo(0x0D, $false, $false, $false)   # Enter opens the flyout
Start-Sleep -Milliseconds 1500
$hit = $null; $hitWin = $null
foreach ($h in [QAInput]::TopWindows()) {
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try {
        $n = $e.Current.Name
        if ($n -match 'VietKB') { W "flyout element type=$($e.Current.ControlType.ProgrammaticName) name=[$n] rect=$($e.Current.BoundingRectangle)"; if (-not $hit) { $hit = $e; $hitWin = $h } }
      } catch {}
    }
  } catch {}
}
if (-not $hit) { W 'flyout=NO_VIETKB'; W 'TRAY-DONE'; exit 0 }
# icon accessible name is the state/tooltip
Start-Sleep -Milliseconds 300
[void](Click-Element $hit)    # opens the tray menu (left click = menu for this app)
Start-Sleep -Milliseconds 1200
$items = @()
foreach ($h in [QAInput]::TopWindows()) {
  try {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
    foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try { if ($e.Current.ControlType.ProgrammaticName -match 'MenuItem') { $items += [pscustomobject]@{ Name = $e.Current.Name; El = $e } } } catch {}
    }
  } catch {}
}
W "menu_items=$($items.Count): $((($items | ForEach-Object { $_.Name }) -join ' | '))"
$toggle = $items | Where-Object { $_.Name -match 'tiếng Việt' } | Select-Object -First 1
if ($toggle) {
  W "tray_toggle_item=[$($toggle.Name)]"
  try { $toggle.El.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $toggle.El) }
  Start-Sleep -Milliseconds 1500
  $ch = Get-TopWindow 'VietKB E2E probe'
  if ($ch) {
    [void][QAInput]::Focus($ch.Hwnd); Start-Sleep -Milliseconds 250
    $el = Get-EditElement $ch.Hwnd; if ($el) { [void](Click-Element $el) }
    Clear-TextArea $ch.Hwnd
    [QAInput]::TypeAscii('ba6t5', 120); Start-Sleep -Milliseconds 600
    $t = ([string](Get-UiaText $ch.Hwnd).Text).TrimEnd("`r", "`n")
    W "after_tray_toggle actual=[$t] state=$(if ($t -match 'ậ') { 'ON' } elseif ($t -ceq 'ba6t5') { 'OFF' } else { 'OTHER' })"
  }
} else { W 'tray_toggle_item=NOT_FOUND'; [QAInput]::Combo(0x1B, $false, $false, $false) }
W 'TRAY-DONE'
