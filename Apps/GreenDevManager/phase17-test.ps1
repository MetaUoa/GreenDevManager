param([switch]$SkipWindow)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$source = Get-Content (Join-Path $appRoot 'src\App.tsx') -Raw
$styles = Get-Content (Join-Path $appRoot 'src\styles.css') -Raw
$html = Get-Content (Join-Path $appRoot 'index.html') -Raw

Write-Host '== Accessibility gates =='
foreach ($marker in @('aria-current', 'aria-label="主要导航"', 'aria-live="polite"', 'href="#main-content"', 'event.ctrlKey', 'PageDown')) { if (-not $source.Contains($marker)) { throw "Missing accessibility marker: $marker" } }
foreach ($marker in @(':focus-visible', '.skip-link', 'prefers-reduced-motion')) { if (-not $styles.Contains($marker)) { throw "Missing accessibility style: $marker" } }
foreach ($marker in @('BootstrapWizard', 'select_frameworks_directory', 'initialize_frameworks_root', '全新下载配置', '接入现有环境')) { if (-not $source.Contains($marker)) { throw "Missing first-run marker: $marker" } }
if ($html -notmatch '<html\s+lang="zh-CN"' -or $html -notmatch 'name="viewport"') { throw 'HTML language or viewport metadata is missing.' }
Write-Host '[OK] keyboard navigation, focus, live status, language and reduced motion'

$routes = @('overview','components','android','manifest','catalog','updater','profiles','recovery','stability','supply','fleet','developer','enterprise','environment','cache','config','diagnostics','logs')
foreach ($route in $routes) { if ($source -notmatch "view === `"$route`"") { throw "GUI route is missing: $route" } }
Write-Host "[OK] $($routes.Count) GUI routes are wired"

if (-not $SkipWindow) {
    Write-Host '== Production WebView navigation and DPI =='
    Add-Type -AssemblyName System.Drawing
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class GreenDevUiNative {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
}
'@
    $executable = Join-Path $appRoot 'GreenDevManager.exe'
    if (-not (Test-Path -LiteralPath $executable)) { throw 'Production executable is missing.' }
    $captureRoot = Join-Path ([IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))) 'Caches\GreenDevManager\phase17-ui'
    New-Item -ItemType Directory -Path $captureRoot -Force | Out-Null
    $process = Start-Process -FilePath $executable -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(15)
        do { Start-Sleep -Milliseconds 300; $process.Refresh() } until ($process.HasExited -or $process.MainWindowHandle -ne 0 -or [DateTime]::UtcNow -ge $deadline)
        if ($process.HasExited -or $process.MainWindowHandle -eq 0) { throw 'Production window did not open.' }
        # user32 virtualizes coordinates for a DPI-unaware PowerShell host. Opt the
        # capture thread into per-monitor v2 so window rectangles and bitmaps use
        # the same physical pixel coordinate space as WebView2.
        [GreenDevUiNative]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
        $dpi = [GreenDevUiNative]::GetDpiForWindow($process.MainWindowHandle)
        $scale = if ($dpi) { $dpi / 96.0 } else { 1.0 }
        [GreenDevUiNative]::SetWindowPos($process.MainWindowHandle, [IntPtr](-1), 40, 40, [int](1240*$scale), [int](820*$scale), 0x0040) | Out-Null
        [GreenDevUiNative]::SetForegroundWindow($process.MainWindowHandle) | Out-Null
        Start-Sleep -Seconds 2
        $shell = New-Object -ComObject WScript.Shell
        $shell.AppActivate($process.Id) | Out-Null
        # Start from the visible Overview navigation button instead of the search input.
        # WebView2 may reserve Ctrl+PageDown while a text input owns focus.
        [GreenDevUiNative]::SetCursorPos([int](118*$scale),[int](212*$scale)) | Out-Null
        [GreenDevUiNative]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
        [GreenDevUiNative]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
        Start-Sleep -Milliseconds 250
        $hashes = @()
        for ($index = 0; $index -lt $routes.Count; $index++) {
            foreach ($size in @(@(960,680),@(1240,820),@(1600,900))) {
                if ($index -gt 0 -and $size[0] -eq 960) {
                    [GreenDevUiNative]::SetForegroundWindow($process.MainWindowHandle) | Out-Null
                    $shell.AppActivate($process.Id) | Out-Null
                    Start-Sleep -Milliseconds 80
                    [Windows.Forms.SendKeys]::SendWait('^{PGDN}')
                    Start-Sleep -Milliseconds 300
                }
                $nativeWidth = [int]($size[0] * $scale); $nativeHeight = [int]($size[1] * $scale)
                [GreenDevUiNative]::SetWindowPos($process.MainWindowHandle, [IntPtr](-1), 40, 40, $nativeWidth, $nativeHeight, 0x0040) | Out-Null
                # Wait for WebView2's layout and compositor to observe the new DPI-scaled viewport.
                Start-Sleep -Milliseconds 180
                $rect = New-Object GreenDevUiNative+RECT; [GreenDevUiNative]::GetWindowRect($process.MainWindowHandle, [ref]$rect) | Out-Null
                $bitmap = New-Object Drawing.Bitmap ($rect.Right-$rect.Left),($rect.Bottom-$rect.Top); $graphics = [Drawing.Graphics]::FromImage($bitmap); $hdc = $graphics.GetHdc()
                try { if (-not [GreenDevUiNative]::PrintWindow($process.MainWindowHandle,$hdc,2)) { throw 'PrintWindow failed.' } } finally { $graphics.ReleaseHdc($hdc) }
                $path = Join-Path $captureRoot "$index-$($size[0])x$($size[1]).png"; $bitmap.Save($path,[Drawing.Imaging.ImageFormat]::Png); $graphics.Dispose(); $bitmap.Dispose()
                if ((Get-Item $path).Length -lt 20000) { throw "Suspiciously empty capture: $path" }
                if ($size[0] -eq 960) { $hashes += (Get-FileHash $path -Algorithm SHA256).Hash }
            }
        }
        if (@($hashes | Select-Object -Unique).Count -lt 8) { throw 'Keyboard route traversal did not produce enough distinct pages.' }
        Write-Host "[OK] $($routes.Count) routes traversed at 960x680, 1240x820, and 1600x900"
    } finally { if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force } }
}
Write-Host 'Phase 17 GUI checks passed.'
