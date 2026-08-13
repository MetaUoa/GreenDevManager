param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('plan', 'import', 'install', 'update')]
    [string]$Action,
    [Parameter(Mandatory = $true)]
    [string]$Id,
    [string]$ManifestPath,
    [string]$ImportArchive
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$curl = Resolve-SystemTool -Name 'curl.exe'
$root = Resolve-FrameworksRoot
if (-not $ManifestPath) { $ManifestPath = Join-Path $root 'Config\greendev\components.json' }
function Invoke-GreenDevFailpoint([string]$Stage) { if ([string]$env:GREENDEV_FAULT_STAGE -eq $Stage) { throw "Injected failure at stage: $Stage" } }

function Resolve-UnderRoot([string]$Relative, [string]$Label) {
    if ([System.IO.Path]::IsPathRooted($Relative) -or $Relative -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "$Label must be a relative path without '..': $Relative"
    }
    $full = [System.IO.Path]::GetFullPath((Join-Path $root $Relative))
    $prefix = [System.IO.Path]::GetFullPath($root).TrimEnd('\') + '\'
    if (-not $full.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) { throw "$Label is outside Frameworks: $full" }
    return $full
}

$manifestRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'Config\greendev')).TrimEnd('\') + '\'
$manifestFull = [System.IO.Path]::GetFullPath($ManifestPath)
if (-not $manifestFull.StartsWith($manifestRoot, [System.StringComparison]::OrdinalIgnoreCase)) { throw "Manifest is outside Config\greendev: $manifestFull" }
$manifest = Get-Content -LiteralPath $manifestFull -Raw | ConvertFrom-Json
if ($manifest.schemaVersion -notin @(1, 2)) { throw "Unsupported manifest schema: $($manifest.schemaVersion)" }
$component = $manifest.components | Where-Object { $_.id -eq $Id } | Select-Object -First 1
if (-not $component) { throw "Unknown manifest component: $Id" }
if (-not $component.enabled) { throw "Manifest component is disabled: $Id" }
if ($component.source.type -notin @('archive', 'msi')) { throw "Unsupported source type: $($component.source.type)" }

function Expand-PackageArchive([string]$Archive, [string]$Destination, [string]$SourceType) {
    $lower = $Archive.ToLowerInvariant()
    if ($SourceType -eq 'msi' -or $lower.EndsWith('.msi')) {
        & msiexec.exe /a $Archive /qn "TARGETDIR=$Destination"
        if ($LASTEXITCODE -ne 0) { throw "MSI administrative extraction failed: $LASTEXITCODE" }
        return
    }
    if ($lower.EndsWith('.zip')) { Expand-Archive -LiteralPath $Archive -DestinationPath $Destination -Force; return }
    if ($lower.EndsWith('.7z')) {
        $sevenZip = Get-Command 7z.exe -ErrorAction SilentlyContinue
        if (-not $sevenZip) { $sevenZip = Get-Item (Join-Path $root 'BuildTools\7-Zip\current\7z.exe') -ErrorAction SilentlyContinue }
        if (-not $sevenZip) { throw '7z.exe is required to extract .7z packages.' }
        & $sevenZip.FullName x $Archive "-o$Destination" -y
        if ($LASTEXITCODE -ne 0) { throw "7z extraction failed: $LASTEXITCODE" }
        return
    }
    if ($lower.EndsWith('.tar.gz') -or $lower.EndsWith('.tgz') -or $lower.EndsWith('.tar.xz')) {
        & tar.exe -xf $Archive -C $Destination
        if ($LASTEXITCODE -ne 0) { throw "tar extraction failed: $LASTEXITCODE" }
        return
    }
    throw "Unsupported package archive: $Archive"
}

$installDir = Resolve-UnderRoot ([string]$component.installDir) 'installDir'
$healthPath = Join-Path $installDir ([string]$component.healthPath)
$archivePath = Resolve-UnderRoot ([string]$component.source.archive) 'archive'
$currentLink = if ($component.currentLink) { Resolve-UnderRoot ([string]$component.currentLink) 'currentLink' } else { $null }
$settingsPath = Join-Path $root 'Config\greendev\install-settings.json'
$lockPath = Join-Path $root 'Config\greendev\package-lock.json'
$pinsPath = Join-Path $root 'Config\greendev\pins.json'
$settings = if (Test-Path -LiteralPath $settingsPath) { Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json } else { [pscustomobject]@{} }
function ConvertTo-StringMap($Value) {
    $map = @{}
    if ($Value) { $Value.PSObject.Properties | ForEach-Object { $map[$_.Name] = $_.Value } }
    return $map
}
$packageLock = if (Test-Path -LiteralPath $lockPath) { ConvertTo-StringMap (Get-Content -LiteralPath $lockPath -Raw | ConvertFrom-Json) } else { @{} }
$pins = if (Test-Path -LiteralPath $pinsPath) { ConvertTo-StringMap (Get-Content -LiteralPath $pinsPath -Raw | ConvertFrom-Json) } else { @{} }

$dependencyProblems = @()
foreach ($dependencyId in @($component.dependsOn)) {
    $dependency = $manifest.components | Where-Object { $_.id -eq $dependencyId } | Select-Object -First 1
    if (-not $dependency) { $dependencyProblems += "Unknown dependency: $dependencyId"; continue }
    $dependencyInstall = Resolve-UnderRoot ([string]$dependency.installDir) "dependency $dependencyId"
    if (-not (Test-Path -LiteralPath (Join-Path $dependencyInstall ([string]$dependency.healthPath)) -PathType Leaf)) {
        $dependencyProblems += "Missing dependency: $dependencyId"
    }
}

$locked = if ($packageLock.ContainsKey($Id)) { $packageLock[$Id] } else { $null }
$expectedHash = [string]$component.source.sha256
if (-not $expectedHash -and $locked -and $locked.version -eq $component.version) { $expectedHash = [string]$locked.sha256 }
$pinnedPath = if ($pins.ContainsKey($Id)) { [string]$pins[$Id] } else { '' }
$pinnedElsewhere = $pinnedPath -and -not [string]::Equals([System.IO.Path]::GetFullPath($pinnedPath), [System.IO.Path]::GetFullPath($installDir), [System.StringComparison]::OrdinalIgnoreCase)

$plan = [ordered]@{
    id = $component.id
    name = $component.name
    version = $component.version
    action = $Action
    archive = $archivePath
    archiveCached = (Test-Path -LiteralPath $archivePath -PathType Leaf)
    source = [string]$component.source.url
    installDir = $installDir
    currentLink = $currentLink
    alreadyInstalled = (Test-Path -LiteralPath $healthPath -PathType Leaf)
    active = ($currentLink -and (Test-Path -LiteralPath $currentLink) -and ([System.IO.Path]::GetFullPath((Get-Item -LiteralPath $currentLink).Target) -eq [System.IO.Path]::GetFullPath($installDir)))
    expectedSha256 = $expectedHash
    checksumReady = [bool]$expectedHash
    pinnedElsewhere = [bool]$pinnedElsewhere
    dependencies = @($component.dependsOn)
    dependencyProblems = $dependencyProblems
}
if ($Action -eq 'plan') { $plan | ConvertTo-Json -Depth 6; exit 0 }

if ($Action -eq 'import') {
    if (-not $ImportArchive) { throw 'ImportArchive is required.' }
    $sourceArchive = [System.IO.Path]::GetFullPath($ImportArchive)
    if (-not (Test-Path -LiteralPath $sourceArchive -PathType Leaf)) { throw "Import archive does not exist: $sourceArchive" }
    $sourceLower = $sourceArchive.ToLowerInvariant()
    if (-not @('.zip', '.7z', '.tar.gz', '.tgz', '.tar.xz', '.msi').Where({ $sourceLower.EndsWith($_) })) { throw 'Supported imports: ZIP, 7Z, TAR.GZ, TGZ, TAR.XZ, MSI.' }
    $archiveParent = Split-Path -Parent $archivePath
    New-Item -ItemType Directory -Path $archiveParent -Force | Out-Null
    $tempImport = "$archivePath.importing"
    Copy-Item -LiteralPath $sourceArchive -Destination $tempImport -Force
    $actualHash = (Get-FileHash -LiteralPath $tempImport -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($component.source.sha256 -and $actualHash -ine [string]$component.source.sha256) {
        Remove-Item -LiteralPath $tempImport -Force
        throw "SHA256 mismatch. Expected $($component.source.sha256), got $actualHash"
    }
    Move-Item -LiteralPath $tempImport -Destination $archivePath -Force
    $packageLock[$Id] = [ordered]@{ version = [string]$component.version; sha256 = $actualHash; archive = [string]$component.source.archive; importedAt = (Get-Date).ToString('o'); source = $sourceArchive }
    $packageLock | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $lockPath -Encoding UTF8
    Write-Host "Imported: $sourceArchive"
    Write-Host "Archive: $archivePath"
    Write-Host "SHA256: $actualHash"
    exit 0
}

if ($dependencyProblems.Count -gt 0) { throw ($dependencyProblems -join '; ') }
if ($Action -eq 'update' -and $pinnedElsewhere) { throw "Component is pinned to another version; update skipped: $pinnedPath" }

if (-not (Test-Path -LiteralPath $healthPath -PathType Leaf)) {
    if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        Invoke-GreenDevFailpoint 'before-download'
        if (-not $expectedHash) { throw 'Online installation is locked until source.sha256 is set or a local archive is imported.' }
        $urls = @()
        if ($settings.mirrors -and $settings.mirrors.$Id) { $urls += @($settings.mirrors.$Id) }
        if ($component.source.url) { $urls += [string]$component.source.url }
        if ($urls.Count -eq 0) { throw 'No online source is configured; import a local ZIP archive.' }
        $archiveParent = Split-Path -Parent $archivePath
        New-Item -ItemType Directory -Path $archiveParent -Force | Out-Null
        $partial = "$archivePath.part"
        $downloaded = $false
        foreach ($url in $urls) {
            Write-Host "Downloading: $url"
            $curlArgs = @('--fail', '--location', '--continue-at', '-', '--retry', '3', '--connect-timeout', '15', '--output', $partial)
            if ($settings.proxyUrl) { $curlArgs += @('--proxy', [string]$settings.proxyUrl) }
            $curlArgs += [string]$url
            & $curl @curlArgs
            if ($LASTEXITCODE -eq 0) { $downloaded = $true; break }
        }
        if (-not $downloaded) { throw 'All download sources failed; partial file is retained for resume.' }
        Move-Item -LiteralPath $partial -Destination $archivePath -Force
    }

    if (-not $expectedHash) { throw 'Archive has no trusted SHA256. Re-import it to create package-lock.json or set source.sha256.' }
    $actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($actualHash -ine $expectedHash.ToUpperInvariant()) { throw "SHA256 mismatch. Expected $expectedHash, got $actualHash" }
    Invoke-GreenDevFailpoint 'after-hash'

    $parent = Split-Path -Parent $installDir
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    $stage = Join-Path $parent ('.greendev-' + $component.id + '-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stage | Out-Null
    try {
        Write-Host 'Extracting to staging directory'
        Expand-PackageArchive $archivePath $stage ([string]$component.source.type)
        $payload = if ($component.archiveRoot) { Join-Path $stage ([string]$component.archiveRoot) } else { $stage }
        if (-not (Test-Path -LiteralPath (Join-Path $payload ([string]$component.healthPath)) -PathType Leaf)) { throw "Health file missing after extraction: $($component.healthPath)" }
        if (Test-Path -LiteralPath $installDir) { throw "Install directory exists but is unhealthy: $installDir" }
        Move-Item -LiteralPath $payload -Destination $installDir
    } finally {
        if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    }
} else {
    Write-Host "Already installed and healthy: $installDir"
}

if ($currentLink) {
    $currentTarget = if (Test-Path -LiteralPath $currentLink) { [System.IO.Path]::GetFullPath((Get-Item -LiteralPath $currentLink).Target) } else { '' }
    if (-not [string]::Equals($currentTarget, [System.IO.Path]::GetFullPath($installDir), [System.StringComparison]::OrdinalIgnoreCase)) {
        $backup = "$currentLink.greendev-backup-$(Get-Date -Format 'yyyyMMddHHmmssfff')"
        if (Test-Path -LiteralPath $currentLink) {
            $currentItem = Get-Item -LiteralPath $currentLink -Force
            if (-not ($currentItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint)) { throw "currentLink is not a directory link: $currentLink" }
            Move-Item -LiteralPath $currentLink -Destination $backup
        }
        try {
            Invoke-GreenDevFailpoint 'before-switch'
            & cmd.exe /d /c "mklink /J `"$currentLink`" `"$installDir`"" | Out-Host
            if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath (Join-Path $currentLink ([string]$component.healthPath)))) { throw 'current junction validation failed.' }
            if (Test-Path -LiteralPath $backup) { Remove-Item -LiteralPath $backup -Force }
        } catch {
            if (Test-Path -LiteralPath $currentLink) { Remove-Item -LiteralPath $currentLink -Force }
            if (Test-Path -LiteralPath $backup) { Move-Item -LiteralPath $backup -Destination $currentLink }
            throw
        }
    }
}

Write-Host "Installed: $($component.name) $($component.version)"
Write-Host "Location: $installDir"
Write-Host 'Existing versions were preserved.'
