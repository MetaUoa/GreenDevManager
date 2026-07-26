# Profile 配置规范

Profile 用于描述一组工具的组合安装方案。

## 示例：Rust 开发环境

```yaml
id: rust-dev
name: Rust Development Environment
description: Rust CLI / Backend / System development environment

tools:
  - id: rust
    version: stable
    components:
      - rustfmt
      - clippy
      - rust-src
      - rust-analyzer

cargo_install:
  - cargo-edit
  - cargo-watch
  - cargo-nextest
  - cargo-audit
  - cargo-deny

scripts:
  - generate: dev-shell.bat
  - generate: cargo-config.toml

verify:
  - rustc --version
  - cargo --version
  - rustup show
```

## 示例：Android 开发环境

```yaml
id: android-dev
name: Android Development Environment

tools:
  - id: java
    version: jdk-21
  - id: gradle
    version: 8.5
  - id: maven
    version: 3.9.11
  - id: android-sdk
    version: latest
  - id: node
    version: 24.18.0

android:
  packages:
    - platform-tools
    - platforms;android-35
    - build-tools;35.0.0
    - cmdline-tools;latest

verify:
  - java -version
  - gradle --version
  - adb version
```

## 字段说明

| 字段 | 说明 |
|---|---|
| `id` | Profile ID |
| `name` | 显示名称 |
| `tools` | 工具列表 |
| `components` | 组件，例如 rustfmt/clippy |
| `scripts` | 需要生成的脚本 |
| `verify` | 检测命令 |
