param([switch]$KeepFixture)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [System.IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$fixture = Join-Path $frameworksRoot 'Caches\GreenDevManager\phase12-fixture'
$manifestPath = Join-Path $frameworksRoot 'Config\greendev\phase12-fixture.json'
$archive = Join-Path $fixture 'fault.zip'
$env:FRAMEWORKS_HOME = $frameworksRoot

if (Test-Path -LiteralPath $fixture) { Remove-Item -LiteralPath $fixture -Recurse -Force }
New-Item -ItemType Directory -Path $fixture -Force | Out-Null
Set-Content -LiteralPath (Join-Path $fixture 'payload.txt') -Value 'fault injection payload' -Encoding ASCII
Compress-Archive -LiteralPath (Join-Path $fixture 'payload.txt') -DestinationPath $archive
$manifest = [ordered]@{
    schemaVersion = 2
    components = @([ordered]@{
        id = 'phase12-fixture'; name = 'Phase 12 Fixture'; version = '1.0.0'; enabled = $true; dependsOn = @()
        installDir = 'Caches\GreenDevManager\phase12-fixture\installed'; currentLink = $null; healthPath = 'payload.txt'; archiveRoot = ''
        source = [ordered]@{ type = 'archive'; url = ''; archive = 'Caches\GreenDevManager\phase12-fixture\fault.zip'; sha256 = ('0' * 64) }
    })
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

try {
    Write-Host '== Fault injection: checksum mismatch =='
    $ErrorActionPreference = 'Continue'
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\manage-component.ps1') -Action install -Id phase12-fixture -ManifestPath $manifestPath 2>&1
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -eq 0 -or ($output -join "`n") -notmatch 'SHA256 mismatch') { throw 'Checksum failure did not stop the transaction.' }
    if (Test-Path -LiteralPath (Join-Path $fixture 'installed')) { throw 'Failed install left a target directory.' }
    Write-Host '[OK] checksum mismatch stopped before extraction'

    Write-Host '== Fault injection: network and disk stages =='
    $validHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
    $manifest.components[0].source.sha256 = $validHash
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    Move-Item -LiteralPath $archive -Destination "$archive.saved"
    $env:GREENDEV_FAULT_STAGE = 'before-download'
    $ErrorActionPreference = 'Continue'
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\manage-component.ps1') -Action install -Id phase12-fixture -ManifestPath $manifestPath 2>&1
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -eq 0 -or ($output -join "`n") -notmatch 'before-download') { throw 'Network failpoint did not stop the task.' }
    Move-Item -LiteralPath "$archive.saved" -Destination $archive
    $env:GREENDEV_FAULT_STAGE = 'after-hash'
    $ErrorActionPreference = 'Continue'
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\manage-component.ps1') -Action install -Id phase12-fixture -ManifestPath $manifestPath 2>&1
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -eq 0 -or ($output -join "`n") -notmatch 'after-hash' -or (Test-Path -LiteralPath (Join-Path $fixture 'installed'))) { throw 'Disk failpoint left a target directory.' }
    Write-Host '[OK] network interruption and simulated disk failure preserved the target'

    Write-Host '== Fault injection: current rollback =='
    $previous = Join-Path $fixture 'previous'; New-Item -ItemType Directory -Path $previous -Force | Out-Null; Set-Content -LiteralPath (Join-Path $previous 'payload.txt') -Value 'previous' -Encoding ASCII
    $current = Join-Path $fixture 'current'; & cmd.exe /d /c "mklink /J `"$current`" `"$previous`"" | Out-Null; if ($LASTEXITCODE -ne 0) { throw 'Fixture junction creation failed.' }
    $manifest.components[0].currentLink = 'Caches\GreenDevManager\phase12-fixture\current'
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    $env:GREENDEV_FAULT_STAGE = 'before-switch'
    $ErrorActionPreference = 'Continue'
    $output = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\manage-component.ps1') -Action update -Id phase12-fixture -ManifestPath $manifestPath 2>&1
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -eq 0 -or ($output -join "`n") -notmatch 'before-switch') { throw 'Switch failpoint did not trigger.' }
    $restored = [IO.Path]::GetFullPath((Get-Item -LiteralPath $current).Target)
    if (-not [string]::Equals($restored,[IO.Path]::GetFullPath($previous),[StringComparison]::OrdinalIgnoreCase)) { throw 'Previous current junction was not restored.' }
    Write-Host '[OK] current junction rolled back to the previous healthy target'

    Write-Host '== Transaction and UI wiring =='
    $rust = Get-Content (Join-Path $appRoot 'src-tauri\src\lib.rs') -Raw
    $ui = Get-Content (Join-Path $appRoot 'src\App.tsx') -Raw
    foreach ($marker in @('pause_task', 'resume_task', 'retry_task', 'write_transaction')) { if (-not $rust.Contains($marker)) { throw "Missing reliability marker: $marker" } }
    foreach ($marker in @('CatalogView', 'UpdaterView', 'ProfilesView')) { if (-not $ui.Contains($marker)) { throw "Missing GUI route: $marker" } }
    Write-Host '[OK] task controls, transaction persistence, and Phase 13-15 routes'
} finally {
    Remove-Item Env:\GREENDEV_FAULT_STAGE -ErrorAction SilentlyContinue
    if (-not $KeepFixture) {
        if (Test-Path -LiteralPath $manifestPath) { Remove-Item -LiteralPath $manifestPath -Force }
        if (Test-Path -LiteralPath $fixture) { Remove-Item -LiteralPath $fixture -Recurse -Force }
    }
}

Write-Host 'Reliability checks passed.'
$global:LASTEXITCODE = 0
