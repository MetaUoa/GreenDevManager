# GreenDev CLI

`greendev.exe` 是 GUI 的命令行入口，读取相同的 `components.json`、`profile-sets.json` 和 Profile Lock，并调用相同的安装脚本。操作事务写入 `Caches\GreenDevManager\transactions`，结果写入 `Logs\GreenDev\operations.jsonl`。

```bat
greendev.exe list [-Json]
greendev.exe doctor
greendev.exe plan COMPONENT
greendev.exe install COMPONENT
greendev.exe update COMPONENT
greendev.exe use COMPONENT TARGET_PATH
greendev.exe profile PROFILE_ID
greendev.exe lock PROFILE_ID
greendev.exe diff PROFILE_ID
greendev.exe validate
greendev.exe audit
```

`install` 与 `update` 强制执行依赖、SHA256、暂存安装和健康检查。`use` 仅切换经过健康验证的目录联接，原版本目录继续保留。`profile` 按档案切换已安装组件；缺失组件保持原状并由后续安装计划处理。

示例：

```bat
greendev.exe list -Json
greendev.exe plan gradle
greendev.exe use gradle BuildTools\Gradle\gradle-9.4.1
greendev.exe lock java-backend
greendev.exe diff java-backend
```
