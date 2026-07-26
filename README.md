# D:\Frameworks

Windows 绿色开发环境根目录。工具解压即用，多版本通过 `current` 切换，缓存与配置集中管理。

## 快速开始

```bat
call D:\Frameworks\Scripts\dev-shell.bat
D:\Frameworks\env-setup.bat
```

## 文档

| 文档 | 说明 |
|---|---|
| [docs/USAGE.md](docs/USAGE.md) | 使用说明（环境、清理、镜像） |
| [docs/DIRECTORY_STRUCTURE.md](docs/DIRECTORY_STRUCTURE.md) | 目录与入口约定 |
| [docs/DOWNLOAD_GUIDE.md](docs/DOWNLOAD_GUIDE.md) | 下载与解压路径 |
| [docs/README.md](docs/README.md) | 文档索引 |

## 入口脚本

```text
Scripts\dev-shell.bat   临时加载环境
env-setup.bat           检测
setup_dev_env.bat       写入用户环境变量
cleanup.bat             缓存清理（默认预览）
auto-setup.bat          安装检测助手
```
