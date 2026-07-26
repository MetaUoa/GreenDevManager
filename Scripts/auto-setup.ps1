param(
    [string]$Lang = "zh"
)

$ErrorActionPreference = "Continue"
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
$isEn = $Lang -match "^(en|english)$"
$components = Get-FrameworksComponents -Root $root

if ($isEn) {
    Write-Host "========================================"
    Write-Host "Frameworks Auto Configuration Helper"
    Write-Host "========================================"
    Write-Host "Root: $root"
    Write-Host ""
    Write-Host "Features:"
    Write-Host "  - Detect installed tools"
    Write-Host "  - Show download locations for missing tools"
    Write-Host "  - Point to the environment setup scripts"
} else {
    Write-Host "========================================"
    Write-Host "Frameworks auto-setup helper"
    Write-Host "========================================"
    Write-Host "Root: $root"
    Write-Host ""
    Write-Host "Detect installed tools / show download hints / setup scripts"
}
Write-Host ""

$found = 0
foreach ($c in $components) {
    if (Test-Path -LiteralPath $c.Detect) {
        $found++
        Write-Host "[OK] $($c.Name): $($c.Detect)"
    } elseif ($isEn) {
        Write-Host "[MISS] $($c.Name)"
        Write-Host "       Download: $($c.Download)"
        Write-Host "       Extract to: $($c.ExtractTo)"
    } else {
        Write-Host "[MISS] $($c.Name)"
        Write-Host "       Download: $($c.Download)"
        Write-Host "       ExtractTo: $($c.ExtractTo)"
    }
}

Write-Host ""
Write-Host "Detection complete: $found / $($components.Count)"
Write-Host ""
Write-Host "Load session env:"
Write-Host "  call $root\Scripts\dev-shell.bat"
Write-Host ""
Write-Host "Verify:"
Write-Host "  $root\env-setup.bat"
Write-Host ""
Write-Host "Write user env vars:"
Write-Host "  $root\setup_dev_env.bat"
