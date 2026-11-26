# Clicks or types into one window (by title). Used by scripts/look.sh.
# Only the development build's window (it runs under WSLg): never the Windows app itself, which
# may be open beside it with a real save. A window owned by truck-save-editor.exe is passed over.
param([string]$Title, [string]$Action, [int]$X = 0, [int]$Y = 0, [string]$Keys = "", [int]$Wheel = 0)
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public class U {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
"@
[U]::SetProcessDPIAware() | Out-Null
$found = [IntPtr]::Zero
[U]::EnumWindows({ param($h, $l) $sb = New-Object System.Text.StringBuilder 256; [U]::GetWindowText($h, $sb, 256) | Out-Null; $owner = 0; [U]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null; $name = (Get-Process -Id $owner -ErrorAction SilentlyContinue).ProcessName; if ([U]::IsWindowVisible($h) -and $sb.ToString() -like "*$Title*" -and $name -ne "truck-save-editor") { $script:found = $h; return $false }; return $true }, [IntPtr]::Zero) | Out-Null
if ($found -eq [IntPtr]::Zero) { Write-Output "NOTFOUND"; exit 1 }
[U]::SetForegroundWindow($found) | Out-Null
Start-Sleep -Milliseconds 200
$r = New-Object U+RECT; [U]::GetWindowRect($found, [ref]$r) | Out-Null
if ($Action -eq "click") {
  [U]::SetCursorPos($r.L + $X, $r.T + $Y) | Out-Null
  Start-Sleep -Milliseconds 80
  [U]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
  [U]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}
if ($Action -eq "scroll") {
  [U]::SetCursorPos($r.L + $X, $r.T + $Y) | Out-Null
  Start-Sleep -Milliseconds 80
  # Negative scrolls down; 120 is one notch.
  $delta = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int32]$Wheel), 0)
  [U]::mouse_event(0x0800, 0, 0, $delta, [UIntPtr]::Zero)
}
if ($Action -eq "hover") {
  [U]::SetCursorPos($r.L + $X, $r.T + $Y) | Out-Null
}
if ($Action -eq "park") {
  # The pointer just outside the window's bottom-right corner: no hover in a screenshot.
  [U]::SetCursorPos($r.R + 40, $r.B + 40) | Out-Null
}
if ($Action -eq "keys") {
  Add-Type -AssemblyName System.Windows.Forms
  [System.Windows.Forms.SendKeys]::SendWait($Keys)
}
Start-Sleep -Milliseconds 300
Write-Output "OK"
