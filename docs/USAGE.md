# D:\Frameworks 使用说明

更新时间: 2026-08-12

## 目录结构

```text
D:\Frameworks
├─ BuildTools     # Gradle / Maven
├─ Caches         # Gradle / Maven / npm / pip / Rust 等缓存
├─ Config         # 工具配置
├─ Databases      # MySQL / SQL 工具
├─ docs           # 文档
├─ downloads      # 离线安装包
├─ Platforms      # Android SDK
├─ ReverseTools   # Ghidra / ILSpy / API Monitor / RWEverything
├─ Runtimes       # Java / Node / Python
├─ Scripts        # 环境脚本
└─ Toolchains     # ACPI / C / Rust
```

## 常用入口

```text
Java      D:\Frameworks\Runtimes\Java\current
Node      D:\Frameworks\Runtimes\Node\current
Python    D:\Frameworks\Runtimes\Python\current
Gradle    D:\Frameworks\BuildTools\Gradle\current
Maven     D:\Frameworks\BuildTools\Maven\current
Android   D:\Frameworks\Platforms\Android\Sdk
C/GCC     D:\Frameworks\Toolchains\C\mingw64
Rust      D:\Frameworks\Toolchains\Rust\current
ACPI      D:\Frameworks\Toolchains\ACPI\iasl
MySQL     D:\Frameworks\Databases\Sql\mysql\current
Ghidra    D:\Frameworks\ReverseTools\Ghidra
```


## 环境管理入口

### GUI 管理器

启动桌面管理器：

```bat
D:\Frameworks\Apps\GreenDevManager\run.bat
```

GUI 可运行快速组件扫描与后台容量统计、普通/深度 Doctor、一键用户环境配置与备份恢复、配置漂移检测，以及带预览确认的缓存清理。配置中心支持表单/源码编辑、格式与字段校验、差异预览、自动同步和历史备份回滚。

Phase 8–11 增加配置外部修改冲突检测、最近 30 份备份的检索与内容预览、异常任务事务恢复、`current` 自动修复、全局任务中心、磁盘趋势、深色主题和页面记忆。

版本管理会枚举已安装的 Java、Node、Python、Gradle、Maven、Rust 和 MySQL，通过健康检查后切换 `current`，失败时恢复原入口；版本固定记录在 `Config\greendev\pins.json`。该流程不移除已安装版本。

Android SDK 页面使用根目录内的 `sdkmanager.bat` 刷新包目录并执行安装或确认卸载。安装与更新页面读取 `Config\greendev\components.json`，先检查依赖、版本固定和 SHA-256，再通过离线 ZIP/7Z/TAR/MSI 或镜像/代理下载归档到 `downloads\packages`，在暂存目录验证健康文件后原子落位；失败时恢复原 `current`，旧版本继续保留。长任务支持阶段、吞吐、ETA、暂停、继续、取消与失败重试。

“诊断与发布”页面集中显示根目录、脚本、写权限、WebView2、清单和日志状态。集成验证与发布命令：

```powershell
cd D:\Frameworks\Apps\GreenDevManager
.\integration-test.ps1
.\release.ps1
```

发布物位于 `Releases\GreenDevManager\<version>`，包含便携 ZIP、NSIS 安装程序和 `SHA256SUMS.txt`。

升级中心通过“联网刷新”读取 Java 21 LTS、Node LTS、Python Stable、Gradle、Maven、Rust Stable 和 MySQL 8.4 LTS 官方目录。采用候选后，具有官方 SHA-256 的组件可直接进入安装计划；缺少校验值的组件先导入离线归档自动锁定哈希。批量计划会按依赖排序，版本固定仍优先生效。

“清单与插件”页面提供 Schema 2 编辑器、自定义组件模板、依赖/路径校验以及可信目录最小权限策略。“应用更新”页面管理 Stable/Beta/Local 通道，可从远程 Feed 下载发布物，验证发布清单、SHA256 和可选 Authenticode 签名；重启替换前保留旧程序，健康窗口异常自动回退。

“环境档案”页面维护个人或团队 Profile，可生成记录版本、哈希、依赖和 current 入口的 Lock，查看当前机器差异，并导出排除凭据特征的离线介质。供应链清单以 CycloneDX 1.5 JSON 写入 `Exports`。

任务中心支持并发上限、优先级、计划时间、暂停/继续、失败重试和事务时间线。恢复中心按项预览配置、环境、Profile、应用回退点和中断事务。团队与合规页面支持 directory/http/git Profile 仓库、机器组、签名要求、允许主机、字段只读锁定和审计导出。

CLI 与 GUI 使用同一份 Manifest、安装脚本、Profile Lock、事务目录和操作日志：

```bat
greendev.exe list
greendev.exe doctor
greendev.exe plan node
greendev.exe install node
greendev.exe use node Runtimes\Node\node-v24.18.0-win-x64
greendev.exe profile java-backend
greendev.exe lock java-backend
greendev.exe diff java-backend
greendev.exe validate
```

诊断与发布页面可导出不含权威配置正文的诊断 ZIP，也可导出/导入跨盘符 Profile。导入前会备份当前 Config，安装目录及旧组件版本不变。

持久操作日志位于 `Logs\GreenDev\operations.jsonl`，崩溃日志位于 `Logs\GreenDev\crash.log`。配置保存前备份位于 `Config\config-backups`。

源码与构建说明见 `Apps\GreenDevManager\README.md`。

### 命令行加载开发环境

临时加载当前终端环境:

```bat
call D:\Frameworks\Scripts\dev-shell.bat
```

PowerShell:

```powershell
. D:\Frameworks\Scripts\dev-shell.ps1
```

手动检测环境:

```bat
D:\Frameworks\env-setup.bat
```

默认检测不启动工具，因此不会重建 Gradle 等缓存。需要同时执行版本探测时:

```bat
D:\Frameworks\env-setup.bat zh deep
```

同步 `Config` 权威副本到工具实际生效位置:

```bat
D:\Frameworks\sync-config.bat
```

永久写入用户/系统环境变量:

```bat
D:\Frameworks\setup_dev_env.bat
```

这些脚本默认使用中文，也可以直接传参切换英文:

```bat
D:\Frameworks\env-setup.bat zh
D:\Frameworks\env-setup.bat en
D:\Frameworks\setup_dev_env.bat zh
D:\Frameworks\setup_dev_env.bat en
D:\Frameworks\auto-setup.bat zh
D:\Frameworks\auto-setup.bat en
D:\Frameworks\cleanup.bat zh
D:\Frameworks\cleanup.bat en
```

`setup_dev_env.bat` 会让你用复选框选择要写入永久用户环境变量的组件。默认全部未选中:

```text
上下键  移动光标
空格    选中/取消当前项
A       全选/全不选
Enter   确认写入
Esc     取消退出
```

也可以直接通过命令行传入编号或名称，适合脚本化执行:

```bat
D:\Frameworks\setup_dev_env.bat zh
D:\Frameworks\setup_dev_env.bat zh java,node,android
D:\Frameworks\setup_dev_env.bat en 1,3,5
```

逗号列表可以不加引号；脚本会自动合并 CMD 拆开的参数。

可选组件:

```text
1  java
2  node
3  gradle
4  maven
5  android
6  rust
7  python
8  c
9  acpi
10 mysql
```

写入前会自动备份当前用户环境变量，备份位置:

```text
D:\Frameworks\Config\env-backups
```

也可以用环境变量指定默认语言:

```bat
set FRAMEWORKS_LANG=en
```

## 清理缓存

默认预览（不删除），级别 `normal`（含 npm / pip / Gradle 依赖缓存）:

```bat
D:\Frameworks\cleanup.bat
```

仅临时/日志（更安全）:

```bat
D:\Frameworks\cleanup.bat safe
```

确认后执行:

```bat
D:\Frameworks\cleanup.bat apply
D:\Frameworks\cleanup.bat apply safe
D:\Frameworks\cleanup.bat apply normal
```

额外清空离线安装包目录内容（不删 downloads 目录本身）:

```bat
D:\Frameworks\cleanup.bat apply downloads
```

额外清理 Gradle Wrapper 发行包缓存（`Caches\Gradle\wrapper\dists`，默认不包含）:

```bat
D:\Frameworks\cleanup.bat apply wrapper
```

预览时显示空目录:

```bat
D:\Frameworks\cleanup.bat showempty
```

跳过 bat 末尾 pause（自动化）:

```bat
set FRAMEWORKS_NOPAUSE=1
D:\Frameworks\cleanup.bat
```

**不会清理**: Android SDK 本体（platforms/build-tools/sources 等）、MySQL 安装与数据、工具安装目录、Maven 本地仓库、`Config` 与 Gradle 配置文件（`gradle.properties` / `init.d`）。



## 中文显示

推荐在 Windows Terminal、PowerShell 或新版 CMD 中运行。如果手动打开的 CMD 出现乱码，可先执行:

```bat
chcp 65001
```

## 验证命令

```bat
java -version
gradle --version
mvn --version
adb version
node --version
rustc --version
cargo --version
gcc --version
mysql --version
iasl -v
```


## 缓存位置

```text
Gradle    D:\Frameworks\Caches\Gradle
Maven     D:\Frameworks\Caches\Maven\repository
npm       D:\Frameworks\Caches\npm
pip       D:\Frameworks\Caches\pip
Rust      D:\Frameworks\Caches\Rust
Android   D:\Frameworks\Caches\Android
```

## Maven / Gradle 国内镜像

权威配置（修改后运行 `sync-config.bat`）:

```text
Maven   D:\Frameworks\Config\maven\settings.xml
Gradle  D:\Frameworks\Config\gradle\gradle.properties
        D:\Frameworks\Config\gradle\init.d\cn-mirrors.init.gradle
Cargo   D:\Frameworks\Config\cargo\config.toml
MySQL   D:\Frameworks\Config\mysql\my.ini.template
```

生效路径:

```text
Maven   %MAVEN_HOME%\conf\settings.xml
        本地仓库: D:\Frameworks\Caches\Maven\repository
Gradle  %GRADLE_USER_HOME%\gradle.properties
        %GRADLE_USER_HOME%\init.d\cn-mirrors.init.gradle
        （GRADLE_USER_HOME = D:\Frameworks\Caches\Gradle）
```

镜像源: 阿里云 `public` / `google` / `gradle-plugin` / `spring` / `snapshots`。

验证:

```bat
call D:\Frameworks\Scripts\dev-shell.bat
mvn -version
mvn help:effective-settings
gradle --version
```

IDEA 使用本机 Maven 时，Settings → Build → Maven → User settings file 可指向:

```text
D:\Frameworks\Config\maven\settings.xml
```

Gradle Wrapper 发行包下载慢时，可改项目 `gradle/wrapper/gradle-wrapper.properties` 的 `distributionUrl` 为腾讯云等镜像，例如:

```text
https://mirrors.cloud.tencent.com/gradle/gradle-8.14.5-bin.zip
```

## 离线安装包

Rust 离线安装文件保留在:

```text
D:\Frameworks\downloads\rust
```

如果不需要离线重装 Rust，可以清理该目录中的安装包。

## 维护规则

1. 新运行时放入 `Runtimes`，例如 Java、Node、Python。
2. 编译器和工具链放入 `Toolchains`，例如 C、Rust、ACPI。
3. 构建工具放入 `BuildTools`，例如 Gradle、Maven。
4. 平台 SDK 放入 `Platforms`，例如 Android SDK。
5. 缓存统一放入 `Caches`，不要散落到工具目录。
6. 下载包和安装包放入 `downloads`。
7. 升级工具时优先更新对应的 `current` 入口，而不是改脚本硬编码路径。
8. PowerShell 组件清单的权威源是 `Scripts\frameworks-common.ps1`，CMD 会话环境入口是 `Scripts\frameworks-env.cmd`；新增组件时需同步维护两者。
9. 脚本通过所在目录推导 `FRAMEWORKS_HOME`（也可用环境变量覆盖），不依赖写死盘符。


## 重要文档

```text
D:\Frameworks\README.md
D:\Frameworks\docs\README.md
D:\Frameworks\docs\DIRECTORY_STRUCTURE.md
D:\Frameworks\docs\DOWNLOAD_GUIDE.md
D:\Frameworks\docs\RUST_ENV.md
```
# 应用更新源

GreenDev Manager 的“应用更新”页面有两种来源：

- `Feed URL` 留空：刷新 `Releases\GreenDevManager\update-feed.json`，属于本地刷新。
- 填写完整 `https://` 或 `http://` Feed 地址：执行联网刷新并缓存到 `Caches\GreenDevManager\app-update-feed.json`。正式版默认使用 `https://github.com/MetaUoa/GreenDevManager/releases/latest/download/update-feed.json`。

界面按钮会分别显示“刷新本地发布”或“联网刷新”。后台 PowerShell、CMD 和 curl 使用 Windows 系统绝对路径，并以 `CREATE_NO_WINDOW` 模式运行，因此不依赖启动管理器时继承的 PATH，也不会弹出控制台窗口。只有用户主动选择“打开终端”时才显示终端。
