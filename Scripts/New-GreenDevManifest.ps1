param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9_-]+$')][string]$Id,
    [string]$Name,
    [string]$Version = '1.0.0',
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$target = if ($OutputPath) { $OutputPath } else { Join-Path $root "Config\greendev\examples\$Id.json" }
$displayName = if ($Name) { $Name } else { $Id }
$manifest = [ordered]@{
    schemaVersion = 2
    components = @([ordered]@{
        id = $Id
        name = $displayName
        version = $Version
        installDir = "Tools\$Id\$Version"
        currentLink = "Tools\$Id\current"
        healthPath = "$Id.exe"
        enabled = $true
        dependsOn = @()
        source = [ordered]@{
            type = 'archive'
            url = "https://HOST/$Id-$Version.zip"
            archive = "downloads\packages\$Id-$Version.zip"
            sha256 = 'SHA256'
        }
    })
}
$parent = Split-Path -Parent $target
New-Item -ItemType Directory -Path $parent -Force | Out-Null
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $target -Encoding UTF8
Write-Host "Manifest template: $target"

