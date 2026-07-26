param(
    [string]$Lang = "zh",
    [string]$Selection = ""
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
$isEn = $Lang -match "^(en|english)$"

function Say([string]$Zh, [string]$En) {
    if ($isEn) { Write-Host $En } else { Write-Host $Zh }
}

function Backup-UserEnvironment {
    $backupDir = Join-Path $root 'Config\env-backups'
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null

    $variables = [ordered]@{}
    $envVars = [Environment]::GetEnvironmentVariables("User")
    foreach ($key in ($envVars.Keys | Sort-Object)) {
        $variables[$key] = [string]$envVars[$key]
    }

    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backupPath = Join-Path $backupDir "user-env-$timestamp.json"
    $backup = [ordered]@{
        createdAt = (Get-Date).ToString("o")
        scope = "User"
        root = $root
        computerName = $env:COMPUTERNAME
        userName = $env:USERNAME
        variables = $variables
    }

    $backup | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $backupPath -Encoding UTF8
    return $backupPath
}

function Select-ComponentsInteractive {
    param(
        [Parameter(Mandatory = $true)]
        [object[]]$Items
    )

    function Write-ChoiceLine([string]$Text) {
        $width = 80
        try {
            if ([Console]::WindowWidth -gt 1) {
                $width = [Math]::Max(20, [Console]::WindowWidth - 1)
            }
        } catch {
            $width = 80
        }
        if ($Text.Length -gt $width) {
            $Text = $Text.Substring(0, $width)
        }
        Write-Host ($Text.PadRight($width))
    }

    $selectedFlags = @($false) * $Items.Count
    $cursor = 0
    $top = [Console]::CursorTop

    while ($true) {
        try { [Console]::SetCursorPosition(0, $top) } catch { }

        for ($i = 0; $i -lt $Items.Count; $i++) {
            $prefix = if ($i -eq $cursor) { ">" } else { " " }
            $mark = if ($selectedFlags[$i]) { "x" } else { " " }
            $line = "{0} [{1}] {2,2}. {3} [{4}]" -f $prefix, $mark, $Items[$i].Id, $Items[$i].Name, $Items[$i].Key
            Write-ChoiceLine $line
        }
        Write-ChoiceLine ""
        if ($isEn) {
            Write-ChoiceLine "Up/Down: move  Space: toggle  A: all/none  Enter: confirm  Esc: cancel"
        } else {
            Write-ChoiceLine "Up/Down move | Space toggle | A all/none | Enter confirm | Esc cancel"
        }

        $key = [Console]::ReadKey($true)
        switch ($key.Key) {
            ([ConsoleKey]::UpArrow) {
                if ($cursor -gt 0) { $cursor-- } else { $cursor = $Items.Count - 1 }
            }
            ([ConsoleKey]::DownArrow) {
                if ($cursor -lt ($Items.Count - 1)) { $cursor++ } else { $cursor = 0 }
            }
            ([ConsoleKey]::Spacebar) {
                $selectedFlags[$cursor] = -not [bool]$selectedFlags[$cursor]
            }
            ([ConsoleKey]::A) {
                $allSelected = $true
                foreach ($flag in $selectedFlags) {
                    if (-not $flag) { $allSelected = $false; break }
                }
                for ($i = 0; $i -lt $selectedFlags.Count; $i++) {
                    $selectedFlags[$i] = -not $allSelected
                }
            }
            ([ConsoleKey]::Enter) {
                $result = @()
                for ($i = 0; $i -lt $Items.Count; $i++) {
                    if ($selectedFlags[$i]) { $result += $Items[$i] }
                }
                Write-Host ""
                return $result
            }
            ([ConsoleKey]::Escape) {
                Write-Host ""
                return @()
            }
        }
    }
}

$components = Get-FrameworksSetupComponents -Root $root

Write-Host ""
Say "=== 配置 Frameworks 永久用户环境变量 ===" "=== Configure persistent Frameworks user environment ==="
Say ("根目录: {0}" -f $root) ("Root: {0}" -f $root)
Write-Host ""

if (-not $Selection) {
    if ([Console]::IsInputRedirected) {
        Say "检测到非交互输入且未指定选择，未写入环境变量。" "Non-interactive input detected with no selection. Nothing was written."
        exit 0
    } else {
        Say "请选择要配置的组件。默认全部未选中。" "Select components to configure. None are selected by default."
        Write-Host ""
        $selected = @(Select-ComponentsInteractive -Items $components)
        if ($null -eq $selected) { $selected = @() }
        if ($selected.Count -eq 1 -and $null -ne $selected[0] -and $selected[0] -is [System.Array]) {
            $selected = @($selected[0])
        }
    }
} else {
    Say "使用命令行选择: $Selection" "Using command line selection: $Selection"
    $Selection = $Selection.Trim()
    if ($Selection -eq "0" -or $Selection -match "^(cancel|quit|exit)$") {
        Say "已取消，没有写入环境变量。" "Cancelled. No environment variables were written."
        exit 0
    }

    if (-not $Selection -or $Selection -match "^(all|a|\*)$" -or $Selection -eq "全部") {
        $selected = @($components)
    } else {
        $tokens = @($Selection -split "[,;，、\s]+" | Where-Object { $_ })
        $selected = @()
        $seen = @{}
        foreach ($token in $tokens) {
            $match = $components | Where-Object {
                $_.Id -eq $token -or $_.Key -ieq $token -or $_.Name -ieq $token
            } | Select-Object -First 1
            if ($match) {
                if (-not $seen.ContainsKey($match.Key)) {
                    $seen[$match.Key] = $true
                    $selected += $match
                }
            } else {
                Say "忽略未知选择: $token" "Ignoring unknown selection: $token"
            }
        }
    }
}

if ($selected.Count -eq 0) {
    Say "没有选择有效组件，未写入环境变量。" "No valid components selected. Nothing was written."
    exit 0
}

$backupPath = Backup-UserEnvironment
Say "已备份当前用户环境变量: $backupPath" "Current user environment variables backed up: $backupPath"

[Environment]::SetEnvironmentVariable("FRAMEWORKS_HOME", $root, "User")

$pathsToAdd = @()
foreach ($component in $selected) {
    Say "配置: $($component.Name)" "Configuring: $($component.Name)"
    foreach ($item in $component.Vars.GetEnumerator()) {
        [Environment]::SetEnvironmentVariable($item.Key, $item.Value, "User")
        Write-Host ("  {0}={1}" -f $item.Key, $item.Value)
    }
    foreach ($path in $component.Paths) {
        if (Test-Path -LiteralPath $path) {
            $pathsToAdd += $path
            Write-Host ("  PATH += {0}" -f $path)
        } else {
            Say "  跳过不存在的 PATH: $path" "  Skipped missing PATH: $path"
        }
    }
}

$existing = @([Environment]::GetEnvironmentVariable("PATH", "User") -split ";" | Where-Object { $_ })
foreach ($path in $pathsToAdd) {
    $found = $false
    foreach ($e in $existing) {
        if ([string]::Equals($e, $path, [System.StringComparison]::OrdinalIgnoreCase)) {
            $found = $true
            break
        }
    }
    if (-not $found) { $existing += $path }
}
[Environment]::SetEnvironmentVariable("PATH", ($existing -join ";"), "User")

Write-Host ""
Say "已写入所选用户环境变量。" "Selected user environment variables have been written."
Say "请重新打开终端让 PATH 生效。" "Reopen your terminal for PATH changes to take effect."
Say "可运行 env-setup.bat 验证当前结构。" "Run env-setup.bat to verify the current structure."
