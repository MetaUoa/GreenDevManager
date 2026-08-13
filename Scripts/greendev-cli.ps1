param(
    [Parameter(Position = 0)][string]$Command = 'help',
    [Parameter(Position = 1)][string]$Target,
    [Parameter(Position = 2)][string]$Value,
    [switch]$Json
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$manifestPath = Join-Path $root 'Config\greendev\components.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$started = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$operationId = "cli-$Command-$started"
$transactionDir = Join-Path $root 'Caches\GreenDevManager\transactions'
$logDir = Join-Path $root 'Logs\GreenDev'
New-Item -ItemType Directory -Path $transactionDir, $logDir -Force | Out-Null
$transaction = Join-Path $transactionDir "$operationId.json"
[ordered]@{ id = $operationId; title = "CLI $Command"; kind = 'cli'; status = 'running'; stage = 'starting'; startedAt = $started; updatedAt = $started } | ConvertTo-Json | Set-Content -LiteralPath $transaction -Encoding UTF8
$output = New-Object System.Collections.Generic.List[string]
$success = $false

function Write-Result($Object) {
    if ($Json) { $Object | ConvertTo-Json -Depth 20 }
    elseif ($Object -is [string]) { Write-Host $Object }
    else { $Object | Format-Table -AutoSize | Out-Host }
}
function Get-Component([string]$Id) { $item = $manifest.components | Where-Object id -eq $Id | Select-Object -First 1; if (-not $item) { throw "Unknown component: $Id" }; $item }
function Resolve-Managed([string]$Relative) { if ([IO.Path]::IsPathRooted($Relative) -or $Relative -match '(^|[\\/])\.\.([\\/]|$)') { throw "Path is outside Frameworks: $Relative" }; [IO.Path]::GetFullPath((Join-Path $root $Relative)) }
function Switch-Component($Component, [string]$TargetPath) {
    if (-not $Component.currentLink) { throw "Component has no current link: $($Component.id)" }
    $targetFull = if ([IO.Path]::IsPathRooted($TargetPath)) { [IO.Path]::GetFullPath($TargetPath) } else { Resolve-Managed $TargetPath }
    $rootPrefix = [IO.Path]::GetFullPath($root).TrimEnd('\') + '\'; if (-not $targetFull.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'Target is outside Frameworks.' }
    if (-not (Test-Path -LiteralPath (Join-Path $targetFull ([string]$Component.healthPath)) -PathType Leaf)) { throw "Target health file is missing: $targetFull" }
    $current = Resolve-Managed ([string]$Component.currentLink); $backup = "$current.greendev-backup-$(Get-Date -Format 'yyyyMMddHHmmssfff')"
    if (Test-Path -LiteralPath $current) { $item = Get-Item -LiteralPath $current -Force; if (-not ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Current path is not a junction: $current" }; Move-Item -LiteralPath $current -Destination $backup }
    try { & cmd.exe /d /c "mklink /J `"$current`" `"$targetFull`"" | Out-Null; if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath (Join-Path $current ([string]$Component.healthPath)))) { throw 'Current link validation failed.' }; if (Test-Path -LiteralPath $backup) { Remove-Item -LiteralPath $backup -Force } }
    catch { if (Test-Path -LiteralPath $current) { Remove-Item -LiteralPath $current -Force }; if (Test-Path -LiteralPath $backup) { Move-Item -LiteralPath $backup -Destination $current }; throw }
    "Switched $($Component.id) -> $targetFull; installed versions preserved."
}

try {
    switch ($Command.ToLowerInvariant()) {
        'help' { Write-Result "greendev list|doctor|plan ID|install ID|update ID|use ID PATH|profile ID|lock ID|diff ID|validate|audit|completion powershell|cmd" }
        'list' { $rows = foreach ($component in $manifest.components) { $install = Resolve-Managed ([string]$component.installDir); [pscustomobject]@{ id = $component.id; version = $component.version; installed = Test-Path -LiteralPath (Join-Path $install ([string]$component.healthPath)); current = if ($component.currentLink) { (Get-Item -LiteralPath (Resolve-Managed ([string]$component.currentLink)) -ErrorAction SilentlyContinue).Target } else { '' } } }; Write-Result $rows }
        'doctor' { & (Join-Path $root 'env-setup.bat') en; if ($LASTEXITCODE -ne 0) { throw 'Doctor reported an error.' } }
        'plan' { & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'Scripts\manage-component.ps1') -Action plan -Id $Target -ManifestPath $manifestPath; if ($LASTEXITCODE -ne 0) { throw 'Plan failed.' } }
        { $_ -in @('install', 'update') } { & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'Scripts\manage-component.ps1') -Action $_ -Id $Target -ManifestPath $manifestPath; if ($LASTEXITCODE -ne 0) { throw "$Command failed." } }
        'use' { Write-Result (Switch-Component (Get-Component $Target) $Value) }
        'profile' { $profiles = Get-Content (Join-Path $root 'Config\greendev\profile-sets.json') -Raw | ConvertFrom-Json; $profile = $profiles.profiles | Where-Object id -eq $Target | Select-Object -First 1; if (-not $profile) { throw "Unknown profile: $Target" }; foreach ($id in $profile.components) { $component = Get-Component $id; $install = Resolve-Managed ([string]$component.installDir); if ($component.currentLink -and (Test-Path -LiteralPath (Join-Path $install ([string]$component.healthPath)))) { Write-Result (Switch-Component $component $install) } }; Write-Result "Profile applied: $Target" }
        'lock' { $profiles = Get-Content (Join-Path $root 'Config\greendev\profile-sets.json') -Raw | ConvertFrom-Json; $profile = $profiles.profiles | Where-Object id -eq $Target | Select-Object -First 1; if (-not $profile) { throw "Unknown profile: $Target" }; $items = foreach ($id in $profile.components) { $component = Get-Component $id; [ordered]@{ id = $id; version = [string]$component.version; installDir = [string]$component.installDir; currentTarget = if ($component.currentLink) { [string](Get-Item -LiteralPath (Resolve-Managed ([string]$component.currentLink)) -ErrorAction SilentlyContinue).Target } else { '' }; sha256 = [string]$component.source.sha256; dependsOn = @($component.dependsOn) } }; $directory = Join-Path $root 'Config\greendev\profile-locks'; New-Item -ItemType Directory -Path $directory -Force | Out-Null; $path = Join-Path $directory "$Target.lock.json"; [ordered]@{ schemaVersion = 1; profileId = $Target; generatedAt = $started; components = @($items) } | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $path -Encoding UTF8; Write-Result "Lock: $path" }
        'diff' { $lock = Get-Content (Join-Path $root "Config\greendev\profile-locks\$Target.lock.json") -Raw | ConvertFrom-Json; $rows = foreach ($item in $lock.components) { $component = Get-Component ([string]$item.id); $installed = Test-Path -LiteralPath (Join-Path (Resolve-Managed ([string]$component.installDir)) ([string]$component.healthPath)); $current = if ($component.currentLink) { [string](Get-Item -LiteralPath (Resolve-Managed ([string]$component.currentLink)) -ErrorAction SilentlyContinue).Target } else { '' }; [pscustomobject]@{ id = $item.id; installed = $installed; versionMatch = [string]$component.version -eq [string]$item.version; currentMatch = $current -eq [string]$item.currentTarget } }; Write-Result $rows }
        'validate' { foreach ($component in $manifest.components) { & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'Scripts\manage-component.ps1') -Action plan -Id $component.id -ManifestPath $manifestPath | Out-Null; if ($LASTEXITCODE -ne 0) { throw "Invalid component: $($component.id)" } }; Write-Result "Manifest valid: $($manifest.components.Count) components" }
        'audit' { $policy = Get-Content (Join-Path $root 'Config\greendev\enterprise-policy.json') -Raw | ConvertFrom-Json; Write-Result ([pscustomobject]@{ machineGroup = $policy.machineGroup; readOnly = $policy.readOnly; lockedFields = @($policy.lockedFields) -join ','; operationsLog = Join-Path $logDir 'operations.jsonl' }) }
        'completion' {
            $commands = 'list','doctor','plan','install','update','use','profile','lock','diff','validate','audit','completion'
            if ($Target -eq 'powershell') {
                Write-Result @"
Register-ArgumentCompleter -Native -CommandName greendev,greendev.exe -ScriptBlock {
    param(`$wordToComplete)
    @('$($commands -join "','")') | Where-Object { `$_ -like "`$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new(`$_, `$_, 'ParameterValue', `$_) }
}
"@
            } elseif ($Target -eq 'cmd') {
                $completionLine = "doskey greendev=`"$root\greendev.exe`" `$*"
                Write-Result "$completionLine`nrem Commands: $($commands -join ', ')"
            } else { throw 'Completion shell must be powershell or cmd.' }
        }
        default { throw "Unknown command: $Command" }
    }
    $success = $true
} finally {
    $finished = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); $result = [ordered]@{ operationId = $operationId; success = $success; title = "CLI $Command"; summary = if ($success) { 'completed' } else { 'failed' }; output = $output -join "`n"; exitCode = if ($success) { 0 } else { 1 }; kind = 'cli'; startedAt = $started; finishedAt = $finished }
    $result | ConvertTo-Json -Compress | Add-Content -LiteralPath (Join-Path $logDir 'operations.jsonl') -Encoding UTF8
    $result['status'] = if ($success) { 'completed' } else { 'failed' }; $result['stage'] = $result['status']; $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "${transaction}.completed.json" -Encoding UTF8
    if (Test-Path -LiteralPath $transaction) { Remove-Item -LiteralPath $transaction -Force }
}
