param(
    [string]$Lang = 'zh',
    [string]$Keys = 'all',
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
$requested = @($Keys -split '[,;\s]+' | Where-Object { $_ } | ForEach-Object { $_.ToLowerInvariant() })
$all = $requested.Count -eq 0 -or $requested -contains 'all' -or $requested -contains '*'

function Is-Requested([string]$Key) {
    return $all -or $requested -contains $Key
}

function Sync-File([string]$Key, [string]$Source, [string]$Destination) {
    if (-not (Is-Requested $Key)) { return }
    if (-not (Test-Path -LiteralPath $Source)) {
        throw "Missing authoritative config: $Source"
    }
    $parent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
    if (-not $Quiet) { Write-Host "[SYNC] $Key -> $Destination" }
}

Sync-File 'rust' (Join-Path $root 'Config\cargo\config.toml') (Join-Path $root 'Toolchains\Rust\cargo-home\config.toml')
Sync-File 'gradle' (Join-Path $root 'Config\gradle\gradle.properties') (Join-Path $root 'Caches\Gradle\gradle.properties')
Sync-File 'gradle' (Join-Path $root 'Config\gradle\init.d\cn-mirrors.init.gradle') (Join-Path $root 'Caches\Gradle\init.d\cn-mirrors.init.gradle')
Sync-File 'maven' (Join-Path $root 'Config\maven\settings.xml') (Join-Path $root 'BuildTools\Maven\current\conf\settings.xml')

if (Is-Requested 'mysql') {
    $template = Join-Path $root 'Config\mysql\my.ini.template'
    $destination = Join-Path $root 'Databases\Sql\mysql\my.ini'
    if (-not (Test-Path -LiteralPath $template)) { throw "Missing authoritative config: $template" }
    $forwardRoot = $root.Replace('\', '/')
    $content = (Get-Content -LiteralPath $template -Raw).Replace('{{FRAMEWORKS_HOME_FWD}}', $forwardRoot)
    $parent = Split-Path -Parent $destination
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Set-Content -LiteralPath $destination -Value $content -Encoding UTF8
    if (-not $Quiet) { Write-Host "[SYNC] mysql -> $destination" }
}

if (-not $Quiet) { Write-Host 'Configuration sync complete.' }
