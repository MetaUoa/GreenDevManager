param(
    [Parameter(Mandatory = $true)]
    [string]$BackupPath,
    [string]$Lang = 'zh'
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
$backupRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'Config\env-backups')).TrimEnd('\') + '\'
$resolved = [System.IO.Path]::GetFullPath($BackupPath)
if (-not $resolved.StartsWith($backupRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Backup is outside Config\env-backups: $resolved"
}
if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
    throw "Backup does not exist: $resolved"
}

$document = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json
if ($document.scope -ne 'User' -or $null -eq $document.variables) {
    throw 'Invalid user environment backup.'
}

$backupDir = Join-Path $root 'Config\env-backups'
$current = [ordered]@{}
$currentVars = [Environment]::GetEnvironmentVariables('User')
foreach ($key in ($currentVars.Keys | Sort-Object)) {
    $current[$key] = [string]$currentVars[$key]
}
$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$safetyPath = Join-Path $backupDir "user-env-before-restore-$timestamp.json"
[ordered]@{
    createdAt = (Get-Date).ToString('o')
    scope = 'User'
    root = $root
    computerName = $env:COMPUTERNAME
    userName = $env:USERNAME
    variables = $current
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $safetyPath -Encoding UTF8

$target = [ordered]@{}
foreach ($property in $document.variables.PSObject.Properties) {
    $target[$property.Name] = [string]$property.Value
}

foreach ($key in @($currentVars.Keys)) {
    if (-not $target.Contains($key)) {
        [Environment]::SetEnvironmentVariable([string]$key, $null, 'User')
    }
}
foreach ($key in $target.Keys) {
    [Environment]::SetEnvironmentVariable([string]$key, [string]$target[$key], 'User')
}

if ($Lang -match '^(en|english)$') {
    Write-Host "Restored user environment: $resolved"
    Write-Host "Safety backup: $safetyPath"
} else {
    Write-Host "已恢复用户环境变量: $resolved"
    Write-Host "恢复前备份: $safetyPath"
}
