param([string]$PolicyPath)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$curl = Resolve-SystemTool -Name 'curl.exe'
if (-not $PolicyPath) { $PolicyPath = Join-Path $root 'Config\greendev\update-policies.json' }
$policyObject = Get-Content -LiteralPath $PolicyPath -Raw | ConvertFrom-Json
$policies = @{}
$policyObject.PSObject.Properties | ForEach-Object { $policies[$_.Name] = [string]$_.Value }
$manifest = Get-Content -LiteralPath (Join-Path $root 'Config\greendev\components.json') -Raw | ConvertFrom-Json
$manifestById = @{}
foreach ($component in $manifest.components) { $manifestById[[string]$component.id] = $component }
$result = [ordered]@{ schemaVersion = 2; generatedAt = (Get-Date).ToUniversalTime().ToString('o'); components = [ordered]@{} }

function Invoke-JsonUrl([string]$Url) {
    $raw = (& $curl --ssl-no-revoke --fail --silent --show-error --location -H 'User-Agent: GreenDevManager/1.1' $Url) -join "`n"
    if ($LASTEXITCODE -ne 0) { throw "Catalog request failed: $Url" }
    return $raw | ConvertFrom-Json
}

function New-Candidate(
    [string]$Id, [string]$Provider, [string]$Version, [string]$Architecture,
    [string]$Channel, [string]$Url, [string]$Sha256, [string]$ArchiveRoot,
    [string]$InstallDir, [string]$ArchivePath, [string]$ComponentName, [string]$Notes
) {
    return [ordered]@{
        id = $Id; provider = $Provider; version = $Version; architecture = $Architecture
        channel = $Channel; url = $Url; sha256 = $Sha256.ToUpperInvariant()
        archiveRoot = $ArchiveRoot; installDir = $InstallDir; archivePath = $ArchivePath
        componentName = $ComponentName; notes = $Notes
    }
}

function Add-Result([string]$Id, [scriptblock]$Probe) {
    try {
        $value = & $Probe
        $candidates = @($value.candidates)
        if ($candidates.Count -gt 0) {
            $default = $candidates[0]
            foreach ($field in @('version', 'url', 'sha256', 'notes')) {
                if (-not $value.Contains($field)) { $value[$field] = $default[$field] }
            }
            $value['defaultCandidateId'] = [string]$default.id
            $value['candidateCount'] = $candidates.Count
        }
        $value['status'] = 'ok'
        $result.components[$Id] = $value
        $count = if ($value.candidateCount) { $value.candidateCount } else { 1 }
        Write-Host "[OK] $Id $($value.version) ($count candidates)"
    } catch {
        $result.components[$Id] = [ordered]@{ status = 'error'; version = ''; url = ''; sha256 = ''; notes = $_.Exception.Message; candidates = @() }
        Write-Warning "[$Id] $($_.Exception.Message)"
    }
}

Add-Result 'node' {
    $releaseData = Invoke-JsonUrl 'https://nodejs.org/dist/index.json'
    $seenMajor = @{}
    $candidates = @()
    foreach ($release in $releaseData) {
        if ($policies.node -eq 'lts' -and -not $release.lts) { continue }
        if (@($release.files) -notcontains 'win-x64-zip') { continue }
        $version = ([string]$release.version).TrimStart('v')
        $major = $version.Split('.')[0]
        if ($seenMajor.ContainsKey($major)) { continue }
        $seenMajor[$major] = $true
        $file = "node-v$version-win-x64.zip"; $base = "https://nodejs.org/dist/v$version"
        $sums = (& $curl --ssl-no-revoke --fail --silent --show-error --location "$base/SHASUMS256.txt") -join "`n"
        if ($LASTEXITCODE -ne 0) { continue }
        $match = [regex]::Match($sums, "(?im)^([0-9a-f]{64})\s+$([regex]::Escape($file))$")
        if (-not $match.Success) { continue }
        $channel = if ($release.lts) { "LTS $($release.lts)" } else { 'Current' }
        $candidates += New-Candidate "node-$major-x64" 'Node.js' $version 'x64' $channel "$base/$file" $match.Groups[1].Value "node-v$version-win-x64" "Runtimes\Node\node-v$version-win-x64" "downloads\packages\$file" 'Node.js' "Node.js $channel"
        if ($candidates.Count -ge 5) { break }
    }
    if (-not $candidates.Count) { throw 'Node.js release was not found.' }
    [ordered]@{ candidates = $candidates }
}
Add-Result 'gradle' {
    $releases = Invoke-JsonUrl 'https://services.gradle.org/versions/all'
    $seenMajor = @{}; $candidates = @()
    foreach ($release in @($releases | Where-Object { -not $_.snapshot -and -not $_.nightly -and -not $_.broken -and [string]$_.version -match '^\d+\.\d+(\.\d+)?$' } | Sort-Object { [version]$_.version } -Descending)) {
        $major = ([string]$release.version).Split('.')[0]
        if ($seenMajor.ContainsKey($major) -or -not $release.checksum) { continue }
        $seenMajor[$major] = $true; $version = [string]$release.version; $file = "gradle-$version-bin.zip"
        $candidates += New-Candidate "gradle-$major" 'Gradle' $version 'universal' 'Stable' ([string]$release.downloadUrl) ([string]$release.checksum) "gradle-$version" "BuildTools\Gradle\gradle-$version" "downloads\packages\$file" 'Gradle' 'Gradle stable release'
        if ($candidates.Count -ge 4) { break }
    }
    if (-not $candidates.Count) { throw 'Gradle release was not found.' }
    [ordered]@{ candidates = $candidates }
}
Add-Result 'maven' {
    [xml]$metadata = (Invoke-WebRequest -UseBasicParsing -Uri 'https://repo1.maven.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml' -TimeoutSec 25).Content
    $version = @($metadata.metadata.versioning.versions.version | Where-Object { $_ -match '^3\.[0-9]+\.[0-9]+$' } | Sort-Object { [version]$_ } -Descending) | Select-Object -First 1
    [ordered]@{ version = $version; url = "https://archive.apache.org/dist/maven/maven-3/$version/binaries/apache-maven-$version-bin.zip"; sha256 = ''; notes = 'Apache Maven release metadata' }
}
Add-Result 'rust' {
    $text = (& $curl --fail --silent --show-error --location 'https://static.rust-lang.org/dist/channel-rust-stable.toml') -join "`n"
    if ($LASTEXITCODE -ne 0) { throw 'Rust stable channel request failed.' }
    $section = [regex]::Match($text, '(?ms)^\[pkg\.rust\]\s*(.*?)(?=^\[)').Groups[1].Value
    $version = [regex]::Match($section, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)').Groups[1].Value
    if (-not $version) { throw 'Rust stable version was not found.' }
    [ordered]@{ version = $version; url = ''; sha256 = ''; notes = 'Rust stable channel; standalone package uses offline import' }
}
Add-Result 'python' {
    $html = (Invoke-WebRequest -UseBasicParsing -Headers @{ 'User-Agent' = 'GreenDevManager/0.11' } -Uri 'https://www.python.org/downloads/windows/' -TimeoutSec 25).Content
    $version = [regex]::Match($html, 'Latest Python 3 Release - Python ([0-9]+\.[0-9]+\.[0-9]+)').Groups[1].Value
    if (-not $version) { throw 'Latest stable Python version was not found.' }
    [ordered]@{ version = $version; url = "https://www.python.org/ftp/python/$version/python-$version-embed-amd64.zip"; sha256 = ''; notes = 'Python stable FTP catalog' }
}
Add-Result 'java' {
    $current = $manifestById['java']
    $majorMatch = [regex]::Match([string]$current.version, '^(?:1\.)?(\d+)')
    $currentMajor = if ($majorMatch.Success) { [int]$majorMatch.Groups[1].Value } else { 21 }
    $majorOrder = @($currentMajor) + @(25, 21, 17, 11, 8 | Where-Object { $_ -ne $currentMajor })
    $candidates = @()
    foreach ($major in $majorOrder) {
        try {
            $packages = Invoke-JsonUrl "https://api.azul.com/metadata/v1/zulu/packages/?java_version=$major&os=windows&arch=x86&hw_bitness=64&archive_type=zip&java_package_type=jdk&release_status=ga&availability_types=CA&latest=true"
            $summary = @($packages | Where-Object { $_.name -match '-ca-jdk' -and $_.name -match '-win_x64\.zip$' -and $_.name -notmatch '-fx-|-crac-' } | Select-Object -First 1)[0]
            if ($summary) {
                $detail = Invoke-JsonUrl "https://api.azul.com/metadata/v1/zulu/packages/$($summary.package_uuid)"
                $version = (@($detail.java_version) -join '.') + "+$($detail.openjdk_build_number)"
                $rootName = ([string]$detail.name) -replace '\.zip$',''
                $candidates += New-Candidate "java-zulu-$major-x64" 'Azul Zulu' $version 'x64' 'LTS' ([string]$detail.download_url) ([string]$detail.sha256_hash) $rootName "Runtimes\Java\jdk-$major\$rootName" "downloads\packages\$($detail.name)" 'Azul Zulu JDK' "Azul Zulu JDK $major LTS"
            }
        } catch { Write-Warning "[java/zulu/$major] $($_.Exception.Message)" }
        try {
            $assets = Invoke-JsonUrl "https://api.adoptium.net/v3/assets/latest/$major/hotspot?architecture=x64&heap_size=normal&image_type=jdk&jvm_impl=hotspot&os=windows&vendor=eclipse"
            $asset = @($assets)[0]
            if ($asset) {
                $version = ([string]$asset.version.openjdk_version) -replace '-LTS$',''
                if ($major -eq 8) { $version = ([string]$asset.release_name) -replace '^jdk','' }
                $rootName = [string]$asset.release_name; $file = [string]$asset.binary.package.name
                $candidates += New-Candidate "java-temurin-$major-x64" 'Eclipse Temurin' $version 'x64' 'LTS' ([string]$asset.binary.package.link) ([string]$asset.binary.package.checksum) $rootName "Runtimes\Java\jdk-$major\temurin-$rootName" "downloads\packages\$file" 'Eclipse Temurin JDK' "Eclipse Temurin JDK $major LTS / HotSpot"
            }
        } catch { Write-Warning "[java/temurin/$major] $($_.Exception.Message)" }
        try {
            $aliasFile = "amazon-corretto-$major-x64-windows-jdk.zip"
            $aliasUrl = "https://corretto.aws/downloads/latest/$aliasFile"
            $sha = ((& $curl --ssl-no-revoke --fail --silent --show-error --location "https://corretto.aws/downloads/latest_sha256/$aliasFile") -join '').Trim()
            if ($LASTEXITCODE -ne 0 -or $sha -notmatch '^[0-9a-fA-F]{64}$') { throw 'Corretto SHA256 was not found.' }
            $headers = (& $curl --ssl-no-revoke --fail --silent --show-error --location --head $aliasUrl) -join "`n"
            if ($LASTEXITCODE -ne 0) { throw 'Corretto redirect was not found.' }
            $redirects = [regex]::Matches($headers, '(?im)^location:\s*(/downloads/resources/([^/]+)/([^\s]+))\s*$')
            if (-not $redirects.Count) { throw 'Corretto versioned URL was not found.' }
            $redirect = $redirects[$redirects.Count - 1]
            $resourceVersion = $redirect.Groups[2].Value
            $file = $redirect.Groups[3].Value
            $parts = $resourceVersion.Split('.')
            if ($major -eq 8) {
                $version = "8u$($parts[1])-b$($parts[2])"
                $rootName = "jdk1.8.0_$($parts[1])"
            } else {
                $version = "$($parts[0]).$($parts[1]).$($parts[2])+$($parts[3])"
                $rootName = "jdk$($parts[0]).$($parts[1]).$($parts[2])_$($parts[3])"
            }
            $url = "https://corretto.aws$($redirect.Groups[1].Value)"
            $candidates += New-Candidate "java-corretto-$major-x64" 'Amazon Corretto' $version 'x64' 'LTS' $url $sha $rootName "Runtimes\Java\jdk-$major\corretto-$resourceVersion" "downloads\packages\$file" 'Amazon Corretto JDK' "Amazon Corretto JDK $major LTS"
        } catch { Write-Warning "[java/corretto/$major] $($_.Exception.Message)" }
    }
    if (-not $candidates.Count) { throw 'No verified JDK package was found.' }
    [ordered]@{ candidates = $candidates }
}
Add-Result 'mysql' {
    $tagData = (& $curl --ssl-no-revoke --fail --silent --show-error --location -H 'User-Agent: GreenDevManager/1.0' 'https://api.github.com/repos/mysql/mysql-server/git/matching-refs/tags/mysql-8.4') | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'MySQL tag catalog request failed.' }
    $versions = foreach ($entry in $tagData) { $name = ([string]$entry.ref).Split('/')[-1]; if ($name -match '^mysql-8\.4\.[0-9]+$') { $name.Substring(6) } }
    $version = @($versions | Sort-Object { [version]$_ } -Descending) | Select-Object -First 1
    if (-not $version) { throw 'MySQL 8.4 LTS version was not found.' }
    [ordered]@{ version = $version; url = "https://dev.mysql.com/get/Downloads/MySQL-8.4/mysql-$version-winx64.zip"; sha256 = ''; notes = 'MySQL Community Server catalog' }
}

$cache = Join-Path $root 'Caches\GreenDevManager\update-catalog.json'
New-Item -ItemType Directory -Path (Split-Path -Parent $cache) -Force | Out-Null
$json = $result | ConvertTo-Json -Depth 8
$temporary = "$cache.tmp"
[System.IO.File]::WriteAllText($temporary, $json + [Environment]::NewLine, (New-Object System.Text.UTF8Encoding($false)))
Move-Item -LiteralPath $temporary -Destination $cache -Force
$json
