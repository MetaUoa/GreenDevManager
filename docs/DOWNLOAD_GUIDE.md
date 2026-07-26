# 绿色版工具下载与安装路径

更新时间: 2026-07-13

安装后请创建/更新对应 `current` junction，再运行 `env-setup.bat` 验证。  
日常使用说明见 [`USAGE.md`](USAGE.md)。

---

## Java JDK

推荐: Azul Zulu OpenJDK（zip 绿色版）

| 版本 | 来源 |
|------|------|
| JDK 21 / 17 / 8 | https://www.azul.com/downloads/?package=jdk#zulu |

```text
解压到: D:\Frameworks\Runtimes\Java\jdk-21   (或 jdk-17 / jdk-8)
入口:   D:\Frameworks\Runtimes\Java\current  -> 实际 JDK 根目录
```

---

## Gradle

| 版本 | 下载 | 用途 |
|------|------|------|
| **8.14.5（推荐默认）** | https://services.gradle.org/distributions/gradle-8.14.5-bin.zip | AGP 8.x 主力 / 本机 current |
| 8.5 | https://services.gradle.org/distributions/gradle-8.5-bin.zip | 中等旧项目兼容 |
| 9.4.1 | https://services.gradle.org/distributions/gradle-9.4.1-bin.zip | AGP 9.2 / 新工程 |

国内发行包镜像: https://mirrors.cloud.tencent.com/gradle/

```text
解压到: D:\Frameworks\BuildTools\Gradle\gradle-8.14.5
入口:   D:\Frameworks\BuildTools\Gradle\current  -> gradle-8.14.5
```

---

## Maven

| 版本 | 下载 |
|------|------|
| 3.9.x | https://maven.apache.org/download.cgi |

```text
解压到: D:\Frameworks\BuildTools\Maven\apache-maven-3.9.11
入口:   D:\Frameworks\BuildTools\Maven\current
配置:   D:\Frameworks\Config\maven\settings.xml  (阿里云镜像)
本地仓: D:\Frameworks\Caches\Maven\repository
```

---

## Node.js

官网: https://nodejs.org/en/download/ （Windows Binary `.zip`）  
国内: https://npmmirror.com/mirrors/node/

```text
解压到: D:\Frameworks\Runtimes\Node\node-vXX.x.x-win-x64
入口:   D:\Frameworks\Runtimes\Node\current
缓存:   D:\Frameworks\Caches\npm
```

当前环境示例: `node-v24.18.0-win-x64`。

---

## Python

官网: https://www.python.org/downloads/windows/  
推荐: Windows embeddable package (64-bit)

```text
解压到: D:\Frameworks\Runtimes\Python\python-3.12
入口:   D:\Frameworks\Runtimes\Python\current  -> python-3.12
缓存:   D:\Frameworks\Caches\pip
```

---

## Go（可选，目录约定）

官网: https://go.dev/dl/

```text
解压到: D:\Frameworks\Runtimes\Go\go-1.xx
入口:   D:\Frameworks\Runtimes\Go\current
```

当前仓库尚未安装 Go 时，脚本不会强制要求。

---

## MinGW-w64 (GCC)

推荐: https://winlibs.com/

```text
解压到: D:\Frameworks\Toolchains\C\mingw64
PATH:   ...\mingw64\bin
```

---

## Rust

官网: https://www.rust-lang.org/tools/install  
离线包可放: `D:\Frameworks\downloads\rust`

```text
实体:   D:\Frameworks\Toolchains\Rust\standalone
入口:   D:\Frameworks\Toolchains\Rust\current  -> standalone
RUSTUP_HOME:  Toolchains\Rust\rustup-home
CARGO_HOME:   Toolchains\Rust\cargo-home
target 缓存:  Caches\Rust\target
```

详见 [`RUST_ENV.md`](RUST_ENV.md)。

---

## Android SDK

Command Line Tools: https://developer.android.com/studio#command-line-tools-only

```text
cmdline-tools: D:\Frameworks\Platforms\Android\Sdk\cmdline-tools\latest
SDK 根:        D:\Frameworks\Platforms\Android\Sdk
```

```bat
sdkmanager "platforms;android-35" "build-tools;35.0.0" "platform-tools"
```

---

## MySQL

官网: https://dev.mysql.com/downloads/mysql/ （zip 归档）

```text
解压到: D:\Frameworks\Databases\Sql\mysql\mysql-8.4.7
入口:   D:\Frameworks\Databases\Sql\mysql\current
```

---

## 安装后步骤

```bat
1. 创建/更新 current junction（或依赖已有 current）
2. D:\Frameworks\auto-setup.bat          检测
3. call D:\Frameworks\Scripts\dev-shell.bat
4. D:\Frameworks\env-setup.bat           验证
5. 可选永久环境: D:\Frameworks\setup_dev_env.bat
```

Maven/Gradle 国内镜像与缓存说明见 [`USAGE.md`](USAGE.md)。
