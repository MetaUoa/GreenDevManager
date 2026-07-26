param(
    [string]$Lang = "zh"
)

$ErrorActionPreference = "Continue"
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
if (-not $env:FRAMEWORKS_HOME) { $env:FRAMEWORKS_HOME = $root }

$isEn = $Lang -match "^(en|english)$"
$components = Get-FrameworksComponents -Root $root

if ($isEn) {
    Write-Host "========================================"
    Write-Host "Frameworks Environment Setup"
    Write-Host "========================================"
} else {
    Write-Host "========================================"
    Write-Host "Frameworks env check"
    Write-Host "========================================"
}
Write-Host "FRAMEWORKS_HOME=$env:FRAMEWORKS_HOME"
Write-Host ""

foreach ($c in $components) {
    $probe = $c.Detect
    # Prefer env-expanded homes when set
    if ($c.Key -eq 'java' -and $env:JAVA_HOME) {
        $probe = Join-Path $env:JAVA_HOME 'bin\java.exe'
    } elseif ($c.Key -eq 'gradle' -and $env:GRADLE_HOME) {
        $probe = Join-Path $env:GRADLE_HOME 'bin\gradle.bat'
    } elseif ($c.Key -eq 'maven' -and $env:MAVEN_HOME) {
        $probe = Join-Path $env:MAVEN_HOME 'bin\mvn.cmd'
    } elseif ($c.Key -eq 'node' -and $env:NODE_HOME) {
        $probe = Join-Path $env:NODE_HOME 'node.exe'
    } elseif ($c.Key -eq 'rust' -and $env:RUST_HOME) {
        $probe = Join-Path $env:RUST_HOME 'bin\rustc.exe'
    } elseif ($c.Key -eq 'mysql' -and $env:MYSQL_HOME) {
        $probe = Join-Path $env:MYSQL_HOME 'bin\mysql.exe'
    } elseif ($c.Key -eq 'android' -and $env:ANDROID_HOME) {
        $probe = Join-Path $env:ANDROID_HOME 'platform-tools\adb.exe'
    }

    if (Test-Path -LiteralPath $probe) {
        Write-Host "[OK] $($c.Name): $probe"
    } else {
        Write-Host "[MISS] $($c.Name): $probe"
    }
}

Write-Host ""
if ($isEn) {
    Write-Host "Environment loaded for this terminal."
    Write-Host "Tool versions:"
} else {
    Write-Host "Session env loaded (if via env-setup.bat)."
    Write-Host "Tool versions:"
}

cmd /c "java -version 2>&1" | Select-Object -First 1
cmd /c "gradle --version 2>&1" | Select-String "Gradle" | Select-Object -First 1
cmd /c "mvn --version 2>&1" | Select-String "Apache Maven" | Select-Object -First 1
cmd /c "adb version 2>&1" | Select-String "Android Debug Bridge" | Select-Object -First 1
cmd /c "node --version 2>&1"
cmd /c "rustc --version 2>&1"
cmd /c "cargo --version 2>&1"
cmd /c "gcc --version 2>&1" | Select-Object -First 1
cmd /c "mysql --version 2>&1"
cmd /c "iasl -v 2>&1" | Select-String "ASL+" | Select-Object -First 1

Write-Host ""
if ($isEn) {
    Write-Host "Close this terminal to discard the temporary environment."
    Write-Host "Run setup_dev_env.bat to write persistent user environment variables."
} else {
    Write-Host "Close terminal to discard temp env. Run setup_dev_env.bat for permanent user env."
}
