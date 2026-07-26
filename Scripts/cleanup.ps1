param(
    [string]$Lang = "zh",
    [ValidateSet("safe", "normal")]
    [string]$Level = "normal",
    [switch]$Apply,
    [switch]$IncludeDownloads,
    [switch]$IncludeWrapper,
    [switch]$ShowEmpty
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot 'frameworks-common.ps1')

$root = Resolve-FrameworksRoot
$isEn = $Lang -match "^(en|english)$"

function Say([string]$Zh, [string]$En) {
    if ($isEn) { Write-Host $En } else { Write-Host $Zh }
}

function Get-PathSizeBytes([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return 0 }
    $sum = (Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) { return 0 }
    return [int64]$sum
}

function Format-Size([int64]$Bytes) {
    if ($Bytes -ge 1GB) { return "{0:N2} GB" -f ($Bytes / 1GB) }
    if ($Bytes -ge 1MB) { return "{0:N2} MB" -f ($Bytes / 1MB) }
    if ($Bytes -ge 1KB) { return "{0:N2} KB" -f ($Bytes / 1KB) }
    return "$Bytes B"
}

function Assert-SafeDeletePath([string]$Path) {
    $rootFull = [System.IO.Path]::GetFullPath($root).TrimEnd('\') + '\'
    $targetFull = [System.IO.Path]::GetFullPath($Path)
    if (-not $targetFull.StartsWith($rootFull, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to delete outside root: $targetFull"
    }
    if ($targetFull.TrimEnd('\') -ieq $rootFull.TrimEnd('\')) {
        throw "Refusing to delete root: $targetFull"
    }

    $blocked = @(
        (Join-Path $root 'Runtimes'),
        (Join-Path $root 'BuildTools'),
        (Join-Path $root 'Toolchains'),
        (Join-Path $root 'Platforms\Android\Sdk\build-tools'),
        (Join-Path $root 'Platforms\Android\Sdk\platforms'),
        (Join-Path $root 'Platforms\Android\Sdk\sources'),
        (Join-Path $root 'Platforms\Android\Sdk\emulator'),
        (Join-Path $root 'Platforms\Android\Sdk\platform-tools'),
        (Join-Path $root 'Platforms\Android\Sdk\cmdline-tools'),
        (Join-Path $root 'Databases'),
        (Join-Path $root 'ReverseTools'),
        (Join-Path $root 'Config'),
        (Join-Path $root 'Scripts'),
        (Join-Path $root 'Caches\Gradle\gradle.properties'),
        (Join-Path $root 'Caches\Gradle\init.d'),
        (Join-Path $root 'Caches\Maven\repository')
    )
    foreach ($b in $blocked) {
        if (-not (Test-Path -LiteralPath $b)) { continue }
        $bFull = [System.IO.Path]::GetFullPath($b).TrimEnd('\')
        if ($targetFull.TrimEnd('\') -ieq $bFull) {
            throw "Refusing to delete protected path: $targetFull"
        }
    }
}

function Clear-DirectoryContents([string]$Path) {
    Assert-SafeDeletePath -Path $Path
    Get-ChildItem -LiteralPath $Path -Force -ErrorAction SilentlyContinue | ForEach-Object {
        if ($_.Name -in @('gradle.properties', 'init.d')) { return }
        Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Remove-PathSafe([string]$Path) {
    Assert-SafeDeletePath -Path $Path
    Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
}

# Mode: delete = remove whole path; empty = clear contents only
# Tier: safe / normal
# NOT cleaning: Android SDK body, MySQL, tool installs, Maven local repo,
# Config authoritative files, gradle.properties/init.d
$allCandidates = @(
    [pscustomobject]@{
        Key = 'rust-target'
        Path = (Join-Path $root 'Caches\Rust\target')
        KindZh = 'Rust 构建缓存 (target)'
        KindEn = 'Rust build cache (target)'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'android-temp'
        Path = (Join-Path $root 'Platforms\Android\Sdk\.temp')
        KindZh = 'Android SDK 临时目录'
        KindEn = 'Android SDK temp directory'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'android-download-intermediates'
        Path = (Join-Path $root 'Platforms\Android\Sdk\.downloadIntermediates')
        KindZh = 'Android SDK 下载中间文件'
        KindEn = 'Android SDK download intermediates'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'gradle-tmp'
        Path = (Join-Path $root 'Caches\Gradle\.tmp')
        KindZh = 'Gradle 临时目录'
        KindEn = 'Gradle temp directory'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'gradle-daemon'
        Path = (Join-Path $root 'Caches\Gradle\daemon')
        KindZh = 'Gradle 守护进程缓存'
        KindEn = 'Gradle daemon cache'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'npm-logs'
        Path = (Join-Path $root 'Caches\npm\_logs')
        KindZh = 'npm 日志'
        KindEn = 'npm logs'
        Tier = 'safe'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'npm-cache'
        Path = (Join-Path $root 'Caches\npm\_cacache')
        KindZh = 'npm 包缓存'
        KindEn = 'npm package cache'
        Tier = 'normal'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'pip-cache'
        Path = (Join-Path $root 'Caches\pip')
        KindZh = 'pip 缓存内容'
        KindEn = 'pip cache contents'
        Tier = 'normal'
        Mode = 'empty'
    },
    [pscustomobject]@{
        Key = 'gradle-caches'
        Path = (Join-Path $root 'Caches\Gradle\caches')
        KindZh = 'Gradle 依赖/构建缓存'
        KindEn = 'Gradle dependency/build caches'
        Tier = 'normal'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'gradle-native'
        Path = (Join-Path $root 'Caches\Gradle\native')
        KindZh = 'Gradle native 缓存'
        KindEn = 'Gradle native cache'
        Tier = 'normal'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'gradle-notifications'
        Path = (Join-Path $root 'Caches\Gradle\notifications')
        KindZh = 'Gradle 通知缓存'
        KindEn = 'Gradle notifications cache'
        Tier = 'normal'
        Mode = 'delete'
    },
    [pscustomobject]@{
        Key = 'gradle-kotlin-profile'
        Path = (Join-Path $root 'Caches\Gradle\kotlin-profile')
        KindZh = 'Gradle Kotlin profile'
        KindEn = 'Gradle Kotlin profile'
        Tier = 'normal'
        Mode = 'delete'
    }
)

$tierRank = @{ safe = 0; normal = 1 }
$levelRank = $tierRank[$Level]
$candidates = @()
foreach ($c in $allCandidates) {
    if ($tierRank[$c.Tier] -le $levelRank) {
        $candidates += $c
    }
}

if ($IncludeDownloads) {
    $candidates += [pscustomobject]@{
        Key = 'rust-downloads'
        Path = (Join-Path $root 'downloads\rust')
        KindZh = 'Rust 离线安装包'
        KindEn = 'Rust offline installers'
        Tier = 'extra'
        Mode = 'empty'
    }
}

if ($IncludeWrapper) {
    $candidates += [pscustomobject]@{
        Key = 'gradle-wrapper-dists'
        Path = (Join-Path $root 'Caches\Gradle\wrapper\dists')
        KindZh = 'Gradle Wrapper 发行包缓存 (可选)'
        KindEn = 'Gradle Wrapper distribution cache (optional)'
        Tier = 'extra'
        Mode = 'delete'
    }
}

Write-Host ""
Say "=== Frameworks 清理检查 ===" "=== Frameworks cleanup check ==="
Say ("根目录: {0}" -f $root) ("Root: {0}" -f $root)
Say ("级别: {0}  (safe=临时/日志; normal=含 npm/pip/gradle 缓存)" -f $Level) ("Level: {0}  (safe=temp/logs; normal=includes npm/pip/gradle caches)" -f $Level)
if ($Apply) {
    Say "模式: 执行删除" "Mode: apply deletions"
} else {
    Say "模式: 预览，不删除任何文件" "Mode: dry-run, no files will be deleted"
}
if (-not $ShowEmpty) {
    Say "空目录: 已隐藏 (加 -ShowEmpty / cleanup.bat showempty 可显示)" "Empty paths: hidden (pass showempty to list them)"
}
Say "不会清理: Android SDK 本体、MySQL、工具安装目录、Maven 本地仓库、Config、gradle.properties/init.d" "Protected: Android SDK body, MySQL, tool installs, Maven local repo, Config, gradle.properties/init.d"
Write-Host ""

$total = 0L
$hiddenEmpty = 0
$existing = @()

foreach ($candidate in $candidates) {
    if (-not (Test-Path -LiteralPath $candidate.Path)) {
        continue
    }

    $size = Get-PathSizeBytes -Path $candidate.Path
    if ($size -le 0 -and -not $ShowEmpty) {
        $hiddenEmpty++
        continue
    }

    $total += $size
    $kind = if ($isEn) { $candidate.KindEn } else { $candidate.KindZh }
    $modeLabel = if ($candidate.Mode -eq 'empty') {
        if ($isEn) { 'empty-contents' } else { '清空内容' }
    } else {
        if ($isEn) { 'delete' } else { '删除目录' }
    }
    $existing += [pscustomobject]@{
        Key = $candidate.Key
        Type = $kind
        Size = (Format-Size $size)
        SizeBytes = $size
        Mode = $modeLabel
        ModeRaw = $candidate.Mode
        Path = $candidate.Path
        Tier = $candidate.Tier
    }
}

if ($existing.Count -eq 0) {
    if ($hiddenEmpty -gt 0) {
        Say ("仅有 {0} 个空目录候选，无实质可释放空间。" -f $hiddenEmpty) ("Only {0} empty candidate(s); nothing substantial to reclaim." -f $hiddenEmpty)
    } else {
        Say "没有发现可清理项。" "No cleanup candidates found."
    }
    exit 0
}

$existing | Sort-Object SizeBytes -Descending | Format-Table Key, Type, Size, Mode, Path -AutoSize
Write-Host ""
if ($hiddenEmpty -gt 0) {
    Say ("另有 {0} 个空目录未列出。" -f $hiddenEmpty) ("{0} empty path(s) omitted." -f $hiddenEmpty)
}
Say ("预计可释放: {0}" -f (Format-Size $total)) ("Estimated reclaimable space: {0}" -f (Format-Size $total))

if (-not $Apply) {
    Write-Host ""
    Say "确认后可运行:" "To delete after review, run:"
    Say "  cleanup.bat apply" "  cleanup.bat apply"
    Say "  cleanup.bat apply safe" "  cleanup.bat apply safe"
    Say "  cleanup.bat apply normal" "  cleanup.bat apply normal"
    Say "  cleanup.bat apply downloads" "  cleanup.bat apply downloads"
    Say "  cleanup.bat apply wrapper     # 额外清理 Gradle Wrapper dists" "  cleanup.bat apply wrapper     # also Gradle Wrapper dists"
    Say "  cleanup.bat showempty         # 预览时显示空目录" "  cleanup.bat showempty         # list empty paths too"
    exit 0
}

Write-Host ""
foreach ($item in $existing) {
    Assert-SafeDeletePath -Path $item.Path
    if ($item.ModeRaw -eq 'empty') {
        Say ("清空: {0}" -f $item.Path) ("Emptying: {0}" -f $item.Path)
        Clear-DirectoryContents -Path $item.Path
    } else {
        Say ("删除: {0}" -f $item.Path) ("Deleting: {0}" -f $item.Path)
        Remove-PathSafe -Path $item.Path
    }
}

Write-Host ""
Say "清理完成。" "Cleanup complete."
Say "提示: Gradle 配置 (gradle.properties / init.d) 已保留。" "Note: Gradle config (gradle.properties / init.d) kept."
