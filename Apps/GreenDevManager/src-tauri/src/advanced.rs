use super::*;

fn greendev_path(root: &Path, name: &str) -> PathBuf {
    root.join(r"Config\greendev").join(name)
}

fn read_json(path: &Path, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(fallback)
}

fn write_json_backup(root: &Path, path: &Path, value: &Value, kind: &str) -> Result<(), String> {
    if path.is_file() {
        let backup = root.join(r"Config\config-backups").join(kind);
        fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
        fs::copy(
            path,
            backup.join(format!("{}-{}.json.bak", kind, now_millis())),
        )
        .map_err(|error| error.to_string())?;
    }
    atomic_config_write(
        path,
        &(serde_json::to_string_pretty(value).map_err(|error| error.to_string())? + "\n"),
    )
}

fn validate_manifest_value(root: &Path, value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if value.get("schemaVersion").and_then(Value::as_u64) != Some(2) {
        errors.push("可视化编辑器要求 Manifest Schema 2。".into());
    }
    let Some(components) = value.get("components").and_then(Value::as_array) else {
        return vec!["components 必须是数组。".into()];
    };
    let mut ids = HashSet::new();
    for (index, item) in components.iter().enumerate() {
        let label = format!("components[{index}]");
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        if id.is_empty()
            || !id
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
        {
            errors.push(format!("{label}.id 仅允许字母、数字、-、_。"));
        }
        if !ids.insert(id.to_string()) {
            errors.push(format!("组件 ID 重复：{id}"));
        }
        for key in ["installDir", "healthPath"] {
            if item
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
            {
                errors.push(format!("{label}.{key} 不能为空。"));
            }
        }
        for key in ["installDir", "currentLink"] {
            if let Some(path) = item
                .get(key)
                .and_then(Value::as_str)
                .filter(|path| !path.is_empty())
            {
                if resolve_relative(root, path).is_err() {
                    errors.push(format!("{label}.{key} 必须位于 Frameworks 内。"));
                }
            }
        }
        let source = item.get("source").unwrap_or(&Value::Null);
        let source_type = source
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("archive");
        if !["archive", "msi"].contains(&source_type) {
            errors.push(format!("{label}.source.type 仅支持 archive/msi。"));
        }
        let archive = source.get("archive").and_then(Value::as_str).unwrap_or("");
        let lower = archive.to_ascii_lowercase();
        if ![".zip", ".7z", ".tar.gz", ".tgz", ".tar.xz", ".msi"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
        {
            errors.push(format!("{label}.source.archive 扩展名不受支持。"));
        }
        if resolve_relative(root, archive).is_err() {
            errors.push(format!("{label}.source.archive 必须位于 Frameworks 内。"));
        }
        if let Some(dependencies) = item.get("dependsOn").and_then(Value::as_array) {
            for dependency in dependencies.iter().filter_map(Value::as_str) {
                if dependency == id {
                    errors.push(format!("{id} 不可依赖自身。"));
                }
            }
        }
    }
    for item in components {
        for dependency in item
            .get("dependsOn")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !ids.contains(dependency) {
                errors.push(format!("依赖定义缺失：{dependency}"));
            }
        }
    }
    errors
}

#[tauri::command]
pub(super) fn get_manifest_editor() -> Result<Value, String> {
    let root = frameworks_root()?;
    let path = greendev_path(&root, "components.json");
    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    Ok(
        json!({"raw": raw, "baseHash": fnv_hash(raw.as_bytes()), "errors": validate_manifest_value(&root, &value), "path": display_path(&path)}),
    )
}

#[tauri::command]
pub(super) fn preview_manifest_editor(raw: String) -> Result<Value, String> {
    let root = frameworks_root()?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|error| format!("JSON 格式错误：{error}"))?;
    let errors = validate_manifest_value(&root, &value);
    Ok(json!({"valid": errors.is_empty(), "errors": errors}))
}

#[tauri::command]
pub(super) fn save_manifest_editor(
    raw: String,
    expected_hash: String,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let path = greendev_path(&root, "components.json");
    let current = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    if fnv_hash(current.as_bytes()) != expected_hash {
        return Err("组件清单已被外部修改，请重新载入后合并。".into());
    }
    let value: Value =
        serde_json::from_str(&raw).map_err(|error| format!("JSON 格式错误：{error}"))?;
    let errors = validate_manifest_value(&root, &value);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    write_json_backup(&root, &path, &value, "manifest")?;
    Ok(finish_operation(
        &root,
        "manifest-edit",
        "保存组件清单",
        started,
        true,
        Some(0),
        format!(
            "已校验并保存 {} 个组件；旧清单已归档。",
            value["components"].as_array().map(Vec::len).unwrap_or(0)
        ),
    ))
}

fn default_trust_policy() -> Value {
    json!({"schemaVersion":1,"requireCatalogSignature":false,"allowLocalManifests":true,"catalogs":[],"pluginPermissions":{"network":false,"process":false,"writeRoots":["downloads\\packages","Caches\\GreenDevManager"]}})
}

#[tauri::command]
pub(super) fn get_trust_policy() -> Result<Value, String> {
    let root = frameworks_root()?;
    Ok(read_json(
        &greendev_path(&root, "trusted-catalogs.json"),
        default_trust_policy(),
    ))
}

#[tauri::command]
pub(super) fn save_trust_policy(policy: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if super::advanced_ops::is_policy_locked(&root, "trustPolicy") {
        return Err("可信目录设置已由企业策略锁定。".into());
    }
    if super::advanced_ops::enterprise_policy(&root)["requireSignedCatalogs"].as_bool()
        == Some(true)
        && policy["requireCatalogSignature"].as_bool() != Some(true)
    {
        return Err("企业策略要求组件目录签名。".into());
    }
    if !policy.get("catalogs").map(Value::is_array).unwrap_or(false) {
        return Err("catalogs 必须是数组。".into());
    }
    write_json_backup(
        &root,
        &greendev_path(&root, "trusted-catalogs.json"),
        &policy,
        "trusted-catalogs",
    )?;
    Ok(finish_operation(
        &root,
        "trust-policy",
        "保存可信目录策略",
        started,
        true,
        Some(0),
        "目录签名要求和插件最小权限已保存。".into(),
    ))
}

fn default_update_settings() -> Value {
    json!({"schemaVersion":1,"channel":"stable","feedUrl":"https://github.com/MetaUoa/GreenDevManager/releases/latest/download/update-feed.json","requireSignature":false,"autoDownload":false})
}

fn local_release_versions(root: &Path) -> Vec<String> {
    let mut values: Vec<_> = fs::read_dir(root.join(r"Releases\GreenDevManager"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    values.sort_by(|left, right| compare_versions(left, right));
    values
}

#[tauri::command]
pub(super) fn get_app_update_status() -> Result<Value, String> {
    let root = frameworks_root()?;
    let settings = read_json(
        &greendev_path(&root, "app-update.json"),
        default_update_settings(),
    );
    let current = env!("CARGO_PKG_VERSION");
    let local = local_release_versions(&root);
    let latest_local = local.last().cloned().unwrap_or_else(|| current.into());
    let feed = read_json(
        &root.join(r"Caches\GreenDevManager\app-update-feed.json"),
        json!({"channels":{}}),
    );
    let channel = settings
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("stable");
    let remote = feed
        .get("channels")
        .and_then(|value| value.get(channel))
        .cloned()
        .unwrap_or(Value::Null);
    let latest_remote = remote.get("version").and_then(Value::as_str).unwrap_or("");
    let candidate = if !latest_remote.is_empty() {
        latest_remote
    } else {
        &latest_local
    };
    let update_available = compare_versions(candidate, current).is_gt();
    let target = if update_available { candidate } else { current };
    Ok(
        json!({"currentVersion":current,"latestLocalVersion":latest_local,"latestRemoteVersion":latest_remote,"targetVersion":target,"updateAvailable":update_available,"settings":settings,"feed":feed,"localVersions":local,"prepared":greendev_path(&root,"pending-app-update.json").is_file()}),
    )
}

#[tauri::command]
pub(super) fn save_app_update_settings(settings: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let channel = settings
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("");
    if super::advanced_ops::is_policy_locked(&root, "appUpdate") {
        return Err("应用更新设置已由企业策略锁定。".into());
    }
    if super::advanced_ops::enterprise_policy(&root)["requireSignedUpdates"].as_bool() == Some(true)
        && settings["requireSignature"].as_bool() != Some(true)
    {
        return Err("企业策略要求应用更新签名。".into());
    }
    if !super::advanced_ops::policy_url_allowed(
        &root,
        "allowedFeedHosts",
        settings
            .get("feedUrl")
            .and_then(Value::as_str)
            .unwrap_or(""),
    ) {
        return Err("更新 Feed 主机不在企业允许列表中。".into());
    }
    if !["stable", "beta", "nightly", "local"].contains(&channel) {
        return Err("更新通道应为 stable、beta、nightly 或 local。".into());
    }
    let feed_url = settings
        .get("feedUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !(feed_url.is_empty() || feed_url.starts_with("https://") || feed_url.starts_with("http://"))
    {
        return Err("Feed URL 应为完整的 HTTP 或 HTTPS 地址。".into());
    }
    write_json_backup(
        &root,
        &greendev_path(&root, "app-update.json"),
        &settings,
        "app-update",
    )?;
    Ok(finish_operation(
        &root,
        "app-update-settings",
        "保存应用更新设置",
        started,
        true,
        Some(0),
        format!("当前通道：{channel}"),
    ))
}

#[tauri::command]
pub(super) fn start_app_feed_task(state: tauri::State<AppState>) -> Result<TaskSnapshot, String> {
    let root = frameworks_root()?;
    let settings = read_json(
        &greendev_path(&root, "app-update.json"),
        default_update_settings(),
    );
    let remote = settings["feedUrl"]
        .as_str()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let script = root.join(r"Scripts\refresh-app-update-feed.ps1");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
    ];
    Ok(start_process_task(
        &state,
        root,
        if remote {
            "联网刷新应用更新源".into()
        } else {
            "刷新本地应用发布".into()
        },
        "app-update-feed".into(),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
pub(super) fn start_app_download_task(
    version: String,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    if !compare_versions(&version, env!("CARGO_PKG_VERSION")).is_gt() {
        return Err("候选版本应高于当前版本。".into());
    }
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\download-app-update.ps1");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-Version".into(),
        version.clone(),
    ];
    Ok(start_process_task(
        &state,
        root,
        format!("下载应用更新 {version}"),
        "app-update-download".into(),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

fn verify_release_directory(
    directory: &Path,
    require_signature: bool,
) -> Result<Vec<String>, String> {
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("release-manifest.json"))
            .map_err(|error| format!("读取发布清单失败：{error}"))?,
    )
    .map_err(|error| error.to_string())?;
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| "发布清单缺少 artifacts。".to_string())?;
    let mut output = Vec::new();
    for artifact in artifacts {
        let name = artifact.get("name").and_then(Value::as_str).unwrap_or("");
        if name.contains(['/', '\\']) {
            return Err("发布物名称越界。".into());
        }
        let path = directory.join(name);
        let expected = artifact.get("sha256").and_then(Value::as_str).unwrap_or("");
        let actual = sha256_file(&path)?;
        if actual != expected.to_uppercase() {
            return Err(format!("发布物哈希不匹配：{name}"));
        }
        output.push(format!("[SHA256 OK] {name}"));
    }
    if require_signature && manifest.get("signed").and_then(Value::as_bool) != Some(true) {
        return Err("更新策略要求签名，但发布清单标记为未签名。".into());
    }
    Ok(output)
}

#[tauri::command]
pub(super) fn prepare_app_update(version: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if !compare_versions(&version, env!("CARGO_PKG_VERSION")).is_gt() {
        return Err("目标版本应高于当前版本。".into());
    }
    let directory = root.join(r"Releases\GreenDevManager").join(&version);
    if !directory.is_dir() {
        return Err("目标版本尚未落入本地 Releases。".into());
    }
    let settings = read_json(
        &greendev_path(&root, "app-update.json"),
        default_update_settings(),
    );
    let lines = verify_release_directory(
        &directory,
        settings
            .get("requireSignature")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    )?;
    let portable = fs::read_dir(&directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with("portable.zip")
        })
        .ok_or_else(|| "发布目录缺少 portable.zip。".to_string())?;
    let stage = root
        .join(r"Caches\GreenDevManager\app-update-stage")
        .join(&version);
    if stage.is_dir() {
        fs::remove_dir_all(&stage).map_err(|error| error.to_string())?;
    }
    extract_archive(&portable.path(), &stage)?;
    let staged_executable = stage.join("GreenDevManager.exe");
    if !staged_executable.is_file() {
        return Err("便携更新包缺少 GreenDevManager.exe。".into());
    }
    if settings
        .get("requireSignature")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let escaped = display_path(&staged_executable).replace('\'', "''");
        let expression =
            format!("(Get-AuthenticodeSignature -LiteralPath '{escaped}').Status -eq 'Valid'");
        let output = background_command(system_program("powershell.exe"))
            .args(["-NoProfile", "-Command", &expression])
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success()
            || !String::from_utf8_lossy(&output.stdout)
                .trim()
                .eq_ignore_ascii_case("true")
        {
            return Err("暂存程序的 Authenticode 签名校验未通过。".into());
        }
    }
    let current_executable = std::env::current_exe()
        .map_err(|error| format!("读取当前程序路径失败：{error}"))?;
    let pending = json!({"schemaVersion":1,"version":version,"preparedAt":now_millis(),"package":display_path(&portable.path()),"stage":display_path(&stage),"currentExecutable":display_path(&current_executable),"state":"verified"});
    write_json_backup(
        &root,
        &greendev_path(&root, "pending-app-update.json"),
        &pending,
        "pending-app-update",
    )?;
    Ok(finish_operation(
        &root,
        "app-update-prepare",
        "准备应用更新",
        started,
        true,
        Some(0),
        format!(
            "版本 {version} 已验证并写入待更新事务。\n{}\n重启前可继续使用当前版本。",
            lines.join("\n")
        ),
    ))
}

#[tauri::command]
pub(super) fn apply_prepared_app_update(app: tauri::AppHandle) -> Result<(), String> {
    let root = frameworks_root()?;
    let pending = greendev_path(&root, "pending-app-update.json");
    if !pending.is_file() {
        return Err("没有已准备的应用更新。".into());
    }
    let script = root.join(r"Scripts\apply-greendev-update.ps1");
    let mut command = background_command(system_program("powershell.exe"));
    command
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &display_path(&script),
            "-PendingPath",
            &display_path(&pending),
            "-CurrentPid",
            &std::process::id().to_string(),
        ])
        .current_dir(&root);
    command
        .spawn()
        .map_err(|error| format!("启动更新引导器失败：{error}"))?;
    app.exit(0);
    Ok(())
}

fn default_profiles() -> Value {
    json!({"schemaVersion":1,"activeProfile":"default","profiles":[{"id":"default","name":"默认开发环境","components":["java","node","python","gradle","maven","rust","mysql"],"teamTemplate":false,"machineOverrides":{}}]})
}

#[tauri::command]
pub(super) fn get_profile_sets() -> Result<Value, String> {
    let root = frameworks_root()?;
    Ok(read_json(
        &greendev_path(&root, "profile-sets.json"),
        default_profiles(),
    ))
}

#[tauri::command]
pub(super) fn save_profile_sets(profiles: Value) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let list = profiles
        .get("profiles")
        .and_then(Value::as_array)
        .ok_or_else(|| "profiles 必须是数组。".to_string())?;
    if super::advanced_ops::is_policy_locked(&root, "profiles") {
        return Err("环境档案已由企业策略锁定。".into());
    }
    let manifest = read_manifest(&root)?;
    let known: HashSet<_> = manifest
        .components
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let mut ids = HashSet::new();
    for profile in list {
        let id = profile.get("id").and_then(Value::as_str).unwrap_or("");
        if id.is_empty() || !ids.insert(id) {
            return Err("Profile ID 为空或重复。".into());
        }
        for component in profile
            .get("components")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !known.contains(component) {
                return Err(format!("Profile {id} 引用了未知组件 {component}。"));
            }
        }
    }
    write_json_backup(
        &root,
        &greendev_path(&root, "profile-sets.json"),
        &profiles,
        "profile-sets",
    )?;
    Ok(finish_operation(
        &root,
        "profiles-save",
        "保存环境档案",
        started,
        true,
        Some(0),
        format!(
            "已保存 {} 个 Profile，机器覆盖项与团队模板标记均保留。",
            list.len()
        ),
    ))
}

fn profile_lock_value(root: &Path, profile_id: &str) -> Result<Value, String> {
    let profiles = read_json(
        &greendev_path(root, "profile-sets.json"),
        default_profiles(),
    );
    let profile = profiles
        .get("profiles")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
        })
        .ok_or_else(|| "Profile 不存在。".to_string())?;
    let selected: HashSet<_> = profile
        .get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let manifest = read_manifest(root)?;
    let pins = read_pins(root);
    let package_lock = read_json(&greendev_path(root, "package-lock.json"), json!({}));
    let mut components = Vec::new();
    for item in manifest
        .components
        .iter()
        .filter(|item| selected.contains(item.id.as_str()))
    {
        let install = resolve_relative(root, &item.install_dir)?;
        let current = item
            .current_link
            .as_deref()
            .and_then(|path| resolve_relative(root, path).ok())
            .and_then(|path| canonical_display(&path));
        components.push(json!({"id":item.id,"version":item.version,"installDir":item.install_dir,"currentTarget":current,"sha256":package_lock.get(&item.id).and_then(|value| value.get("sha256")).and_then(Value::as_str).unwrap_or(&item.source.sha256),"installed":install.join(&item.health_path).is_file(),"pinned":pins.contains_key(&item.id),"dependsOn":item.depends_on}));
    }
    Ok(
        json!({"schemaVersion":1,"profileId":profile_id,"generatedAt":now_millis(),"appVersion":env!("CARGO_PKG_VERSION"),"components":components,"machineOverrides":profile.get("machineOverrides").cloned().unwrap_or(json!({}))}),
    )
}

#[tauri::command]
pub(super) fn build_profile_lock(profile_id: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let lock = profile_lock_value(&root, &profile_id)?;
    let directory = greendev_path(&root, "profile-locks");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{profile_id}.lock.json"));
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&lock).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "profile-lock",
        "生成 Profile 锁文件",
        started,
        true,
        Some(0),
        format!(
            "锁文件: {}\n记录 {} 个组件的版本、哈希、依赖与 current 目标。",
            display_path(&path),
            lock["components"].as_array().map(Vec::len).unwrap_or(0)
        ),
    ))
}

#[tauri::command]
pub(super) fn diff_profile(profile_id: String) -> Result<Value, String> {
    let root = frameworks_root()?;
    let expected = read_json(
        &greendev_path(&root, "profile-locks").join(format!("{profile_id}.lock.json")),
        profile_lock_value(&root, &profile_id)?,
    );
    let actual = profile_lock_value(&root, &profile_id)?;
    let mut rows = Vec::new();
    for item in expected
        .get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        let live = actual
            .get("components")
            .and_then(Value::as_array)
            .and_then(|items| {
                items
                    .iter()
                    .find(|v| v.get("id").and_then(Value::as_str) == Some(id))
            });
        let fields = ["version", "currentTarget", "sha256", "installed"];
        let changes: Vec<_> = fields
            .into_iter()
            .filter(|key| item.get(*key) != live.and_then(|v| v.get(*key)))
            .collect();
        rows.push(json!({"id":id,"state":if changes.is_empty(){"matched"}else{"drifted"},"changes":changes,"expected":item,"actual":live}));
    }
    Ok(
        json!({"profileId":profile_id,"matched":rows.iter().all(|v|v["state"]=="matched"),"rows":rows}),
    )
}

fn contains_secret(name: &str, content: &[u8]) -> bool {
    let lower = name.to_ascii_lowercase();
    if [
        "token",
        "secret",
        "password",
        "credential",
        "private",
        ".key",
        ".pem",
    ]
    .iter()
    .any(|part| lower.contains(part))
    {
        return true;
    }
    let text = String::from_utf8_lossy(content).to_ascii_lowercase();
    [
        "authorization: bearer",
        "client_secret",
        "access_token",
        "private_key",
    ]
    .iter()
    .any(|part| text.contains(part))
}

#[tauri::command]
pub(super) fn export_offline_profile(profile_id: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let lock = profile_lock_value(&root, &profile_id)?;
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("offline-{profile_id}-{started}"));
    fs::create_dir_all(stage.join("packages")).map_err(|e| e.to_string())?;
    fs::write(
        stage.join("profile.lock.json"),
        serde_json::to_vec_pretty(&lock).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let manifest = read_manifest(&root)?;
    let selected: HashSet<_> = lock["components"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(Value::as_str))
        .collect();
    let mut copied = 0;
    for item in manifest
        .components
        .iter()
        .filter(|item| selected.contains(item.id.as_str()))
    {
        let source = resolve_relative(&root, &item.source.archive)?;
        if source.is_file() {
            let bytes = fs::read(&source).map_err(|e| e.to_string())?;
            if contains_secret(
                source.file_name().and_then(|v| v.to_str()).unwrap_or(""),
                &bytes,
            ) {
                continue;
            }
            fs::write(
                stage
                    .join("packages")
                    .join(source.file_name().unwrap_or_default()),
                bytes,
            )
            .map_err(|e| e.to_string())?;
            copied += 1;
        }
    }
    fs::copy(
        greendev_path(&root, "components.json"),
        stage.join("components.json"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(stage.join("README.txt"),"GreenDev offline profile media\r\nPackages are hash-locked; secrets and credentials are excluded.\r\n").map_err(|e|e.to_string())?;
    let destination = root
        .join("Exports")
        .join(format!("GreenDevOffline-{profile_id}-{started}.zip"));
    archive_directory(&stage, &destination)?;
    let _ = fs::remove_dir_all(&stage);
    Ok(finish_operation(
        &root,
        "profile-offline",
        "导出离线环境介质",
        started,
        true,
        Some(0),
        format!(
            "离线包: {}\n收录 {copied} 个本地归档；敏感文件名和凭据特征已排除。",
            display_path(&destination)
        ),
    ))
}

#[tauri::command]
pub(super) fn export_incremental_profile(profile_id: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let lock = profile_lock_value(&root, &profile_id)?;
    let index_path = greendev_path(&root, "offline-index.json");
    let previous: Value = fs::read_to_string(&index_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(json!({"profiles":{}}));
    let old = previous
        .get("profiles")
        .and_then(|v| v.get(&profile_id))
        .cloned()
        .unwrap_or(json!({}));
    let manifest = read_manifest(&root)?;
    let selected: HashSet<_> = lock["components"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(Value::as_str))
        .collect();
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("incremental-{profile_id}-{started}"));
    fs::create_dir_all(stage.join("packages")).map_err(|e| e.to_string())?;
    let mut current = serde_json::Map::new();
    let mut copied = 0;
    for item in manifest
        .components
        .iter()
        .filter(|item| selected.contains(item.id.as_str()))
    {
        let source = resolve_relative(&root, &item.source.archive)?;
        if !source.is_file() {
            continue;
        }
        let hash = sha256_file(&source)?;
        current.insert(item.id.clone(), json!(hash));
        if old.get(&item.id).and_then(Value::as_str) != Some(&hash) {
            fs::copy(
                &source,
                stage
                    .join("packages")
                    .join(source.file_name().unwrap_or_default()),
            )
            .map_err(|e| e.to_string())?;
            copied += 1;
        }
    }
    fs::write(
        stage.join("profile.lock.json"),
        serde_json::to_vec_pretty(&lock).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        stage.join("incremental-index.json"),
        serde_json::to_vec_pretty(
            &json!({"profileId":profile_id,"base":old,"current":current,"generatedAt":started}),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let destination = root
        .join("Exports")
        .join(format!("GreenDevIncremental-{profile_id}-{started}.zip"));
    archive_directory(&stage, &destination)?;
    let _ = fs::remove_dir_all(&stage);
    let mut next = previous;
    if !next["profiles"].is_object() {
        next["profiles"] = json!({});
    }
    next["profiles"][&profile_id] = Value::Object(current);
    atomic_config_write(
        &index_path,
        &(serde_json::to_string_pretty(&next).map_err(|e| e.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "profile-incremental",
        "导出增量离线介质",
        started,
        true,
        Some(0),
        format!(
            "增量包: {}\n包含 {copied} 个相对上次索引发生变化的归档。",
            display_path(&destination)
        ),
    ))
}

#[tauri::command]
pub(super) fn export_supply_chain_inventory() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let manifest = read_manifest(&root)?;
    let locks = read_json(&greendev_path(&root, "package-lock.json"), json!({}));
    let advisories = read_json(
        &greendev_path(&root, "advisories.json"),
        json!({"advisories":[]}),
    );
    let components:Vec<_>=manifest.components.iter().map(|item|json!({"type":"application","name":item.name,"version":item.version,"purl":format!("pkg:generic/{}@{}",item.id,item.version),"sha256":locks.get(&item.id).and_then(|v|v.get("sha256")).and_then(Value::as_str).unwrap_or(&item.source.sha256),"license":"NOASSERTION"})).collect();
    let inventory = json!({"bomFormat":"CycloneDX","specVersion":"1.5","version":1,"metadata":{"timestamp":now_millis(),"component":{"name":"GreenDev Environment","version":env!("CARGO_PKG_VERSION")}},"components":components,"vulnerabilities":advisories["advisories"]});
    let path = root
        .join("Exports")
        .join(format!("GreenDev-SBOM-{started}.cdx.json"));
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(
        &path,
        serde_json::to_vec_pretty(&inventory).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(finish_operation(
        &root,
        "supply-chain",
        "导出供应链清单",
        started,
        true,
        Some(0),
        format!(
            "CycloneDX 清单: {}\n{} 个组件，{} 条本地漏洞通告。",
            display_path(&path),
            components.len(),
            advisories["advisories"]
                .as_array()
                .map(Vec::len)
                .unwrap_or(0)
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_schema_two_manifest_passes_advanced_validation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        let value: Value = serde_json::from_str(
            &fs::read_to_string(root.join(r"Config\greendev\components.json")).unwrap(),
        )
        .unwrap();
        assert!(validate_manifest_value(&root, &value).is_empty());
    }
    #[test]
    fn unsafe_manifest_paths_and_duplicate_ids_are_reported() {
        let root = Path::new(r"D:\Frameworks");
        let value = json!({"schemaVersion":2,"components":[{"id":"x","installDir":"..\\outside","healthPath":"x.exe","source":{"type":"archive","archive":"a.zip"}},{"id":"x","installDir":"Tools\\x","healthPath":"x.exe","source":{"type":"archive","archive":"a.zip"}}]});
        let errors = validate_manifest_value(root, &value);
        assert!(errors.iter().any(|value| value.contains("重复")));
        assert!(errors.iter().any(|value| value.contains("Frameworks")));
    }
    #[test]
    fn secret_scanner_catches_names_and_content() {
        assert!(contains_secret("access-token.txt", b"value"));
        assert!(contains_secret("settings.txt", b"client_secret=fixture"));
        assert!(!contains_secret("component.zip", b"ordinary fixture"));
    }
}
