# D:\Frameworks 使用说明

更新时间: 2026-07-13

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
Gradle    D:\Frameworks\BuildTools\Gradle\current
Maven     D:\Frameworks\BuildTools\Maven\current
Android   D:\Frameworks\Platforms\Android\Sdk
C/GCC     D:\Frameworks\Toolchains\C\mingw64
Rust      D:\Frameworks\Toolchains\Rust\current
ACPI      D:\Frameworks\Toolchains\ACPI\iasl
MySQL     D:\Frameworks\Databases\Sql\mysql\current
Ghidra    D:\Frameworks\ReverseTools\Ghidra
```


## 加载开发环境

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

批处理脚本已经在开头设置 UTF-8 代码页:

```bat
chcp 65001 >nul
```

推荐在 Windows Terminal、PowerShell 或新版 CMD 中运行。如果手动打开的 CMD 仍出现乱码，先执行:

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

权威配置（改这里，再同步到生效路径）:

```text
Maven   D:\Frameworks\Config\maven\settings.xml
Gradle  D:\Frameworks\Config\gradle\gradle.properties
        D:\Frameworks\Config\gradle\init.d\cn-mirrors.init.gradle
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
8. 组件清单与环境变量定义的权威源是 `Scripts\frameworks-common.ps1`；改组件时优先改此文件。
9. 脚本通过所在目录推导 `FRAMEWORKS_HOME`（也可用环境变量覆盖），不依赖写死盘符。


## 重要文档

```text
D:\Frameworks\README.md
D:\Frameworks\docs\README.md
D:\Frameworks\docs\DIRECTORY_STRUCTURE.md
D:\Frameworks\docs\DOWNLOAD_GUIDE.md
D:\Frameworks\docs\RUST_ENV.md
```
