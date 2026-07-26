#requires -Version 7.0
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function tools_uninstall_final_nowinget_v3 {

  # ==========================
  # Config
  # ==========================
  $UseProxy = $true
  $ProxyUrl = "http://127.0.0.1:7890"

  # 是否清理用户环境变量（PNPM_HOME / PATH 中的 pnpm 目录）
  # 默认 false：不动注册表/用户环境（更安全）
  #$CleanupUserEnv = $false
  $CleanupUserEnv = $true  

  $pnpmPackages = @(
    "snow-ai",
    "@iflow-ai/iflow-cli",
    "@google/gemini-cli",
    "@qwen-code/qwen-code",
    "@anthropic-ai/claude-code",
    "@openai/codex"
  )

  $bins = @("claude", "codex", "gemini", "snow", "qwen", "iflow")

  $patched  = 'exec node  "/mnt/c/Users'
  $original = 'exec node  "C:/Users'

  if ($UseProxy) { $env:HTTP_PROXY=$ProxyUrl; $env:HTTPS_PROXY=$ProxyUrl }

  # ==========================
  # Helpers
  # ==========================
  function Write-Section([string]$t) {
    Write-Host ""
    Write-Host ("=" * 72)
    Write-Host $t
    Write-Host ("=" * 72)
  }

  function Get-PnpmHome {
    $pnpmHome = $env:PNPM_HOME
    if (-not $pnpmHome) { $pnpmHome = Join-Path $env:LOCALAPPDATA "pnpm" }
    return $pnpmHome
  }

  function Get-PnpmGlobalImporterDir {
    # 你日志里是 ...\global\5（pnpm 大版本目录）
    # 这里用 glob 找第一个存在的版本目录
    $globalRoot = Join-Path $env:LOCALAPPDATA "pnpm\global"
    if (-not (Test-Path -LiteralPath $globalRoot)) { return $null }

    $candidates = Get-ChildItem -LiteralPath $globalRoot -Directory -ErrorAction SilentlyContinue |
                  Sort-Object Name -Descending
    foreach ($d in $candidates) {
      return $d.FullName
    }
    return $null
  }

  function Test-PnpmGlobalManifestOK {
    $importer = Get-PnpmGlobalImporterDir
    if (-not $importer) { return $false }
    return (Test-Path -LiteralPath (Join-Path $importer "package.json")) `
        -or (Test-Path -LiteralPath (Join-Path $importer "package.yaml")) `
        -or (Test-Path -LiteralPath (Join-Path $importer "package.json5"))
  }

  function Unpatch-FileIfNeeded([string]$path, [string]$name) {
    try {
      if (-not (Test-Path -LiteralPath $path)) { return $false }
      $c = Get-Content -Raw -LiteralPath $path
      if ($c -notmatch [regex]::Escape($patched)) { return $false }
      Set-Content -LiteralPath $path -Value ($c -replace [regex]::Escape($patched), $original) -Encoding utf8
      Write-Host "[unpatch] reverted: $name -> $path"
      return $true
    } catch {
      Write-Warning "[unpatch] failed: $name -> $path : $($_.Exception.Message)"
      return $false
    }
  }

  function Unpatch-BestEffort([string[]]$Names) {
    $pnpmHome = Get-PnpmHome
    $exts = @(".ps1", ".cmd")
    foreach ($n in $Names) {
      $did = $false

      $cmd = Get-Command $n -ErrorAction SilentlyContinue
      if ($cmd -and $cmd.Source -and (Test-Path -LiteralPath $cmd.Source)) {
        $did = Unpatch-FileIfNeeded -path $cmd.Source -name $n
      } else {
        Write-Host "[unpatch] command not found: $n (will try PNPM_HOME shims)"
      }

      foreach ($ext in $exts) {
        $p = Join-Path $pnpmHome ($n + $ext)
        if (Unpatch-FileIfNeeded -path $p -name $n) { $did = $true }
      }

      if (-not $did) { Write-Host "[unpatch] no change needed: $n" }
    }
  }

  function Remove-PnpmShims([string[]]$Names) {
    $pnpmHome = Get-PnpmHome
    if (-not (Test-Path -LiteralPath $pnpmHome)) {
      Write-Host "[clean] PNPM_HOME not found: $pnpmHome"
      return
    }
    Write-Host "[clean] PNPM_HOME=$pnpmHome"
    $exts = @("", ".cmd", ".ps1", ".exe")
    foreach ($n in $Names) {
      foreach ($ext in $exts) {
        $f = Join-Path $pnpmHome ($n + $ext)
        if (Test-Path -LiteralPath $f) {
          try { Remove-Item -Force -LiteralPath $f; Write-Host "[clean] removed $f" }
          catch { Write-Warning "[clean] failed remove $f : $($_.Exception.Message)" }
        }
      }
    }
  }

  function Pnpm-UninstallGlobals-Safe([string[]]$Pkgs) {
    if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
      Write-Warning "[pnpm] pnpm not found; skip pnpm uninstall"
      return
    }

    if (-not (Test-PnpmGlobalManifestOK)) {
      Write-Host "[pnpm] global manifest missing -> skip pnpm uninstall -g (avoid error spam)"
      return
    }

    foreach ($pkg in $Pkgs) {
      try {
        Write-Host "[pnpm] uninstall -g $pkg"
        & pnpm uninstall -g $pkg | Out-Host
      } catch {
        Write-Warning "[pnpm] uninstall failed: $pkg : $($_.Exception.Message)"
      }
    }
  }

  function Pnpm-DeepClean {
    if (Get-Command pnpm -ErrorAction SilentlyContinue) {
      try { Write-Host "[pnpm] store prune"; & pnpm store prune | Out-Host }
      catch { Write-Warning "[pnpm] store prune failed: $($_.Exception.Message)" }
    }

    $storeDir  = Join-Path $env:LOCALAPPDATA "pnpm\store"
    $globalDir = Join-Path $env:LOCALAPPDATA "pnpm\global"

    if (Test-Path -LiteralPath $storeDir) {
      try { Remove-Item -Recurse -Force -LiteralPath $storeDir; Write-Host "[clean] removed store dir: $storeDir" }
      catch { Write-Warning "[clean] failed remove store dir: $storeDir : $($_.Exception.Message)" }
    } else { Write-Host "[clean] store dir not found: $storeDir" }

    if (Test-Path -LiteralPath $globalDir) {
      try { Remove-Item -Recurse -Force -LiteralPath $globalDir; Write-Host "[clean] removed global dir: $globalDir" }
      catch { Write-Warning "[clean] failed remove global dir: $globalDir : $($_.Exception.Message)" }
    } else { Write-Host "[clean] global dir not found: $globalDir" }

    Remove-PnpmShims -Names $bins
  }

  function Cleanup-UserEnvIfEnabled {
    if (-not $CleanupUserEnv) { return }

    Write-Section "4) Cleanup user environment variables (optional)"

    try {
      $pnpmHome = Get-PnpmHome

      # 清 PNPM_HOME（用户级）
      $curPNPM_HOME = [Environment]::GetEnvironmentVariable("PNPM_HOME", "User")
      if ($curPNPM_HOME) {
        [Environment]::SetEnvironmentVariable("PNPM_HOME", $null, "User")
        Write-Host "[env] removed User PNPM_HOME"
      } else {
        Write-Host "[env] User PNPM_HOME not set"
      }

      # 清 Path 里的 pnpmHome（用户级）
      $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
      if ($userPath) {
        $parts = $userPath -split ';' | Where-Object { $_ -and ($_ -ne $pnpmHome) }
        $newPath = ($parts -join ';')
        if ($newPath -ne $userPath) {
          [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
          Write-Host "[env] removed PNPM_HOME from User Path"
        } else {
          Write-Host "[env] PNPM_HOME not present in User Path"
        }
      } else {
        Write-Host "[env] User Path empty/unset"
      }

      Write-Host "[env] NOTE: 需要新开终端/重新登录后生效"
    } catch {
      Write-Warning "[env] cleanup failed: $($_.Exception.Message)"
    }
  }

  function Verify-After {
    Write-Section "Verify (after reopen terminal recommended)"
    Write-Host "[check] where.exe commands (should be empty):"
    try { & where.exe @bins 2>$null | Out-Host } catch {}

    if (Get-Command pnpm -ErrorAction SilentlyContinue) {
      Write-Host ""
      Write-Host "[check] pnpm -g list --depth 0"
      try { & pnpm -g list --depth 0 | Out-Host } catch {}
    }
  }

  # ==========================
  # Run
  # ==========================
  Write-Host ("[env] PS={0} User={1}" -f $PSVersionTable.PSVersion, $env:USERNAME)

  Write-Section "1) Revert WSL path patch (best effort)"
  Unpatch-BestEffort -Names $bins

  Write-Section "2) Uninstall global AI CLI tools (pnpm -g, safe)"
  Pnpm-UninstallGlobals-Safe -Pkgs $pnpmPackages

  Write-Section "3) Deep clean pnpm store/global + PNPM_HOME shims"
  Pnpm-DeepClean

  Cleanup-UserEnvIfEnabled
  Verify-After

  Write-Host ""
  Write-Host "Done. 建议：关闭所有终端窗口后重新打开，再跑 verify。"
}

function tools_uninstall_final_nowinget_v3_run {
  $log = Join-Path $env:TEMP ("tools_uninstall_final_nowinget_v3_" + (Get-Date -Format "yyyyMMdd_HHmmss") + ".log")
  try {
    Start-Transcript -Path $log -Force | Out-Null
    tools_uninstall_final_nowinget_v3
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

tools_uninstall_final_nowinget_v3_run
