# D:\Frameworks 目录文件结构说明

更新时间: 2026-08-12

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
├─ Apps              # GreenDev Manager 等本地管理应用
├─ BuildTools        # Gradle、Maven
├─ Caches            # 各工具缓存
├─ Config            # 权威配置、GUI 组件清单、版本固定与环境备份
├─ Databases         # MySQL 等
├─ docs              # 使用说明与设计文档
├─ downloads         # 离线安装包
├─ Logs              # GreenDev Manager 持久操作日志
├─ Exports           # 诊断包和跨盘符 Profile（运行时生成）
├─ Platforms         # Android SDK
├─ ReverseTools      # Ghidra、ILSpy、API Monitor 等
├─ Runtimes          # Java、Node、Python
├─ Releases          # GUI 便携包、安装程序与校验清单
├─ Scripts           # 环境脚本实现
├─ Toolchains        # C、Rust、ACPI
├─ auto-setup.bat
├─ cleanup.bat
├─ env-setup.bat
├─ setup_dev_env.bat
└─ sync-config.bat
```

## 管理应用 `Apps`

```text
Apps
└─ GreenDevManager
   ├─ src                 # React/TypeScript 界面
   ├─ src-tauri           # Rust/Tauri 本地后端
   ├─ GreenDevManager.exe # 本地构建产物（不进 Git）
   ├─ WebView2Loader.dll  # GNU 构建运行库（不进 Git）
   ├─ build.ps1
   ├─ integration-test.ps1
   ├─ e2e-test.ps1
   ├─ release.ps1
   ├─ CHANGELOG.md
   └─ run.bat
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
   ├─ current          # junction -> python-3.13
   └─ python-3.13      # 根目录内 Python 实体
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
   └─ cargo-home
```

入口:

```text
RUST_HOME     Toolchains\Rust\current
CARGO_HOME    Toolchains\Rust\cargo-home
C/GCC         Toolchains\C\mingw64
ACPI          Toolchains\ACPI\iasl
```

## 构建工具 `BuildTools`

```text
BuildTools
├─ Gradle
│  ├─ current          # -> gradle-8.14.5
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
      ├─ platform-tools
      ├─ platforms
      └─ sources          # 仅保留最近版本；emulator 已移除，需要时 sdkmanager 安装
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
      ├─ resources         # 当前 data / logs / files
      ├─ backups           # 经校验的历史数据压缩归档
      └─ my.ini            # basedir/datadir 均指向本目录
```

启动: `mysqld --defaults-file=D:\Frameworks\Databases\Sql\mysql\my.ini`（未注册 Windows 服务）。

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
├─ pip
└─ Rust\target         # CARGO_TARGET_DIR
```

`cleanup.bat` 可清理缓存子集；**不会**清理 Android SDK 本体、MySQL、工具安装目录、Maven 本地仓库、Config 与 Gradle 配置文件。

## 配置 `Config`（权威副本）

```text
Config
├─ cargo
├─ config-backups       # GUI 保存前的配置快照
├─ env-backups
├─ greendev
│  ├─ components.json   # 组件版本、来源、依赖、健康检查与 SHA-256
│  ├─ install-settings.json # 代理与镜像覆盖
│  ├─ update-policies.json  # stable / lts 更新策略
│  ├─ package-lock.json # 本地导入包的 SHA-256（运行时生成）
│  └─ pins.json         # 版本固定
├─ gradle
│  ├─ gradle.properties
│  └─ init.d\cn-mirrors.init.gradle
├─ maven
│  └─ settings.xml     # 阿里云镜像 + 本地仓库路径
├─ mysql
│  └─ my.ini.template  # 根据 FRAMEWORKS_HOME 生成活动配置
├─ npm
└─ pip
```

修改配置时优先改 `Config\`，再同步到生效路径（见 `USAGE.md`）。

## 下载 `downloads`

```text
downloads
├─ packages            # 清单下载/导入缓存
└─ rust                # Rust 离线安装包
```

## 脚本 `Scripts`

```text
Scripts
├─ frameworks-common.ps1   # PowerShell 组件清单 / 路径解析 / 会话环境
├─ frameworks-env.cmd      # CMD 会话环境集中入口
├─ auto-setup.ps1
├─ cleanup.ps1
├─ dev-shell.bat
├─ dev-shell.ps1
├─ env-setup-output.ps1
├─ manage-component.ps1   # 组件计划、导入、安装与更新
├─ manage-component-batch.ps1 # 按依赖顺序批量更新
├─ refresh-update-catalog.ps1 # 刷新官方版本目录
├─ setup-dev-env.ps1
└─ sync-config.ps1
```


顶层入口:

```text
auto-setup.bat     -> Scripts\auto-setup.ps1
cleanup.bat        -> Scripts\cleanup.ps1
env-setup.bat      -> 设置环境 + Scripts\env-setup-output.ps1
setup_dev_env.bat  -> Scripts\setup-dev-env.ps1
sync-config.bat    -> Scripts\sync-config.ps1
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
MYSQL_HOME          Databases\Sql\mysql\current
PIP_CACHE_DIR       Caches\pip
PIP_INDEX_URL       https://pypi.tuna.tsinghua.edu.cn/simple
```

## PATH 入口（按组件，存在才写入）

```text
Runtimes\Java\current\bin
Runtimes\Node\current
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
