param([Parameter(Mandatory)][string]$Path)

$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Path).Path
$manifestPath = if (Test-Path -LiteralPath $resolved -PathType Container) { Join-Path $resolved 'plugin.json' } else { $resolved }
$plugin = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$errors = [Collections.Generic.List[string]]::new()
if ($plugin.schemaVersion -ne 1) { $errors.Add('schemaVersion must be 1.') }
if ($plugin.id -notmatch '^[A-Za-z0-9_-]+$') { $errors.Add('id contains unsupported characters.') }
if (-not $plugin.name) { $errors.Add('name is required.') }
$known = @('network', 'process', 'writeRoots')
foreach ($permission in @($plugin.permissions.psobject.Properties.Name)) {
    if ($permission -notin $known) { $errors.Add("Unknown permission: $permission") }
}
[pscustomobject]@{ valid = $errors.Count -eq 0; errors = @($errors); manifest = $manifestPath; permissions = $plugin.permissions } | ConvertTo-Json -Depth 8
if ($errors.Count) { exit 2 }

