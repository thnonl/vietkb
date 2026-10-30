. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -Namespace T -Name K4 -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool ctrl, bool shift) {
  INPUT[] a = new INPUT[6]; int i = 0;
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; i++; }
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (shift) { a[i].type = 1; a[i].u.ki.wVk = 0x10; a[i].u.ki.dwFlags = 2; i++; }
  if (ctrl) { a[i].type = 1; a[i].u.ki.wVk = 0x11; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
$vs = Get-TopWindow 'scratch-vsc.txt'
if (-not $vs) { Write-Host 'already closed'; exit 0 }
[void][QAInput]::Focus($vs.Hwnd); Start-Sleep -Milliseconds 300
[QAInput]::Combo(0x1B, $false, $false, $false)   # dismiss the dialog (Cancel)
Start-Sleep -Milliseconds 500
[void][T.K4]::Tap(0x57, $true, $true)            # Ctrl+Shift+W
Start-Sleep -Milliseconds 1000
$hit = $null
foreach ($h in [QAInput]::TopWindows()) {
  if ([QAInput]::Title($h) -match 'scratch-vsc') {
    foreach ($e in [System.Windows.Automation.AutomationElement]::FromHandle($h).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
      try {
        $ct = $e.Current.ControlType.ProgrammaticName
        $n = $e.Current.Name
        if ($ct -eq 'ControlType.Button' -and ($n -eq "Don't Save" -or $n -eq 'Không lưu' -or $n -eq 'Do not save')) { $hit = $e }
      } catch {}
    }
  }
}
if ($hit) {
  Write-Host "click dont-save [$($hit.Current.Name)]"
  try { $hit.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $hit) }
  Start-Sleep -Milliseconds 1200
}
Write-Host "vscode_gone=$(-not [bool](Get-TopWindow 'scratch-vsc.txt'))"

