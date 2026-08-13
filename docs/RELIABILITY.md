# 可靠性与恢复

GreenDev Manager 1.0 RC 将任务快照和执行规格分别写入 `Caches\GreenDevManager\transactions`。任务处于 queued、running 或 paused 时异常退出，下一次启动会把旧记录标记为 restarted，并以增加后的 attempt 恢复；暂停任务仍保持暂停，下载 `.part` 和组件旧版本保持原样。含令牌、密码或凭据参数的任务只保存展示快照，不持久化可执行恢复规格。

`Config\greendev\reliability-policy.json` 管理日志阈值、性能预算和 Stable/Beta/Nightly 验证矩阵。任务并发上限在每次领取时动态读取；队列内暂停任务会释放并发槽。`operations.jsonl` 达到阈值后移动到 `Logs\GreenDev\archive`，历史归档不做自动移除。

GUI“稳定性中心”可查看队列恢复次数、日志容量、单实例状态并运行本机性能基线。基线写入 `Caches\GreenDevManager\performance-baseline.json`。
