param(
    [Parameter(Mandatory = $true)]
    [string]$Ids,
    [string]$ManifestPath
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
if (-not $ManifestPath) { $ManifestPath = Join-Path $root 'Config\greendev\components.json' }
$items = @($Ids -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
if ($items.Count -eq 0) { throw 'At least one component ID is required.' }
if ($items | Where-Object { $_ -notmatch '^[a-z0-9][a-z0-9._-]{0,63}$' }) { throw 'A component ID has invalid characters.' }

$index = 0
foreach ($id in $items) {
    $index++
    Write-Host "[$index/$($items.Count)] Updating $id"
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'manage-component.ps1') -Action update -Id $id -ManifestPath $ManifestPath
    if ($LASTEXITCODE -ne 0) { throw "Component update failed: $id" }
}
Write-Host "Batch completed: $($items -join ', ')"
