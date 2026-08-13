# D:\Frameworks

Windows 绿色开发环境根目录。工具解压即用，多版本通过 `current` 切换，缓存与配置集中管理。

## GUI 管理器

[下载 GreenDev Manager 最新安装包与便携包](https://github.com/MetaUoa/GreenDevManager/releases/latest)

```bat
D:\Frameworks\Apps\GreenDevManager\run.bat
```

当前支持后台容量扫描与趋势、Doctor、环境备份恢复、多版本安全切换、Android SDK、持久任务队列、统一恢复中心、Schema 2 清单、应用自更新、供应链清单、团队 Profile/Lock/增量介质、企业合规、审计导出以及 GUI/CLI 双入口。已安装旧版本不会被自动移除。

## 快速开始

```bat
call D:\Frameworks\Scripts\dev-shell.bat
D:\Frameworks\env-setup.bat
D:\Frameworks\greendev.exe list
```

## 文档

| 文档 | 说明 |
|---|---|
| [docs/USAGE.md](docs/USAGE.md) | 使用说明（环境、清理、镜像） |
| [docs/DIRECTORY_STRUCTURE.md](docs/DIRECTORY_STRUCTURE.md) | 目录与入口约定 |
| [docs/DOWNLOAD_GUIDE.md](docs/DOWNLOAD_GUIDE.md) | 下载与解压路径 |
| [docs/README.md](docs/README.md) | 文档索引 |
| [Apps/GreenDevManager/README.md](Apps/GreenDevManager/README.md) | GUI 管理器开发与构建 |

## 入口脚本

```text
Scripts\dev-shell.bat   临时加载环境
env-setup.bat           检测
setup_dev_env.bat       写入用户环境变量
sync-config.bat         同步权威配置到生效位置
cleanup.bat             缓存清理（默认预览）
auto-setup.bat          安装检测助手
```
