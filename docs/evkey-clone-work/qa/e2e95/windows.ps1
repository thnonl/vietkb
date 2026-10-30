. 'E:\vietkb\docs\evkey-clone-work\qa\e2e95\lib95.ps1'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public class WEnum {
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
 [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
}
'@
foreach ($h in [QAInput]::TopWindows()) {
  $t = [QAInput]::Title($h)
  if (-not $t) { continue }
  $c = New-Object System.Text.StringBuilder 256
  [void][WEnum]::GetClassNameW($h, $c, 256)
  $r = New-Object WEnum+RECT
  [void][WEnum]::GetWindowRect($h, [ref]$r)
  $pid2 = [QAInput]::Pid($h)
  $pn = (Get-Process -Id $pid2 -ErrorAction SilentlyContinue).ProcessName
  Write-Host ("{0,-8} {1,-24} {2,6},{3,-6} {4,-24} [{5}]" -f $h, $pn, $r.L, $r.T, $c.ToString(), $t)
}
