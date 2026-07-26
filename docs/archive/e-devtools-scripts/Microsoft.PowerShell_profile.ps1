function tools_update {
    <#
	Microsoft.PowerShell_profile.ps1
    .SYNOPSIS
    更新常用 AI CLI 工具，并修复 Scoop(nodejs-lts) 下相关启动脚本的 WSL 路径兼容问题。

    .DESCRIPTION
    1) 临时清空 HTTP/HTTPS 代理环境变量，避免 pnpm 拉包走代理导致异常
    2) 使用 pnpm 全局安装/更新一组 AI CLI 工具
    3) 扫描并修补 D:\Green\Scoop\persist\nodejs-lts\bin 目录下指定脚本：
       将脚本中的 `exec node  "C:/Users...` 替换为 `exec node  "/mnt/c/Users...`
       以便在 WSL/跨环境调用时不因路径格式不兼容而失败
    #>

    # ========== 配置区 ==========
    $nodeBinDir = "D:\Green\Scoop\persist\nodejs-lts\bin"

    # 需要更新的全局 npm 包（pnpm -g）
    $packages = @(
        "snow-ai",
        "@iflow-ai/iflow-cli@latest",
        "@google/gemini-cli@latest",
        "@qwen-code/qwen-code@latest",
        "@anthropic-ai/claude-code@latest",
        "@openai/codex@latest"
    )

    # 需要修补的启动脚本文件名（位于 $nodeBinDir 下）
    $binsToPatch = @("claude", "codex", "gemini", "snow", "qwen", "iflow")

    # 替换规则：Windows 用户目录路径 -> WSL 挂载路径
    $from = 'exec node  "C:/Users'
    $to   = 'exec node  "/mnt/c/Users'

    # ========== 1) 禁用代理 ==========
    # 临时清空代理变量，避免 pnpm 下载/鉴权/证书问题
    #$env:HTTP_PROXY  = $null
    #$env:HTTPS_PROXY = $null
	$env:HTTP_PROXY = "http://127.0.0.1:7890"; 
	$env:HTTPS_PROXY = "http://127.0.0.1:7890"	

    # ========== 2) 更新/安装 CLI 工具 ==========
    foreach ($pkg in $packages) {
        Write-Host "[pnpm] install -g $pkg"
        pnpm install -g $pkg
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "[pnpm] 安装失败：$pkg（退出码 $LASTEXITCODE），继续执行后续步骤。"
        }
    }

    # ========== 3) 修补启动脚本 ==========
    foreach ($bin in $binsToPatch) {
        $path = Join-Path $nodeBinDir $bin

        # 文件不存在就跳过（避免报错）
        if (-not (Test-Path $path)) {
            Write-Warning "[patch] 未找到文件：$path，跳过。"
            continue
        }

        # 读取全文（-Raw 让其变成一个字符串，便于整体 replace）
        $content = Get-Content -Path $path -Raw

        # 如果包含目标字符串才进行替换，避免无意义写回
        if ($content -notmatch [regex]::Escape($from)) {
            Write-Host "[patch] 无需修改：$bin"
            continue
        }

        # 替换并写回（UTF-8）
        $newContent = $content -replace [regex]::Escape($from), $to
        Set-Content -Path $path -Value $newContent -Encoding utf8

        Write-Host "[patch] 已修补：$bin"
    }

    Write-Host "完成：工具更新 + 脚本路径修补。"
}

增加如下功能
#WinGet
$progressPreference = 'silentlyContinue'
Install-PackageProvider -Name NuGet -Force | Out-Null
Install-Module -Name Microsoft.WinGet.Client -Force -Repository PSGallery | Out-Null
Write-Host "Using Repair-WinGetPackageManager cmdlet to bootstrap WinGet..."
Repair-WinGetPackageManager -AllUsers

#PowerShell 7
winget install Microsoft.PowerShell