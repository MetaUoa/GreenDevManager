@echo off
setlocal EnableExtensions
for %%I in ("%~dp0..\..") do set "FRAMEWORKS_HOME=%%~fI"
if not exist "%~dp0GreenDevManager.exe" (
  echo GreenDevManager.exe is missing. Run build.ps1 first.
  exit /b 1
)
start "" "%~dp0GreenDevManager.exe"
