param([Parameter(Mandatory = $true)][ValidateSet('preview', 'apply')][string]$Action)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')
$root = Resolve-FrameworksRoot
$curl = Resolve-SystemTool -Name 'curl.exe'
$policy = Get-Content (Join-Path $root 'Config\greendev\enterprise-policy.json') -Raw | ConvertFrom-Json
$repository = $policy.teamRepository
$cache = Join-Path $root 'Caches\GreenDevManager\team-repository'
$incoming = Join-Path $cache 'profile-sets.json'
New-Item -ItemType Directory -Path $cache -Force | Out-Null

switch ([string]$repository.kind) {
    'directory' {
        if (-not $repository.path) { throw 'Team repository directory is not configured.' }
        $source = Join-Path ([System.IO.Path]::GetFullPath([string]$repository.path)) 'profile-sets.json'
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Team profile file is missing: $source" }
        Copy-Item -LiteralPath $source -Destination $incoming -Force
    }
    'http' {
        if (-not $repository.url) { throw 'Team repository URL is not configured.' }
        & $curl --fail --location --retry 3 --output "$incoming.part" ([string]$repository.url)
        if ($LASTEXITCODE -ne 0) { throw "Team profile download failed: $LASTEXITCODE" }
        Move-Item -LiteralPath "$incoming.part" -Destination $incoming -Force
    }
    'git' {
        if (-not $repository.url) { throw 'Team Git URL is not configured.' }
        $gitRoot = Join-Path $cache 'git'
        if (-not (Test-Path -LiteralPath (Join-Path $gitRoot '.git'))) {
            & git.exe clone --depth 1 --branch ([string]$repository.branch) ([string]$repository.url) $gitRoot
        } else { & git.exe -C $gitRoot fetch --depth 1 origin ([string]$repository.branch); & git.exe -C $gitRoot reset --hard FETCH_HEAD }
        if ($LASTEXITCODE -ne 0) { throw "Team Git synchronization failed: $LASTEXITCODE" }
        Copy-Item -LiteralPath (Join-Path $gitRoot 'profile-sets.json') -Destination $incoming -Force
    }
    default { throw "Unsupported team repository kind: $($repository.kind)" }
}

$team = Get-Content -LiteralPath $incoming -Raw | ConvertFrom-Json
if ($team.schemaVersion -ne 1 -or -not $team.profiles) { throw 'Team profile schema is invalid.' }
$localPath = Join-Path $root 'Config\greendev\profile-sets.json'
$local = Get-Content -LiteralPath $localPath -Raw | ConvertFrom-Json
$localIds = @($local.profiles | ForEach-Object { $_.id })
$teamIds = @($team.profiles | ForEach-Object { $_.id })
$added = @($teamIds | Where-Object { $_ -notin $localIds })
$changed = @($team.profiles | Where-Object { $candidate = $_; $existing = $local.profiles | Where-Object id -eq $candidate.id | Select-Object -First 1; $existing -and (($existing | ConvertTo-Json -Depth 20 -Compress) -ne ($candidate | ConvertTo-Json -Depth 20 -Compress)) } | ForEach-Object id)
Write-Host "Team profiles: $($teamIds.Count); added: $($added.Count); changed: $($changed.Count)"
if ($added) { Write-Host "Added: $($added -join ', ')" }
if ($changed) { Write-Host "Changed: $($changed -join ', ')" }
if ($Action -eq 'preview') { exit 0 }

$history = Join-Path $root 'Config\config-backups\team-profiles'
New-Item -ItemType Directory -Path $history -Force | Out-Null
Copy-Item -LiteralPath $localPath -Destination (Join-Path $history "profile-sets-$(Get-Date -Format 'yyyyMMddHHmmssfff').json.bak")
$mergedProfiles = New-Object System.Collections.ArrayList
foreach ($existing in @($local.profiles)) { [void]$mergedProfiles.Add($existing) }
foreach ($candidate in @($team.profiles)) {
    $existing = $mergedProfiles | Where-Object id -eq $candidate.id | Select-Object -First 1
    if ($existing) {
        $machineOverrides = $existing.machineOverrides
        [void]$mergedProfiles.Remove($existing)
        if ($machineOverrides) { $candidate.machineOverrides = $machineOverrides }
    }
    $candidate.teamTemplate = $true
    [void]$mergedProfiles.Add($candidate)
}
$merged = [ordered]@{ schemaVersion = 1; activeProfile = [string]$local.activeProfile; profiles = @($mergedProfiles) }
$merged | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath "$localPath.importing" -Encoding UTF8
Move-Item -LiteralPath "$localPath.importing" -Destination $localPath -Force
Write-Host 'Team profiles merged; machine overrides and local-only profiles were preserved.'
