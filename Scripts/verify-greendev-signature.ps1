param(
    [Parameter(Mandatory)][string]$Path,
    [string]$PolicyPath
)

$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Path).Path
$policy = if ($PolicyPath -and (Test-Path -LiteralPath $PolicyPath)) { Get-Content -LiteralPath $PolicyPath -Raw | ConvertFrom-Json } else { [pscustomobject]@{ requireDetachedSignatures = $false; requireTrustedChain = $false; trustedSignerThumbprints = @(); revokedSignerThumbprints = @() } }
$trusted = @($policy.trustedSignerThumbprints | ForEach-Object { ($_ -replace '\s', '').ToUpperInvariant() })
$revoked = @($policy.revokedSignerThumbprints | ForEach-Object { ($_ -replace '\s', '').ToUpperInvariant() })

function Test-DetachedSignature([string]$File) {
    $sidecar = "$File.sig.json"
    if (-not (Test-Path -LiteralPath $sidecar)) {
        if ($policy.requireDetachedSignatures) { throw "Detached signature is required: $File" }
        return [pscustomobject]@{ file = $File; signed = $false; valid = $true; signer = '' }
    }
    $document = Get-Content -LiteralPath $sidecar -Raw | ConvertFrom-Json
    if ($document.schemaVersion -ne 1 -or $document.algorithm -ne 'RSA-PSS-SHA256') { throw "Unsupported signature document: $sidecar" }
    if ($document.fileName -ne [IO.Path]::GetFileName($File)) { throw "Signature file name mismatch: $File" }
    $actualHash = (Get-FileHash -LiteralPath $File -Algorithm SHA256).Hash
    if ($actualHash -ne $document.sha256) { throw "Hash mismatch: $File" }
    $certificate = [Security.Cryptography.X509Certificates.X509Certificate2]::new([Convert]::FromBase64String($document.certificate))
    $thumbprint = ($certificate.Thumbprint -replace '\s', '').ToUpperInvariant()
    $declaredThumbprint = ([string]$document.thumbprint -replace '\s', '').ToUpperInvariant()
    if ($declaredThumbprint -ne $thumbprint) { throw "Certificate thumbprint mismatch: $File" }
    $signedAt = [DateTimeOffset]::Parse([string]$document.signedAt).UtcDateTime
    if ($signedAt -lt $certificate.NotBefore.ToUniversalTime() -or $signedAt -gt $certificate.NotAfter.ToUniversalTime()) { throw "Certificate was not valid at signing time: $File" }
    if ($revoked -contains $thumbprint) { throw "Signer is revoked: $thumbprint" }
    if ($trusted.Count -and $trusted -notcontains $thumbprint) { throw "Signer is outside the trust list: $thumbprint" }
    if ($policy.requireTrustedChain) {
        $chain = [Security.Cryptography.X509Certificates.X509Chain]::new()
        if (-not $chain.Build($certificate)) { throw "Certificate chain validation failed: $thumbprint" }
    }
    $rsa = [Security.Cryptography.X509Certificates.RSACertificateExtensions]::GetRSAPublicKey($certificate)
    $valid = $rsa.VerifyData([IO.File]::ReadAllBytes($File), [Convert]::FromBase64String($document.signature), [Security.Cryptography.HashAlgorithmName]::SHA256, [Security.Cryptography.RSASignaturePadding]::Pss)
    if (-not $valid) { throw "Signature verification failed: $File" }
    [pscustomobject]@{ file = $File; signed = $true; valid = $true; signer = $thumbprint }
}

$results = @()
if (Test-Path -LiteralPath $resolved -PathType Container) {
    $manifestPath = Join-Path $resolved 'release-manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) { throw 'release-manifest.json is missing.' }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    foreach ($artifact in @($manifest.artifacts)) {
        $file = Join-Path $resolved $artifact.name
        if (-not (Test-Path -LiteralPath $file)) { throw "Artifact is missing: $($artifact.name)" }
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $artifact.sha256) { throw "Artifact checksum mismatch: $($artifact.name)" }
        $results += Test-DetachedSignature $file
    }
    $results += Test-DetachedSignature $manifestPath
} else {
    $results += Test-DetachedSignature $resolved
}
$results | ConvertTo-Json -Depth 5
