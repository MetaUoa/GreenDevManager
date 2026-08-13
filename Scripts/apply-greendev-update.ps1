param(
    [Parameter(Mandatory = $true)][string]$PendingPath,
    [Parameter(Mandatory = $true)][int]$CurrentPid
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$pendingFull = [System.IO.Path]::GetFullPath($PendingPath)
$expectedPending = [System.IO.Path]::GetFullPath((Join-Path $root 'Config\greendev\pending-app-update.json'))
if (-not [string]::Equals($pendingFull, $expectedPending, [StringComparison]::OrdinalIgnoreCase)) { throw 'Pending update path mismatch.' }
$pending = Get-Content -LiteralPath $pendingFull -Raw | ConvertFrom-Json
$stage = [System.IO.Path]::GetFullPath([string]$pending.stage)
$stageRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'Caches\GreenDevManager\app-update-stage')).TrimEnd('\') + '\'
$appRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'Apps\GreenDevManager'))
if (-not $stage.StartsWith($stageRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Update stage is outside the managed cache.' }
if (-not (Test-Path -LiteralPath (Join-Path $stage 'GreenDevManager.exe') -PathType Leaf)) { throw 'Staged executable is missing.' }

$deadline = [DateTime]::UtcNow.AddSeconds(30)
while ((Get-Process -Id $CurrentPid -ErrorAction SilentlyContinue) -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 250 }
if (Get-Process -Id $CurrentPid -ErrorAction SilentlyContinue) { throw 'GreenDev Manager did not exit before the update deadline.' }

$stamp = Get-Date -Format 'yyyyMMddHHmmssfff'
$backup = Join-Path $root "Caches\GreenDevManager\app-update-backups\$($pending.version)-$stamp"
New-Item -ItemType Directory -Path $backup -Force | Out-Null
$managedFiles = @(
    [pscustomobject]@{ Name = 'GreenDevManager.exe'; Target = (Join-Path $appRoot 'GreenDevManager.exe') },
    [pscustomobject]@{ Name = 'WebView2Loader.dll'; Target = (Join-Path $appRoot 'WebView2Loader.dll') },
    [pscustomobject]@{ Name = 'greendev.exe'; Target = (Join-Path $root 'greendev.exe') }
)
foreach ($file in $managedFiles) { if (Test-Path -LiteralPath $file.Target) { Copy-Item -LiteralPath $file.Target -Destination (Join-Path $backup $file.Name) } }

try {
    foreach ($file in $managedFiles) {
        $source = Join-Path $stage $file.Name
        if (-not (Test-Path -LiteralPath $source)) { continue }
        $temporary = "$($file.Target).updating"
        Copy-Item -LiteralPath $source -Destination $temporary -Force
        Move-Item -LiteralPath $temporary -Destination $file.Target -Force
    }
    $process = Start-Process -FilePath (Join-Path $appRoot 'GreenDevManager.exe') -PassThru
    Start-Sleep -Seconds 5
    if ($process.HasExited) { throw "Updated application exited during health window: $($process.ExitCode)" }
    Move-Item -LiteralPath $pendingFull -Destination "$pendingFull.applied-$stamp.json" -Force
} catch {
    foreach ($file in $managedFiles) { $saved = Join-Path $backup $file.Name; if (Test-Path -LiteralPath $saved) { Copy-Item -LiteralPath $saved -Destination $file.Target -Force } }
    Move-Item -LiteralPath $pendingFull -Destination "$pendingFull.failed-$stamp.json" -Force
    Start-Process -FilePath (Join-Path $appRoot 'GreenDevManager.exe') | Out-Null
    throw
}
