param(
    [string]$Lang = 'zh',
    [switch]$Deep
)

$ErrorActionPreference = 'Continue'
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
if (-not $env:FRAMEWORKS_HOME) { $env:FRAMEWORKS_HOME = $root }
$isEn = $Lang -match '^(en|english)$'
$components = Get-FrameworksComponents -Root $root
$issues = 0
$warnings = 0

function Say([string]$Zh, [string]$En) {
    if ($isEn) { Write-Host $En } else { Write-Host $Zh }
}

function Test-PathChecked([string]$Path, [ref]$Failure) {
    try {
        $exists = Test-Path -LiteralPath $Path -ErrorAction Stop
        $Failure.Value = $null
        return $exists
    } catch {
        $Failure.Value = $_.Exception.Message
        return $false
    }
}

function Get-Sha256([string]$Path) {
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            return ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '')
        } finally {
            $sha.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

Write-Host '========================================'
Say 'Frameworks Environment Doctor' 'Frameworks Environment Doctor'
Write-Host '========================================'
Write-Host "FRAMEWORKS_HOME=$env:FRAMEWORKS_HOME"
Write-Host ''

Say '-- current links --' '-- current links --'
$currentLinks = @(
    'Runtimes\Java\current',
    'Runtimes\Node\current',
    'Runtimes\Python\current',
    'BuildTools\Gradle\current',
    'BuildTools\Maven\current',
    'Toolchains\Rust\current',
    'Databases\Sql\mysql\current'
)
$rootPrefix = $root.TrimEnd('\') + '\'
foreach ($relative in $currentLinks) {
    $path = Join-Path $root $relative
    try {
        $item = Get-Item -LiteralPath $path -Force -ErrorAction Stop
        if (-not $item.LinkType) {
            Write-Host "[WARN] $relative is not a junction/symlink"
            $warnings++
            continue
        }
        $target = [string]($item.Target | Select-Object -First 1)
        if (-not [System.IO.Path]::IsPathRooted($target)) {
            $target = [System.IO.Path]::GetFullPath((Join-Path $item.Parent.FullName $target))
        }
        $failure = $null
        $exists = Test-PathChecked $target ([ref]$failure)
        if (-not $target.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            Write-Host "[EXTERNAL] $relative -> $target"
            $issues++
        } elseif (-not $exists) {
            Write-Host "[BROKEN] $relative -> $target$(if ($failure) { " ($failure)" })"
            $issues++
        } else {
            Write-Host "[OK] $relative -> $target"
        }
    } catch {
        Write-Host "[MISS] $relative ($($_.Exception.Message))"
        $issues++
    }
}

Write-Host ''
Say '-- components --' '-- components --'
foreach ($c in $components) {
    $probe = $c.Detect
    if ($c.Key -eq 'java' -and $env:JAVA_HOME) { $probe = Join-Path $env:JAVA_HOME 'bin\java.exe' }
    elseif ($c.Key -eq 'gradle' -and $env:GRADLE_HOME) { $probe = Join-Path $env:GRADLE_HOME 'bin\gradle.bat' }
    elseif ($c.Key -eq 'maven' -and $env:MAVEN_HOME) { $probe = Join-Path $env:MAVEN_HOME 'bin\mvn.cmd' }
    elseif ($c.Key -eq 'node' -and $env:NODE_HOME) { $probe = Join-Path $env:NODE_HOME 'node.exe' }
    elseif ($c.Key -eq 'rust' -and $env:RUST_HOME) { $probe = Join-Path $env:RUST_HOME 'bin\rustc.exe' }
    elseif ($c.Key -eq 'mysql' -and $env:MYSQL_HOME) { $probe = Join-Path $env:MYSQL_HOME 'bin\mysql.exe' }
    elseif ($c.Key -eq 'android' -and $env:ANDROID_HOME) { $probe = Join-Path $env:ANDROID_HOME 'platform-tools\adb.exe' }

    $failure = $null
    if (Test-PathChecked $probe ([ref]$failure)) {
        Write-Host "[OK] $($c.Name): $probe"
    } elseif ($failure) {
        Write-Host "[ERROR] $($c.Name): $probe ($failure)"
        $issues++
    } else {
        Write-Host "[MISS] $($c.Name): $probe"
        $issues++
    }
}

$npmProbe = Join-Path $root 'Runtimes\Node\current\npm.cmd'
$npmFailure = $null
if (Test-PathChecked $npmProbe ([ref]$npmFailure)) {
    Write-Host "[OK] npm: $npmProbe"
} else {
    Write-Host "[MISS] npm: $npmProbe$(if ($npmFailure) { " ($npmFailure)" })"
    $issues++
}

Write-Host ''
Say '-- authoritative configuration --' '-- authoritative configuration --'
$configPairs = @(
    @('Config\cargo\config.toml', 'Toolchains\Rust\cargo-home\config.toml'),
    @('Config\gradle\gradle.properties', 'Caches\Gradle\gradle.properties'),
    @('Config\gradle\init.d\cn-mirrors.init.gradle', 'Caches\Gradle\init.d\cn-mirrors.init.gradle'),
    @('Config\maven\settings.xml', 'BuildTools\Maven\current\conf\settings.xml')
)
foreach ($pair in $configPairs) {
    $source = Join-Path $root $pair[0]
    $active = Join-Path $root $pair[1]
    try {
        if (-not (Test-Path -LiteralPath $active -ErrorAction Stop)) {
            Write-Host "[MISS] $($pair[1])"
            $issues++
        } elseif ((Get-Sha256 $source) -ne (Get-Sha256 $active)) {
            Write-Host "[DRIFT] $($pair[1])"
            $issues++
        } else {
            Write-Host "[OK] $($pair[1])"
        }
    } catch {
        Write-Host "[ERROR] $($pair[1]): $($_.Exception.Message)"
        $issues++
    }
}

$mysqlIni = Join-Path $root 'Databases\Sql\mysql\my.ini'
$expectedData = $root.Replace('\', '/') + '/Databases/Sql/mysql/resources/data'
if ((Test-Path -LiteralPath $mysqlIni) -and (Get-Content -LiteralPath $mysqlIni -Raw) -match [regex]::Escape("datadir=$expectedData")) {
    Write-Host '[OK] Databases\Sql\mysql\my.ini'
} else {
    Write-Host '[DRIFT] Databases\Sql\mysql\my.ini'
    $issues++
}

$mysqlRoot = Join-Path $root 'Databases\Sql\mysql'
Get-ChildItem -LiteralPath $mysqlRoot -Directory -Filter 'mysql-*' -ErrorAction SilentlyContinue | ForEach-Object {
    $legacyMysqlData = Join-Path $_.FullName 'resources\data'
    if (Test-Path -LiteralPath $legacyMysqlData) {
        Write-Host "[WARN] legacy MySQL data tree present: $legacyMysqlData"
        $script:warnings++
    }
}

Write-Host ''
if ($Deep) {
    Say '-- tool versions (deep) --' '-- tool versions (deep) --'
    cmd /c "java -version 2>&1" | Select-Object -First 1
    cmd /c "gradle --version 2>&1" | Select-String 'Gradle' | Select-Object -First 1
    cmd /c "mvn --version 2>&1" | Select-String 'Apache Maven' | Select-Object -First 1
    cmd /c "adb version 2>&1" | Select-String 'Android Debug Bridge' | Select-Object -First 1
    cmd /c "node --version 2>&1"
    cmd /c "npm --version 2>&1"
    cmd /c "python --version 2>&1"
    cmd /c "rustc --version 2>&1"
    cmd /c "cargo --version 2>&1"
    cmd /c "gcc --version 2>&1" | Select-Object -First 1
    cmd /c "mysql --version 2>&1"
    cmd /c "iasl -v 2>&1" | Select-String 'ASL+' | Select-Object -First 1
} else {
    Say '-- tool versions skipped; use env-setup.bat en deep for executable probes --' '-- tool versions skipped; use env-setup.bat en deep for executable probes --'
}

Write-Host ''
Say ("Doctor complete: {0} issue(s), {1} warning(s)." -f $issues, $warnings) ("Doctor complete: {0} issue(s), {1} warning(s)." -f $issues, $warnings)
if ($issues -gt 0) { exit 1 }
