@echo off
rem Central CMD environment definition. Call from another batch file or CMD session.
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
set "PIP_INDEX_URL=https://pypi.tuna.tsinghua.edu.cn/simple"
set "npm_config_cache=%FRAMEWORKS_HOME%\Caches\npm"
set "npm_config_registry=https://registry.npmmirror.com"
set "MYSQL_HOME=%FRAMEWORKS_HOME%\Databases\Sql\mysql\current"

if /I not "%FRAMEWORKS_ENV_LOADED%"=="%FRAMEWORKS_HOME%" set "PATH=%ANDROID_HOME%\platform-tools;%ANDROID_HOME%\cmdline-tools\latest\bin;%JAVA_HOME%\bin;%GRADLE_HOME%\bin;%MAVEN_HOME%\bin;%NODE_HOME%;%FRAMEWORKS_HOME%\Runtimes\Python\current;%FRAMEWORKS_HOME%\Runtimes\Python\current\Scripts;%RUST_HOME%\bin;%FRAMEWORKS_HOME%\Toolchains\C\mingw64\bin;%FRAMEWORKS_HOME%\Toolchains\ACPI\iasl;%MYSQL_HOME%\bin;%FRAMEWORKS_HOME%\ReverseTools\Ghidra;%PATH%"
set "FRAMEWORKS_ENV_LOADED=%FRAMEWORKS_HOME%"
