# D:\Frameworks 目录文件结构说明

更新时间: 2026-07-13

## 总体定位

`D:\Frameworks` 是本机绿色开发环境根目录，用于集中管理运行时、工具链、构建工具、平台 SDK、数据库工具、逆向工具、缓存、配置、脚本和离线安装包。

核心原则:

1. 可执行工具集中放在固定分类目录。
2. 缓存和配置从工具目录中分离，统一放入 `Caches` 和 `Config`。
3. 版本切换优先通过 `current` 入口完成，脚本不硬编码具体版本号。
4. 临时解压目录、测试目录和无效入口不长期保留。

## 顶层结构

```text
D:\Frameworks
├─ BuildTools        # Gradle、Maven
├─ Caches            # 各工具缓存
├─ Config            # 权威配置（Maven/Gradle 镜像等）
├─ Databases         # MySQL 等
├─ docs              # 使用说明与设计文档
├─ downloads         # 离线安装包
├─ Platforms         # Android SDK
├─ ReverseTools      # Ghidra、ILSpy、API Monitor 等
├─ Runtimes          # Java、Node、Python
├─ Scripts           # 环境脚本实现
├─ Toolchains        # C、Rust、ACPI
├─ auto-setup.bat
├─ cleanup.bat
├─ env-setup.bat
└─ setup_dev_env.bat
```

## 运行时 `Runtimes`

```text
Runtimes
├─ Java
│  ├─ current          # junction -> jdk-21 内实际 JDK 根
│  ├─ jdk-8
│  ├─ jdk-17
│  └─ jdk-21
├─ Node
│  ├─ current          # junction -> node-v24.x
│  └─ node-v24.18.0-win-x64
└─ Python
   └─ current          # 预留，安装后创建
```

入口:

```text
JAVA_HOME   Runtimes\Java\current
NODE_HOME   Runtimes\Node\current
Python PATH Runtimes\Python\current  (+ Scripts)
```

## 工具链 `Toolchains`

```text
Toolchains
├─ ACPI\iasl
├─ C\mingw64
└─ Rust
   ├─ current          # junction -> standalone
   ├─ standalone       # rustc/cargo 实体
   ├─ cargo-home
   └─ rustup-home
```

入口:

```text
RUST_HOME     Toolchains\Rust\current
RUSTUP_HOME   Toolchains\Rust\rustup-home
CARGO_HOME    Toolchains\Rust\cargo-home
C/GCC         Toolchains\C\mingw64
ACPI          Toolchains\ACPI\iasl
```

## 构建工具 `BuildTools`

```text
BuildTools
├─ Gradle
│  ├─ current          # -> gradle-8.14.5
│  ├─ gradle-7.6.6
│  ├─ gradle-8.5
│  ├─ gradle-8.9
│  ├─ gradle-8.11.1
│  ├─ gradle-8.13
│  ├─ gradle-8.14.5
│  └─ gradle-9.4.1
└─ Maven
   ├─ current          # -> apache-maven-3.9.11
   ├─ apache-maven-3.8.9
   └─ apache-maven-3.9.11
```

## 平台 `Platforms`

```text
Platforms
└─ Android
   └─ Sdk
      ├─ build-tools
      ├─ cmake
      ├─ cmdline-tools
      ├─ emulator
      ├─ platform-tools
      ├─ platforms
      └─ sources
```

```text
ANDROID_HOME / ANDROID_SDK_ROOT   Platforms\Android\Sdk
```

## 数据库 `Databases`

```text
Databases
└─ Sql
   └─ mysql
      ├─ current           # junction -> mysql-8.4.7
      ├─ mysql-8.4.7
      ├─ resources
      └─ my.ini
```

```text
MYSQL_HOME   Databases\Sql\mysql\current
```

## 逆向工具 `ReverseTools`

```text
ReverseTools
├─ Android             # jadx、apktool 等
├─ API Monitor
├─ Ghidra              # 根目录即安装体；current 仅为 ghidraRun 包装
├─ ILSpy
└─ RWEverything
```

PATH 使用 `ReverseTools\Ghidra`（不要依赖 `Ghidra\current` 作为工具根）。

## 缓存 `Caches`

```text
Caches
├─ Android
├─ Gradle              # GRADLE_USER_HOME（含 gradle.properties / init.d 生效副本）
├─ Maven\repository    # Maven 本地仓库
├─ npm
├─ NuGet
├─ pip
└─ Rust\target         # CARGO_TARGET_DIR
```

`cleanup.bat` 可清理缓存子集；**不会**清理 Android SDK 本体、MySQL、工具安装目录、Maven 本地仓库、Config 与 Gradle 配置文件。

## 配置 `Config`（权威副本）

```text
Config
├─ cargo
├─ env-backups
├─ gradle
│  ├─ gradle.properties
│  └─ init.d\cn-mirrors.init.gradle
├─ maven
│  └─ settings.xml     # 阿里云镜像 + 本地仓库路径
├─ npm
└─ pip
```

修改配置时优先改 `Config\`，再同步到生效路径（见 `USAGE.md`）。

## 下载 `downloads`

```text
downloads
└─ rust                # 离线安装包
```

## 脚本 `Scripts`

```text
Scripts
├─ frameworks-common.ps1   # 组件清单 / 路径解析 / 会话环境（单源）
├─ auto-setup.ps1
├─ cleanup.ps1
├─ dev-shell.bat
├─ dev-shell.ps1
├─ env-setup-output.ps1
└─ setup-dev-env.ps1
```


顶层入口:

```text
auto-setup.bat     -> Scripts\auto-setup.ps1
cleanup.bat        -> Scripts\cleanup.ps1
env-setup.bat      -> 设置环境 + Scripts\env-setup-output.ps1
setup_dev_env.bat  -> Scripts\setup-dev-env.ps1
dev-shell          -> Scripts\dev-shell.bat / .ps1
```

约定:

1. `.bat` 尽量保持 ASCII 包装。
2. 含中文的 `.ps1` 使用 UTF-8 BOM。
3. 默认中文；`en` 或 `FRAMEWORKS_LANG=en` 切英文。
4. 路径统一走 `current` / `MYSQL_HOME` / `RUST_HOME`，不写死版本号。

## 环境变量对应关系

```text
FRAMEWORKS_HOME     D:\Frameworks
JAVA_HOME           Runtimes\Java\current
NODE_HOME           Runtimes\Node\current
npm_config_cache    Caches\npm
GRADLE_HOME         BuildTools\Gradle\current
GRADLE_USER_HOME    Caches\Gradle
MAVEN_HOME          BuildTools\Maven\current
MAVEN_OPTS          -Dmaven.repo.local=...\Caches\Maven\repository
ANDROID_HOME        Platforms\Android\Sdk
ANDROID_SDK_ROOT    同 ANDROID_HOME
ANDROID_USER_HOME   Caches\Android
CARGO_HOME          Toolchains\Rust\cargo-home
CARGO_TARGET_DIR    Caches\Rust\target
RUST_HOME           Toolchains\Rust\current
RUSTUP_HOME         Toolchains\Rust\rustup-home
MYSQL_HOME          Databases\Sql\mysql\current
PIP_CACHE_DIR       Caches\pip
```

## PATH 入口（按组件，存在才写入）

```text
Runtimes\Java\current\bin
Runtimes\Node\current
Runtimes\Node\current\node_modules\npm\bin
BuildTools\Gradle\current\bin
BuildTools\Maven\current\bin
Platforms\Android\Sdk\platform-tools
Platforms\Android\Sdk\cmdline-tools\latest\bin
Toolchains\Rust\current\bin
Runtimes\Python\current
Runtimes\Python\current\Scripts
Toolchains\C\mingw64\bin
Toolchains\ACPI\iasl
Databases\Sql\mysql\current\bin
ReverseTools\Ghidra
```

## 新增 / 升级规则

1. 运行时 → `Runtimes`；工具链 → `Toolchains`；构建 → `BuildTools`；平台 SDK → `Platforms`；库 → `Databases`；逆向 → `ReverseTools`。
2. 下载包装 `downloads`；缓存进 `Caches`；配置进 `Config`。
3. 新版本与旧版本并存于版本子目录，验证后再改 `current`。
4. 改 `current` 后跑 `env-setup.bat` 验证。
5. 清理前确认不是 `current` 指向的实体目录。
