@echo off
rem Load Frameworks development environment for this terminal only.
for %%I in ("%~dp0..") do set "FRAMEWORKS_HOME=%%~fI"

set "ANDROID_HOME=%FRAMEWORKS_HOME%\Platforms\Android\Sdk"
set "ANDROID_SDK_ROOT=%ANDROID_HOME%"
set "ANDROID_USER_HOME=%FRAMEWORKS_HOME%\Caches\Android"

set "JAVA_HOME=%FRAMEWORKS_HOME%\Runtimes\Java\current"
set "NODE_HOME=%FRAMEWORKS_HOME%\Runtimes\Node\current"
set "GRADLE_HOME=%FRAMEWORKS_HOME%\BuildTools\Gradle\current"
set "GRADLE_USER_HOME=%FRAMEWORKS_HOME%\Caches\Gradle"
set "MAVEN_HOME=%FRAMEWORKS_HOME%\BuildTools\Maven\current"
set "MAVEN_OPTS=-Dmaven.repo.local=%FRAMEWORKS_HOME%\Caches\Maven\repository"

set "CARGO_HOME=%FRAMEWORKS_HOME%\Toolchains\Rust\cargo-home"
set "CARGO_TARGET_DIR=%FRAMEWORKS_HOME%\Caches\Rust\target"
set "RUST_HOME=%FRAMEWORKS_HOME%\Toolchains\Rust\current"

set "PIP_CACHE_DIR=%FRAMEWORKS_HOME%\Caches\pip"
set "npm_config_cache=%FRAMEWORKS_HOME%\Caches\npm"
set "MYSQL_HOME=%FRAMEWORKS_HOME%\Databases\Sql\mysql\current"

call :add_path "%ANDROID_HOME%\platform-tools"
call :add_path "%ANDROID_HOME%\cmdline-tools\latest\bin"
call :add_path "%JAVA_HOME%\bin"
call :add_path "%GRADLE_HOME%\bin"
call :add_path "%MAVEN_HOME%\bin"
call :add_path "%NODE_HOME%"
call :add_path "%NODE_HOME%\node_modules\npm\bin"
call :add_path "%FRAMEWORKS_HOME%\Runtimes\Python\current"
call :add_path "%FRAMEWORKS_HOME%\Runtimes\Python\current\Scripts"
call :add_path "%RUST_HOME%\bin"
call :add_path "%FRAMEWORKS_HOME%\Toolchains\C\mingw64\bin"
call :add_path "%FRAMEWORKS_HOME%\Toolchains\ACPI\iasl"
call :add_path "%MYSQL_HOME%\bin"
call :add_path "%FRAMEWORKS_HOME%\ReverseTools\Ghidra"

echo Frameworks environment loaded. FRAMEWORKS_HOME=%FRAMEWORKS_HOME%
goto :eof

:add_path
if exist "%~1" set "PATH=%~1;%PATH%"
goto :eof
