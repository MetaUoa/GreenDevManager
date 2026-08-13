param([switch]$SkipWindow)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$source = Get-Content (Join-Path $appRoot 'src\App.tsx') -Raw
$rust = Get-Content (Join-Path $appRoot 'src-tauri\src\lib.rs') -Raw
$release = Get-Content (Join-Path $appRoot 'release.ps1') -Raw

Write-Host '== First-run static gates =='
foreach ($marker in @('BootstrapWizard', '全新下载配置', '接入现有环境', 'select_frameworks_directory', 'initialize_frameworks_root')) {
    if (-not $source.Contains($marker)) { throw "Missing first-run UI marker: $marker" }
}
foreach ($marker in @('BOOTSTRAP_MANIFEST_URL', 'safe_bootstrap_entry', 'persist_frameworks_root', '全新初始化需要选择空目录')) {
    if (-not $rust.Contains($marker)) { throw "Missing bootstrap core marker: $marker" }
}
foreach ($marker in @('bootstrap-manifest.json', 'GreenDevManager-bootstrap-', 'Compress-Archive')) {
    if (-not $release.Contains($marker)) { throw "Missing bootstrap release marker: $marker" }
}
Write-Host '[OK] first-run UI, root persistence, archive confinement and release payload are wired'

if ($SkipWindow) { Write-Host 'First-run gates passed.'; exit 0 }

$executable = Join-Path $appRoot 'GreenDevManager.exe'
$loader = Join-Path $appRoot 'WebView2Loader.dll'
if (-not (Test-Path -LiteralPath $executable) -or -not (Test-Path -LiteralPath $loader)) { throw 'Production executable is missing.' }
$isolated = Join-Path $env:TEMP 'GreenDevManager-FirstRun-Test'
New-Item -ItemType Directory -Path $isolated -Force | Out-Null
Copy-Item -LiteralPath $executable, $loader -Destination $isolated -Force
$savedRoot = $env:FRAMEWORKS_HOME
$savedDisable = $env:GREENDEV_DISABLE_SAVED_ROOT
Remove-Item Env:FRAMEWORKS_HOME -ErrorAction SilentlyContinue
$env:GREENDEV_DISABLE_SAVED_ROOT = '1'
$process = $null
try {
    $process = Start-Process -FilePath (Join-Path $isolated 'GreenDevManager.exe') -WorkingDirectory $isolated -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do { Start-Sleep -Milliseconds 250; $process.Refresh() } until ($process.HasExited -or $process.MainWindowHandle -ne 0 -or [DateTime]::UtcNow -ge $deadline)
    if ($process.HasExited -or $process.MainWindowHandle -eq 0) { throw 'First-run window did not open.' }
    Add-Type -AssemblyName System.Drawing
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class GreenDevBootstrapNative {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdc, uint flags);
}
'@
    [GreenDevBootstrapNative]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
    $dpi = [GreenDevBootstrapNative]::GetDpiForWindow($process.MainWindowHandle)
    $scale = if ($dpi) { $dpi / 96.0 } else { 1.0 }
    [GreenDevBootstrapNative]::SetWindowPos($process.MainWindowHandle,[IntPtr](-1),40,40,[int](1240*$scale),[int](820*$scale),0x0040) | Out-Null
    Start-Sleep -Seconds 2
    $rect = New-Object GreenDevBootstrapNative+RECT
    [GreenDevBootstrapNative]::GetWindowRect($process.MainWindowHandle,[ref]$rect) | Out-Null
    $bitmap = New-Object Drawing.Bitmap ($rect.Right-$rect.Left),($rect.Bottom-$rect.Top)
    $graphics = [Drawing.Graphics]::FromImage($bitmap); $hdc = $graphics.GetHdc()
    try { if (-not [GreenDevBootstrapNative]::PrintWindow($process.MainWindowHandle,$hdc,2)) { throw 'First-run PrintWindow failed.' } } finally { $graphics.ReleaseHdc($hdc) }
    $captureRoot = Join-Path $frameworksRoot 'Caches\GreenDevManager\bootstrap-ui'
    New-Item -ItemType Directory -Path $captureRoot -Force | Out-Null
    $capture = Join-Path $captureRoot 'first-run-1240x820.png'
    $bitmap.Save($capture,[Drawing.Imaging.ImageFormat]::Png); $graphics.Dispose(); $bitmap.Dispose()
    if ((Get-Item $capture).Length -lt 20000) { throw 'First-run capture is suspiciously empty.' }
    Write-Host "[OK] isolated installer-style launch rendered first-run wizard: $capture"
} finally {
    if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    if ($null -eq $savedRoot) { Remove-Item Env:FRAMEWORKS_HOME -ErrorAction SilentlyContinue } else { $env:FRAMEWORKS_HOME = $savedRoot }
    if ($null -eq $savedDisable) { Remove-Item Env:GREENDEV_DISABLE_SAVED_ROOT -ErrorAction SilentlyContinue } else { $env:GREENDEV_DISABLE_SAVED_ROOT = $savedDisable }
}
Write-Host 'First-run gates passed.'
