# 远程节点与分批发布

节点注册表位于 `Config\greendev\remote-nodes.json`，支持 `local`、`winrm` 和 `agent` 传输类型。配置只保存 `credentialRef`，口令和令牌由外部凭据存储解析。

GUI“远程节点”按机器组和标签生成批次，记录维护窗口、审批点与失败回滚要求。点击“暂存并等待审批”只写入 `Caches\GreenDevManager\fleet-rollouts`，此时不会触发节点变更。

“刷新只读清单”经 local/WinRM/Agent 采集组件版本和 current 入口，结果缓存到 `Caches\GreenDevManager\fleet-inventory.json`。采集过程不更改节点配置。

轻量 Agent 命令：

```powershell
Scripts\greendev-agent.ps1 -Action inventory
Scripts\greendev-agent.ps1 -Action plan -PlanPath PLAN.json
Scripts\greendev-agent.ps1 -Action apply -PlanPath PLAN.json
Scripts\greendev-agent.ps1 -Action rollback -TransactionId ID
```
