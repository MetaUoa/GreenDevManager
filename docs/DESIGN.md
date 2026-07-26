# GreenDev Manager 设计方案

## 1. 项目定位

GreenDev Manager 是一款用于 Windows 绿色开发环境的一键下载、安装、更新、切换和检测工具。

它的核心目标是：

- 所有开发工具集中安装到指定目录，例如 `D:\Frameworks`
- 尽量不污染系统环境变量和用户目录
- 支持多版本共存与快速切换
- 支持 manifest 描述式安装
- 支持 profile 一键组合安装开发环境
- 支持 CLI 优先，GUI 后续扩展

推荐技术路线：

```text
Rust CLI Core -> Rust Service/Core Library -> GUI Frontend(Tauri/egui/Avalonia 可选)
```

---

## 2. 目标目录结构

```text
D:\Frameworks\
├── Runtimes\
│   ├── Java\
│   ├── Node\
│   ├── Rust\
│   ├── Python\
│   └── Go\
├── BuildTools\
│   ├── Gradle\
│   ├── Maven\
│   ├── CMake\
│   └── Ninja\
├── Platforms\
│   ├── Android\
│   ├── Linux\
│   └── Windows\
├── Toolchains\
│   ├── C\
│   ├── LLVM\
│   └── MSYS2\
├── Databases\
│   ├── MySQL\
│   ├── PostgreSQL\
│   └── Redis\
├── ReverseTools\
│   ├── Ghidra\
│   ├── jadx\
│   ├── apktool\
│   ├── ILSpy\
│   └── Frida\
├── Apps\
│   ├── GreenDevManager\
│   └── UniGetUI\
├── Caches\
│   ├── Gradle\
│   ├── Maven\
│   ├── npm\
│   ├── pip\
│   ├── Rust\
│   └── Android\
├── Config\
│   ├── cargo\
│   ├── gradle\
│   ├── maven\
│   ├── npm\
│   └── pip\
├── Scripts\
│   ├── dev-shell.bat
│   ├── dev-shell.ps1
│   ├── env-setup.bat
│   └── check-env.ps1
├── manifests\
├── profiles\
├── downloads\
├── logs\
└── docs\
```

---

## 3. 核心功能

### 3.1 工具管理

- 查询可安装工具
- 安装指定版本
- 下载压缩包或安装器
- 校验 SHA256
- 自动解压
- 识别压缩包根目录
- 创建 `current` 目录联接
- 生成环境变量脚本
- 验证命令是否可用

### 3.2 多版本管理

示例：

```text
D:\Frameworks\Runtimes\Java\
├── jdk-8\
├── jdk-17\
├── jdk-21\
└── current -> jdk-21
```

切换命令：

```bat
greendev use java jdk-21
greendev use gradle 8.5
greendev use rust stable
```

### 3.3 Profile 一键环境

Profile 用于组合多个工具，例如：

- Android 开发环境
- Java 后端开发环境
- Rust 开发环境
- C/C++ 开发环境
- 逆向分析环境

示例：

```bat
greendev profile android-dev
greendev profile rust-dev
```

### 3.4 环境检测

```bat
greendev doctor
greendev doctor java
greendev doctor android
greendev doctor rust
```

检测内容：

- 工具目录是否存在
- current 是否有效
- 环境变量是否正确
- PATH 是否包含必要路径
- 命令版本是否符合预期
- 缓存目录是否已重定向

---

## 4. CLI 命令设计

```bat
greendev list
greendev search <keyword>
greendev install <tool> [version]
greendev uninstall <tool> [version]
greendev update [tool]
greendev use <tool> <version>
greendev versions <tool>
greendev profile <profile-id>
greendev doctor [tool]
greendev env
greendev cache clean [tool]
greendev config get <key>
greendev config set <key> <value>
```

---

## 5. Rust 项目结构

```text
GreenDevManager\
├── Cargo.toml
├── src\
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── manifest.rs
│   ├── profile.rs
│   ├── downloader.rs
│   ├── installer.rs
│   ├── archive.rs
│   ├── env.rs
│   ├── version.rs
│   ├── doctor.rs
│   ├── cache.rs
│   ├── link.rs
│   └── utils.rs
├── manifests\
├── profiles\
├── templates\
└── docs\
```

---

## 6. 推荐依赖

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["stream", "rustls-tls"] }
indicatif = "0.17"
sha2 = "0.10"
zip = "2"
sevenz-rust = "0.6"
tar = "0.4"
flate2 = "1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
directories = "5"
fs_extra = "1"
```

---

## 7. 开发阶段规划

### 第一阶段：CLI MVP

支持：

- 配置根目录
- 读取 manifest
- 下载文件
- 解压 zip
- 安装 Java / Gradle / Maven / Node
- 生成 env 脚本
- doctor 检测

### 第二阶段：完善绿色化

支持：

- Rust 安装
- Android SDK commandline tools
- Maven / Gradle / npm / Cargo 缓存重定向
- current 版本切换
- profile 一键安装

### 第三阶段：GUI

可选方案：

- Tauri：Rust 后端 + Web UI
- egui：纯 Rust GUI
- Avalonia：C# GUI + Rust CLI Core

### 第四阶段：插件化

- 用户自定义 manifest
- 镜像源配置
- 工具市场
- 离线包导入
- 企业内网源

---

## 8. 设计原则

1. CLI 优先，GUI 后置
2. manifest 驱动，不把工具下载逻辑写死在代码里
3. 所有路径可配置
4. 所有安装操作可重复执行
5. 默认不修改系统环境变量
6. 优先生成临时环境脚本
7. 支持断点续传和校验
8. 保留旧版本，切换 current
9. 缓存集中到 `D:\Frameworks\Caches`
10. 日志完整，方便定位失败原因
