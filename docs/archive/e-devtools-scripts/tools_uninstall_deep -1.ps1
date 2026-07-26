#requires -Version 7.0
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function tools_uninstall_deep_nowinget {
  $UseProxy = $true
  $ProxyUrl = "http://127.0.0.1:7890"

  # pnpm 全局安装的
  $pnpmPackages = @(
    "snow-ai",
    "@iflow-ai/iflow-cli",
    "@google/gemini-cli",
    "@qwen-code/qwen-code",
    "@anthropic-ai/claude-code",
    "@openai/codex"
  )

  # 脚本 patch 过的命令名
  $binsToPatch = @("claude","codex","gemini","snow","qwen","iflow")

  # patch 替换规则（卸载时做反向替换）
  $patched  = 'exec node  "/mnt/c/Users'
  $original = 'exec node  "C:/Users'

  if ($UseProxy) { $env:HTTP_PROXY=$ProxyUrl; $env:HTTPS_PROXY=$ProxyUrl }

  function Write-Section([string]$t){ Write-Host ""; Write-Host ("="*72); Write-Host $t; Write-Host ("="*72) }

  function Unpatch([string]$name){
    $cmd = Get-Command $name -ErrorAction SilentlyContinue
    if (-not $cmd) { Write-Host "[unpatch] command not found: $name"; return }
    if (-not $cmd.Source -or -not (Test-Path -LiteralPath $cmd.Source)) { Write-Warning "[unpatch] not file-based: $name"; return }
    try {
      $p = $cmd.Source
      $c = Get-Content -Raw -LiteralPath $p
      if ($c -notmatch [regex]::Escape($patched)) { Write-Host "[unpatch] no change needed: $name -> $p"; return }
      Set-Content -LiteralPath $p -Value ($c -replace [regex]::Escape($patched), $original) -Encoding utf8
      Write-Host "[unpatch] reverted: $name -> $p"
    } catch { Write-Warning "[unpatch] failed: $name : $($_.Exception.Message)" }
  }

  function Pnpm-UninstallGlobals([string[]]$pkgs){
    if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
      Write-Warning "[pnpm] pnpm not found; nothing to uninstall"
      return
    }
    foreach ($pkg in $pkgs) {
      try {
        Write-Host "[pnpm] uninstall -g $pkg"
        & pnpm uninstall -g $pkg | Out-Host
      } catch {
        Write-Warning "[pnpm] uninstall failed: $pkg : $($_.Exception.Message)"
      }
    }
  }

  function Pnpm-DeepClean {
    if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) { return }

    Write-Host "[pnpm] store prune"
    try { & pnpm store prune | Out-Host } catch { Write-Warning "[pnpm] store prune failed: $($_.Exception.Message)" }

    Write-Host "[pnpm] cache clean --all (best effort)"
    try { & pnpm cache clean --all | Out-Host } catch { Write-Warning "[pnpm] cache clean failed: $($_.Exception.Message)" }

    # 清理 PNPM_HOME 里残留的 shim/脚本（只删我们这批命令）
    $pnpmHome = $env:PNPM_HOME
    if (-not $pnpmHome) { $pnpmHome = Join-Path $env:LOCALAPPDATA "pnpm" }
    if (Test-Path -LiteralPath $pnpmHome) {
      Write-Host "[clean] PNPM_HOME=$pnpmHome"
      foreach ($n in @("claude","codex","gemini","snow","qwen","iflow")) {
        foreach ($ext in @("",".cmd",".ps1",".exe")) {
          $f = Join-Path $pnpmHome ($n + $ext)
          if (Test-Path -LiteralPath $f) {
            try { Remove-Item -Force -LiteralPath $f; Write-Host "[clean] removed $f" }
            catch { Write-Warning "[clean] failed remove $f : $($_.Exception.Message)" }
          }
        }
      }
    }
  }

  Write-Section "1) Revert WSL path patch (best effort)"
  foreach ($b in $binsToPatch) { Unpatch $b }

  Write-Section "2) Uninstall global AI CLI tools (pnpm -g)"
  Pnpm-UninstallGlobals $pnpmPackages

  Write-Section "3) Deep clean pnpm store/cache + PNPM_HOME shims"
  Pnpm-DeepClean

  Write-Host ""
  Write-Host "Done. 关闭并重新打开终端，确保命令缓存/PATH 刷新。"
}

function tools_uninstall_deep_nowinget_run {
  $log = Join-Path $env:TEMP ("tools_uninstall_deep_nowinget_" + (Get-Date -Format "yyyyMMdd_HHmmss") + ".log")
  try {
    Start-Transcript -Path $log -Force | Out-Null
    tools_uninstall_deep_nowinget
    Stop-Transcript | Out-Null
    Write-Host ("Log: {0}" -f $log)
  } catch {
    try { Stop-Transcript | Out-Null } catch {}
    Write-Host ("ERROR: {0}" -f $_.Exception.Message)
    Write-Host ("Log: {0}" -f $log)
  } finally {
    Read-Host "Press Enter to close"
  }
}

tools_uninstall_deep_nowinget_run
