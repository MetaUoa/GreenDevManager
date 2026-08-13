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

## Phase 6–7：安装与发布闭环（已完成）

- 强制校验、原子安装、失败回退、诊断和 Windows 发布物

## Phase 8：稳定性与数据安全（已完成）

- 配置冲突检测、备份保留/预览、事务恢复、current 修复和端到端测试

## Phase 9：组件升级中心（已完成）

- 官方目录刷新、LTS/Stable 策略、候选采用、依赖排序、批量更新和版本回退

## Phase 10：体验与可视化（已完成）

- 全局任务中心、搜索、主题、页面记忆、磁盘趋势、行号编辑器和诊断包

## Phase 11：发布与维护自动化（已完成）

- Profile 迁移、Manifest Schema 迁移、发布清单、可选签名、CI 和变更日志

## Phase 12：质量与可靠性（已完成）

- 任务暂停/继续/失败重试、阶段事务快照、吞吐与 ETA 字段
- 启动恢复、安装回退、故障注入、高 DPI 和生产 WebView2 回归

## Phase 13：Manifest 与插件化（已完成）

- Schema 2 可视化编辑、自定义组件与依赖校验
- ZIP、7Z、TAR.GZ、TGZ、TAR.XZ、MSI 安装来源
- 可信目录、签名要求、网络/进程/写目录最小权限

## Phase 14：自更新与供应链（已完成）

- Stable/Beta/Local 通道、本地或远程 Feed、SHA256/签名策略和待更新事务
- CycloneDX 1.5 组件清单、本地漏洞通告与发布 Feed

## Phase 15：多档案与多机器复现（已完成）

- Profile/Lock、环境差异、团队模板与机器覆盖项
- 离线介质导出、组件包哈希锁定和敏感信息排除

## Phase 16：生产稳定性（已完成）

- 持久任务队列、并发/优先级/计划时间、暂停/继续、事务时间线和统一恢复中心
- 网络、磁盘阶段、current 切换失败注入与自动回滚

## Phase 17：GUI 自动化与体验（已完成）

- 14 页面键盘循环导航、跳转链接、焦点、ARIA Live、Reduced Motion
- 960×680、1240×820、1600×900 生产 WebView2 自动截图回归

## Phase 18：团队与企业管理（已完成）

- directory/http/git 团队 Profile 仓库、差异预览与保留机器覆盖的合并
- 企业签名/主机/字段锁定策略、机器组、合规扫描、审计包和增量离线介质

## Phase 19：CLI 与扩展生态（已完成）

- `greendev.exe` 与 GUI 共用 Manifest、安装脚本、Profile Lock、事务和 JSONL 日志
- list/doctor/plan/install/update/use/profile/lock/diff/validate/audit
- Manifest JSON Schema 和自定义组件示例

## Phase 20：稳定性收敛（已完成）

- 任务快照与执行规格双持久化，异常重启后重排队并复用下载断点
- Windows 单实例、日志容量观察与只归档、性能基线和三通道验证策略
- SDK、签名与 GUI 静态门禁纳入集成测试

## Phase 21：签名与供应链（已完成）

- Authenticode 可选签名和 RSA-PSS-SHA256 分离签名工具
- 可信/吊销指纹、密钥轮换和许可证策略
- 发布级 CycloneDX 1.5 SBOM、来源证明、清单与制品哈希验证

## Phase 22：远程机器（已完成）

- 节点注册表、机器组、标签、local/winrm/agent 传输契约和凭据引用
- 分批比例、维护窗口、逐批预览、审批暂存和失败回滚模型
- 轻量 Agent 的 inventory/plan/apply/rollback 事务入口

## Phase 23：生态与体验（已完成）

- Manifest 模板生成器、Plugin Schema 和最小权限检查器
- CLI PowerShell/CMD 补全、18 页面导航与 `Ctrl+K` 命令搜索
- zh-CN/en-US 资源约定、键盘提示和迁移/排障文档
