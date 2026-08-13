param(
    [switch]$SkipBuild,
    [switch]$SkipInstaller,
    [string]$SignThumbprint,
    [string]$ReleaseNotes,
    [string]$ReleaseBaseUrl,
    [switch]$OnlineCargo,
    [long]$SourceDateEpoch = 0,
    [ValidateSet('stable', 'beta', 'nightly', 'local')]
    [string]$Channel = 'stable'
)

$ErrorActionPreference = 'Stop'
$appRoot = $PSScriptRoot
$frameworksRoot = [System.IO.Path]::GetFullPath((Join-Path $appRoot '..\..'))
$version = (Get-Content (Join-Path $appRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json).version
$releaseRoot = Join-Path $frameworksRoot "Releases\GreenDevManager\$version"
$stage = Join-Path $frameworksRoot 'Caches\GreenDevManager\release-stage'
$portableNode = Join-Path $frameworksRoot 'Runtimes\Node\current\node.exe'
$node = if (Test-Path -LiteralPath $portableNode) { $portableNode } else { (Get-Command node.exe -ErrorAction Stop).Source }
$nodeHome = Split-Path -Parent $node
$npm = if (Test-Path -LiteralPath (Join-Path $nodeHome 'npm.cmd')) { Join-Path $nodeHome 'npm.cmd' } else { (Get-Command npm.cmd -ErrorAction Stop).Source }
$portableRust = Join-Path $frameworksRoot 'Toolchains\Rust\current\bin\rustc.exe'
$rust = if (Test-Path -LiteralPath $portableRust) { $portableRust } else { (Get-Command rustc.exe -ErrorAction Stop).Source }
$rustBin = Split-Path -Parent $rust
$mingwBin = Join-Path $frameworksRoot 'Toolchains\C\mingw64\bin'
$env:FRAMEWORKS_HOME = $frameworksRoot
$env:CARGO_HOME = if (Test-Path (Join-Path $env:USERPROFILE '.cargo\registry\src')) { Join-Path $env:USERPROFILE '.cargo' } else { Join-Path $frameworksRoot 'Toolchains\Rust\cargo-home' }
$env:CARGO_TARGET_DIR = Join-Path $frameworksRoot 'Caches\Rust\target'
$env:CARGO_NET_OFFLINE = if ($OnlineCargo) { 'false' } else { 'true' }
$env:SOURCE_DATE_EPOCH = if ($SourceDateEpoch -gt 0) { [string]$SourceDateEpoch } else { '' }
$toolPaths = @($nodeHome, $rustBin)
if (Test-Path -LiteralPath $mingwBin) { $toolPaths += $mingwBin }
$env:PATH = "$($toolPaths -join ';');$env:PATH"

if (-not $SkipBuild) { & (Join-Path $appRoot 'build.ps1') }
if (-not (Test-Path (Join-Path $appRoot 'GreenDevManager.exe'))) { throw 'GreenDevManager.exe is missing.' }

function Invoke-CodeSign([string]$Path) {
    if (-not $SignThumbprint) { return }
    $signTool = Get-Command signtool.exe -ErrorAction Stop
    & $signTool.Source sign /sha1 $SignThumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Path
    if ($LASTEXITCODE -ne 0) { throw "Code signing failed: $Path" }
}

Invoke-CodeSign (Join-Path $appRoot 'GreenDevManager.exe')
Invoke-CodeSign (Join-Path $frameworksRoot 'greendev.exe')

New-Item -ItemType Directory -Path $releaseRoot -Force | Out-Null
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage | Out-Null
Copy-Item (Join-Path $appRoot 'GreenDevManager.exe'), (Join-Path $appRoot 'WebView2Loader.dll'), (Join-Path $appRoot 'README.md') -Destination $stage
Copy-Item (Join-Path $frameworksRoot 'greendev.exe') -Destination $stage
Copy-Item (Join-Path $appRoot 'portable-run.bat') -Destination (Join-Path $stage 'run.bat')
$zip = Join-Path $releaseRoot "GreenDevManager-$version-win-x64-portable.zip"
if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal

if (-not $SkipInstaller) {
    Push-Location $appRoot
    try {
        $tauriArguments = @('run', 'tauri', 'build', '--', '--bundles', 'nsis')
        if ($SignThumbprint) {
            $signingConfig = [ordered]@{ bundle = [ordered]@{ windows = [ordered]@{ certificateThumbprint = $SignThumbprint; digestAlgorithm = 'sha256'; timestampUrl = 'http://timestamp.digicert.com'; tsp = $true } } } | ConvertTo-Json -Depth 6 -Compress
            $tauriArguments += @('--config', $signingConfig)
        }
        & $npm @tauriArguments
        if ($LASTEXITCODE -ne 0) { throw 'NSIS build failed.' }
    } finally { Pop-Location }
    $installers = @(Get-ChildItem (Join-Path $env:CARGO_TARGET_DIR 'release\bundle\nsis') -Filter '*.exe' -ErrorAction Stop | Where-Object { $_.Name -like "*_$version`_*-setup.exe" })
    if ($installers.Count -ne 1) { throw "Expected exactly one NSIS installer for version $version, found $($installers.Count)." }
    foreach ($installer in $installers) {
        $destination = Join-Path $releaseRoot "GreenDevManager-$version-win-x64-setup.exe"
        Copy-Item -LiteralPath $installer.FullName -Destination $destination -Force
        Invoke-CodeSign $destination
    }
}

$notesPath = Join-Path $releaseRoot 'RELEASE_NOTES.md'
$notes = if ($ReleaseNotes) { $ReleaseNotes } else { "GreenDev Manager $version`r`n`r`n- Phase 20-23: restart-resumable queue, reliability budgets, signed supply-chain workflow, remote rollout staging, Manifest SDK and CLI completions.`r`n- Existing component versions, logs and configuration backups are preserved." }
Set-Content -LiteralPath $notesPath -Value $notes -Encoding UTF8

$generatedAt = if ($SourceDateEpoch -gt 0) { [DateTimeOffset]::FromUnixTimeSeconds($SourceDateEpoch).UtcDateTime.ToString('o') } else { (Get-Date).ToUniversalTime().ToString('o') }
$packageLockPath = Join-Path $appRoot 'package-lock.json'
# Windows PowerShell 5 rejects the package-lock v3 root package's empty key,
# so normalize it through the bundled Node runtime before ConvertFrom-Json.
$npmJson = & $node -e "const p=require(process.argv[1]);process.stdout.write(JSON.stringify(Object.entries(p.packages||{}).filter(([k,v])=>k&&v.version).map(([k,v])=>({name:k.split('node_modules/').pop(),version:String(v.version),license:v.license?String(v.license):'NOASSERTION'}))))" $packageLockPath
if ($LASTEXITCODE -ne 0) { throw 'package-lock normalization failed.' }
$npmPackages = @($npmJson | ConvertFrom-Json)
$npmComponents = foreach ($package in $npmPackages) {
    $name = [string]$package.name; $dependencyVersion = [string]$package.version
    [ordered]@{ type = 'library'; group = 'npm'; name = $name; version = $dependencyVersion; purl = "pkg:npm/$name@$dependencyVersion"; licenses = @([ordered]@{ license = [ordered]@{ name = [string]$package.license } }) }
}
$cargoLockText = Get-Content (Join-Path $appRoot 'src-tauri\Cargo.lock') -Raw
$cargoComponents = foreach ($match in [regex]::Matches($cargoLockText, '(?ms)\[\[package\]\]\s*name\s*=\s*"([^"]+)"\s*version\s*=\s*"([^"]+)"')) {
    $name = $match.Groups[1].Value; $dependencyVersion = $match.Groups[2].Value
    [ordered]@{ type = 'library'; group = 'cargo'; name = $name; version = $dependencyVersion; purl = "pkg:cargo/$name@$dependencyVersion"; licenses = @([ordered]@{ license = [ordered]@{ name = 'NOASSERTION' } }) }
}
$environmentManifest = Get-Content (Join-Path $frameworksRoot 'Config\greendev\components.json') -Raw | ConvertFrom-Json
$environmentComponents = foreach ($component in $environmentManifest.components) {
    [ordered]@{ type = 'application'; group = 'greendev-environment'; name = [string]$component.id; version = [string]$component.version; purl = "pkg:generic/$($component.id)@$($component.version)"; hashes = if ($component.source.sha256) { @([ordered]@{ alg = 'SHA-256'; content = [string]$component.source.sha256 }) } else { @() }; licenses = @([ordered]@{ license = [ordered]@{ name = 'NOASSERTION' } }) }
}
$seenPurls = @{}
$allComponents = foreach ($component in @($npmComponents) + @($cargoComponents) + @($environmentComponents)) { if (-not $seenPurls.ContainsKey($component.purl)) { $seenPurls[$component.purl] = $true; $component } }
$advisoriesPath = Join-Path $frameworksRoot 'Config\greendev\advisories.json'
$advisories = if (Test-Path $advisoriesPath) { @((Get-Content $advisoriesPath -Raw | ConvertFrom-Json).advisories) } else { @() }
$serialSeed = [Text.Encoding]::UTF8.GetBytes("GreenDev Manager|$version|$SourceDateEpoch")
$serialBytes = [Security.Cryptography.SHA256]::Create().ComputeHash($serialSeed)[0..15]
$sbom = [ordered]@{ bomFormat = 'CycloneDX'; specVersion = '1.5'; serialNumber = "urn:uuid:$([guid]::new([byte[]]$serialBytes))"; version = 1; metadata = [ordered]@{ timestamp = $generatedAt; tools = @([ordered]@{ vendor = 'GreenDev'; name = 'release.ps1'; version = $version }); component = [ordered]@{ type = 'application'; name = 'GreenDev Manager'; version = $version } }; components = @($allComponents); vulnerabilities = $advisories }
$sbom | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $releaseRoot 'release-sbom.cdx.json') -Encoding UTF8

$bootstrapStage = Join-Path $frameworksRoot 'Caches\GreenDevManager\bootstrap-release-stage'
if (Test-Path -LiteralPath $bootstrapStage) { Remove-Item -LiteralPath $bootstrapStage -Recurse -Force }
New-Item -ItemType Directory -Path $bootstrapStage | Out-Null
foreach ($name in @('README.md', 'env-setup.bat', 'setup_dev_env.bat', 'auto-setup.bat', 'cleanup.bat', 'sync-config.bat', 'greendev.exe')) {
    $source = Join-Path $frameworksRoot $name
    if (Test-Path -LiteralPath $source -PathType Leaf) { Copy-Item -LiteralPath $source -Destination $bootstrapStage }
}
Copy-Item -LiteralPath (Join-Path $frameworksRoot 'Scripts') -Destination $bootstrapStage -Recurse
New-Item -ItemType Directory -Path (Join-Path $bootstrapStage 'Config') | Out-Null
foreach ($name in @('cargo', 'gradle', 'greendev', 'maven', 'mysql', 'npm', 'pip')) {
    $source = Join-Path $frameworksRoot "Config\$name"
    if (Test-Path -LiteralPath $source -PathType Container) { Copy-Item -LiteralPath $source -Destination (Join-Path $bootstrapStage 'Config') -Recurse }
}
foreach ($name in @('pins.json', 'package-lock.json', 'pending-app-update.json')) {
    $localState = Join-Path $bootstrapStage "Config\greendev\$name"
    if (Test-Path -LiteralPath $localState) { Remove-Item -LiteralPath $localState -Force }
}
$bootstrapZip = Join-Path $releaseRoot "GreenDevManager-bootstrap-$version.zip"
if (Test-Path -LiteralPath $bootstrapZip) { Remove-Item -LiteralPath $bootstrapZip -Force }
Compress-Archive -Path (Join-Path $bootstrapStage '*') -DestinationPath $bootstrapZip -CompressionLevel Optimal
$bootstrapUrl = if ($ReleaseBaseUrl) { "$($ReleaseBaseUrl.TrimEnd('/'))/$([Uri]::EscapeDataString((Split-Path -Leaf $bootstrapZip)))" } else { Split-Path -Leaf $bootstrapZip }
$bootstrapManifest = [ordered]@{
    schemaVersion = 1
    version = $version
    generatedAt = $generatedAt
    url = $bootstrapUrl
    size = (Get-Item -LiteralPath $bootstrapZip).Length
    sha256 = (Get-FileHash -LiteralPath $bootstrapZip -Algorithm SHA256).Hash
}
$bootstrapManifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $releaseRoot 'bootstrap-manifest.json') -Encoding UTF8

$gitCommit = (& git -C $frameworksRoot rev-parse HEAD 2>$null)
$materials = @(
    [ordered]@{ uri = 'git+local'; digest = [ordered]@{ sha1 = [string]$gitCommit } },
    [ordered]@{ uri = 'package-lock.json'; digest = [ordered]@{ sha256 = (Get-FileHash (Join-Path $appRoot 'package-lock.json') -Algorithm SHA256).Hash } },
    [ordered]@{ uri = 'src-tauri/Cargo.lock'; digest = [ordered]@{ sha256 = (Get-FileHash (Join-Path $appRoot 'src-tauri\Cargo.lock') -Algorithm SHA256).Hash } },
    [ordered]@{ uri = 'Config/greendev/components.json'; digest = [ordered]@{ sha256 = (Get-FileHash (Join-Path $frameworksRoot 'Config\greendev\components.json') -Algorithm SHA256).Hash } }
)
$builderId = if ($env:GITHUB_ACTIONS -eq 'true') { 'github-actions/greendev-release' } else { 'greendev-local-release.ps1' }
$provenance = [ordered]@{ schemaVersion = 1; predicateType = 'https://slsa.dev/provenance/v1'; subject = 'GreenDev Manager'; version = $version; buildType = 'tauri-v2-nsis-portable'; builder = [ordered]@{ id = $builderId }; invocation = [ordered]@{ channel = $Channel; offlineCargo = -not $OnlineCargo; signed = [bool]$SignThumbprint; sourceDateEpoch = $SourceDateEpoch; deterministicMetadata = ($SourceDateEpoch -gt 0) }; materials = $materials; generatedAt = $generatedAt }
$provenance | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $releaseRoot 'provenance.json') -Encoding UTF8

$artifacts = Get-ChildItem -LiteralPath $releaseRoot -File | Where-Object {
    $_.Name -notin @('SHA256SUMS.txt', 'release-manifest.json', 'update-feed.json', 'bootstrap-manifest.json') -and
        $_.Name -notlike 'GreenDevManager-bootstrap-*.zip' -and
        -not $_.Name.EndsWith('.sig.json')
}
$checksumPath = Join-Path $releaseRoot 'SHA256SUMS.txt'
$checksumArtifacts = @($artifacts) + @(
    Get-Item -LiteralPath $bootstrapZip
    Get-Item -LiteralPath (Join-Path $releaseRoot 'bootstrap-manifest.json')
)
$lines = foreach ($artifact in $checksumArtifacts) { $hash = (Get-FileHash -LiteralPath $artifact.FullName -Algorithm SHA256).Hash; "$hash  $($artifact.Name)" }
Set-Content -LiteralPath $checksumPath -Value $lines -Encoding ASCII
$releaseManifest = [ordered]@{
    schemaVersion = 1
    version = $version
    generatedAt = $generatedAt
    signed = [bool]$SignThumbprint
    channel = $Channel
    sbom = [ordered]@{ format = 'CycloneDX'; specVersion = '1.5'; components = @($allComponents).Count }
    provenance = [ordered]@{ predicateType = $provenance.predicateType; materials = @($materials).Count }
    artifacts = @($artifacts | ForEach-Object {
        $entry = [ordered]@{ name = $_.Name; size = $_.Length; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
        if ($ReleaseBaseUrl) { $entry.url = "$($ReleaseBaseUrl.TrimEnd('/'))/$([Uri]::EscapeDataString($_.Name))" }
        $entry
    })
}
$releaseManifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $releaseRoot 'release-manifest.json') -Encoding UTF8
if ($SignThumbprint) {
    $signScript = Join-Path $frameworksRoot 'Scripts\sign-greendev-artifact.ps1'
    foreach ($artifact in $artifacts) { & $signScript -Path $artifact.FullName -Thumbprint $SignThumbprint }
    & $signScript -Path (Join-Path $releaseRoot 'release-manifest.json') -Thumbprint $SignThumbprint
}
$feedPath = Join-Path $frameworksRoot 'Releases\GreenDevManager\update-feed.json'
$feed = if (Test-Path -LiteralPath $feedPath) { Get-Content -LiteralPath $feedPath -Raw | ConvertFrom-Json } else { [pscustomobject]@{ schemaVersion = 1; channels = [pscustomobject]@{} } }
$entry = [ordered]@{ version = $version; publishedAt = $generatedAt; manifest = "$version/release-manifest.json"; signed = [bool]$SignThumbprint }
$feed.channels | Add-Member -NotePropertyName $Channel -NotePropertyValue $entry -Force
$feed | Add-Member -NotePropertyName generatedAt -NotePropertyValue $generatedAt -Force
$feed | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $feedPath -Encoding UTF8
if ($SignThumbprint) { & (Join-Path $frameworksRoot 'Scripts\sign-greendev-artifact.ps1') -Path $feedPath -Thumbprint $SignThumbprint }
if ($ReleaseBaseUrl) {
    $publicFeed = [ordered]@{
        schemaVersion = 1
        channels = [ordered]@{
            $Channel = [ordered]@{
                version = $version
                publishedAt = $generatedAt
                manifest = "$($ReleaseBaseUrl.TrimEnd('/'))/release-manifest.json"
                signed = [bool]$SignThumbprint
            }
        }
        generatedAt = $generatedAt
    }
    $publicFeed | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $releaseRoot 'update-feed.json') -Encoding UTF8
}
Write-Host "Release: $releaseRoot"
$artifacts | Select-Object Name, Length
