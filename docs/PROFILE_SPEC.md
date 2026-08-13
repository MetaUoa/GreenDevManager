# Profile 与 Lock

Profile 定义位于 `Config\greendev\profile-sets.json`：

```json
{
  "id": "java-backend",
  "name": "Java 后端",
  "components": ["java", "gradle", "maven", "mysql"],
  "teamTemplate": true,
  "machineOverrides": {}
}
```

Lock 位于 `Config\greendev\profile-locks\PROFILE.lock.json`，记录组件版本、归档 SHA256、依赖、安装目录和 current 目标。GUI 与 `greendev.exe lock/diff/profile` 使用相同文件。

团队仓库支持：

- `directory`：共享目录中的 `profile-sets.json`。
- `http`：直接下载 Profile 集合。
- `git`：指定仓库与分支。

团队同步先展示新增与变更。应用时归档本机文件，按 ID 合并团队 Profile，同时保留本机 `machineOverrides` 和仅本机 Profile。完整离线介质包含所有可用归档；增量介质仅包含相对上次导出索引发生变化的归档。凭据特征文件不会进入离线包。
