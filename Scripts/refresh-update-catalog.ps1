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
$result = [ordered]@{ schemaVersion = 1; generatedAt = (Get-Date).ToUniversalTime().ToString('o'); components = [ordered]@{} }

function Add-Result([string]$Id, [scriptblock]$Probe) {
    try {
        $value = & $Probe
        $value['status'] = 'ok'
        $result.components[$Id] = $value
        Write-Host "[OK] $Id $($value.version)"
    } catch {
        $result.components[$Id] = [ordered]@{ status = 'error'; version = ''; url = ''; sha256 = ''; notes = $_.Exception.Message }
        Write-Warning "[$Id] $($_.Exception.Message)"
    }
}

Add-Result 'node' {
    $releaseData = (& $curl --ssl-no-revoke --fail --silent --show-error --location 'https://nodejs.org/dist/index.json') | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Node.js release index request failed.' }
    $release = $null
    foreach ($entry in $releaseData) { if ($policies.node -ne 'lts' -or $entry.lts) { $release = $entry; break } }
    if (-not $release) { throw 'Node.js release was not found.' }
    $version = ([string]$release.version).TrimStart('v')
    $file = "node-v$version-win-x64.zip"; $base = "https://nodejs.org/dist/v$version"
    $sums = & $curl --ssl-no-revoke --fail --silent --show-error --location "$base/SHASUMS256.txt"
    if ($LASTEXITCODE -ne 0) { throw 'Node.js checksum request failed.' }
    $sums = $sums -join "`n"
    $match = [regex]::Match($sums, "(?im)^([0-9a-f]{64})\s+$([regex]::Escape($file))$")
    $channel = if ($release.lts) { 'LTS' } else { 'Stable' }
    [ordered]@{ version = $version; url = "$base/$file"; sha256 = $match.Groups[1].Value.ToUpperInvariant(); notes = "Node.js $channel" }
}
Add-Result 'gradle' {
    $release = Invoke-RestMethod -Uri 'https://services.gradle.org/versions/current' -TimeoutSec 25
    [ordered]@{ version = [string]$release.version; url = [string]$release.downloadUrl; sha256 = ([string]$release.checksum).ToUpperInvariant(); notes = 'Gradle current release' }
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
    $packageData = (& $curl --ssl-no-revoke --fail --silent --show-error --location 'https://api.azul.com/metadata/v1/zulu/packages/?java_version=21&os=windows&arch=x86&hw_bitness=64&archive_type=zip&java_package_type=jdk&release_status=ga&availability_types=CA&latest=true') | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Azul package catalog request failed.' }
    $package = $null
    foreach ($entry in $packageData) { if ($entry.download_url -match '-ca-jdk[0-9].*-win_x64\.zip$' -and $entry.download_url -notmatch '-fx-|-crac-') { $package = $entry; break } }
    if (-not $package) { throw 'Azul JDK package was not found.' }
    $version = (@($package.java_version) -join '.')
    [ordered]@{ version = $version; url = [string]$package.download_url; sha256 = ([string]$package.sha256_hash).ToUpperInvariant(); notes = 'Azul Zulu JDK 21 LTS' }
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
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $cache -Encoding UTF8
$result | ConvertTo-Json -Depth 8
