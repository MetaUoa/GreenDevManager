param([string]$FeedUrl, [switch]$Local)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$curl = Resolve-SystemTool -Name 'curl.exe'
$settingsPath = Join-Path $root 'Config\greendev\app-update.json'
$cachePath = Join-Path $root 'Caches\GreenDevManager\app-update-feed.json'
$settings = if (Test-Path -LiteralPath $settingsPath) { Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json } else { [pscustomobject]@{ feedUrl = '' } }
if (-not $Local -and -not $FeedUrl) { $FeedUrl = [string]$settings.feedUrl }

New-Item -ItemType Directory -Path (Split-Path -Parent $cachePath) -Force | Out-Null
if ($FeedUrl) {
    $temporary = "$cachePath.part"
    & $curl --fail --location --retry 3 --connect-timeout 15 --output $temporary $FeedUrl
    if ($LASTEXITCODE -ne 0) { throw "Update feed request failed: $LASTEXITCODE" }
    $feed = Get-Content -LiteralPath $temporary -Raw | ConvertFrom-Json
    if (-not $feed.channels) { throw 'Update feed is missing channels.' }
    Move-Item -LiteralPath $temporary -Destination $cachePath -Force
    Write-Host "Remote feed cached: $FeedUrl"
    exit 0
}

$releaseRoot = Join-Path $root 'Releases\GreenDevManager'
$publishedFeed = Join-Path $releaseRoot 'update-feed.json'
if (Test-Path -LiteralPath $publishedFeed -PathType Leaf) {
    $feed = Get-Content -LiteralPath $publishedFeed -Raw | ConvertFrom-Json
    if (-not $feed.channels) { throw 'Local release feed is missing channels.' }
    foreach ($property in @($feed.channels.psobject.Properties)) {
        $entry = $property.Value
        $reference = [string]$entry.manifest
        if ($reference -and -not [IO.Path]::IsPathRooted($reference)) {
            $entry.manifest = [IO.Path]::GetFullPath((Join-Path $releaseRoot $reference))
        }
        $entry | Add-Member -NotePropertyName source -NotePropertyValue 'local' -Force
    }
} else {
    $versions = @(Get-ChildItem -LiteralPath $releaseRoot -Directory -ErrorAction SilentlyContinue | Sort-Object { [System.Management.Automation.SemanticVersion]$_.Name })
    $latest = $versions | Select-Object -Last 1
    $entry = if ($latest) {
        $manifest = Join-Path $latest.FullName 'release-manifest.json'
        [ordered]@{ version = $latest.Name; manifest = $manifest; source = 'local'; publishedAt = (Get-Item $latest.FullName).LastWriteTimeUtc.ToString('o') }
    } else { [ordered]@{ version = '0.0.0'; source = 'local'; publishedAt = (Get-Date).ToUniversalTime().ToString('o') } }
    $feed = [pscustomobject]@{ schemaVersion = 1; generatedAt = (Get-Date).ToUniversalTime().ToString('o'); channels = [pscustomobject]@{ stable = $entry; beta = $entry } }
}
if (-not $feed.channels.local) {
    $localSource = if ($feed.channels.beta) { $feed.channels.beta } else { $feed.channels.stable }
    $feed.channels | Add-Member -NotePropertyName local -NotePropertyValue $localSource -Force
}
$feed.generatedAt = (Get-Date).ToUniversalTime().ToString('o')
$feed | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $cachePath -Encoding UTF8
Write-Host "Local update feed: stable=$($feed.channels.stable.version), beta=$($feed.channels.beta.version), local=$($feed.channels.local.version)"
