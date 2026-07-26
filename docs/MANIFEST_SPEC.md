# Manifest 配置规范

Manifest 用于描述一个工具如何下载、安装、配置、验证。

## 基础结构

```yaml
id: gradle
name: Gradle
category: BuildTools
install_dir: BuildTools/Gradle
homepage: https://gradle.org

versions:
  - version: 8.5
    url: https://services.gradle.org/distributions/gradle-8.5-bin.zip
    archive_type: zip
    root_dir: gradle-8.5
    sha256: ""

current: 8.5

env:
  GRADLE_HOME: "{install_path}/current"

path:
  - "{install_path}/current/bin"

cache:
  GRADLE_USER_HOME: "{frameworks_home}/Caches/Gradle"

verify:
  - command: "{install_path}/current/bin/gradle.bat --version"
```

## 字段说明

| 字段 | 说明 |
|---|---|
| `id` | 工具唯一 ID |
| `name` | 显示名称 |
| `category` | 分类 |
| `install_dir` | 相对于 Frameworks 根目录的安装目录 |
| `versions` | 可安装版本列表 |
| `url` | 下载地址 |
| `archive_type` | `zip` / `7z` / `tar.gz` / `exe` |
| `root_dir` | 解压后的根目录 |
| `sha256` | 校验值 |
| `env` | 环境变量 |
| `path` | 需要加入 PATH 的路径 |
| `cache` | 缓存重定向变量 |
| `verify` | 安装后检测命令 |

## 路径变量

| 变量 | 含义 |
|---|---|
| `{frameworks_home}` | 绿色环境根目录，例如 `D:/Frameworks` |
| `{install_path}` | 当前工具安装目录 |
| `{version}` | 当前版本号 |
| `{downloads}` | 下载缓存目录 |
| `{cache}` | 缓存目录 |
