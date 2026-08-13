$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$config = Get-Content (Join-Path $root 'Config\greendev\remote-nodes.json') -Raw | ConvertFrom-Json
$results = foreach ($node in @($config.nodes)) {
    $started = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    try {
        $inventory = switch ([string]$node.transport) {
            'local' {
                & (Join-Path $root 'Scripts\greendev-agent.ps1') -Action inventory | ConvertFrom-Json
            }
            'winrm' {
                $remoteRoot = [string]$node.root
                if (-not $remoteRoot) { throw "WinRM node '$($node.id)' must define its Frameworks root." }
                Invoke-Command -ComputerName ([string]$node.host) -ScriptBlock {
                    param($FrameworksRoot)
                    & (Join-Path $FrameworksRoot 'Scripts\greendev-agent.ps1') -Action inventory
                } -ArgumentList $remoteRoot | Out-String | ConvertFrom-Json
            }
            'agent' {
                $headers = @{}
                if ($node.credentialRef) {
                    $token = [Environment]::GetEnvironmentVariable([string]$node.credentialRef, 'Process')
                    if (-not $token) { $token = [Environment]::GetEnvironmentVariable([string]$node.credentialRef, 'User') }
                    if ($token) { $headers.Authorization = "Bearer $token" }
                }
                Invoke-RestMethod -Method Get -Uri "$([string]$node.endpoint)/v1/inventory" -Headers $headers
            }
            default { throw "Unsupported transport: $($node.transport)" }
        }
        [ordered]@{ id = $node.id; status = 'online'; checkedAt = $started; transport = $node.transport; group = $node.group; inventory = @($inventory); error = '' }
    } catch {
        [ordered]@{ id = $node.id; status = 'offline'; checkedAt = $started; transport = $node.transport; group = $node.group; inventory = @(); error = $_.Exception.Message }
    }
}
[ordered]@{ schemaVersion = 1; generatedAt = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); nodes = @($results) } | ConvertTo-Json -Depth 20
