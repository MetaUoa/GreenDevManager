# Shared Frameworks helpers. Dot-source from Scripts\*.ps1 only.
# Provides: Resolve-FrameworksRoot, Get-FrameworksComponents,
#           Get-FrameworksSetupComponents, Set-FrameworksSessionEnvironment,
#           Add-FrameworksPath

function Resolve-FrameworksRoot {
    param([string]$Hint)

    if ($Hint -and (Test-Path -LiteralPath $Hint)) {
        return [System.IO.Path]::GetFullPath($Hint)
    }

    if ($env:FRAMEWORKS_HOME -and (Test-Path -LiteralPath $env:FRAMEWORKS_HOME)) {
        return [System.IO.Path]::GetFullPath($env:FRAMEWORKS_HOME)
    }

    # This file lives in <root>\Scripts
    $here = $PSScriptRoot
    if (-not $here -and $MyInvocation.MyCommand.Path) {
        $here = Split-Path -Parent $MyInvocation.MyCommand.Path
    }
    if (-not $here) {
        throw 'Unable to resolve Frameworks root (PSScriptRoot empty).'
    }
    return [System.IO.Path]::GetFullPath((Join-Path $here '..'))
}

function Get-FrameworksComponents {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )

    $Root = $Root.TrimEnd('\')

    return @(
        [pscustomobject]@{
            Id = '1'; Key = 'java'; Name = 'Java'
            SetupSelectable = $true
            Vars = @{ JAVA_HOME = "$Root\Runtimes\Java\current" }
            Paths = @("$Root\Runtimes\Java\current\bin")
            Detect = "$Root\Runtimes\Java\current\bin\java.exe"
            Download = 'https://www.azul.com/downloads/?package=jdk#zulu'
            ExtractTo = "$Root\Runtimes\Java\jdk-21"
        },
        [pscustomobject]@{
            Id = '2'; Key = 'node'; Name = 'Node.js / npm'
            SetupSelectable = $true
            Vars = @{
                NODE_HOME = "$Root\Runtimes\Node\current"
                npm_config_cache = "$Root\Caches\npm"
            }
            Paths = @(
                "$Root\Runtimes\Node\current",
                "$Root\Runtimes\Node\current\node_modules\npm\bin"
            )
            Detect = "$Root\Runtimes\Node\current\node.exe"
            Download = 'https://nodejs.org/en/download/'
            ExtractTo = "$Root\Runtimes\Node"
        },
        [pscustomobject]@{
            Id = '3'; Key = 'gradle'; Name = 'Gradle'
            SetupSelectable = $true
            Vars = @{
                GRADLE_HOME = "$Root\BuildTools\Gradle\current"
                GRADLE_USER_HOME = "$Root\Caches\Gradle"
            }
            Paths = @("$Root\BuildTools\Gradle\current\bin")
            Detect = "$Root\BuildTools\Gradle\current\bin\gradle.bat"
            Download = 'https://gradle.org/releases/'
            ExtractTo = "$Root\BuildTools\Gradle\gradle-8.14.5"
        },
        [pscustomobject]@{
            Id = '4'; Key = 'maven'; Name = 'Maven'
            SetupSelectable = $true
            Vars = @{
                MAVEN_HOME = "$Root\BuildTools\Maven\current"
                MAVEN_OPTS = "-Dmaven.repo.local=$Root\Caches\Maven\repository"
            }
            Paths = @("$Root\BuildTools\Maven\current\bin")
            Detect = "$Root\BuildTools\Maven\current\bin\mvn.cmd"
            Download = 'https://maven.apache.org/download.cgi'
            ExtractTo = "$Root\BuildTools\Maven\apache-maven-3.9.11"
        },
        [pscustomobject]@{
            Id = '5'; Key = 'android'; Name = 'Android SDK'
            SetupSelectable = $true
            Vars = @{
                ANDROID_HOME = "$Root\Platforms\Android\Sdk"
                ANDROID_SDK_ROOT = "$Root\Platforms\Android\Sdk"
                ANDROID_USER_HOME = "$Root\Caches\Android"
            }
            Paths = @(
                "$Root\Platforms\Android\Sdk\platform-tools",
                "$Root\Platforms\Android\Sdk\cmdline-tools\latest\bin"
            )
            Detect = "$Root\Platforms\Android\Sdk\platform-tools\adb.exe"
            Download = 'https://developer.android.com/studio#command-line-tools-only'
            ExtractTo = "$Root\Platforms\Android\Sdk\cmdline-tools\latest"
        },
        [pscustomobject]@{
            Id = '6'; Key = 'rust'; Name = 'Rust / Cargo'
            SetupSelectable = $true
            Vars = @{
                CARGO_HOME = "$Root\Toolchains\Rust\cargo-home"
                CARGO_TARGET_DIR = "$Root\Caches\Rust\target"
                RUST_HOME = "$Root\Toolchains\Rust\current"
            }
            Paths = @("$Root\Toolchains\Rust\current\bin")
            Detect = "$Root\Toolchains\Rust\current\bin\rustc.exe"
            Download = 'https://www.rust-lang.org/tools/install'
            ExtractTo = "$Root\Toolchains\Rust\standalone"
        },
        [pscustomobject]@{
            Id = '7'; Key = 'python'; Name = 'Python / pip cache'
            SetupSelectable = $true
            Vars = @{
                PIP_CACHE_DIR = "$Root\Caches\pip"
                PIP_INDEX_URL = 'https://pypi.tuna.tsinghua.edu.cn/simple'
            }
            Paths = @(
                "$Root\Runtimes\Python\current",
                "$Root\Runtimes\Python\current\Scripts"
            )
            Detect = "$Root\Runtimes\Python\current\python.exe"
            Download = 'https://www.python.org/downloads/windows/'
            ExtractTo = "$Root\Runtimes\Python\python-3.12"
        },
        [pscustomobject]@{
            Id = '8'; Key = 'c'; Name = 'C / GCC'
            SetupSelectable = $true
            Vars = @{}
            Paths = @("$Root\Toolchains\C\mingw64\bin")
            Detect = "$Root\Toolchains\C\mingw64\bin\gcc.exe"
            Download = 'https://winlibs.com/'
            ExtractTo = "$Root\Toolchains\C\mingw64"
        },
        [pscustomobject]@{
            Id = '9'; Key = 'acpi'; Name = 'ACPI / iasl'
            SetupSelectable = $true
            Vars = @{}
            Paths = @("$Root\Toolchains\ACPI\iasl")
            Detect = "$Root\Toolchains\ACPI\iasl\iasl.exe"
            Download = 'https://www.intel.com/content/www/us/en/developer/topic-technology/open/acpica/download.html'
            ExtractTo = "$Root\Toolchains\ACPI\iasl"
        },
        [pscustomobject]@{
            Id = '10'; Key = 'mysql'; Name = 'MySQL'
            SetupSelectable = $true
            Vars = @{ MYSQL_HOME = "$Root\Databases\Sql\mysql\current" }
            Paths = @("$Root\Databases\Sql\mysql\current\bin")
            Detect = "$Root\Databases\Sql\mysql\current\bin\mysql.exe"
            Download = 'https://dev.mysql.com/downloads/mysql/'
            ExtractTo = "$Root\Databases\Sql\mysql"
        },
        [pscustomobject]@{
            Id = '11'; Key = 'ghidra'; Name = 'Ghidra'
            SetupSelectable = $false
            Vars = @{}
            Paths = @("$Root\ReverseTools\Ghidra")
            Detect = "$Root\ReverseTools\Ghidra\ghidraRun.bat"
            Download = 'https://ghidra-sre.org/'
            ExtractTo = "$Root\ReverseTools\Ghidra"
        }
    )
}

function Get-FrameworksSetupComponents {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )
    return @(Get-FrameworksComponents -Root $Root | Where-Object { $_.SetupSelectable -ne $false })
}

function Add-FrameworksPath {
    param([string]$Path)
    if (-not $Path) { return }
    if (-not (Test-Path -LiteralPath $Path)) { return }

    $parts = @()
    if ($env:PATH) {
        $parts = @($env:PATH -split ';' | Where-Object { $_ })
    }
    foreach ($p in $parts) {
        if ([string]::Equals($p, $Path, [System.StringComparison]::OrdinalIgnoreCase)) {
            return
        }
    }
    if ($env:PATH) {
        $env:PATH = "$Path;$env:PATH"
    } else {
        $env:PATH = $Path
    }
}

function Set-FrameworksSessionEnvironment {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )

    $Root = $Root.TrimEnd('\')
    $env:FRAMEWORKS_HOME = $Root

    $components = Get-FrameworksComponents -Root $Root
    foreach ($component in $components) {
        foreach ($key in @($component.Vars.Keys)) {
            Set-Item -Path "Env:$key" -Value ([string]$component.Vars[$key])
        }
        foreach ($path in $component.Paths) {
            Add-FrameworksPath -Path $path
        }
    }
}
