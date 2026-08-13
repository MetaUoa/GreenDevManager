param([switch]$SkipWindow)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$source = Get-Content (Join-Path $appRoot 'src-tauri\src\lib.rs') -Raw
$phaseSource = Get-Content (Join-Path $appRoot 'src-tauri\src\phase20_23.rs') -Raw

Write-Host '== Phase 20-23 static gates =='
foreach ($marker in @('PersistedTask', 'restore_persisted_tasks', 'acquire_single_instance', 'write_task_record')) { if (-not $source.Contains($marker)) { throw "Missing reliability marker: $marker" } }
foreach ($marker in @('get_reliability_status', 'get_supply_chain_status', 'preview_fleet_rollout', 'get_ecosystem_status')) { if (-not $phaseSource.Contains($marker)) { throw "Missing platform command: $marker" } }
foreach ($name in @('reliability-policy.json','supply-chain-policy.json','remote-nodes.json')) { Get-Content (Join-Path $frameworksRoot "Config\greendev\$name") -Raw | ConvertFrom-Json | Out-Null }
Write-Host '[OK] persistent queue, policies, supply chain, fleet and ecosystem commands'

Write-Host '== SDK and signature fixtures =='
$fixture = Join-Path $frameworksRoot 'Caches\GreenDevManager\phase23-sdk-gate.json'
& (Join-Path $frameworksRoot 'Scripts\New-GreenDevManifest.ps1') -Id phase23-gate -OutputPath $fixture | Out-Null
if ((Get-Content $fixture -Raw | ConvertFrom-Json).components[0].id -ne 'phase23-gate') { throw 'Manifest SDK output mismatch.' }
$pluginResult = & (Join-Path $frameworksRoot 'Scripts\Test-GreenDevPlugin.ps1') -Path (Join-Path $frameworksRoot 'Config\greendev\examples\plugin.json') | ConvertFrom-Json
if (-not $pluginResult.valid) { throw 'Plugin validation failed.' }
Write-Host '[OK] Manifest SDK and plugin permission validation'

$inventory = & (Join-Path $frameworksRoot 'Scripts\greendev-fleet-inventory.ps1') | ConvertFrom-Json
if ($inventory.schemaVersion -ne 1 -or @($inventory.nodes).Count -ne 0) { throw 'Empty fleet inventory fixture mismatch.' }
Write-Host '[OK] empty fleet read-only inventory'

Write-Host '== PATH-independent update source =='
$savedPath = $env:PATH
try {
    $env:PATH = Join-Path $frameworksRoot 'Runtimes\Node\current'
    & 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe' -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\refresh-app-update-feed.ps1') -Local | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Update source refresh failed with a stripped PATH.' }
} finally { $env:PATH = $savedPath }
$feed = Get-Content (Join-Path $frameworksRoot 'Caches\GreenDevManager\app-update-feed.json') -Raw | ConvertFrom-Json
if (-not $feed.channels.stable -or -not $feed.channels.beta) { throw 'Update source channels were not preserved.' }
foreach ($channel in @($feed.channels.stable, $feed.channels.beta)) {
    if (-not (Test-Path -LiteralPath ([string]$channel.manifest) -PathType Leaf)) { throw "Local feed manifest is unresolved: $($channel.manifest)" }
}
Write-Host '[OK] background update tools resolve without the system PATH'

if (-not $SkipWindow) {
    Write-Host '== Single instance gate =='
    $executable = Join-Path $appRoot 'GreenDevManager.exe'
    $first = Start-Process -FilePath $executable -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(12)
        do { Start-Sleep -Milliseconds 250; $first.Refresh() } until ($first.MainWindowHandle -ne 0 -or $first.HasExited -or [DateTime]::UtcNow -ge $deadline)
        if ($first.HasExited -or $first.MainWindowHandle -eq 0) { throw 'Primary window did not start.' }
        $second = Start-Process -FilePath $executable -PassThru
        if (-not $second.WaitForExit(5000)) { Stop-Process -Id $second.Id -Force; throw 'Second instance stayed active.' }
        Write-Host '[OK] second launch exits while primary remains active'
    } finally { if ($first -and -not $first.HasExited) { Stop-Process -Id $first.Id -Force } }
}
Write-Host 'Phase 20-23 gates passed.'
