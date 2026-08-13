param(
    [ValidateSet('inventory', 'plan', 'apply', 'rollback')][string]$Action = 'inventory',
    [string]$PlanPath,
    [string]$TransactionId
)

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$transactions = Join-Path $root 'Caches\GreenDevManager\agent-transactions'
New-Item -ItemType Directory -Path $transactions -Force | Out-Null
switch ($Action) {
    'inventory' {
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'greendev-cli.ps1') list -Json
    }
    'plan' {
        if (-not $PlanPath) { throw 'PlanPath is required.' }
        $plan = Get-Content -LiteralPath $PlanPath -Raw | ConvertFrom-Json
        [pscustomobject]@{ accepted = $true; protocol = 1; node = $env:COMPUTERNAME; planId = $plan.id; previewOnly = $true } | ConvertTo-Json
    }
    'apply' {
        if (-not $PlanPath) { throw 'PlanPath is required.' }
        $plan = Get-Content -LiteralPath $PlanPath -Raw | ConvertFrom-Json
        $manifest = Get-Content (Join-Path $root 'Config\greendev\components.json') -Raw | ConvertFrom-Json
        $component = $manifest.components | Where-Object id -eq $plan.componentId | Select-Object -First 1
        if (-not $component) { throw "Component is not present in the node manifest: $($plan.componentId)" }
        if ($plan.version -and [string]$component.version -ne [string]$plan.version) { throw "Node manifest version is $($component.version), requested $($plan.version)." }
        $previousTarget = if ($component.currentLink) { [string](Get-Item -LiteralPath (Join-Path $root ([string]$component.currentLink)) -ErrorAction SilentlyContinue).Target } else { '' }
        $record = [ordered]@{ schemaVersion = 1; id = $plan.id; state = 'running'; approvedAt = (Get-Date).ToUniversalTime().ToString('o'); componentId = $plan.componentId; previousTarget = $previousTarget; plan = $plan }
        $record | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $transactions "$($plan.id).json") -Encoding UTF8
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'greendev-cli.ps1') update ([string]$plan.componentId)
        if ($LASTEXITCODE -ne 0) { throw "Component update failed: $($plan.componentId)" }
        $record['state'] = 'completed'; $record['completedAt'] = (Get-Date).ToUniversalTime().ToString('o')
        $record | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $transactions "$($plan.id).json") -Encoding UTF8
        [pscustomobject]@{ accepted = $true; transactionId = $plan.id; state = 'completed'; componentId = $plan.componentId } | ConvertTo-Json
    }
    'rollback' {
        if (-not $TransactionId) { throw 'TransactionId is required.' }
        $path = Join-Path $transactions "$TransactionId.json"
        $record = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
        if ($record.previousTarget) {
            & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'greendev-cli.ps1') use ([string]$record.componentId) ([string]$record.previousTarget)
            if ($LASTEXITCODE -ne 0) { throw "Component rollback failed: $($record.componentId)" }
        }
        $record | Add-Member -NotePropertyName state -NotePropertyValue 'rolled-back' -Force
        $record | Add-Member -NotePropertyName rolledBackAt -NotePropertyValue (Get-Date).ToUniversalTime().ToString('o') -Force
        $record | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $path -Encoding UTF8
        [pscustomobject]@{ accepted = $true; transactionId = $TransactionId; state = 'rolled-back' } | ConvertTo-Json
    }
}
