@echo off
setlocal EnableExtensions
set "LANG_ARG=%~1"
set "DEEP_ARG="
if not defined LANG_ARG if defined FRAMEWORKS_LANG set "LANG_ARG=%FRAMEWORKS_LANG%"
if not defined LANG_ARG set "LANG_ARG=zh"
if /I "%~2"=="deep" set "DEEP_ARG=-Deep"

call "%~dp0Scripts\frameworks-env.cmd"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Scripts\env-setup-output.ps1" -Lang "%LANG_ARG%" %DEEP_ARG%
if /I not "%FRAMEWORKS_NOPAUSE%"=="1" pause
