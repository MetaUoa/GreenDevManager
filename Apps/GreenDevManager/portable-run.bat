@echo off
setlocal EnableExtensions
if not "%~1"=="" set "FRAMEWORKS_HOME=%~f1"
if not exist "%~dp0GreenDevManager.exe" (
  echo GreenDevManager.exe is missing.
  exit /b 1
)
if defined FRAMEWORKS_HOME if exist "%FRAMEWORKS_HOME%\env-setup.bat" goto launch
for %%I in ("%~dp0" "%~dp0.." "%~dp0..\.." "%~dp0..\..\.." "%~dp0..\..\..\..") do (
  if exist "%%~fI\env-setup.bat" set "FRAMEWORKS_HOME=%%~fI"
)
if not defined FRAMEWORKS_HOME (
  echo Frameworks root was not found.
  echo Usage: run.bat D:\Frameworks
  exit /b 2
)
:launch
start "" "%~dp0GreenDevManager.exe"
