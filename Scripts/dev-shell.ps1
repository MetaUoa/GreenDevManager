# Load Frameworks development environment for this PowerShell session only.
$common = Join-Path $PSScriptRoot 'frameworks-common.ps1'
. $common

$root = Resolve-FrameworksRoot
Set-FrameworksSessionEnvironment -Root $root

Write-Host "Frameworks environment loaded. FRAMEWORKS_HOME=$env:FRAMEWORKS_HOME"
