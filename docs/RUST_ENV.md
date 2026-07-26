# Rust 绿色开发环境

更新时间: 2026-07-13

## 目录结构（当前实装）

```text
D:\Frameworks\Toolchains\Rust\
├── current\              # junction -> standalone
├── standalone\           # rustc / cargo 实体安装
├── rustup-home\
│   ├── toolchains\
│   ├── update-hashes\
│   └── settings.toml
└── cargo-home\
    ├── bin\
    ├── registry\
    ├── git\
    └── config.toml
```

构建产物缓存:

```text
D:\Frameworks\Caches\Rust\target      # CARGO_TARGET_DIR
```

离线安装包:

```text
D:\Frameworks\downloads\rust
```

## 环境变量

脚本（`dev-shell` / `setup_dev_env`）写入:

```bat
set RUST_HOME=D:\Frameworks\Toolchains\Rust\current
set RUSTUP_HOME=D:\Frameworks\Toolchains\Rust\rustup-home
set CARGO_HOME=D:\Frameworks\Toolchains\Rust\cargo-home
set CARGO_TARGET_DIR=D:\Frameworks\Caches\Rust\target
set PATH=%RUST_HOME%\bin;%PATH%
```

说明:

- 日常使用走 `RUST_HOME`（`current` → `standalone`）。
- 若使用 rustup 管理工具链，保留 `RUSTUP_HOME`。
- 依赖/registry 在 `CARGO_HOME`；编译 target 在 `Caches\Rust\target`。

## 安装流程（参考）

```bat
set RUSTUP_HOME=D:\Frameworks\Toolchains\Rust\rustup-home
set CARGO_HOME=D:\Frameworks\Toolchains\Rust\cargo-home
D:\Frameworks\downloads\rust\rustup-init.exe -y --no-modify-path
rustup default stable
rustup component add rustfmt clippy rust-src
```

或解压 standalone 发行版到 `Toolchains\Rust\standalone`，并保证:

```text
Toolchains\Rust\current  ->  standalone
```

## Cargo 配置

文件: `Toolchains\Rust\cargo-home\config.toml`

建议:

```toml
[build]
target-dir = "D:/Frameworks/Caches/Rust/target"

[net]
git-fetch-with-cli = true
```

国内 crates 镜像可按需自行添加 `[source.crates-io]` / `replace-with`。

## 检测

```bat
call D:\Frameworks\Scripts\dev-shell.bat
rustc --version
cargo --version
rustup show
```

## 清理

Rust 构建缓存可用:

```bat
D:\Frameworks\cleanup.bat apply safe
```

会清理 `Caches\Rust\target`，不会删除 `standalone` / `rustup-home` / `cargo-home` 工具本身。
