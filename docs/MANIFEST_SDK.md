# Manifest 与插件 SDK

组件 Manifest 使用 `Config\greendev\schema\components.schema.json`，插件描述使用 `Config\greendev\schema\plugin.schema.json`。

```powershell
Scripts\New-GreenDevManifest.ps1 -Id custom-tool -Name "Custom Tool" -Version 1.0.0
Scripts\Test-GreenDevPlugin.ps1 -Path Config\greendev\examples\plugin.json
greendev completion powershell
greendev completion cmd
```

模板生成到 examples 目录，不直接修改活动组件清单。插件权限只识别 `network`、`process` 和 `writeRoots`，导入前应先审查权限差异。
