@echo off
setlocal EnableExtensions
set "FW_LANG=zh"
set "FW_SEL="

rem Optional language: zh | en | english
if /I "%~1"=="zh" goto :set_lang_zh
if /I "%~1"=="en" goto :set_lang_en
if /I "%~1"=="english" goto :set_lang_en
if defined FRAMEWORKS_LANG set "FW_LANG=%FRAMEWORKS_LANG%"
goto :collect_sel

:set_lang_zh
set "FW_LANG=zh"
shift
goto :collect_sel

:set_lang_en
set "FW_LANG=en"
shift
goto :collect_sel

:collect_sel
if "%~1"=="" goto :run
set "FW_SEL=%~1"
shift

:collect_sel_loop
if "%~1"=="" goto :run
set "FW_SEL=%FW_SEL%,%~1"
shift
goto :collect_sel_loop

:run
if not "%FW_SEL%"=="" goto :run_with_sel
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Scripts\setup-dev-env.ps1" -Lang "%FW_LANG%"
goto :finish

:run_with_sel
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Scripts\setup-dev-env.ps1" -Lang "%FW_LANG%" -Selection "%FW_SEL%"
goto :finish

:finish
if /I "%FRAMEWORKS_NOPAUSE%"=="1" goto :eof
echo.
pause
goto :eof
