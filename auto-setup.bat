@echo off
setlocal
set "LANG_ARG=%~1"
if not defined LANG_ARG if defined FRAMEWORKS_LANG set "LANG_ARG=%FRAMEWORKS_LANG%"
if not defined LANG_ARG set "LANG_ARG=zh"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Scripts\auto-setup.ps1" -Lang "%LANG_ARG%"
if /I not "%FRAMEWORKS_NOPAUSE%"=="1" pause
