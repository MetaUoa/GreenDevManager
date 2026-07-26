function tools_update {

    # ========= 配置区 =========
    $ProxyMode = "disable"  # disable / enable
    $ProxyUrl  = "http://127.0.0.1:7890"

    $Packages = @(
        "snow-ai",
        "@iflow-ai/iflow-cli@latest",
        "@google/gemini-cli@latest",
        "@qwen-code/qwen-code@latest",
        "@anthropic-ai/claude-code@latest",
        "@openai/codex@latest"
    )

    $BinsToPatch = @("claude", "codex", "gemini", "snow", "qwen", "iflow")
    $From = 'exec node  "C:/Users'
    $To   = 'exec node  "/mnt/c/Users'
    $ScoopInstallRetries = 3

    # ========= 日志 =========
    $log = Join-Path $env:TEMP ("tools_update_" + (Get-Date -Format "yyyyMMdd_HHmmss") + ".log")

    function _log([string]$msg) {
        $line = "[{0}] {1}" -f (Get-Date -Format "HH:mm:ss"), $msg
        Add-Content -Path $log -Value $line
        Write-Host $msg
    }
    function _warn([string]$msg) {
        $line = "[{0}] WARNING {1}" -f (Get-Date -Format "HH:mm:ss"), $msg
        Add-Content -Path $log -Value $line
        Write-Warning $msg
    }
    function Add-ToPathIfMissing([string]$dir) {
        if (-not $dir) { return }
        if (-not (Test-Path -LiteralPath $dir)) { return }
        if ($env:Path -notlike "*$dir*") { $env:Path = "$dir;$env:Path" }
    }

    $oldProgress = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'

    try {
        _log "Log: $log"

        # 1) 代理
        if ($ProxyMode -eq "disable") {
            $env:HTTP_PROXY  = $null
            $env:HTTPS_PROXY = $null
            _log "[proxy] disabled"
        } else {
            $env:HTTP_PROXY  = $ProxyUrl
            $env:HTTPS_PROXY = $ProxyUrl
            _log "[proxy] enabled: $ProxyUrl"
        }

        # 2) 强行安装 Scoop（如果没有）
        function Ensure-Scoop {
            if (Get-Command scoop -ErrorAction SilentlyContinue) { return $true }

            foreach ($shim in @(
                (Join-Path $env:USERPROFILE "scoop\shims"),
                (Join-Path $env:LOCALAPPDATA "scoop\shims"),
                (Join-Path $env:USERPROFILE "AppData\Local\scoop\shims")
            )) {
                $scoopCmd = Join-Path $shim "scoop.cmd"
                if (Test-Path -LiteralPath $scoopCmd) {
                    Add-ToPathIfMissing $shim
                    if (Get-Command scoop -ErrorAction SilentlyContinue) {
                        _log "[scoop] found: $scoopCmd"
                        return $true
                    }
                }
            }

            _log "[scoop] not found -> force install"
            try { Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned -Force | Out-Null } catch {}
            try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch {}

            for ($i = 1; $i -le $ScoopInstallRetries; $i++) {
                try {
                    _log "[scoop] install attempt $i/$ScoopInstallRetries ..."
                    irm get.scoop.sh | iex
                } catch {
                    _warn ("[scoop] install attempt failed: " + $_.Exception.Message)
                    Start-Sleep -Seconds 2
                }

                Add-ToPathIfMissing (Join-Path $env:USERPROFILE "scoop\shims")
                Add-ToPathIfMissing (Join-Path $env:LOCALAPPDATA "scoop\shims")

                if (Get-Command scoop -ErrorAction SilentlyContinue) {
                    _log "[scoop] installed OK"
                    return $true
                }
            }

            return $false
        }

        if (-not (Ensure-Scoop)) {
            _warn "[scoop] FAILED. Check network/proxy. Abort."
            _log "Log: $log"
            return
        }

        # 3) 计算 Scoop root / persist bin（自动适配）
        function Get-ScoopRoot {
            try {
                $rp = (& scoop config root_path 2>$null)
                if ($rp) {
                    $rp = $rp.ToString().Trim()
                    if ($rp) { return $rp }
                }
            } catch {}
            return (Join-Path $env:USERPROFILE "scoop")
        }

        $ScoopRoot  = Get-ScoopRoot
        $NodeBinDir = Join-Path $ScoopRoot "persist\nodejs-lts\bin"
        _log "[scoop] root: $ScoopRoot"
        _log "[patch] target bin dir: $NodeBinDir"

        # 4) 安装 nodejs-lts / pnpm（缺失则装）
        if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
            _log "[scoop] node not found -> scoop install nodejs-lts"
            scoop install nodejs-lts
        }
        if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
            _log "[scoop] pnpm not found -> scoop install pnpm"
            scoop install pnpm
        }

        if (-not (Get-Command node -ErrorAction SilentlyContinue)) { _warn "[node] still not found. Reopen terminal then rerun."; return }
        if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) { _warn "[pnpm] still not found. Reopen terminal then rerun."; return }

        _log ("[env] node=" + (node -v))
        _log ("[env] pnpm=" + (pnpm -v))

        # 5) pnpm setup（解决 global bin dir）
        _log "[pnpm] setup"
        pnpm setup

        if (-not $env:PNPM_HOME) { $env:PNPM_HOME = Join-Path $env:LOCALAPPDATA "pnpm" }
        if (-not (Test-Path -LiteralPath $env:PNPM_HOME)) {
            New-Item -ItemType Directory -Path $env:PNPM_HOME -Force | Out-Null
        }
        Add-ToPathIfMissing $env:PNPM_HOME
        pnpm config set global-bin-dir $env:PNPM_HOME | Out-Null
        _log ("[pnpm-env] PNPM_HOME=" + $env:PNPM_HOME)

        # 6) 安装/更新 CLI
        foreach ($pkg in $Packages) {
            _log "[pnpm] install -g $pkg"
            pnpm install -g $pkg
            if ($LASTEXITCODE -ne 0) {
                _warn "[pnpm] failed: $pkg (exit code $LASTEXITCODE)"
            }
        }

        # 7) patch（Scoop persist）
        if (-not (Test-Path $NodeBinDir)) {
            _warn "[patch] Node bin dir not found. Maybe you are using PNPM_HOME shims; patch not needed."
            _log "Done. Log: $log"
            return
        }

        foreach ($bin in $BinsToPatch) {
            $path = Join-Path $NodeBinDir $bin
            if (-not (Test-Path $path)) {
                _warn "[patch] missing: $path (skip)"
                continue
            }

            try {
                $content = Get-Content -Path $path -Raw
                if ($content -notmatch [regex]::Escape($From)) {
                    _log "[patch] no change needed: $bin"
                    continue
                }

                $newContent = $content -replace [regex]::Escape($From), $To
                Set-Content -Path $path -Value $newContent -Encoding utf8
                _log "[patch] patched: $bin"
            } catch {
                _warn ("[patch] failed: " + $bin + " : " + $_.Exception.Message)
            }
        }

        _log "Done. Log: $log"
    }
    finally {
        $ProgressPreference = $oldProgress
    }
}
