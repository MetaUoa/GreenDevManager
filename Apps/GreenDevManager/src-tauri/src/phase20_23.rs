use super::*;

fn config_path(root: &Path, name: &str) -> PathBuf {
    root.join(r"Config\greendev").join(name)
}

fn read_value(path: &Path, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(fallback)
}

fn save_policy(root: &Path, name: &str, kind: &str, value: &Value) -> Result<(), String> {
    let path = config_path(root, name);
    if path.is_file() {
        let backup = root.join(r"Config\config-backups").join(kind);
        fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
        fs::copy(
            &path,
            backup.join(format!("{kind}-{}.json.bak", now_millis())),
        )
        .map_err(|error| error.to_string())?;
    }
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(value).map_err(|error| error.to_string())? + "\n"),
    )
}

fn count_named(directory: &Path, suffix: &str) -> usize {
    fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(suffix))
        .count()
}

fn reliability_policy() -> Value {
    json!({
        "schemaVersion": 1,
        "logRotateBytes": 5 * 1024 * 1024,
        "preserveLogArchives": true,
        "resumeInterruptedTasks": true,
        "orphanStageAction": "report",
        "performanceBudgetMs": {"dashboard": 500, "manifest": 150, "logs": 250},
        "releaseChannels": ["stable", "beta", "nightly"]
    })
}

#[tauri::command]
pub(super) fn get_reliability_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    let transactions = transaction_directory(&root);
    let logs = root.join(r"Logs\GreenDev");
    let active_log = logs.join("operations.jsonl");
    let policy = read_value(
        &config_path(&root, "reliability-policy.json"),
        reliability_policy(),
    );
    let baseline = read_value(
        &root.join(r"Caches\GreenDevManager\performance-baseline.json"),
        Value::Null,
    );
    Ok(json!({
        "policy": policy,
        "queue": {
            "pending": count_named(&transactions, ".task.json")
                .saturating_sub(count_named(&transactions, ".completed.task.json"))
                .saturating_sub(count_named(&transactions, ".restarted.task.json")),
            "completed": count_named(&transactions, ".completed.task.json"),
            "restarted": count_named(&transactions, ".restarted.task.json")
        },
        "logs": {
            "activeBytes": fs::metadata(active_log).map(|item| item.len()).unwrap_or(0),
            "archives": count_named(&logs.join("archive"), ".jsonl"),
            "crashBytes": fs::metadata(logs.join("crash.log")).map(|item| item.len()).unwrap_or(0)
        },
        "singleInstance": true,
        "baseline": baseline,
        "generatedAt": now_millis()
    }))
}

#[tauri::command]
pub(super) fn save_reliability_policy(policy: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let rotate = policy["logRotateBytes"].as_u64().unwrap_or(0);
    if !(1024 * 1024..=1024 * 1024 * 1024).contains(&rotate) {
        return Err("日志轮转阈值应在 1 MiB–1 GiB。".into());
    }
    if policy["preserveLogArchives"].as_bool() != Some(true) {
        return Err("绿色环境策略要求保留日志归档。".into());
    }
    save_policy(
        &root,
        "reliability-policy.json",
        "reliability-policy",
        &policy,
    )?;
    Ok(finish_operation(
        &root,
        "reliability-policy",
        "保存可靠性策略",
        started,
        true,
        Some(0),
        format!("日志阈值 {rotate} 字节；归档保留；中断任务重排队。"),
    ))
}

#[tauri::command]
pub(super) fn archive_operation_log() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let logs = root.join(r"Logs\GreenDev");
    let source = logs.join("operations.jsonl");
    if !source.is_file() || fs::metadata(&source).map(|item| item.len()).unwrap_or(0) == 0 {
        return Ok(finish_operation(
            &root,
            "log-archive",
            "归档操作日志",
            started,
            true,
            Some(0),
            "当前日志为空，没有生成归档。".into(),
        ));
    }
    let archive = logs.join("archive");
    fs::create_dir_all(&archive).map_err(|error| error.to_string())?;
    let destination = archive.join(format!("operations-{started}.jsonl"));
    fs::rename(&source, &destination).map_err(|error| error.to_string())?;
    Ok(finish_operation(
        &root,
        "log-archive",
        "归档操作日志",
        started,
        true,
        Some(0),
        format!(
            "已归档到 {}，历史归档完整保留。",
            display_path(&destination)
        ),
    ))
}

#[tauri::command]
pub(super) fn run_performance_baseline() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let measure = |operation: &mut dyn FnMut()| {
        let instant = std::time::Instant::now();
        operation();
        instant.elapsed().as_millis() as u64
    };
    let mut dashboard = || {
        let _ = definitions()
            .into_iter()
            .map(|item| component_status(&root, item))
            .collect::<Vec<_>>();
    };
    let mut manifest = || {
        let _ = read_manifest(&root);
    };
    let mut logs = || {
        let _ = get_operation_logs(Some(150));
    };
    let dashboard_ms = measure(&mut dashboard);
    let manifest_ms = measure(&mut manifest);
    let logs_ms = measure(&mut logs);
    let value = json!({
        "schemaVersion": 1,
        "generatedAt": now_millis(),
        "appVersion": env!("CARGO_PKG_VERSION"),
        "measurementsMs": {"dashboard": dashboard_ms, "manifest": manifest_ms, "logs": logs_ms}
    });
    let path = root.join(r"Caches\GreenDevManager\performance-baseline.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&value).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "performance-baseline",
        "运行性能基线",
        started,
        true,
        Some(0),
        format!(
            "dashboard={dashboard_ms}ms\nmanifest={manifest_ms}ms\nlogs={logs_ms}ms\n{}",
            display_path(&path)
        ),
    ))
}

fn supply_chain_policy() -> Value {
    json!({
        "schemaVersion": 1,
        "requireCodeSigning": false,
        "requireDetachedSignatures": false,
        "requireTrustedChain": false,
        "allowUnsignedLocal": true,
        "trustedSignerThumbprints": [],
        "revokedSignerThumbprints": [],
        "keyRotationDays": 365,
        "allowedLicenses": ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"],
        "deniedLicenses": []
    })
}

fn latest_release(root: &Path) -> Option<PathBuf> {
    let release_root = root.join(r"Releases\GreenDevManager");
    let mut directories: Vec<_> = fs::read_dir(release_root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();
    directories.sort_by(|left, right| {
        compare_versions(
            &left.file_name().to_string_lossy(),
            &right.file_name().to_string_lossy(),
        )
    });
    directories.last().map(|entry| entry.path())
}

#[tauri::command]
pub(super) fn get_supply_chain_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    let policy = read_value(
        &config_path(&root, "supply-chain-policy.json"),
        supply_chain_policy(),
    );
    let release = latest_release(&root);
    let release_path = release
        .as_ref()
        .map(|path| display_path(path))
        .unwrap_or_default();
    let exists = |name: &str| {
        release
            .as_ref()
            .map(|path| path.join(name).is_file())
            .unwrap_or(false)
    };
    let manifest = release
        .as_ref()
        .map(|path| read_value(&path.join("release-manifest.json"), Value::Null))
        .unwrap_or(Value::Null);
    Ok(json!({
        "policy": policy,
        "releasePath": release_path,
        "releaseVersion": manifest["version"].as_str().unwrap_or(""),
        "checks": [
            {"id":"manifest","healthy":exists("release-manifest.json"),"detail":"发布清单"},
            {"id":"signature","healthy":exists("release-manifest.json.sig.json") || policy["requireDetachedSignatures"].as_bool()!=Some(true),"detail":"分离签名"},
            {"id":"provenance","healthy":exists("provenance.json"),"detail":"构建来源证明"},
            {"id":"sbom","healthy":exists("release-sbom.cdx.json"),"detail":"CycloneDX SBOM"},
            {"id":"codesign","healthy":manifest["signed"].as_bool()==Some(true) || policy["requireCodeSigning"].as_bool()!=Some(true),"detail":"Windows 代码签名"}
        ],
        "generatedAt": now_millis()
    }))
}

#[tauri::command]
pub(super) fn save_supply_chain_policy(policy: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if policy["keyRotationDays"].as_u64().unwrap_or(0) < 30 {
        return Err("密钥轮换周期至少为 30 天。".into());
    }
    for key in [
        "trustedSignerThumbprints",
        "revokedSignerThumbprints",
        "allowedLicenses",
        "deniedLicenses",
    ] {
        if !policy[key].is_array() {
            return Err(format!("{key} 必须是数组。"));
        }
    }
    save_policy(
        &root,
        "supply-chain-policy.json",
        "supply-chain-policy",
        &policy,
    )?;
    Ok(finish_operation(
        &root,
        "supply-chain-policy",
        "保存供应链策略",
        started,
        true,
        Some(0),
        "签名信任、吊销、密钥轮换与许可证规则已保存。".into(),
    ))
}

#[tauri::command]
pub(super) fn verify_supply_chain() -> Result<OperationResult, String> {
    let root = frameworks_root()?;
    let release = latest_release(&root).ok_or_else(|| "没有本地发布目录。".to_string())?;
    run_batch(
        "supply-chain-verify",
        "验证发布供应链",
        "verify-greendev-signature.ps1",
        &[
            "-Path",
            &display_path(&release),
            "-PolicyPath",
            &display_path(&config_path(&root, "supply-chain-policy.json")),
        ],
    )
}

fn fleet_default() -> Value {
    json!({
        "schemaVersion": 1,
        "agentProtocol": 1,
        "defaultBatchPercent": 20,
        "maintenanceWindow": {"start":"02:00","end":"05:00"},
        "nodes": [],
        "rollouts": []
    })
}

fn validate_fleet(value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let mut ids = HashSet::new();
    let Some(nodes) = value["nodes"].as_array() else {
        return vec!["nodes 必须是数组。".into()];
    };
    for node in nodes {
        let id = node["id"].as_str().unwrap_or("");
        if id.is_empty() || !ids.insert(id.to_ascii_lowercase()) {
            errors.push(format!("节点 ID 缺失或重复：{id}"));
        }
        if !["local", "winrm", "agent"].contains(&node["transport"].as_str().unwrap_or("")) {
            errors.push(format!("节点 {id} transport 仅支持 local/winrm/agent。"));
        }
        if node["credentialRef"]
            .as_str()
            .unwrap_or("")
            .contains("password")
        {
            errors.push(format!("节点 {id} 应使用凭据引用，不在配置中保存口令。"));
        }
    }
    errors
}

fn fleet_rollouts(root: &Path) -> Vec<Value> {
    let directory = root.join(r"Caches\GreenDevManager\fleet-rollouts");
    let mut values: Vec<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                && !entry.file_name().to_string_lossy().contains(".before-")
        })
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|text| serde_json::from_str::<Value>(&text).ok())
        .collect();
    values.sort_by_key(|value| {
        std::cmp::Reverse(
            value["events"]
                .as_array()
                .and_then(|events| events.last())
                .and_then(|event| event["at"].as_u64())
                .unwrap_or(0),
        )
    });
    values
}

#[tauri::command]
pub(super) fn get_fleet_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    let value = read_value(&config_path(&root, "remote-nodes.json"), fleet_default());
    let errors = validate_fleet(&value);
    let nodes = value["nodes"].as_array().cloned().unwrap_or_default();
    let inventory = read_value(
        &root.join(r"Caches\GreenDevManager\fleet-inventory.json"),
        json!({"schemaVersion":1,"generatedAt":0,"nodes":[]}),
    );
    let online = inventory["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| node["status"] == "online")
        .count();
    Ok(json!({
        "config": value,
        "nodeCount": nodes.len(),
        "onlineCount": online,
        "errors": errors,
        "rollouts": fleet_rollouts(&root),
        "inventory": inventory,
        "generatedAt": now_millis()
    }))
}

#[tauri::command]
pub(super) fn start_fleet_inventory_task(
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\greendev-fleet-inventory.ps1");
    let output = root.join(r"Caches\GreenDevManager\fleet-inventory.json");
    Ok(start_process_task(
        &state,
        root,
        "采集远程只读清单".into(),
        "fleet-inventory".into(),
        "powershell.exe".into(),
        vec![
            "-NoProfile".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-File".into(),
            display_path(&script),
        ],
        vec![],
        Some(output),
    ))
}

#[tauri::command]
pub(super) fn save_fleet_config(config: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let errors = validate_fleet(&config);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    save_policy(&root, "remote-nodes.json", "remote-nodes", &config)?;
    Ok(finish_operation(
        &root,
        "fleet-config",
        "保存远程节点配置",
        started,
        true,
        Some(0),
        format!(
            "已保存 {} 个节点；敏感值仅接受凭据引用。",
            config["nodes"].as_array().map(Vec::len).unwrap_or(0)
        ),
    ))
}

#[tauri::command]
pub(super) fn preview_fleet_rollout(request: Value) -> Result<Value, String> {
    let root = frameworks_root()?;
    let config = read_value(&config_path(&root, "remote-nodes.json"), fleet_default());
    let group = request["group"].as_str().unwrap_or("");
    let tags: HashSet<_> = request["tags"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let selected: Vec<_> = config["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| group.is_empty() || node["group"] == group)
        .filter(|node| {
            tags.is_empty()
                || node["tags"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|tag| tags.contains(tag))
        })
        .cloned()
        .collect();
    let percent = request["batchPercent"]
        .as_u64()
        .or_else(|| config["defaultBatchPercent"].as_u64())
        .unwrap_or(20)
        .clamp(1, 100) as usize;
    let batch_size = (selected.len() * percent).div_ceil(100);
    let batches: Vec<Value> = selected
        .chunks(batch_size.max(1))
        .enumerate()
        .map(|(index, values)| json!({"index":index+1,"nodes":values.iter().map(|node|node["id"].clone()).collect::<Vec<_>>()}))
        .collect();
    Ok(json!({
        "id": operation_id("rollout-plan"),
        "componentId": request["componentId"],
        "version": request["version"],
        "group": group,
        "nodeCount": selected.len(),
        "batchPercent": percent,
        "batches": batches,
        "maintenanceWindow": config["maintenanceWindow"],
        "approvalRequired": true,
        "rollbackOnFailure": true,
        "createdAt": now_millis()
    }))
}

#[tauri::command]
pub(super) fn stage_fleet_rollout(plan: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if plan["approvalRequired"].as_bool() != Some(true) || !plan["batches"].is_array() {
        return Err("发布计划结构无效。".into());
    }
    let directory = root.join(r"Caches\GreenDevManager\fleet-rollouts");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let id = plan["id"].as_str().unwrap_or("rollout");
    let record = json!({"schemaVersion":1,"status":"awaiting-approval","plan":plan,"events":[{"at":started,"state":"staged","detail":"已暂存，等待审批"}]});
    let path = directory.join(format!("{id}.json"));
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&record).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "fleet-rollout",
        "暂存分批发布",
        started,
        true,
        Some(0),
        format!(
            "发布事务已保存：{}\n当前状态：awaiting-approval；尚未修改远程节点。",
            display_path(&path)
        ),
    ))
}

#[tauri::command]
pub(super) fn set_fleet_rollout_state(
    id: String,
    action: String,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let directory = root.join(r"Caches\GreenDevManager\fleet-rollouts");
    let path = directory.join(format!("{id}.json"));
    let mut record = read_value(&path, Value::Null);
    if record.is_null() {
        return Err("分批发布事务不存在。".into());
    }
    let current = record["status"].as_str().unwrap_or("").to_string();
    let next = match (current.as_str(), action.as_str()) {
        ("awaiting-approval", "approve") => "approved",
        ("approved" | "running", "pause") => "paused",
        ("paused", "resume") => "approved",
        ("approved" | "paused" | "running" | "failed" | "completed", "rollback") => {
            "rollback-requested"
        }
        _ => return Err(format!("状态转换不适用：{current} -> {action}")),
    };
    fs::copy(&path, directory.join(format!("{id}.before-{started}.json")))
        .map_err(|error| error.to_string())?;
    record["status"] = Value::String(next.into());
    let events = record
        .get_mut("events")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "发布事件列表缺失。".to_string())?;
    events.push(json!({"at":started,"state":next,"detail":format!("状态操作：{action}")}));
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&record).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "fleet-state",
        "更新分批发布状态",
        started,
        true,
        Some(0),
        format!("{id}: {current} -> {next}\n修改前记录已归档。"),
    ))
}

#[tauri::command]
pub(super) fn start_fleet_rollout_task(
    id: String,
    action: String,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    if !["apply", "rollback"].contains(&action.as_str()) {
        return Err("远程执行仅支持 apply 或 rollback。".into());
    }
    let root = frameworks_root()?;
    let record = read_value(
        &root
            .join(r"Caches\GreenDevManager\fleet-rollouts")
            .join(format!("{id}.json")),
        Value::Null,
    );
    let required = if action == "apply" {
        "approved"
    } else {
        "rollback-requested"
    };
    if record["status"] != required {
        return Err(format!("事务需处于 {required} 状态。"));
    }
    let script = root.join(r"Scripts\greendev-fleet.ps1");
    Ok(start_process_task(
        &state,
        root,
        if action == "apply" {
            "执行分批发布".into()
        } else {
            "执行远程回滚".into()
        },
        format!("fleet-{action}"),
        "powershell.exe".into(),
        vec![
            "-NoProfile".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-File".into(),
            display_path(&script),
            "-Id".into(),
            id,
            "-Action".into(),
            action,
        ],
        vec![],
        None,
    ))
}

#[tauri::command]
pub(super) fn get_ecosystem_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    Ok(json!({
        "manifestSchema": display_path(&config_path(&root, r"schema\components.schema.json")),
        "example": display_path(&config_path(&root, r"examples\custom-component.json")),
        "commands": ["New-GreenDevManifest.ps1", "Test-GreenDevPlugin.ps1", "greendev completion powershell", "greendev completion cmd"],
        "locales": ["zh-CN", "en-US"],
        "pluginPermissions": ["network", "process", "writeRoots"],
        "generatedAt": now_millis()
    }))
}

#[tauri::command]
pub(super) fn generate_manifest_template(component_id: String) -> Result<OperationResult, String> {
    if component_id.is_empty()
        || !component_id
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
    {
        return Err("组件 ID 仅允许字母、数字、-、_。".into());
    }
    run_batch(
        "manifest-sdk",
        "生成 Manifest 模板",
        "New-GreenDevManifest.ps1",
        &["-Id", &component_id],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_rejects_duplicate_nodes_and_inline_passwords() {
        let value = json!({"nodes":[
            {"id":"node-a","transport":"agent","credentialRef":"password-inline"},
            {"id":"NODE-A","transport":"agent","credentialRef":"vault:node-a"}
        ]});
        let errors = validate_fleet(&value);
        assert!(errors.iter().any(|item| item.contains("重复")));
        assert!(errors.iter().any(|item| item.contains("凭据引用")));
    }

    #[test]
    fn empty_fleet_is_a_valid_starting_point() {
        assert!(validate_fleet(&fleet_default()).is_empty());
    }
}
