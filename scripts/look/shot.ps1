# Captures one window (by title) to a PNG, or a part of it (-Crop "x,y,width,height", in the
# window's pixels). Used by scripts/look.sh.
param([string]$Title, [string]$Out, [string]$Crop = "")
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$found = [IntPtr]::Zero
[W]::EnumWindows({ param($h, $l) $sb = New-Object System.Text.StringBuilder 256; [W]::GetWindowText($h, $sb, 256) | Out-Null; if ([W]::IsWindowVisible($h) -and $sb.ToString() -like "*$Title*") { $script:found = $h; return $false }; return $true }, [IntPtr]::Zero) | Out-Null
if ($found -eq [IntPtr]::Zero) { Write-Output "NOTFOUND"; exit 1 }
$r = New-Object W+RECT; [W]::GetWindowRect($found, [ref]$r) | Out-Null
$w = $r.R - $r.L; $hgt = $r.B - $r.T
$bmp = New-Object System.Drawing.Bitmap $w, $hgt
$g = [System.Drawing.Graphics]::FromImage($bmp); $hdc = $g.GetHdc()
[W]::PrintWindow($found, $hdc, 2) | Out-Null
$g.ReleaseHdc($hdc); $g.Dispose()
if ($Crop -ne "") {
  $c = $Crop.Split(",") | ForEach-Object { [int]$_ }
  $part = $bmp.Clone((New-Object System.Drawing.Rectangle $c[0], $c[1], $c[2], $c[3]), $bmp.PixelFormat)
  $bmp.Dispose(); $bmp = $part; $w = $c[2]; $hgt = $c[3]
}
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png); Write-Output "OK $w x $hgt"
