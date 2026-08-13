param([switch]$SkipBuild, [switch]$SkipIntegration)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [System.IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$env:FRAMEWORKS_HOME = $frameworksRoot

if (-not $SkipIntegration) { & (Join-Path $appRoot 'integration-test.ps1') }
if (-not $SkipBuild) { & (Join-Path $appRoot 'build.ps1') }
$executable = Join-Path $appRoot 'GreenDevManager.exe'
if (-not (Test-Path -LiteralPath $executable)) { throw 'GreenDevManager.exe is missing.' }
$bytes = [System.IO.File]::ReadAllBytes($executable)
$ascii = [System.Text.Encoding]::ASCII.GetString($bytes)
if (-not $ascii.Contains('tauri://localhost')) { throw 'Production custom protocol marker was not found.' }
if (-not $ascii.Contains('/assets/index-')) { throw 'Embedded frontend asset marker was not found.' }
if (-not $ascii.Contains('get_app_update_status')) { throw 'Phase 14 updater command marker was not found.' }
if (-not $ascii.Contains('get_profile_sets')) { throw 'Phase 15 profile command marker was not found.' }
Write-Host '[OK] embedded frontend protocol'

$process = Start-Process -FilePath $executable -PassThru
try {
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do {
        Start-Sleep -Milliseconds 400
        $process.Refresh()
    } until ($process.HasExited -or $process.MainWindowHandle -ne 0 -or [DateTime]::UtcNow -ge $deadline)
    if ($process.HasExited) { throw "Application exited during startup: $($process.ExitCode)" }
    if ($process.MainWindowHandle -eq 0) { throw 'Application window was not created.' }
    if (-not $process.Responding) { throw 'Application window is not responding.' }
    if ($process.MainWindowTitle -ne 'GreenDev Manager') { throw "Unexpected window title: $($process.MainWindowTitle)" }
    Write-Host "[OK] production window: $($process.MainWindowTitle)"
} finally {
    if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
}
& (Join-Path $appRoot 'phase17-test.ps1')
& (Join-Path $appRoot 'phase20-test.ps1') -SkipWindow
