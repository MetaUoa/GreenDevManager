param(
    [Parameter(Mandatory)][string]$Path,
    [Parameter(Mandatory)][string]$Thumbprint,
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Path).Path
$normalized = ($Thumbprint -replace '\s', '').ToUpperInvariant()
$certificate = Get-ChildItem Cert:\CurrentUser\My, Cert:\LocalMachine\My -ErrorAction SilentlyContinue |
    Where-Object { ($_.Thumbprint -replace '\s', '').ToUpperInvariant() -eq $normalized } |
    Select-Object -First 1
if (-not $certificate) { throw "Signing certificate was not found: $normalized" }
$rsa = [Security.Cryptography.X509Certificates.RSACertificateExtensions]::GetRSAPrivateKey($certificate)
if (-not $rsa) { throw 'The signing certificate has no accessible RSA private key.' }
$bytes = [System.IO.File]::ReadAllBytes($resolved)
$signature = $rsa.SignData($bytes, [Security.Cryptography.HashAlgorithmName]::SHA256, [Security.Cryptography.RSASignaturePadding]::Pss)
$target = if ($OutputPath) { $OutputPath } else { "$resolved.sig.json" }
$document = [ordered]@{
    schemaVersion = 1
    algorithm = 'RSA-PSS-SHA256'
    fileName = [IO.Path]::GetFileName($resolved)
    sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
    signedAt = (Get-Date).ToUniversalTime().ToString('o')
    thumbprint = $certificate.Thumbprint
    certificate = [Convert]::ToBase64String($certificate.RawData)
    signature = [Convert]::ToBase64String($signature)
}
$document | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $target -Encoding UTF8
Write-Host "Signature: $target"
