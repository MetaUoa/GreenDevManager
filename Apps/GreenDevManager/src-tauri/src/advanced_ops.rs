use super::*;

fn greendev(root: &Path, name: &str) -> PathBuf {
    root.join(r"Config\greendev").join(name)
}
fn read_value(path: &Path, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(fallback)
}

pub(super) fn enterprise_policy(root: &Path) -> Value {
    read_value(
        &greendev(root, "enterprise-policy.json"),
        json!({"schemaVersion":1,"readOnly":false,"lockedFields":[],"requireSignedCatalogs":false,"requireSignedUpdates":false,"allowedFeedHosts":[],"allowedProxyHosts":[],"machineGroup":"default","teamRepository":{"kind":"directory","path":"","url":"","branch":"main"}}),
    )
}
pub(super) fn is_policy_locked(root: &Path, field: &str) -> bool {
    enterprise_policy(root)
        .get("lockedFields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|value| value == field)
}
pub(super) fn policy_url_allowed(root: &Path, field: &str, url: &str) -> bool {
    let allowed = enterprise_policy(root)
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if allowed.is_empty() || url.is_empty() {
        return true;
    }
    let host = url
        .split_once("://")
        .map(|(_, tail)| tail)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    allowed
        .iter()
        .filter_map(Value::as_str)
        .any(|candidate| candidate.eq_ignore_ascii_case(host))
}

fn add_file_items(root: &Path, directory: &Path, kind: &str, title: &str, items: &mut Vec<Value>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            add_file_items(root, &path, kind, title, items);
        } else {
            let relative = path
                .strip_prefix(root)
                .map(display_path)
                .unwrap_or_else(|_| display_path(&path));
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            items.push(json!({"id":fnv_hash(relative.as_bytes()),"kind":kind,"title":title,"path":display_path(&path),"relativePath":relative,"createdAt":modified,"sizeBytes":entry.metadata().map(|m|m.len()).unwrap_or(0),"canRestore":true}));
        }
    }
}

fn recovery_items(root: &Path) -> Vec<Value> {
    let mut items = Vec::new();
    add_file_items(
        root,
        &root.join(r"Config\config-backups"),
        "config",
        "配置备份",
        &mut items,
    );
    add_file_items(
        root,
        &root.join(r"Config\env-backups"),
        "environment",
        "环境变量备份",
        &mut items,
    );
    add_file_items(
        root,
        &transaction_directory(root),
        "transaction",
        "任务事务",
        &mut items,
    );
    for relative in [
        r"Caches\GreenDevManager\app-update-backups",
        r"Config\profile-import-backups",
    ] {
        let directory = root.join(relative);
        if let Ok(entries) = fs::read_dir(&directory) {
            for entry in entries.filter_map(Result::ok).filter(|e| e.path().is_dir()) {
                let path = entry.path();
                let rel = path
                    .strip_prefix(root)
                    .map(display_path)
                    .unwrap_or_else(|_| display_path(&path));
                items.push(json!({"id":fnv_hash(rel.as_bytes()),"kind":if relative.starts_with("Caches"){"app"}else{"profile"},"title":if relative.starts_with("Caches"){"应用程序回退点"}else{"Profile 导入前备份"},"path":display_path(&path),"relativePath":rel,"createdAt":entry.metadata().ok().and_then(|m|m.modified().ok()).and_then(|t|t.duration_since(UNIX_EPOCH).ok()).map(|d|d.as_millis() as u64).unwrap_or(0),"sizeBytes":directory_size(&path),"canRestore":true}));
            }
        }
    }
    items.sort_by_key(|item| std::cmp::Reverse(item["createdAt"].as_u64().unwrap_or(0)));
    items
}

#[tauri::command]
pub(super) fn get_recovery_center() -> Result<Value, String> {
    let root = frameworks_root()?;
    let items = recovery_items(&root);
    Ok(
        json!({"items":items,"pendingTransactions":items.iter().filter(|item|item["kind"]=="transaction"&&item["path"].as_str().map(|p|p.ends_with(".json")&&!p.contains(".completed.")&&!p.contains(".recovered.")).unwrap_or(false)).count()}),
    )
}

#[tauri::command]
pub(super) fn preview_recovery_item(id: String) -> Result<Value, String> {
    let root = frameworks_root()?;
    let item = recovery_items(&root)
        .into_iter()
        .find(|item| item["id"] == id)
        .ok_or_else(|| "恢复项不存在。".to_string())?;
    let path = PathBuf::from(item["path"].as_str().unwrap_or(""));
    let hash = if path.is_file() {
        sha256_file(&path).unwrap_or_default()
    } else {
        String::new()
    };
    Ok(
        json!({"item":item,"sha256":hash,"preview":if path.is_file(){fs::read_to_string(&path).unwrap_or_else(|_|"二进制备份，将按原样恢复。".into()).chars().take(12000).collect::<String>()}else{format!("目录恢复点，共 {}。",directory_size(&path))}}),
    )
}

#[tauri::command]
pub(super) fn restore_recovery_item(id: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let item = recovery_items(&root)
        .into_iter()
        .find(|item| item["id"] == id)
        .ok_or_else(|| "恢复项不存在。".to_string())?;
    let kind = item["kind"].as_str().unwrap_or("");
    let path = PathBuf::from(item["path"].as_str().unwrap_or(""));
    match kind {
        "config" => {
            let relative = path
                .strip_prefix(root.join(r"Config\config-backups"))
                .map_err(|_| "配置备份路径异常。".to_string())?;
            let mut parts = relative.components();
            let config_id = parts
                .next()
                .and_then(|p| p.as_os_str().to_str())
                .ok_or_else(|| "配置 ID 缺失。".to_string())?;
            let file = path
                .file_name()
                .and_then(|p| p.to_str())
                .ok_or_else(|| "备份文件名缺失。".to_string())?;
            if find_config_definition(config_id).is_ok() {
                rollback_config(config_id.into(), file.into())
            } else {
                let target_name = match config_id {
                    "manifest" => "components.json",
                    "team-profiles" | "profile-sets" => "profile-sets.json",
                    "trusted-catalogs" => "trusted-catalogs.json",
                    "app-update" => "app-update.json",
                    "enterprise-policy" => "enterprise-policy.json",
                    "task-policy" => "task-policy.json",
                    "reliability-policy" => "reliability-policy.json",
                    "supply-chain-policy" => "supply-chain-policy.json",
                    "remote-nodes" => "remote-nodes.json",
                    "pending-app-update" => "pending-app-update.json",
                    _ => return Err(format!("恢复中心尚未映射配置类型：{config_id}")),
                };
                let target = greendev(&root, target_name);
                if path
                    .extension()
                    .and_then(|v| v.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("bak"))
                    .unwrap_or(false)
                {
                    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    let _: Value = serde_json::from_str(&content)
                        .map_err(|e| format!("备份 JSON 校验失败：{e}"))?;
                }
                let before = root.join(r"Config\config-backups\recovery-before");
                fs::create_dir_all(&before).map_err(|e| e.to_string())?;
                if target.is_file() {
                    fs::copy(&target, before.join(format!("{target_name}-{started}.bak")))
                        .map_err(|e| e.to_string())?;
                }
                let temporary = target.with_extension("recovering");
                fs::copy(&path, &temporary).map_err(|e| e.to_string())?;
                fs::rename(&temporary, &target).map_err(|e| e.to_string())?;
                Ok(finish_operation(
                    &root,
                    "config-recovery",
                    "恢复管理配置",
                    started,
                    true,
                    Some(0),
                    format!(
                        "{} -> {}\n恢复前状态已归档。",
                        display_path(&path),
                        display_path(&target)
                    ),
                ))
            }
        }
        "environment" => restore_environment_backup(
            path.file_name()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .into(),
        ),
        "transaction" => {
            recover_transactions(&root);
            Ok(finish_operation(
                &root,
                "recovery-center",
                "恢复任务事务",
                started,
                true,
                Some(0),
                format!("已重新执行启动恢复扫描：{}", display_path(&path)),
            ))
        }
        "profile" => {
            let backup = root
                .join(r"Config\profile-restore-before")
                .join(started.to_string());
            copy_profile_tree(&root.join("Config"), &backup)?;
            copy_profile_tree(&path, &root.join("Config"))?;
            let sync = sync_config_key(&root, "all").unwrap_or_else(|e| e);
            Ok(finish_operation(
                &root,
                "profile-recovery",
                "恢复 Profile 配置",
                started,
                true,
                Some(0),
                format!(
                    "恢复点: {}\n恢复前备份: {}\n{}",
                    display_path(&path),
                    display_path(&backup),
                    sync
                ),
            ))
        }
        "app" => {
            if !path.join("GreenDevManager.exe").is_file() {
                return Err("应用回退点缺少 GreenDevManager.exe。".into());
            }
            let current_executable = std::env::current_exe()
                .map_err(|error| format!("读取当前程序路径失败：{error}"))?;
            let pending = json!({"schemaVersion":1,"version":format!("rollback-{started}"),"preparedAt":started,"stage":display_path(&path),"currentExecutable":display_path(&current_executable),"state":"verified-rollback"});
            atomic_config_write(
                &greendev(&root, "pending-app-update.json"),
                &(serde_json::to_string_pretty(&pending).map_err(|e| e.to_string())? + "\n"),
            )?;
            Ok(finish_operation(
                &root,
                "app-rollback-prepare",
                "准备应用回退",
                started,
                true,
                Some(0),
                "应用回退点已写入待更新事务，可在应用更新页重启应用。".into(),
            ))
        }
        _ => Err("恢复类型不受支持。".into()),
    }
}

fn compliance(root: &Path) -> Value {
    let policy = enterprise_policy(root);
    let manifest = read_manifest(root);
    let updates = read_value(&greendev(root, "app-update.json"), json!({}));
    let trust = read_value(&greendev(root, "trusted-catalogs.json"), json!({}));
    let mut checks = Vec::new();
    checks.push(json!({"id":"manifest","healthy":manifest.is_ok(),"detail":manifest.as_ref().map(|m|format!("Schema {} · {} 个组件",m.schema_version,m.components.len())).unwrap_or_else(|e|e.clone())}));
    let checksums = manifest
        .as_ref()
        .map(|m| {
            m.components
                .iter()
                .filter(|c| !c.source.sha256.is_empty())
                .count()
        })
        .unwrap_or(0);
    let total = manifest.as_ref().map(|m| m.components.len()).unwrap_or(0);
    checks.push(json!({"id":"checksums","healthy":checksums==total,"detail":format!("{checksums}/{total} 个组件在共享清单锁定 SHA256；本机 package-lock 另行补充")}));
    let signed_updates = !policy["requireSignedUpdates"].as_bool().unwrap_or(false)
        || updates["requireSignature"].as_bool() == Some(true);
    checks
        .push(json!({"id":"signed-updates","healthy":signed_updates,"detail":"应用更新签名策略"}));
    let signed_catalogs = !policy["requireSignedCatalogs"].as_bool().unwrap_or(false)
        || trust["requireCatalogSignature"].as_bool() == Some(true);
    checks.push(
        json!({"id":"signed-catalogs","healthy":signed_catalogs,"detail":"组件目录签名策略"}),
    );
    json!({"policy":policy,"checks":checks,"healthy":checks.iter().all(|v|v["healthy"].as_bool()==Some(true)),"generatedAt":now_millis()})
}

#[tauri::command]
pub(super) fn get_enterprise_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    Ok(compliance(&root))
}

#[tauri::command]
pub(super) fn save_enterprise_policy(policy: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let existing = enterprise_policy(&root);
    if existing["readOnly"].as_bool() == Some(true) {
        return Err("企业策略处于只读锁定状态。".into());
    }
    if !policy["lockedFields"].is_array() {
        return Err("lockedFields 必须是数组。".into());
    }
    let path = greendev(&root, "enterprise-policy.json");
    if path.is_file() {
        let backup = root.join(r"Config\config-backups\enterprise-policy");
        fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        fs::copy(
            &path,
            backup.join(format!("enterprise-policy-{started}.json.bak")),
        )
        .map_err(|e| e.to_string())?;
    }
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&policy).map_err(|e| e.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "enterprise-policy",
        "保存企业策略",
        started,
        true,
        Some(0),
        "机器组、可信要求、仓库和字段锁定已保存；旧策略已归档。".into(),
    ))
}

#[tauri::command]
pub(super) fn start_team_sync_task(
    action: String,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    if !["preview", "apply"].contains(&action.as_str()) {
        return Err("团队同步操作无效。".into());
    }
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\sync-team-profiles.ps1");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-Action".into(),
        action.clone(),
    ];
    Ok(start_process_task(
        &state,
        root,
        if action == "apply" {
            "应用团队 Profile".into()
        } else {
            "预览团队 Profile 差异".into()
        },
        format!("team-sync-{action}"),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
pub(super) fn export_audit_bundle() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("audit-{started}"));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    for name in [
        "enterprise-policy.json",
        "profile-sets.json",
        "trusted-catalogs.json",
        "app-update.json",
        "components.json",
    ] {
        let source = greendev(&root, name);
        if source.is_file() {
            fs::copy(&source, stage.join(name)).map_err(|e| e.to_string())?;
        }
    }
    let operations = root.join(r"Logs\GreenDev\operations.jsonl");
    if operations.is_file() {
        fs::copy(operations, stage.join("operations.jsonl")).map_err(|e| e.to_string())?;
    }
    fs::write(
        stage.join("compliance.json"),
        serde_json::to_vec_pretty(&compliance(&root)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let destination = root
        .join("Exports")
        .join(format!("GreenDevAudit-{started}.zip"));
    archive_directory(&stage, &destination)?;
    let _ = fs::remove_dir_all(&stage);
    Ok(finish_operation(
        &root,
        "audit-export",
        "导出审计包",
        started,
        true,
        Some(0),
        format!("审计包: {}", display_path(&destination)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_enterprise_allowlist_accepts_urls() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        assert!(policy_url_allowed(
            &root,
            "allowedFeedHosts",
            "https://updates.example/feed.json"
        ));
    }
    #[test]
    fn compliance_report_has_required_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        let report = compliance(&root);
        assert!(report["checks"].as_array().map(Vec::len).unwrap_or(0) >= 4);
    }
}
