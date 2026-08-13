# GreenDev Manager

用于管理 Windows 绿色开发环境的 Tauri 桌面应用。当前稳定版 `1.0.1` 已完成首次启动在线初始化。

## 下载

- [Windows x64 安装包](https://github.com/MetaUoa/GreenDevManager/releases/latest/download/GreenDevManager-1.0.1-win-x64-setup.exe)
- [Windows x64 便携包](https://github.com/MetaUoa/GreenDevManager/releases/latest/download/GreenDevManager-1.0.1-win-x64-portable.zip)
- [最新发布与校验文件](https://github.com/MetaUoa/GreenDevManager/releases/latest)

## 功能

- 快速组件清单先显示，环境与缓存容量在后台扫描
- 普通/深度 Doctor、结构化操作结果与持久 JSONL 日志
- 用户环境变量选择配置、自动备份、历史快照恢复
- Config 权威配置内容指纹、漂移检测和一键同步
- safe/normal 缓存清理预览与确认执行
- Java、Node、Python、Gradle、Maven、Rust、MySQL 多版本枚举
- `current` 健康校验切换、失败回退和版本固定；不移除已安装版本
- Android SDK 本地/在线包目录、安装和确认卸载
- `Config\greendev\components.json` 清单驱动的归档下载、校验、暂存安装和更新
- Android 与清单任务的进度、取消和结果追踪
- Gradle、Maven、Cargo、MySQL、npm、pip 配置的表单/源码双模式编辑、校验、差异预览与备份回滚
- 组件依赖和固定状态预检、离线 ZIP/7Z/TAR/MSI 导入、镜像/代理、断点重试、强制 SHA-256 校验及失败回滚
- 运行环境诊断、崩溃日志、集成测试、便携 ZIP 与 NSIS 安装包发布
- 配置冲突检测、最近 30 份备份保留/检索/预览、任务事务恢复和 current 自动修复
- LTS/Stable 官方目录刷新、候选采用、依赖排序批量更新和一键版本回退
- 全局搜索、深色模式、页面记忆、任务中心、磁盘趋势和脱敏诊断包
- 跨盘符 Profile 导入导出、Manifest Schema 迁移、发布清单、可选代码签名和 CI 门禁
- 后台任务暂停/继续/重试、阶段事务、下载吞吐和异常恢复回归
- Stable/Beta/Nightly/Local 应用更新通道、远程 Feed、本地发布物验证和待更新事务
- Manifest Schema 2 可视化编辑、自定义组件、ZIP/7Z/TAR/MSI 与可信目录最小权限
- 多 Profile、版本/哈希/依赖锁定、环境差异、团队模板、机器覆盖和离线介质
- CycloneDX 供应链清单与本地漏洞通告库存
- 持久任务队列、并发/优先级/计划时间、完整事务时间线和统一恢复中心
- 18 页面键盘导航、ARIA/焦点门禁、三种窗口尺寸与生产 WebView2 自动回归
- `greendev.exe` CLI：list/doctor/plan/install/update/use/profile/lock/diff/validate/audit/completion
- directory/http/git 团队 Profile 仓库、差异合并、机器覆盖保留、合规扫描与审计包
- 重启可恢复任务规格、单实例、日志只归档、性能预算和稳定性中心
- Authenticode/RSA-PSS 签名链、吊销/信任策略、CycloneDX 与 SLSA 来源证明
- 远程节点注册表、分批发布预览、审批暂存与轻量 Agent 协议
- Manifest/Plugin Schema、模板生成器、权限检查和 `Ctrl+K` 命令搜索

## 运行

```bat
D:\Frameworks\Apps\GreenDevManager\run.bat
```

安装版首次启动未发现环境时会显示目录向导。选择空目录可下载并校验最新 Bootstrap 包；选择已有目录会验证 `Scripts`、`Config` 与 `env-setup.bat`。根目录可位于任意盘符，保存后由 `FRAMEWORKS_HOME` 和应用本地配置共同发现。

GNU 构建产物由 `GreenDevManager.exe` 与同目录 `WebView2Loader.dll` 组成。

## 开发

```powershell
cd D:\Frameworks\Apps\GreenDevManager
npm install
cmd /c "call ..\..\Scripts\dev-shell.bat && npm run tauri dev"
```

## 构建绿色 EXE

```powershell
powershell -ExecutionPolicy Bypass -File .\build.ps1
```

## 验证与发布

```powershell
powershell -ExecutionPolicy Bypass -File .\integration-test.ps1
powershell -ExecutionPolicy Bypass -File .\e2e-test.ps1
powershell -ExecutionPolicy Bypass -File .\release.ps1
```

发布物写入 `Releases\GreenDevManager\<version>`。便携包可在 Frameworks 目录树内直接启动；放在其他位置时执行 `run.bat D:\Frameworks`。

签名发布可传入本机证书指纹：

```powershell
.\release.ps1 -SignThumbprint CERT_THUMBPRINT
```

组件清单只接受 Frameworks 根目录内的相对目标路径。安装包默认缓存到 `downloads\packages`；在线安装必须先在清单或本地包锁中提供可信 SHA-256。离线 ZIP/7Z/TAR/MSI 导入会计算并记录校验值。
