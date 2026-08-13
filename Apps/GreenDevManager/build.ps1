$ErrorActionPreference = 'Stop'

$appRoot = $PSScriptRoot
$frameworksRoot = [System.IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$portableNode = Join-Path $frameworksRoot 'Runtimes\Node\current\node.exe'
$node = if (Test-Path -LiteralPath $portableNode) { $portableNode } else { (Get-Command node.exe -ErrorAction Stop).Source }
$nodeHome = Split-Path -Parent $node
$npm = if (Test-Path -LiteralPath (Join-Path $nodeHome 'npm.cmd')) { Join-Path $nodeHome 'npm.cmd' } else { (Get-Command npm.cmd -ErrorAction Stop).Source }
$portableRust = Join-Path $frameworksRoot 'Toolchains\Rust\current\bin\rustc.exe'
$rust = if (Test-Path -LiteralPath $portableRust) { $portableRust } else { (Get-Command rustc.exe -ErrorAction Stop).Source }
$rustBin = Split-Path -Parent $rust
$mingwBin = Join-Path $frameworksRoot 'Toolchains\C\mingw64\bin'
$cargo = if (Test-Path -LiteralPath (Join-Path $rustBin 'cargo.exe')) { Join-Path $rustBin 'cargo.exe' } else { (Get-Command cargo.exe -ErrorAction Stop).Source }

$env:FRAMEWORKS_HOME = $frameworksRoot
$portableCargoHome = Join-Path $frameworksRoot 'Toolchains\Rust\cargo-home'
$userCargoHome = Join-Path $env:USERPROFILE '.cargo'
$env:CARGO_HOME = if (Test-Path -LiteralPath (Join-Path $userCargoHome 'registry\src')) { $userCargoHome } else { $portableCargoHome }
$env:CARGO_TARGET_DIR = Join-Path $frameworksRoot 'Caches\Rust\target'
$toolPaths = @($nodeHome, $rustBin)
if (Test-Path -LiteralPath $mingwBin) { $toolPaths += $mingwBin }
$env:PATH = "$($toolPaths -join ';');$env:PATH"

Push-Location $appRoot
try {
    & $npm run build
    if ($LASTEXITCODE -ne 0) { throw 'Frontend build failed.' }

    Push-Location (Join-Path $appRoot 'src-tauri')
    try {
        Write-Host "Cargo cache: $env:CARGO_HOME"
        $cargoArguments = @('build', '--release', '--features', 'custom-protocol')
        if ($env:CARGO_NET_OFFLINE -ne 'false') { $cargoArguments += '--offline' }
        & $cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) { throw 'Rust release build failed.' }
    } finally {
        Pop-Location
    }

    $source = Join-Path $env:CARGO_TARGET_DIR 'release\greendev-manager.exe'
    $destination = Join-Path $appRoot 'GreenDevManager.exe'
    Copy-Item -LiteralPath $source -Destination $destination -Force
    $cliSource = Join-Path $env:CARGO_TARGET_DIR 'release\greendev.exe'
    $cliDestination = Join-Path $frameworksRoot 'greendev.exe'
    if (-not (Test-Path -LiteralPath $cliSource)) { throw 'greendev.exe CLI was not produced.' }
    Copy-Item -LiteralPath $cliSource -Destination $cliDestination -Force
    $webViewLoader = Get-ChildItem (Join-Path $env:CARGO_TARGET_DIR 'release\build') -Recurse -Filter 'WebView2Loader.dll' |
        Where-Object { $_.Directory.Name -eq 'x64' } |
        Select-Object -First 1
    if (-not $webViewLoader) { throw 'WebView2Loader.dll was not produced by the GNU build.' }
    Copy-Item -LiteralPath $webViewLoader.FullName -Destination (Join-Path $appRoot 'WebView2Loader.dll') -Force
    Write-Host "Built: $destination"
    Write-Host "CLI: $cliDestination"
} finally {
    Pop-Location
}
