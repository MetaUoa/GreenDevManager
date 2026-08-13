param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9_-]+$')][string]$Id,
    [Parameter(Mandatory)][ValidateSet('apply', 'rollback')][string]$Action
)

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$config = Get-Content (Join-Path $root 'Config\greendev\remote-nodes.json') -Raw | ConvertFrom-Json
$recordPath = Join-Path $root "Caches\GreenDevManager\fleet-rollouts\$Id.json"
$record = Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
$expected = if ($Action -eq 'apply') { 'approved' } else { 'rollback-requested' }
if ($record.status -ne $expected) { throw "Rollout state must be $expected." }
$record.status = 'running'
$record.events += [pscustomobject]@{ at = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); state = 'running'; detail = "$Action started" }
$record | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $recordPath -Encoding UTF8

function Invoke-Node($Node, $Plan, [string]$NodeAction) {
    $planRoot = Join-Path $root 'Caches\GreenDevManager\fleet-agent-plans'
    New-Item -ItemType Directory -Path $planRoot -Force | Out-Null
    $planPath = Join-Path $planRoot "$Id-$($Node.id)-$NodeAction.json"
    $Plan | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $planPath -Encoding UTF8
    switch ([string]$Node.transport) {
        'local' {
            & (Join-Path $root 'Scripts\greendev-agent.ps1') -Action $NodeAction -PlanPath $planPath -TransactionId $Id
            if ($LASTEXITCODE -ne 0) { throw "Local agent failed: $($Node.id)" }
        }
        'winrm' {
            $remoteRoot = if ($Node.root) { [string]$Node.root } else { 'D:\Frameworks' }
            $planJson = $Plan | ConvertTo-Json -Depth 20 -Compress
            Invoke-Command -ComputerName ([string]$Node.host) -ScriptBlock {
                param($FrameworksRoot, $RemoteAction, $PlanJson, $RolloutId)
                $planDirectory = Join-Path $FrameworksRoot 'Caches\GreenDevManager\fleet-agent-plans'
                New-Item -ItemType Directory -Path $planDirectory -Force | Out-Null
                $remotePlan = Join-Path $planDirectory "$RolloutId-winrm.json"
                Set-Content -LiteralPath $remotePlan -Value $PlanJson -Encoding UTF8
                & (Join-Path $FrameworksRoot 'Scripts\greendev-agent.ps1') -Action $RemoteAction -PlanPath $remotePlan -TransactionId $RolloutId
            } -ArgumentList $remoteRoot, $NodeAction, $planJson, $Id
        }
        'agent' {
            $headers = @{}
            if ($Node.credentialRef) {
                $token = [Environment]::GetEnvironmentVariable([string]$Node.credentialRef, 'Process')
                if (-not $token) { $token = [Environment]::GetEnvironmentVariable([string]$Node.credentialRef, 'User') }
                if ($token) { $headers.Authorization = "Bearer $token" }
            }
            Invoke-RestMethod -Method Post -Uri "$([string]$Node.endpoint)/v1/rollouts/$NodeAction" -Headers $headers -ContentType 'application/json' -Body ($Plan | ConvertTo-Json -Depth 20)
        }
        default { throw "Unsupported transport: $($Node.transport)" }
    }
}

$failed = $false
$paused = $false
try {
    foreach ($batch in @($record.plan.batches)) {
        $fresh = Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
        if ($fresh.status -eq 'paused') { throw 'Rollout paused at batch boundary.' }
        foreach ($nodeId in @($batch.nodes)) {
            $node = $config.nodes | Where-Object id -eq $nodeId | Select-Object -First 1
            if (-not $node) { throw "Node is not registered: $nodeId" }
            try {
                Invoke-Node $node $record.plan $Action | Out-Host
                $record.events += [pscustomobject]@{ at = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); state = 'node-completed'; node = $nodeId; detail = "$Action completed" }
            } catch {
                $record.events += [pscustomobject]@{ at = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); state = 'node-failed'; node = $nodeId; detail = $_.Exception.Message }
                $failed = $true
                throw
            } finally {
                $record | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $recordPath -Encoding UTF8
            }
        }
    }
} catch {
    if ($_.Exception.Message -like '*paused at batch boundary*') { $paused = $true } else { $failed = $true }
    throw
} finally {
    $record.status = if ($paused) { 'paused' } elseif ($failed) { 'failed' } elseif ($Action -eq 'rollback') { 'rolled-back' } else { 'completed' }
    $record.events += [pscustomobject]@{ at = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); state = $record.status; detail = "$Action finished" }
    $record | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $recordPath -Encoding UTF8
}
