@echo off
setlocal EnableExtensions
set "LANG_ARG="
set "LEVEL_ARG=normal"
set "APPLY_ARG="
set "DOWNLOADS_ARG="
set "WRAPPER_ARG="
set "SHOWEMPTY_ARG="

:parse
if "%~1"=="" goto run
if /I "%~1"=="apply" (
  set "APPLY_ARG=-Apply"
  shift
  goto parse
)
if /I "%~1"=="downloads" (
  set "DOWNLOADS_ARG=-IncludeDownloads"
  shift
  goto parse
)
if /I "%~1"=="wrapper" (
  set "WRAPPER_ARG=-IncludeWrapper"
  shift
  goto parse
)
if /I "%~1"=="showempty" (
  set "SHOWEMPTY_ARG=-ShowEmpty"
  shift
  goto parse
)
if /I "%~1"=="safe" (
  set "LEVEL_ARG=safe"
  shift
  goto parse
)
if /I "%~1"=="normal" (
  set "LEVEL_ARG=normal"
  shift
  goto parse
)
if /I "%~1"=="zh" (
  set "LANG_ARG=zh"
  shift
  goto parse
)
if /I "%~1"=="en" (
  set "LANG_ARG=en"
  shift
  goto parse
)
if /I "%~1"=="english" (
  set "LANG_ARG=en"
  shift
  goto parse
)
if not defined LANG_ARG set "LANG_ARG=%~1"
shift
goto parse

:run
if not defined LANG_ARG if defined FRAMEWORKS_LANG set "LANG_ARG=%FRAMEWORKS_LANG%"
if not defined LANG_ARG set "LANG_ARG=zh"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Scripts\cleanup.ps1" -Lang "%LANG_ARG%" -Level "%LEVEL_ARG%" %APPLY_ARG% %DOWNLOADS_ARG% %WRAPPER_ARG% %SHOWEMPTY_ARG%
if /I not "%FRAMEWORKS_NOPAUSE%"=="1" pause
