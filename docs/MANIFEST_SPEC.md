# Manifest Schema 2

权威文件为 `Config\greendev\components.json`，JSON Schema 位于 `Config\greendev\schema\components.schema.json`，自定义组件模板位于 `Config\greendev\examples\custom-component.json`。

```json
{
  "schemaVersion": 2,
  "components": [
    {
      "id": "custom-tool",
      "name": "Custom Tool",
      "version": "1.0.0",
      "enabled": true,
      "dependsOn": [],
      "installDir": "Toolchains\\Custom\\custom-tool-1.0.0",
      "currentLink": "Toolchains\\Custom\\current",
      "healthPath": "bin\\custom-tool.exe",
      "archiveRoot": "custom-tool-1.0.0",
      "source": {
        "type": "archive",
        "url": "https://HOST/custom-tool.zip",
        "archive": "downloads\\packages\\custom-tool.zip",
        "sha256": "SHA256"
      }
    }
  ]
}
```

约束：

- 所有目标和归档路径均为 Frameworks 内的相对路径，不接受 `..`。
- 支持 ZIP、7Z、TAR.GZ、TGZ、TAR.XZ 和 MSI。
- 在线安装必须在 Manifest 或本机 package lock 中提供 SHA256。
- `dependsOn` 在批量计划中进行拓扑排序；循环和缺失依赖会阻止执行。
- 插件目录只提供声明式 Manifest，网络、进程和写入目录由可信策略控制。
