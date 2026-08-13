param([switch]$SkipFrontend, [switch]$SkipRust)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [System.IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$node = Join-Path $frameworksRoot 'Runtimes\Node\current\npm.cmd'
$cargo = Join-Path $frameworksRoot 'Toolchains\Rust\current\bin\cargo.exe'
$env:FRAMEWORKS_HOME = $frameworksRoot
$env:CARGO_HOME = if (Test-Path (Join-Path $env:USERPROFILE '.cargo\registry\src')) { Join-Path $env:USERPROFILE '.cargo' } else { Join-Path $frameworksRoot 'Toolchains\Rust\cargo-home' }
$env:CARGO_TARGET_DIR = Join-Path $frameworksRoot 'Caches\Rust\target'
$env:PATH = "$(Join-Path $frameworksRoot 'Toolchains\Rust\current\bin');$(Join-Path $frameworksRoot 'Toolchains\C\mingw64\bin');$env:PATH"

Write-Host '== PowerShell syntax =='
$scripts = @('manage-component.ps1', 'manage-component-batch.ps1', 'refresh-update-catalog.ps1', 'refresh-app-update-feed.ps1', 'download-app-update.ps1', 'apply-greendev-update.ps1', 'sync-team-profiles.ps1', 'greendev-cli.ps1', 'greendev-agent.ps1', 'greendev-fleet.ps1', 'greendev-fleet-inventory.ps1', 'sign-greendev-artifact.ps1', 'verify-greendev-signature.ps1', 'New-GreenDevManifest.ps1', 'Test-GreenDevPlugin.ps1', 'restore-user-env.ps1', 'sync-config.ps1', 'setup-dev-env.ps1', 'cleanup.ps1')
foreach ($name in $scripts) {
    $path = Join-Path $frameworksRoot "Scripts\$name"
    $tokens = $null; $errors = $null
    [System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors) | Out-Null
    if ($errors.Count) { throw "$name syntax: $($errors[0].Message)" }
    Write-Host "[OK] $name"
}
foreach ($name in @('build.ps1', 'release.ps1', 'e2e-test.ps1', 'reliability-test.ps1', 'phase17-test.ps1', 'phase20-test.ps1', 'bootstrap-test.ps1')) {
    $path = Join-Path $appRoot $name
    $tokens = $null; $errors = $null
    [System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors) | Out-Null
    if ($errors.Count) { throw "$name syntax: $($errors[0].Message)" }
    Write-Host "[OK] $name"
}

Write-Host '== Reliability and fault injection =='
& (Join-Path $appRoot 'reliability-test.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Reliability checks failed.' }
& (Join-Path $appRoot 'phase17-test.ps1') -SkipWindow
& (Join-Path $appRoot 'phase20-test.ps1') -SkipWindow

Write-Host '== Production frontend wiring =='
$cargoManifest = Get-Content (Join-Path $appRoot 'src-tauri\Cargo.toml') -Raw
$buildScript = Get-Content (Join-Path $appRoot 'build.ps1') -Raw
if ($cargoManifest -notmatch 'custom-protocol\s*=\s*\["tauri/custom-protocol"\]') { throw 'Tauri custom-protocol feature is missing.' }
if ($buildScript -notmatch "--features(?:\s+custom-protocol|',\s*'custom-protocol)") { throw 'Portable build does not enable custom-protocol.' }
if ($cargoManifest -notmatch 'default-run\s*=\s*"greendev-manager"') { throw 'GUI is not the default bundle target.' }
Write-Host '[OK] release builds load embedded frontend assets'

Write-Host '== Manifest plans =='
$manifest = Get-Content (Join-Path $frameworksRoot 'Config\greendev\components.json') -Raw | ConvertFrom-Json
if ($manifest.schemaVersion -notin @(1, 2)) { throw 'Manifest schema mismatch.' }
foreach ($component in $manifest.components) {
    $json = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\manage-component.ps1') -Action plan -Id $component.id
    if ($LASTEXITCODE -ne 0) { throw "Plan failed: $($component.id)" }
    $plan = $json | ConvertFrom-Json
    if ($plan.id -ne $component.id) { throw "Plan ID mismatch: $($component.id)" }
    Write-Host "[OK] $($component.id) -> $($plan.installDir)"
}

Write-Host '== Environment doctor =='
$env:FRAMEWORKS_NOPAUSE = '1'
& (Join-Path $frameworksRoot 'env-setup.bat') en
if ($LASTEXITCODE -ne 0) { throw 'Environment doctor failed.' }

Write-Host '== CLI shared core =='
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\greendev-cli.ps1') validate
if ($LASTEXITCODE -ne 0) { throw 'CLI manifest validation failed.' }
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $frameworksRoot 'Scripts\greendev-cli.ps1') list -Json | ConvertFrom-Json | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'CLI JSON listing failed.' }

if (-not $SkipFrontend) {
    Write-Host '== Frontend build =='
    Push-Location $appRoot
    try { & $node run build; if ($LASTEXITCODE -ne 0) { throw 'Frontend build failed.' } } finally { Pop-Location }
}
if (-not $SkipRust) {
    Write-Host '== Rust tests =='
    & $cargo test --release --offline --features custom-protocol --manifest-path (Join-Path $appRoot 'src-tauri\Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed.' }
}
Write-Host 'Integration checks passed.'
