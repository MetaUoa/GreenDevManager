# Frameworks 文档

本目录为 `D:\Frameworks` 绿色开发环境的说明与 GreenDev Manager 设计文档。

## 日常使用（优先读这些）

| 文件 | 说明 |
|---|---|
| [`USAGE.md`](USAGE.md) | 使用说明：加载环境、清理、镜像、验证 |
| [`DIRECTORY_STRUCTURE.md`](DIRECTORY_STRUCTURE.md) | 当前目录结构与入口约定 |
| [`DOWNLOAD_GUIDE.md`](DOWNLOAD_GUIDE.md) | 绿色版工具下载与解压路径 |
| [`RUST_ENV.md`](RUST_ENV.md) | Rust 绿色环境布局与环境变量 |

## GreenDev Manager 设计与实现

| 文件 | 说明 |
|---|---|
| [`DESIGN.md`](DESIGN.md) | 总体设计 |
| [`ROADMAP.md`](ROADMAP.md) | 开发路线图 |
| [`CLI_SPEC.md`](CLI_SPEC.md) | CLI 命令设计 |
| [`MANIFEST_SPEC.md`](MANIFEST_SPEC.md) | 工具 manifest 规范 |
| [`PROFILE_SPEC.md`](PROFILE_SPEC.md) | 环境 profile 规范 |

## 顶层脚本入口

```text
D:\Frameworks\Scripts\dev-shell.bat   临时加载环境
D:\Frameworks\env-setup.bat           检测环境
D:\Frameworks\setup_dev_env.bat       写入用户环境变量
D:\Frameworks\sync-config.bat         同步权威配置
D:\Frameworks\cleanup.bat             缓存清理（默认预览）
D:\Frameworks\auto-setup.bat          安装检测助手
```

详细步骤见 [`USAGE.md`](USAGE.md)。
