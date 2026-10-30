. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type -Namespace T -Name K3 -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx; public int dy; public uint data; public uint flags; public uint time; public IntPtr extra; }
[StructLayout(LayoutKind.Sequential)] public struct HARDINPUT { public uint msg; public ushort p1; public ushort p2; }
[StructLayout(LayoutKind.Explicit)] public struct InputUnion { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public HARDINPUT hi; }
[StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public InputUnion u; }
public static void Tap(ushort vk, bool alt) {
  INPUT[] a = new INPUT[4]; int i = 0;
  if (alt) { a[i].type = 1; a[i].u.ki.wVk = 0x12; i++; }
  a[i].type = 1; a[i].u.ki.wVk = vk; i++;
  a[i].type = 1; a[i].u.ki.wVk = vk; a[i].u.ki.dwFlags = 2; i++;
  if (alt) { a[i].type = 1; a[i].u.ki.wVk = 0x12; a[i].u.ki.dwFlags = 2; i++; }
  SendInput((uint)i, a, Marshal.SizeOf(typeof(INPUT)));
}
'@
$vs = Get-TopWindow 'scratch-vsc.txt'
if ($vs) {
  [void][QAInput]::Focus($vs.Hwnd); Start-Sleep -Milliseconds 400
  [void][T.K3]::Tap(0x73, $true)   # Alt+F4
  Start-Sleep -Milliseconds 1200
  foreach ($h in [QAInput]::TopWindows()) {
    $t = [QAInput]::Title($h)
    if ($t -match 'scratch-vsc') {
      Write-Host "dialog? title=[$t]"
      $root = [System.Windows.Automation.AutomationElement]::FromHandle($h)
      foreach ($e in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        try {
          $n = $e.Current.Name
          if ($e.Current.ControlType.ProgrammaticName -match 'Button' -and $n) { Write-Host "  button=[$n]" }
        } catch {}
      }
    }
  }
  $btn = $null
  foreach ($h in [QAInput]::TopWindows()) {
    $t = [QAInput]::Title($h)
    if ($t -match 'scratch-vsc') {
      foreach ($e in [System.Windows.Automation.AutomationElement]::FromHandle($h).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        try { if ($e.Current.Name -match "Don't Save|Không lưu|Do not save") { $btn = $e; break } } catch {}
      }
    }
  }
  if ($btn) {
    Write-Host "clicking [$($btn.Current.Name)]"
    try { $btn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch { [void](Click-Element $btn) }
    Start-Sleep -Milliseconds 1000
  }
  Write-Host "vscode_gone=$(-not [bool](Get-TopWindow 'scratch-vsc.txt'))"
}
