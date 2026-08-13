param([Parameter(Mandatory = $true)][string]$Version)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$curl = Resolve-SystemTool -Name 'curl.exe'
$settings = Get-Content (Join-Path $root 'Config\greendev\app-update.json') -Raw | ConvertFrom-Json
$feed = Get-Content (Join-Path $root 'Caches\GreenDevManager\app-update-feed.json') -Raw | ConvertFrom-Json
$entry = $feed.channels.([string]$settings.channel)
if (-not $entry -or [string]$entry.version -ne $Version) { throw "Feed does not contain $Version on channel $($settings.channel)." }
$releaseDir = Join-Path $root "Releases\GreenDevManager\$Version"
New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null

function Resolve-FeedUri([string]$Reference) {
    if ([Uri]::IsWellFormedUriString($Reference, [UriKind]::Absolute)) { return $Reference }
    if (-not $settings.feedUrl) { throw "Relative feed reference has no remote base: $Reference" }
    return ([Uri]::new([Uri]$settings.feedUrl, $Reference)).AbsoluteUri
}

$manifestReference = [string]$entry.manifest
if (-not $manifestReference) { throw 'Update entry has no manifest reference.' }
$manifestPath = Join-Path $releaseDir 'release-manifest.json'
if (Test-Path -LiteralPath $manifestReference -PathType Leaf) { Copy-Item -LiteralPath $manifestReference -Destination $manifestPath -Force }
else {
    $temporary = "$manifestPath.part"
    & $curl --fail --location --continue-at - --retry 3 --output $temporary (Resolve-FeedUri $manifestReference)
    if ($LASTEXITCODE -ne 0) { throw "Manifest download failed: $LASTEXITCODE" }
    Move-Item -LiteralPath $temporary -Destination $manifestPath -Force
}
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ([string]$manifest.version -ne $Version) { throw "Manifest version mismatch: $($manifest.version)" }
$manifestUri = Resolve-FeedUri $manifestReference
$portableFound = $false
foreach ($artifact in @($manifest.artifacts)) {
    $name = [string]$artifact.name
    if (-not $name -or [System.IO.Path]::GetFileName($name) -ne $name) { throw "Invalid artifact name: $name" }
    if ($name -like '*portable.zip') { $portableFound = $true }
    $artifactPath = Join-Path $releaseDir $name
    $artifactReference = if ($artifact.url) { Resolve-FeedUri ([string]$artifact.url) } else { ([Uri]::new([Uri]$manifestUri, $name)).AbsoluteUri }
    $partial = "$artifactPath.part"
    & $curl --fail --location --continue-at - --retry 3 --output $partial $artifactReference
    if ($LASTEXITCODE -ne 0) { throw "Artifact download failed ($name): $LASTEXITCODE" }
    $actual = (Get-FileHash -LiteralPath $partial -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($actual -ne ([string]$artifact.sha256).ToUpperInvariant()) { throw "Artifact SHA256 mismatch ($name): $actual" }
    Move-Item -LiteralPath $partial -Destination $artifactPath -Force
    Write-Host "Downloaded and verified: $artifactPath"
}
if (-not $portableFound) { throw 'Release manifest has no portable ZIP.' }
