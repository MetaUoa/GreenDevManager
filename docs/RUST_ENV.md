# Rust 绿色开发环境

更新时间: 2026-07-26

## 目录结构（当前实装）

```text
D:\Frameworks\Toolchains\Rust\
├── current\              # junction -> standalone
├── standalone\           # rustc / cargo 实体安装（唯一工具链）
└── cargo-home\
    ├── registry\
    └── config.toml       # 生效副本；权威副本在 Config\cargo\config.toml
```

构建产物缓存:

```text
D:\Frameworks\Caches\Rust\target      # CARGO_TARGET_DIR
```

> rustup-home 已移除（2026-07）。当前只维护 standalone 单套工具链，
> 升级方式见下文；如需改用 rustup 管理，需重新引入 RUSTUP_HOME。

## 环境变量

脚本（`dev-shell` / `setup_dev_env`）写入:

```bat
set RUST_HOME=D:\Frameworks\Toolchains\Rust\current
set CARGO_HOME=D:\Frameworks\Toolchains\Rust\cargo-home
set CARGO_TARGET_DIR=D:\Frameworks\Caches\Rust\target
set PATH=%RUST_HOME%\bin;%PATH%
```

说明:

- 日常使用走 `RUST_HOME`（`current` → `standalone`）。
- 依赖/registry 在 `CARGO_HOME`；编译 target 在 `Caches\Rust\target`。
- 共享 target 目录的取舍：省磁盘，但不同项目 feature/依赖差异会触发重编译。

## 升级流程（standalone 方式）

1. 下载新版 standalone 发行包（`rust-x.xx.x-x86_64-pc-windows-gnu.tar.xz`）。
2. 解压为 `Toolchains\Rust\rust-x.xx`（或直接替换 `standalone`）。
3. 重建 junction: `mklink /J current <新目录>`（管理员或开发者模式）。
4. `rustc --version` 验证。

## Cargo 配置

权威副本: `Config\cargo\config.toml`，生效位置: `cargo-home\config.toml`。

当前内容（已配 rsproxy 国内镜像）:

```toml
[build]
target-dir = "D:/Frameworks/Caches/Rust/target"

[net]
git-fetch-with-cli = true

[source.crates-io]
replace-with = "rsproxy-sparse"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
```

停用镜像：注释掉 `replace-with` 一行。

## 检测

```bat
call D:\Frameworks\Scripts\dev-shell.bat
rustc --version
cargo --version
```

## 清理

Rust 构建缓存可用:

```bat
D:\Frameworks\cleanup.bat apply safe
```

会清理 `Caches\Rust\target`，不会删除 `standalone` / `cargo-home` 工具本身。
