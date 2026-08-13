# Changelog

## 1.1.0 — 2026-08-13

- 安装与更新页面新增绿色发行版、版本和架构候选选择器。
- JDK 官方目录覆盖 8、11、17、21、25，并提供 Azul Zulu、Eclipse Temurin、Amazon Corretto 三种 Windows x64 ZIP。
- Node.js 提供多个 LTS 主版本，Gradle 提供多个稳定主版本，候选均携带独立下载地址、目录结构和 SHA-256。
- 修复 Windows PowerShell 5.1 写入目录缓存时的 UTF-8 BOM 兼容问题，联网刷新结果可立即被 GUI 读取。
- 归档根目录与上游命名不一致时，可依据健康文件自动识别唯一顶层目录；旧版本继续保留。

## 1.0.1 — 2026-08-13

- 未发现环境根目录时进入首次启动向导，可浏览选择任意盘符目录。
- 支持接入现有环境，或向空目录下载 SHA-256 锁定的 Bootstrap 配置、脚本与 CLI。
- 根目录同时保存到应用本地状态和用户级 `FRAMEWORKS_HOME`，安装版可在环境目录外启动。
- 应用更新改为替换实际运行程序路径；WinRM 节点必须显式配置各自根目录。

## 1.0.0 — 2026-08-13

- 首个 Stable 公开版本，汇总 Phase 1–23 的环境管理、任务恢复、自更新、供应链和远程节点能力。
- 默认接入 GitHub 更新 Feed，支持从 `1.0.0-rc.1` 联网发现、下载、SHA-256 校验与准备升级。
- 后台 PowerShell 任务统一隐藏控制台窗口，并兼容 GUI 进程 PATH 不完整的场景。
- 提供 Windows x64 NSIS 安装包、便携包、CycloneDX SBOM、SLSA 来源证明与 SHA-256 校验文件。

## 1.0.0-rc.1 — 2026-08-13

- Phase 20：任务执行规格持久化、异常重启自动重排队、Windows 单实例、日志只归档、性能预算与 Stable/Beta/Nightly 门禁。
- Phase 21：Authenticode、RSA-PSS 分离签名、信任/吊销与轮换策略、CycloneDX SBOM、许可证策略和 SLSA 来源证明。
- Phase 22：远程节点注册表、local/winrm/agent 传输契约、分批发布预览、维护窗口、审批暂存和回滚事务模型。
- Phase 23：Manifest/Plugin Schema、模板与权限校验工具、PowerShell/CMD 补全、18 页面 GUI 和 `Ctrl+K` 命令搜索。
- 旧组件版本、配置备份、日志归档与既有发布物全部保留。

## 0.19.0 — 2026-08-13

- Phase 16：持久任务调度、并发/优先级/计划时间、事务时间线、统一恢复中心与三类故障注入。
- Phase 17：14 页面键盘循环导航、焦点/ARIA/动态状态门禁、多尺寸生产 WebView2 回归和完成通知。
- Phase 18：团队 Profile 仓库、差异合并、机器覆盖、增量介质、企业字段锁定、合规扫描和审计包。
- Phase 19：`greendev.exe`、共享 Manifest/Profile/事务/JSONL 核心、Schema 与自定义组件示例。
- 所有旧组件版本、应用回退点和配置备份继续保留。

## 0.15.0 — 2026-08-12

- Phase 12：任务暂停/继续/重试、阶段事务快照、吞吐指标、故障注入和高 DPI 回归。
- Phase 13：Manifest Schema 2 编辑器、自定义组件、ZIP/7Z/TAR/MSI、可信目录与插件权限隔离。
- Phase 14：Stable/Beta/Local 自更新源、发布物哈希/签名策略、待更新事务与 CycloneDX SBOM。
- Phase 15：多 Profile、Lock、差异、团队模板、机器覆盖、敏感信息排除和离线环境介质。
- 所有组件旧版本、配置备份及应用发布物继续保留。

## 0.11.0 — 2026-08-12

- 配置冲突检测、备份保留/预览、安装事务恢复与 current 自动修复。
- 更新候选、策略展示、依赖排序批量更新和一键版本回退。
- 全局搜索、主题与页面持久化、任务中心、容量趋势和诊断包导出。
- 配置迁移包、Manifest Schema 迁移、发布清单、可选代码签名与 CI 门禁。

## 0.7.0 — 2026-08-12

- 完成配置编辑、校验安装、诊断与安装包发布。
