# 开发路线图

## Phase 1：CLI MVP

目标：做出可用的 `greendev.exe`。

功能：

- `greendev list`
- `greendev install java jdk-21`
- `greendev install gradle 8.5`
- `greendev install maven 3.9.11`
- `greendev use java jdk-21`
- `greendev doctor`
- 生成 `Scripts\dev-shell.bat`

## Phase 2：绿色化增强

功能：

- 统一缓存目录
- Rust 安装支持
- Node.js 支持
- Android SDK commandline tools 支持
- current 目录联接切换
- 下载 SHA256 校验
- 断点续传

## Phase 3：Profile 套装

支持：

- `android-dev`
- `java-backend`
- `rust-dev`
- `cpp-dev`
- `reverse-analysis`

命令：

```bat
greendev profile android-dev
greendev profile rust-dev
```

## Phase 4：GUI

推荐：

- Tauri GUI
- Rust CLI 作为核心能力

页面：

- 首页仪表盘
- 工具市场
- 已安装工具
- 环境套装
- 版本切换
- 缓存管理
- 下载任务
- 日志中心

## Phase 5：插件化和离线包

功能：

- 自定义 manifest
- 离线导入 zip/7z
- 企业内网镜像
- manifest 签名校验
- 多机器同步配置
