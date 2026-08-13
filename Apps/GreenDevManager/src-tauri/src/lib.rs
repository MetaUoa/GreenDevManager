use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
mod advanced;
mod advanced_ops;
mod phase20_23;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ComponentStatus {
    id: String,
    name: String,
    category: String,
    installed: bool,
    healthy: bool,
    version: String,
    executable: String,
    current_path: Option<String>,
    current_target: Option<String>,
    environment_variables: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheEntry {
    id: String,
    name: String,
    path: String,
    size_bytes: u64,
    protected: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dashboard {
    root: String,
    total_size_bytes: u64,
    cache_size_bytes: u64,
    storage_ready: bool,
    installed_count: usize,
    healthy_count: usize,
    components: Vec<ComponentStatus>,
    caches: Vec<CacheEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageMetrics {
    total_size_bytes: u64,
    cache_size_bytes: u64,
    caches: Vec<CacheEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperationResult {
    operation_id: String,
    success: bool,
    title: String,
    summary: String,
    output: String,
    exit_code: Option<i32>,
    kind: String,
    started_at: u64,
    finished_at: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigStatus {
    id: String,
    name: String,
    source_path: String,
    deployed_path: Option<String>,
    source_hash: String,
    deployed_hash: Option<String>,
    state: String,
    detail: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigField {
    key: String,
    label: String,
    value: String,
    kind: String,
    help: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigBackup {
    file_name: String,
    created_at: String,
    size_bytes: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigDocument {
    id: String,
    name: String,
    format: String,
    source_path: String,
    raw: String,
    base_hash: String,
    fields: Vec<ConfigField>,
    backups: Vec<ConfigBackup>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigPreview {
    valid: bool,
    errors: Vec<String>,
    diff: String,
    rendered: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentBackup {
    file_name: String,
    path: String,
    created_at: String,
    variable_count: usize,
    root: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionEntry {
    version: String,
    path: String,
    current: bool,
    pinned: bool,
    healthy: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionInventory {
    component_id: String,
    component_name: String,
    supports_switching: bool,
    current_path: Option<String>,
    versions: Vec<VersionEntry>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AndroidPackage {
    id: String,
    version: String,
    description: String,
    installed: bool,
    obsolete: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestComponentStatus {
    id: String,
    name: String,
    desired_version: String,
    install_dir: String,
    current_link: Option<String>,
    source_url: String,
    archive_path: String,
    enabled: bool,
    installed: bool,
    active: bool,
    archive_cached: bool,
    checksum_ready: bool,
    pinned_elsewhere: bool,
    dependencies: Vec<String>,
    blocked_reason: String,
    state: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallPlan {
    component_id: String,
    action: String,
    steps: Vec<String>,
    blockers: Vec<String>,
    archive_path: String,
    source_url: String,
    expected_sha256: String,
    ready: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct InstallSettings {
    proxy_url: String,
    mirrors: HashMap<String, Vec<String>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticItem {
    id: String,
    name: String,
    healthy: bool,
    detail: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticReport {
    app_version: String,
    generated_at: u64,
    healthy_count: usize,
    items: Vec<DiagnosticItem>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshot {
    id: String,
    title: String,
    kind: String,
    status: String,
    progress: u8,
    message: String,
    cancelable: bool,
    pausable: bool,
    retryable: bool,
    stage: String,
    bytes_processed: u64,
    bytes_total: u64,
    bytes_per_second: u64,
    eta_seconds: Option<u64>,
    attempt: u32,
    priority: u8,
    scheduled_at: u64,
    queue_position: usize,
    timeline: Vec<TaskEvent>,
    started_at: u64,
    updated_at: u64,
    result: Option<OperationResult>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskEvent {
    at: u64,
    stage: String,
    message: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskPolicy {
    max_concurrent: usize,
    default_priority: u8,
    notifications: bool,
}

impl Default for TaskPolicy {
    fn default() -> Self {
        Self {
            max_concurrent: 2,
            default_priority: 50,
            notifications: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoragePoint {
    recorded_at: u64,
    total_size_bytes: u64,
    cache_size_bytes: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogCandidate {
    id: String,
    provider: String,
    version: String,
    architecture: String,
    channel: String,
    url: String,
    sha256: String,
    archive_root: String,
    install_dir: String,
    archive_path: String,
    component_name: String,
    notes: String,
    checksum_ready: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCandidate {
    component_id: String,
    name: String,
    current_version: String,
    target_version: String,
    update_available: bool,
    installed: bool,
    active: bool,
    pinned: bool,
    checksum_ready: bool,
    catalog_available: bool,
    install_ready: bool,
    can_adopt: bool,
    policy: String,
    release_notes: String,
    candidates: Vec<CatalogCandidate>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchInstallPlan {
    component_ids: Vec<String>,
    ordered_ids: Vec<String>,
    steps: Vec<String>,
    blockers: Vec<String>,
    ready: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupPreview {
    file_name: String,
    content: String,
    source_hash: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MaintenanceStatus {
    current_version: String,
    latest_local_version: String,
    update_available: bool,
    release_directory: String,
    crash_log: String,
    pending_transactions: usize,
    build_mode: String,
}

struct TaskEntry {
    snapshot: TaskSnapshot,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    spec: TaskSpec,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskSpec {
    root: PathBuf,
    title: String,
    kind: String,
    program: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    cache_output: Option<PathBuf>,
    attempt: u32,
    priority: u8,
    scheduled_at: u64,
    #[serde(default)]
    start_paused: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedTask {
    schema_version: u32,
    snapshot: TaskSnapshot,
    spec: TaskSpec,
}

#[derive(Default)]
struct AppState {
    tasks: Arc<Mutex<HashMap<String, TaskEntry>>>,
}

#[derive(Clone, Copy)]
struct ComponentDefinition {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    executable: &'static str,
    current: Option<&'static str>,
    variables: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct VersionDefinition {
    id: &'static str,
    name: &'static str,
    base: &'static str,
    current: &'static str,
    health: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestDocument {
    schema_version: u32,
    components: Vec<ManifestComponent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestComponent {
    id: String,
    name: String,
    version: String,
    enabled: bool,
    install_dir: String,
    current_link: Option<String>,
    health_path: String,
    #[serde(default)]
    #[allow(dead_code)]
    archive_root: String,
    #[serde(default)]
    depends_on: Vec<String>,
    source: ManifestSource,
}

#[derive(Deserialize)]
struct ManifestSource {
    #[serde(rename = "type")]
    source_type: String,
    url: String,
    archive: String,
    #[serde(default)]
    sha256: String,
}

fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition {
            id: "java",
            name: "Java",
            category: "运行时",
            executable: r"Runtimes\Java\current\bin\java.exe",
            current: Some(r"Runtimes\Java\current"),
            variables: &["JAVA_HOME"],
        },
        ComponentDefinition {
            id: "node",
            name: "Node.js / npm",
            category: "运行时",
            executable: r"Runtimes\Node\current\node.exe",
            current: Some(r"Runtimes\Node\current"),
            variables: &["NODE_HOME", "npm_config_cache", "npm_config_registry"],
        },
        ComponentDefinition {
            id: "python",
            name: "Python / pip",
            category: "运行时",
            executable: r"Runtimes\Python\current\python.exe",
            current: Some(r"Runtimes\Python\current"),
            variables: &["PIP_CACHE_DIR", "PIP_INDEX_URL"],
        },
        ComponentDefinition {
            id: "gradle",
            name: "Gradle",
            category: "构建工具",
            executable: r"BuildTools\Gradle\current\bin\gradle.bat",
            current: Some(r"BuildTools\Gradle\current"),
            variables: &["GRADLE_HOME", "GRADLE_USER_HOME"],
        },
        ComponentDefinition {
            id: "maven",
            name: "Maven",
            category: "构建工具",
            executable: r"BuildTools\Maven\current\bin\mvn.cmd",
            current: Some(r"BuildTools\Maven\current"),
            variables: &["MAVEN_HOME", "MAVEN_OPTS"],
        },
        ComponentDefinition {
            id: "android",
            name: "Android SDK",
            category: "平台",
            executable: r"Platforms\Android\Sdk\platform-tools\adb.exe",
            current: None,
            variables: &["ANDROID_HOME", "ANDROID_SDK_ROOT", "ANDROID_USER_HOME"],
        },
        ComponentDefinition {
            id: "rust",
            name: "Rust / Cargo",
            category: "工具链",
            executable: r"Toolchains\Rust\current\bin\rustc.exe",
            current: Some(r"Toolchains\Rust\current"),
            variables: &["RUST_HOME", "CARGO_HOME", "CARGO_TARGET_DIR"],
        },
        ComponentDefinition {
            id: "c",
            name: "C / GCC",
            category: "工具链",
            executable: r"Toolchains\C\mingw64\bin\gcc.exe",
            current: None,
            variables: &[],
        },
        ComponentDefinition {
            id: "acpi",
            name: "ACPI / iasl",
            category: "工具链",
            executable: r"Toolchains\ACPI\iasl\iasl.exe",
            current: None,
            variables: &[],
        },
        ComponentDefinition {
            id: "mysql",
            name: "MySQL",
            category: "数据库",
            executable: r"Databases\Sql\mysql\current\bin\mysql.exe",
            current: Some(r"Databases\Sql\mysql\current"),
            variables: &["MYSQL_HOME"],
        },
        ComponentDefinition {
            id: "ghidra",
            name: "Ghidra",
            category: "逆向工具",
            executable: r"ReverseTools\Ghidra\ghidraRun.bat",
            current: None,
            variables: &[],
        },
    ]
}

fn version_definitions() -> Vec<VersionDefinition> {
    vec![
        VersionDefinition {
            id: "java",
            name: "Java",
            base: r"Runtimes\Java",
            current: r"Runtimes\Java\current",
            health: r"bin\java.exe",
        },
        VersionDefinition {
            id: "node",
            name: "Node.js",
            base: r"Runtimes\Node",
            current: r"Runtimes\Node\current",
            health: "node.exe",
        },
        VersionDefinition {
            id: "python",
            name: "Python",
            base: r"Runtimes\Python",
            current: r"Runtimes\Python\current",
            health: "python.exe",
        },
        VersionDefinition {
            id: "gradle",
            name: "Gradle",
            base: r"BuildTools\Gradle",
            current: r"BuildTools\Gradle\current",
            health: r"bin\gradle.bat",
        },
        VersionDefinition {
            id: "maven",
            name: "Maven",
            base: r"BuildTools\Maven",
            current: r"BuildTools\Maven\current",
            health: r"bin\mvn.cmd",
        },
        VersionDefinition {
            id: "rust",
            name: "Rust",
            base: r"Toolchains\Rust",
            current: r"Toolchains\Rust\current",
            health: r"bin\rustc.exe",
        },
        VersionDefinition {
            id: "mysql",
            name: "MySQL",
            base: r"Databases\Sql\mysql",
            current: r"Databases\Sql\mysql\current",
            health: r"bin\mysql.exe",
        },
    ]
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn operation_id(kind: &str) -> String {
    format!(
        "{}-{}-{}",
        kind,
        now_millis(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn version_key(value: &str) -> Vec<u64> {
    value
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    match (
        semver::Version::parse(left.trim_start_matches('v')),
        semver::Version::parse(right.trim_start_matches('v')),
    ) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => version_key(left).cmp(&version_key(right)),
    }
}

const BOOTSTRAP_MANIFEST_URL: &str =
    "https://github.com/MetaUoa/GreenDevManager/releases/latest/download/bootstrap-manifest.json";

fn bootstrap_config_path() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("GreenDevManager")
        .join("root.json")
}

fn saved_frameworks_root() -> Option<PathBuf> {
    let value: Value =
        serde_json::from_str(&fs::read_to_string(bootstrap_config_path()).ok()?).ok()?;
    let path = PathBuf::from(value.get("root")?.as_str()?);
    is_frameworks_root(&path).then_some(path)
}

fn frameworks_root() -> Result<PathBuf, String> {
    if let Ok(value) = env::var("FRAMEWORKS_HOME") {
        let path = PathBuf::from(value);
        if is_frameworks_root(&path) {
            return path.canonicalize().map_err(|error| error.to_string());
        }
    }
    if env::var_os("GREENDEV_DISABLE_SAVED_ROOT").is_none() {
        if let Some(path) = saved_frameworks_root() {
            return path.canonicalize().map_err(|error| error.to_string());
        }
    }
    let mut candidates = Vec::new();
    if let Ok(current) = env::current_dir() {
        candidates.push(current);
    }
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    for candidate in candidates {
        for ancestor in candidate.ancestors() {
            if is_frameworks_root(ancestor) {
                return ancestor.canonicalize().map_err(|error| error.to_string());
            }
        }
    }
    Err("未找到 Frameworks 根目录，请设置 FRAMEWORKS_HOME。".into())
}

fn is_frameworks_root(path: &Path) -> bool {
    path.join("Scripts").is_dir() && path.join("env-setup.bat").is_file()
}

fn persist_frameworks_root(path: &Path) -> Result<(), String> {
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    let config_path = bootstrap_config_path();
    let parent = config_path
        .parent()
        .ok_or_else(|| "根目录配置路径无效。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = parent.join(format!("root-{}.tmp", now_millis()));
    let content = serde_json::to_vec_pretty(&json!({
        "schemaVersion": 1,
        "root": display_path(&canonical),
        "savedAt": now_millis()
    }))
    .map_err(|error| error.to_string())?;
    if config_path.is_file() {
        atomic_config_write(
            &config_path,
            &String::from_utf8(content).map_err(|error| error.to_string())?,
        )?;
    } else {
        fs::write(&temporary, content).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &config_path).map_err(|error| error.to_string())?;
    }
    env::set_var("FRAMEWORKS_HOME", &canonical);
    let _ = background_command(system_program("reg.exe"))
        .args([
            "add",
            r"HKCU\Environment",
            "/v",
            "FRAMEWORKS_HOME",
            "/t",
            "REG_SZ",
            "/d",
            &display_path(&canonical),
            "/f",
        ])
        .output();
    Ok(())
}

fn copy_bootstrap_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!("初始化包包含符号链接：{}", entry.path().display()));
        }
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_bootstrap_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn download_file(url: &str, destination: &Path) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("初始化下载地址必须使用 HTTPS。".into());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let output = background_command(system_program("curl.exe"))
        .args([
            "--fail",
            "--location",
            "--retry",
            "3",
            "--connect-timeout",
            "15",
            "--output",
            &display_path(destination),
            url,
        ])
        .output()
        .map_err(|error| format!("启动下载失败：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "下载失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn safe_bootstrap_entry(value: &str) -> bool {
    let entry = Path::new(value.trim());
    !entry.as_os_str().is_empty()
        && !entry.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn validate_bootstrap_archive(path: &Path) -> Result<(), String> {
    let output = background_command(system_program("tar.exe"))
        .args(["-t", "-f", &display_path(path)])
        .output()
        .map_err(|error| format!("读取初始化包目录失败：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !safe_bootstrap_entry(line) {
            return Err(format!("初始化包包含越界路径：{line}"));
        }
    }
    Ok(())
}

#[tauri::command]
fn get_bootstrap_status() -> Value {
    match frameworks_root() {
        Ok(root) => json!({
            "configured": true,
            "root": display_path(&root),
            "currentVersion": env!("CARGO_PKG_VERSION"),
            "manifestUrl": BOOTSTRAP_MANIFEST_URL
        }),
        Err(_) => json!({
            "configured": false,
            "root": "",
            "currentVersion": env!("CARGO_PKG_VERSION"),
            "manifestUrl": BOOTSTRAP_MANIFEST_URL
        }),
    }
}

#[tauri::command]
fn select_frameworks_directory() -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        let script = r#"Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; $dialog.Description = 'Select or create a GreenDev environment directory'; $dialog.ShowNewFolderButton = $true; if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { [Console]::OutputEncoding = [Text.Encoding]::UTF8; [Console]::Write($dialog.SelectedPath) }"#;
        let output = background_command(system_program("powershell.exe"))
            .args(["-NoProfile", "-STA", "-Command", script])
            .output()
            .map_err(|error| format!("打开目录选择器失败：{error}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok((!selected.is_empty()).then_some(selected))
    }
    #[cfg(not(windows))]
    Ok(None)
}

#[tauri::command]
fn initialize_frameworks_root(
    path: String,
    mode: String,
    state: tauri::State<AppState>,
) -> Result<Value, String> {
    let selected = PathBuf::from(path.trim());
    if !selected.is_absolute() {
        return Err("请选择绝对目录。".into());
    }
    match mode.as_str() {
        "existing" => {
            if !is_frameworks_root(&selected) {
                return Err(
                    "所选目录不是现有 GreenDev 环境：缺少 Scripts 和 env-setup.bat。".into(),
                );
            }
        }
        "fresh" => {
            fs::create_dir_all(&selected).map_err(|error| format!("创建目录失败：{error}"))?;
            if fs::read_dir(&selected)
                .map_err(|error| error.to_string())?
                .next()
                .is_some()
            {
                return Err("全新初始化需要选择空目录，以保护已有文件。".into());
            }
            let cache = bootstrap_config_path()
                .parent()
                .ok_or_else(|| "初始化缓存路径无效。".to_string())?
                .join("bootstrap");
            fs::create_dir_all(&cache).map_err(|error| error.to_string())?;
            let manifest_url = env::var("GREENDEV_BOOTSTRAP_MANIFEST_URL")
                .unwrap_or_else(|_| BOOTSTRAP_MANIFEST_URL.into());
            let manifest_path = cache.join("bootstrap-manifest.json");
            download_file(&manifest_url, &manifest_path)?;
            let manifest: Value = serde_json::from_str(
                &fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?,
            )
            .map_err(|error| format!("初始化清单格式错误：{error}"))?;
            let version = manifest["version"]
                .as_str()
                .ok_or_else(|| "初始化清单缺少 version。".to_string())?;
            let url = manifest["url"]
                .as_str()
                .ok_or_else(|| "初始化清单缺少 url。".to_string())?;
            let expected = manifest["sha256"]
                .as_str()
                .ok_or_else(|| "初始化清单缺少 sha256。".to_string())?
                .to_uppercase();
            let archive = cache.join(format!("GreenDevManager-bootstrap-{version}.zip"));
            download_file(url, &archive)?;
            let actual = sha256_file(&archive)?;
            if actual != expected {
                return Err(format!("初始化包 SHA-256 校验失败：{actual}"));
            }
            validate_bootstrap_archive(&archive)?;
            let stage = cache.join(format!("stage-{version}"));
            if stage.is_dir() {
                fs::remove_dir_all(&stage).map_err(|error| error.to_string())?;
            }
            extract_archive(&archive, &stage)?;
            if !is_frameworks_root(&stage) {
                return Err("初始化包结构不完整。".into());
            }
            copy_bootstrap_tree(&stage, &selected)?;
            if !is_frameworks_root(&selected) {
                return Err("初始化后的目录校验失败。".into());
            }
        }
        _ => return Err("初始化模式无效。".into()),
    }
    persist_frameworks_root(&selected)?;
    let root = frameworks_root()?;
    let _ = restore_persisted_tasks(&state, &root);
    recover_transactions(&root);
    Ok(json!({
        "configured": true,
        "root": display_path(&root),
        "currentVersion": env!("CARGO_PKG_VERSION"),
        "manifestUrl": BOOTSTRAP_MANIFEST_URL,
        "mode": mode
    }))
}

fn system_program(name: &str) -> PathBuf {
    let windows = env::var_os("WINDIR")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let candidate = match name.to_ascii_lowercase().as_str() {
        "cmd.exe" => windows.join(r"System32\cmd.exe"),
        "powershell.exe" => windows.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
        "curl.exe" => windows.join(r"System32\curl.exe"),
        "reg.exe" => windows.join(r"System32\reg.exe"),
        "tar.exe" => windows.join(r"System32\tar.exe"),
        _ => PathBuf::from(name),
    };
    if candidate.is_file() {
        candidate
    } else {
        PathBuf::from(name)
    }
}

fn background_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_file() => entry.metadata().map(|meta| meta.len()).unwrap_or(0),
                Ok(kind) if kind.is_dir() && !kind.is_symlink() => directory_size(&path),
                _ => 0,
            }
        })
        .sum()
}

fn display_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
}

fn canonical_display(path: &Path) -> Option<String> {
    fs::canonicalize(path)
        .ok()
        .map(|value| display_path(&value))
}

fn component_status(root: &Path, definition: ComponentDefinition) -> ComponentStatus {
    let executable = root.join(definition.executable);
    let current_path = definition.current.map(|value| root.join(value));
    let current_target = current_path
        .as_ref()
        .and_then(|path| canonical_display(path));
    let version = current_target
        .as_ref()
        .and_then(|target| Path::new(target).file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            if definition.id == "android" {
                "SDK".into()
            } else if definition.id == "ghidra" {
                "Standalone".into()
            } else {
                "固定入口".into()
            }
        });
    let installed = executable.is_file();
    ComponentStatus {
        id: definition.id.into(),
        name: definition.name.into(),
        category: definition.category.into(),
        installed,
        healthy: installed
            && current_path
                .as_ref()
                .map(|path| path.exists())
                .unwrap_or(true),
        version,
        executable: display_path(&executable),
        current_path: current_path.as_ref().map(|path| display_path(path)),
        current_target,
        environment_variables: definition
            .variables
            .iter()
            .map(|value| (*value).into())
            .collect(),
    }
}

fn cache_entries(root: &Path, calculate: bool) -> Vec<CacheEntry> {
    [
        ("gradle", "Gradle", r"Caches\Gradle", false),
        ("maven", "Maven 本地仓库", r"Caches\Maven\repository", true),
        ("npm", "npm", r"Caches\npm", false),
        ("pip", "pip", r"Caches\pip", false),
        ("rust", "Rust target", r"Caches\Rust\target", false),
        ("android", "Android 用户缓存", r"Caches\Android", false),
    ]
    .into_iter()
    .map(|(id, name, relative, protected)| {
        let path = root.join(relative);
        CacheEntry {
            id: id.into(),
            name: name.into(),
            path: display_path(&path),
            size_bytes: if calculate { directory_size(&path) } else { 0 },
            protected,
        }
    })
    .collect()
}

fn append_log(root: &Path, result: &OperationResult) {
    let directory = root.join(r"Logs\GreenDev");
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let active = directory.join("operations.jsonl");
    let rotate_bytes = fs::read_to_string(root.join(r"Config\greendev\reliability-policy.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')).ok())
        .and_then(|value| value["logRotateBytes"].as_u64())
        .unwrap_or(5 * 1024 * 1024);
    if fs::metadata(&active)
        .map(|metadata| metadata.len() >= rotate_bytes)
        .unwrap_or(false)
    {
        let archive = directory.join("archive");
        if fs::create_dir_all(&archive).is_ok() {
            let _ = fs::rename(
                &active,
                archive.join(format!("operations-{}.jsonl", now_millis())),
            );
        }
    }
    if let Ok(mut line) = serde_json::to_string(result) {
        line.push('\n');
        use std::io::Write;
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(active)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn finish_operation(
    root: &Path,
    kind: &str,
    title: &str,
    started_at: u64,
    success: bool,
    exit_code: Option<i32>,
    output: String,
) -> OperationResult {
    let summary = output
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(if success {
            "操作完成"
        } else {
            "操作异常"
        })
        .trim()
        .to_string();
    let result = OperationResult {
        operation_id: operation_id(kind),
        success,
        title: title.into(),
        summary,
        output: output.trim().into(),
        exit_code,
        kind: kind.into(),
        started_at,
        finished_at: now_millis(),
    };
    append_log(root, &result);
    result
}

fn run_batch(
    kind: &str,
    title: &str,
    script: &str,
    arguments: &[&str],
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let script_path = root.join(script);
    if !script_path.is_file() {
        return Err(format!("脚本不存在：{}", display_path(&script_path)));
    }
    let mut command_line = format!(
        "set FRAMEWORKS_NOPAUSE=1&& call \"{}\"",
        display_path(&script_path)
    );
    for argument in arguments {
        command_line.push(' ');
        command_line.push_str(argument);
    }
    let output = background_command(system_program("cmd.exe"))
        .args(["/d", "/c", &command_line])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("启动脚本失败：{error}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&stderr);
    }
    Ok(finish_operation(
        &root,
        kind,
        title,
        started,
        output.status.success(),
        output.status.code(),
        text,
    ))
}

#[tauri::command]
fn get_dashboard() -> Result<Dashboard, String> {
    let root = frameworks_root()?;
    let components: Vec<_> = definitions()
        .into_iter()
        .map(|definition| component_status(&root, definition))
        .collect();
    let installed_count = components
        .iter()
        .filter(|component| component.installed)
        .count();
    let healthy_count = components
        .iter()
        .filter(|component| component.healthy)
        .count();
    Ok(Dashboard {
        root: display_path(&root),
        total_size_bytes: 0,
        cache_size_bytes: 0,
        storage_ready: false,
        installed_count,
        healthy_count,
        components,
        caches: cache_entries(&root, false),
    })
}

#[tauri::command]
fn scan_storage() -> Result<StorageMetrics, String> {
    let root = frameworks_root()?;
    let caches = cache_entries(&root, true);
    let cache_size_bytes = caches.iter().map(|cache| cache.size_bytes).sum();
    let total_size_bytes = [
        "BuildTools",
        "Caches",
        "Databases",
        "Platforms",
        "ReverseTools",
        "Runtimes",
        "Toolchains",
    ]
    .iter()
    .map(|name| directory_size(&root.join(name)))
    .sum();
    let history_path = root.join(r"Logs\GreenDev\storage-history.jsonl");
    if let Some(parent) = history_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut line) = serde_json::to_string(&StoragePoint {
        recorded_at: now_millis(),
        total_size_bytes,
        cache_size_bytes,
    }) {
        line.push('\n');
        use std::io::Write;
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(history_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
    Ok(StorageMetrics {
        total_size_bytes,
        cache_size_bytes,
        caches,
    })
}

#[tauri::command]
fn get_storage_history(limit: Option<usize>) -> Result<Vec<StoragePoint>, String> {
    let root = frameworks_root()?;
    let text =
        fs::read_to_string(root.join(r"Logs\GreenDev\storage-history.jsonl")).unwrap_or_default();
    let mut points: Vec<_> = text
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<StoragePoint>(line).ok())
        .take(limit.unwrap_or(30).min(180))
        .collect();
    points.reverse();
    Ok(points)
}

#[tauri::command]
fn run_doctor(deep: bool) -> Result<OperationResult, String> {
    let args: Vec<&str> = if deep { vec!["en", "deep"] } else { vec!["en"] };
    run_batch(
        "doctor",
        if deep {
            "深度环境检查"
        } else {
            "环境检查"
        },
        "env-setup.bat",
        &args,
    )
}

#[tauri::command]
fn sync_configs() -> Result<OperationResult, String> {
    run_batch("config", "同步权威配置", "sync-config.bat", &["en"])
}

#[tauri::command]
fn preview_cleanup(level: String, include_wrapper: bool) -> Result<OperationResult, String> {
    let safe_level = if level == "safe" { "safe" } else { "normal" };
    let mut args = vec!["en", safe_level];
    if include_wrapper {
        args.push("wrapper");
    }
    run_batch("cleanup-preview", "缓存清理预览", "cleanup.bat", &args)
}

#[tauri::command]
fn apply_cleanup(level: String, include_wrapper: bool) -> Result<OperationResult, String> {
    let safe_level = if level == "safe" { "safe" } else { "normal" };
    let mut args = vec!["apply", "en", safe_level];
    if include_wrapper {
        args.push("wrapper");
    }
    run_batch("cleanup", "清理缓存", "cleanup.bat", &args)
}

#[tauri::command]
fn configure_environment(components: Vec<String>) -> Result<OperationResult, String> {
    const ALLOWED: &[&str] = &[
        "java", "node", "gradle", "maven", "android", "rust", "python", "c", "acpi", "mysql",
    ];
    let selected: Vec<&str> = components
        .iter()
        .map(String::as_str)
        .filter(|value| ALLOWED.contains(value))
        .collect();
    if selected.is_empty() {
        return Err("请至少选择一个组件。".into());
    }
    let selection = selected.join(",");
    run_batch(
        "environment",
        "配置用户环境",
        "setup_dev_env.bat",
        &["en", &selection],
    )
}

#[tauri::command]
fn list_environment_backups() -> Result<Vec<EnvironmentBackup>, String> {
    let root = frameworks_root()?;
    let directory = root.join(r"Config\env-backups");
    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir(&directory) else {
        return Ok(result);
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(document) = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .ok_or(())
        else {
            continue;
        };
        let variables = document
            .get("variables")
            .and_then(Value::as_object)
            .map(|value| value.len())
            .unwrap_or(0);
        result.push(EnvironmentBackup {
            file_name: entry.file_name().to_string_lossy().into_owned(),
            path: display_path(&path),
            created_at: document
                .get("createdAt")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            variable_count: variables,
            root: document
                .get("root")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
        });
    }
    result.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(result)
}

#[tauri::command]
fn restore_environment_backup(file_name: String) -> Result<OperationResult, String> {
    if file_name.contains(['\\', '/']) || !file_name.ends_with(".json") {
        return Err("备份文件名无效。".into());
    }
    let root = frameworks_root()?;
    let path = root.join(r"Config\env-backups").join(&file_name);
    let path_text = display_path(&path);
    let started = now_millis();
    let script = root.join(r"Scripts\restore-user-env.ps1");
    let output = background_command(system_program("powershell.exe"))
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &display_path(&script),
            "-BackupPath",
            &path_text,
            "-Lang",
            "zh",
        ])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| error.to_string())?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(finish_operation(
        &root,
        "environment-restore",
        "恢复用户环境",
        started,
        output.status.success(),
        output.status.code(),
        text,
    ))
}

fn fnv_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016X}")
}

fn config_item(
    root: &Path,
    id: &str,
    name: &str,
    source: &str,
    deployed: Option<&str>,
    generated: bool,
) -> ConfigStatus {
    let source_path = root.join(source);
    let source_bytes = fs::read(&source_path).unwrap_or_default();
    let expected = if generated {
        String::from_utf8_lossy(&source_bytes)
            .replace(
                "{{FRAMEWORKS_HOME_FWD}}",
                &display_path(root).replace('\\', "/"),
            )
            .into_bytes()
    } else {
        source_bytes.clone()
    };
    let (deployed_path, deployed_hash, state, detail) = if let Some(target) = deployed {
        let path = root.join(target);
        match fs::read(&path) {
            Ok(bytes) if bytes == expected => (
                Some(display_path(&path)),
                Some(fnv_hash(&bytes)),
                "synced".into(),
                "内容与权威副本一致".into(),
            ),
            Ok(bytes) => (
                Some(display_path(&path)),
                Some(fnv_hash(&bytes)),
                "drifted".into(),
                "生效副本存在内容漂移".into(),
            ),
            Err(_) => (
                Some(display_path(&path)),
                None,
                "missing".into(),
                "生效副本缺失".into(),
            ),
        }
    } else {
        (None, None, "reference".into(), "由统一环境变量加载".into())
    };
    ConfigStatus {
        id: id.into(),
        name: name.into(),
        source_path: display_path(&source_path),
        deployed_path,
        source_hash: fnv_hash(&source_bytes),
        deployed_hash,
        state,
        detail,
    }
}

#[tauri::command]
fn get_config_statuses() -> Result<Vec<ConfigStatus>, String> {
    let root = frameworks_root()?;
    Ok(vec![
        config_item(
            &root,
            "gradle",
            "Gradle properties",
            r"Config\gradle\gradle.properties",
            Some(r"Caches\Gradle\gradle.properties"),
            false,
        ),
        config_item(
            &root,
            "gradle-init",
            "Gradle init.d",
            r"Config\gradle\init.d\cn-mirrors.init.gradle",
            Some(r"Caches\Gradle\init.d\cn-mirrors.init.gradle"),
            false,
        ),
        config_item(
            &root,
            "maven",
            "Maven",
            r"Config\maven\settings.xml",
            Some(r"BuildTools\Maven\current\conf\settings.xml"),
            false,
        ),
        config_item(
            &root,
            "cargo",
            "Cargo",
            r"Config\cargo\config.toml",
            Some(r"Toolchains\Rust\cargo-home\config.toml"),
            false,
        ),
        config_item(
            &root,
            "mysql",
            "MySQL",
            r"Config\mysql\my.ini.template",
            Some(r"Databases\Sql\mysql\my.ini"),
            true,
        ),
        config_item(&root, "npm", "npm", r"Config\npm\.npmrc", None, false),
        config_item(&root, "pip", "pip", r"Config\pip\pip.ini", None, false),
    ])
}

#[derive(Clone, Copy)]
struct ConfigDefinition {
    id: &'static str,
    name: &'static str,
    format: &'static str,
    source: &'static str,
    sync_key: Option<&'static str>,
}

fn config_definitions() -> Vec<ConfigDefinition> {
    vec![
        ConfigDefinition {
            id: "gradle",
            name: "Gradle properties",
            format: "properties",
            source: r"Config\gradle\gradle.properties",
            sync_key: Some("gradle"),
        },
        ConfigDefinition {
            id: "gradle-init",
            name: "Gradle init.d",
            format: "groovy",
            source: r"Config\gradle\init.d\cn-mirrors.init.gradle",
            sync_key: Some("gradle"),
        },
        ConfigDefinition {
            id: "maven",
            name: "Maven settings",
            format: "xml",
            source: r"Config\maven\settings.xml",
            sync_key: Some("maven"),
        },
        ConfigDefinition {
            id: "cargo",
            name: "Cargo",
            format: "toml",
            source: r"Config\cargo\config.toml",
            sync_key: Some("rust"),
        },
        ConfigDefinition {
            id: "mysql",
            name: "MySQL",
            format: "ini-template",
            source: r"Config\mysql\my.ini.template",
            sync_key: Some("mysql"),
        },
        ConfigDefinition {
            id: "npm",
            name: "npm",
            format: "properties",
            source: r"Config\npm\.npmrc",
            sync_key: None,
        },
        ConfigDefinition {
            id: "pip",
            name: "pip",
            format: "ini",
            source: r"Config\pip\pip.ini",
            sync_key: None,
        },
    ]
}

fn find_config_definition(id: &str) -> Result<ConfigDefinition, String> {
    config_definitions()
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| "未知配置项。".into())
}

fn line_setting(content: &str, key: &str) -> String {
    content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                return None;
            }
            let (left, right) = trimmed.split_once('=')?;
            if left.trim() == key {
                Some(right.trim().trim_matches('"').to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn xml_tag_value(content: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    content
        .find(&open)
        .and_then(|start| {
            let value_start = start + open.len();
            content[value_start..]
                .find(&close)
                .map(|end| content[value_start..value_start + end].trim().to_string())
        })
        .unwrap_or_default()
}

fn toml_section_value(content: &str, section: &str, key: &str) -> String {
    let mut active = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            active = &trimmed[1..trimmed.len() - 1] == section;
            continue;
        }
        if active {
            if let Some((left, right)) = trimmed.split_once('=') {
                if left.trim() == key {
                    return right.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    String::new()
}

fn field(key: &str, label: &str, value: String, kind: &str, help: &str) -> ConfigField {
    ConfigField {
        key: key.into(),
        label: label.into(),
        value,
        kind: kind.into(),
        help: help.into(),
    }
}

fn extract_config_fields(id: &str, content: &str) -> Vec<ConfigField> {
    match id {
        "gradle" => vec![
            field(
                "org.gradle.jvmargs",
                "JVM 参数",
                line_setting(content, "org.gradle.jvmargs"),
                "text",
                "例如 -Xmx2g -Dfile.encoding=UTF-8",
            ),
            field(
                "org.gradle.parallel",
                "并行构建",
                line_setting(content, "org.gradle.parallel"),
                "boolean",
                "允许 Gradle 并行执行项目",
            ),
            field(
                "org.gradle.caching",
                "构建缓存",
                line_setting(content, "org.gradle.caching"),
                "boolean",
                "启用可重建构建缓存",
            ),
            field(
                "systemProp.http.proxyHost",
                "代理主机",
                line_setting(content, "systemProp.http.proxyHost"),
                "text",
                "留空表示不设置",
            ),
            field(
                "systemProp.http.proxyPort",
                "代理端口",
                line_setting(content, "systemProp.http.proxyPort"),
                "number",
                "1–65535",
            ),
        ],
        "maven" => vec![
            field(
                "localRepository",
                "本地仓库",
                xml_tag_value(content, "localRepository"),
                "path",
                "推荐使用 ${env.FRAMEWORKS_HOME}",
            ),
            field(
                "offline",
                "离线模式",
                xml_tag_value(content, "offline"),
                "boolean",
                "启用后 Maven 不访问远程仓库",
            ),
            field(
                "proxy.active",
                "代理启用",
                xml_tag_value(content, "active"),
                "boolean",
                "Maven 代理开关",
            ),
            field(
                "proxy.host",
                "代理主机",
                xml_tag_value(content, "host"),
                "text",
                "本机代理通常为 127.0.0.1",
            ),
            field(
                "proxy.port",
                "代理端口",
                xml_tag_value(content, "port"),
                "number",
                "1–65535",
            ),
        ],
        "cargo" => vec![
            field(
                "replace-with",
                "crates.io 替换源",
                toml_section_value(content, "source.crates-io", "replace-with"),
                "text",
                "对应 source 段名称",
            ),
            field(
                "registry",
                "Sparse 索引",
                toml_section_value(content, "source.rsproxy-sparse", "registry"),
                "url",
                "必须使用 https 或 sparse+https",
            ),
            field(
                "git-fetch-with-cli",
                "使用 Git CLI",
                toml_section_value(content, "net", "git-fetch-with-cli"),
                "boolean",
                "复杂代理环境更稳定",
            ),
        ],
        "mysql" => vec![
            field(
                "port",
                "端口",
                line_setting(content, "port"),
                "number",
                "1–65535",
            ),
            field(
                "bind-address",
                "监听地址",
                line_setting(content, "bind-address"),
                "text",
                "绿色环境默认 127.0.0.1",
            ),
            field(
                "max_connections",
                "最大连接数",
                line_setting(content, "max_connections"),
                "number",
                "正整数",
            ),
            field(
                "innodb_buffer_pool_size",
                "InnoDB 缓冲池",
                line_setting(content, "innodb_buffer_pool_size"),
                "size",
                "例如 2G、512M",
            ),
            field(
                "slow_query_log",
                "慢查询日志",
                line_setting(content, "slow_query_log"),
                "boolean-number",
                "1 启用，0 停用",
            ),
        ],
        "npm" => vec![
            field(
                "registry",
                "Registry",
                line_setting(content, "registry"),
                "url",
                "npm 包索引地址",
            ),
            field(
                "cache",
                "缓存位置",
                line_setting(content, "cache"),
                "path",
                "支持 ${FRAMEWORKS_HOME}",
            ),
            field(
                "strict-ssl",
                "严格 TLS",
                {
                    let value = line_setting(content, "strict-ssl");
                    if value.is_empty() {
                        "true".into()
                    } else {
                        value
                    }
                },
                "boolean",
                "推荐保持启用",
            ),
        ],
        "pip" => vec![
            field(
                "index-url",
                "索引地址",
                line_setting(content, "index-url"),
                "url",
                "pip simple index",
            ),
            field(
                "timeout",
                "超时秒数",
                line_setting(content, "timeout"),
                "number",
                "正整数",
            ),
            field(
                "trusted-host",
                "信任主机",
                line_setting(content, "trusted-host"),
                "text",
                "只填写主机名",
            ),
        ],
        _ => Vec::new(),
    }
}

fn replace_line_setting(content: &str, key: &str, value: &str, replace_all: bool) -> String {
    let mut found = false;
    let mut output = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let matches = !trimmed.starts_with(['#', ';'])
            && trimmed
                .split_once('=')
                .map(|(left, _)| left.trim() == key)
                .unwrap_or(false);
        if matches && (!found || replace_all) {
            output.push(format!("{key}={value}"));
            found = true;
        } else {
            output.push(line.to_string());
        }
    }
    if !found {
        output.push(format!("{key}={value}"));
    }
    let mut rendered = output.join("\n");
    rendered.push('\n');
    rendered
}

fn replace_xml_tag(content: &str, tag: &str, value: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let Some(start) = content.find(&open) else {
        return content.to_string();
    };
    let value_start = start + open.len();
    let Some(end) = content[value_start..].find(&close) else {
        return content.to_string();
    };
    format!(
        "{}{}{}",
        &content[..value_start],
        value,
        &content[value_start + end..]
    )
}

fn replace_toml_section_value(
    content: &str,
    section: &str,
    key: &str,
    value: &str,
    quoted: bool,
) -> String {
    let mut active = false;
    let mut found = false;
    let mut output = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            active = &trimmed[1..trimmed.len() - 1] == section;
        }
        if active
            && trimmed
                .split_once('=')
                .map(|(left, _)| left.trim() == key)
                .unwrap_or(false)
        {
            output.push(if quoted {
                format!("{key} = \"{value}\"")
            } else {
                format!("{key} = {value}")
            });
            found = true;
        } else {
            output.push(line.to_string());
        }
    }
    if !found {
        output.push(format!("\n[{section}]"));
        output.push(if quoted {
            format!("{key} = \"{value}\"")
        } else {
            format!("{key} = {value}")
        });
    }
    let mut rendered = output.join("\n");
    rendered.push('\n');
    rendered
}

fn render_config_fields(id: &str, original: &str, fields: &HashMap<String, String>) -> String {
    let mut content = original.to_string();
    match id {
        "gradle" | "npm" | "pip" => {
            for (key, value) in fields {
                content = replace_line_setting(&content, key, value, false);
            }
        }
        "mysql" => {
            for (key, value) in fields {
                content = replace_line_setting(&content, key, value, key == "port");
            }
        }
        "maven" => {
            for (key, value) in fields {
                let tag = match key.as_str() {
                    "proxy.active" => "active",
                    "proxy.host" => "host",
                    "proxy.port" => "port",
                    other => other,
                };
                content = replace_xml_tag(&content, tag, value);
            }
        }
        "cargo" => {
            if let Some(value) = fields.get("replace-with") {
                content = replace_toml_section_value(
                    &content,
                    "source.crates-io",
                    "replace-with",
                    value,
                    true,
                );
            }
            if let Some(value) = fields.get("registry") {
                content = replace_toml_section_value(
                    &content,
                    "source.rsproxy-sparse",
                    "registry",
                    value,
                    true,
                );
                content = replace_toml_section_value(
                    &content,
                    "registries.rsproxy",
                    "index",
                    value,
                    true,
                );
            }
            if let Some(value) = fields.get("git-fetch-with-cli") {
                content =
                    replace_toml_section_value(&content, "net", "git-fetch-with-cli", value, false);
            }
        }
        _ => {}
    }
    content
}

fn validate_config_content(definition: ConfigDefinition, content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if content.is_empty() {
        errors.push("配置内容为空。".into());
    }
    if content.len() > 512 * 1024 {
        errors.push("配置内容超过 512 KiB。".into());
    }
    if content.contains('\0') {
        errors.push("配置包含 NUL 字符。".into());
    }
    match definition.format {
        "xml" => {
            let mut reader = quick_xml::Reader::from_str(content);
            reader.config_mut().trim_text(true);
            loop {
                match reader.read_event() {
                    Ok(quick_xml::events::Event::Eof) => break,
                    Ok(_) => {}
                    Err(error) => {
                        errors.push(format!("XML 格式错误：{error}"));
                        break;
                    }
                }
            }
            if !content.contains("<settings") {
                errors.push("Maven 配置缺少 settings 根元素。".into());
            }
        }
        "toml" => {
            if let Err(error) =
                toml::from_str::<toml::Value>(content.trim_start_matches('\u{feff}'))
            {
                errors.push(format!("TOML 格式错误：{error}"));
            }
        }
        "ini" | "ini-template" | "properties" => {
            for (index, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with(['#', ';', '[']) {
                    continue;
                }
                if !trimmed.contains('=') && definition.format == "properties" {
                    errors.push(format!("第 {} 行缺少 '='。", index + 1));
                    if errors.len() >= 8 {
                        break;
                    }
                }
            }
        }
        "groovy" => {
            let opens = content.chars().filter(|value| *value == '{').count();
            let closes = content.chars().filter(|value| *value == '}').count();
            if opens != closes {
                errors.push("Groovy 花括号数量不匹配。".into());
            }
        }
        _ => {}
    }
    if definition.id == "mysql" && !content.contains("{{FRAMEWORKS_HOME_FWD}}") {
        errors.push("MySQL 模板必须保留 {{FRAMEWORKS_HOME_FWD}}。".into());
    }
    for item in extract_config_fields(definition.id, content) {
        if item.kind == "number"
            && (!item.value.chars().all(|value| value.is_ascii_digit())
                || item.value.parse::<u32>().unwrap_or(0) == 0)
        {
            errors.push(format!("{} 必须是正整数。", item.label));
        }
        if item.kind == "boolean"
            && !["true", "false"].contains(&item.value.to_ascii_lowercase().as_str())
        {
            errors.push(format!("{} 必须是 true 或 false。", item.label));
        }
        if item.kind == "url"
            && !(item.value.starts_with("https://")
                || item.value.starts_with("http://")
                || item.value.starts_with("sparse+https://"))
        {
            errors.push(format!("{} 的 URL 格式无效。", item.label));
        }
    }
    errors
}

fn config_diff(old: &str, new: &str) -> String {
    if old == new {
        return "内容未发生变化。".into();
    }
    let old_lines: Vec<_> = old.lines().collect();
    let new_lines: Vec<_> = new.lines().collect();
    let mut output = Vec::new();
    let max = old_lines.len().max(new_lines.len());
    for index in 0..max {
        match (old_lines.get(index), new_lines.get(index)) {
            (Some(left), Some(right)) if left == right => {}
            (Some(left), Some(right)) => {
                output.push(format!("- {:04} {left}", index + 1));
                output.push(format!("+ {:04} {right}", index + 1));
            }
            (Some(left), None) => output.push(format!("- {:04} {left}", index + 1)),
            (None, Some(right)) => output.push(format!("+ {:04} {right}", index + 1)),
            _ => {}
        }
        if output.len() >= 240 {
            output.push("… 差异过长，已截断。".into());
            break;
        }
    }
    output.join("\n")
}

fn config_backup_directory(root: &Path, id: &str) -> PathBuf {
    root.join(r"Config\config-backups").join(id)
}

fn list_config_backups_at(root: &Path, id: &str) -> Vec<ConfigBackup> {
    let directory = config_backup_directory(root, id);
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.filter_map(Result::ok) {
            if !entry.path().is_file() {
                continue;
            }
            let metadata = entry.metadata().ok();
            result.push(ConfigBackup {
                file_name: entry.file_name().to_string_lossy().into_owned(),
                created_at: metadata
                    .as_ref()
                    .and_then(|item| item.modified().ok())
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|value| value.as_millis().to_string())
                    .unwrap_or_default(),
                size_bytes: metadata.map(|item| item.len()).unwrap_or(0),
            });
        }
    }
    result.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    result
}

#[tauri::command]
fn get_config_document(id: String) -> Result<ConfigDocument, String> {
    let root = frameworks_root()?;
    let definition = find_config_definition(&id)?;
    let path = root.join(definition.source);
    let raw = fs::read_to_string(&path).map_err(|error| format!("读取配置失败：{error}"))?;
    Ok(ConfigDocument {
        id,
        name: definition.name.into(),
        format: definition.format.into(),
        source_path: display_path(&path),
        base_hash: fnv_hash(raw.as_bytes()),
        fields: extract_config_fields(definition.id, &raw),
        backups: list_config_backups_at(&root, definition.id),
        raw,
    })
}

fn preview_config_internal(
    root: &Path,
    id: &str,
    raw: Option<String>,
    fields: Option<HashMap<String, String>>,
) -> Result<ConfigPreview, String> {
    let definition = find_config_definition(id)?;
    let path = root.join(definition.source);
    let original = fs::read_to_string(&path).map_err(|error| format!("读取配置失败：{error}"))?;
    let rendered =
        raw.unwrap_or_else(|| render_config_fields(id, &original, &fields.unwrap_or_default()));
    let errors = validate_config_content(definition, &rendered);
    Ok(ConfigPreview {
        valid: errors.is_empty(),
        errors,
        diff: config_diff(&original, &rendered),
        rendered,
    })
}

#[tauri::command]
fn preview_config_change(
    id: String,
    raw: Option<String>,
    fields: Option<HashMap<String, String>>,
) -> Result<ConfigPreview, String> {
    preview_config_internal(&frameworks_root()?, &id, raw, fields)
}

fn atomic_config_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "配置路径没有父目录。".to_string())?;
    let temp = parent.join(format!(".greendev-{}.tmp", now_millis()));
    let old = parent.join(format!(".greendev-{}.old", now_millis()));
    fs::write(&temp, content.as_bytes()).map_err(|error| format!("写入临时配置失败：{error}"))?;
    fs::rename(path, &old).map_err(|error| {
        let _ = fs::remove_file(&temp);
        format!("暂存原配置失败：{error}")
    })?;
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::rename(&old, path);
        let _ = fs::remove_file(&temp);
        return Err(format!("替换配置失败，已恢复原文件：{error}"));
    }
    fs::remove_file(&old).map_err(|error| format!("配置已写入，但清理临时副本失败：{error}"))?;
    Ok(())
}

fn sync_config_key(root: &Path, key: &str) -> Result<String, String> {
    let script = root.join(r"Scripts\sync-config.ps1");
    let output = background_command(system_program("powershell.exe"))
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &display_path(&script),
            "-Lang",
            "zh",
            "-Keys",
            key,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("同步配置失败：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[tauri::command]
fn apply_config_change(
    id: String,
    raw: Option<String>,
    fields: Option<HashMap<String, String>>,
    expected_hash: Option<String>,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let definition = find_config_definition(&id)?;
    let path = root.join(definition.source);
    if let Some(expected) = expected_hash {
        let current = fs::read(&path).map_err(|error| format!("读取配置失败：{error}"))?;
        let current_hash = fnv_hash(&current);
        if current_hash != expected {
            return Err(format!(
                "配置已被其他程序修改。当前指纹 {current_hash}，请重新载入后合并变更。"
            ));
        }
    }
    let preview = preview_config_internal(&root, &id, raw, fields)?;
    if !preview.valid {
        return Err(preview.errors.join("\n"));
    }
    if preview.diff == "内容未发生变化。" {
        return Ok(finish_operation(
            &root,
            "config-edit",
            "保存配置",
            started,
            true,
            Some(0),
            "内容未发生变化。".into(),
        ));
    }
    let backup_dir = config_backup_directory(&root, definition.id);
    fs::create_dir_all(&backup_dir).map_err(|error| error.to_string())?;
    let backup_name = format!("{}-{}.bak", definition.id, now_millis());
    fs::copy(&path, backup_dir.join(&backup_name))
        .map_err(|error| format!("备份配置失败：{error}"))?;
    prune_config_backups(&root, definition.id, 30);
    atomic_config_write(&path, &preview.rendered)?;
    if let Some(key) = definition.sync_key {
        if let Err(error) = sync_config_key(&root, key) {
            let _ = fs::copy(backup_dir.join(&backup_name), &path);
            let _ = sync_config_key(&root, key);
            return Err(format!("同步失败，已恢复权威配置：{error}"));
        }
    }
    Ok(finish_operation(
        &root,
        "config-edit",
        &format!("保存 {}", definition.name),
        started,
        true,
        Some(0),
        format!("备份: {}\n{}", backup_name, preview.diff),
    ))
}

fn prune_config_backups(root: &Path, id: &str, keep: usize) {
    let directory = config_backup_directory(root, id);
    let mut entries: Vec<_> = fs::read_dir(&directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    let remove_count = entries.len().saturating_sub(keep);
    for entry in entries.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
}

#[tauri::command]
fn preview_config_backup(id: String, file_name: String) -> Result<BackupPreview, String> {
    if file_name.contains(['\\', '/']) || !file_name.ends_with(".bak") {
        return Err("配置备份文件名无效。".into());
    }
    let root = frameworks_root()?;
    let definition = find_config_definition(&id)?;
    let path = config_backup_directory(&root, definition.id).join(&file_name);
    let content = fs::read_to_string(path).map_err(|error| format!("读取配置备份失败：{error}"))?;
    Ok(BackupPreview {
        file_name,
        source_hash: fnv_hash(content.as_bytes()),
        content,
    })
}

#[tauri::command]
fn rollback_config(id: String, file_name: String) -> Result<OperationResult, String> {
    if file_name.contains(['\\', '/']) || !file_name.ends_with(".bak") {
        return Err("配置备份文件名无效。".into());
    }
    let started = now_millis();
    let root = frameworks_root()?;
    let definition = find_config_definition(&id)?;
    let backup = config_backup_directory(&root, definition.id).join(&file_name);
    let restored =
        fs::read_to_string(&backup).map_err(|error| format!("读取配置备份失败：{error}"))?;
    let errors = validate_config_content(definition, &restored);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let path = root.join(definition.source);
    let safety = config_backup_directory(&root, definition.id).join(format!(
        "{}-before-rollback-{}.bak",
        definition.id,
        now_millis()
    ));
    fs::copy(&path, &safety).map_err(|error| format!("创建回滚安全备份失败：{error}"))?;
    atomic_config_write(&path, &restored)?;
    if let Some(key) = definition.sync_key {
        sync_config_key(&root, key)?;
    }
    Ok(finish_operation(
        &root,
        "config-rollback",
        &format!("回滚 {}", definition.name),
        started,
        true,
        Some(0),
        format!("已恢复: {file_name}\n回滚前快照: {}", display_path(&safety)),
    ))
}

fn pins_path(root: &Path) -> PathBuf {
    root.join(r"Config\greendev\pins.json")
}

fn read_pins(root: &Path) -> HashMap<String, String> {
    fs::read_to_string(pins_path(root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn find_version_definition(id: &str) -> Result<VersionDefinition, String> {
    version_definitions()
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| "该组件使用固定入口，不支持版本切换。".into())
}

fn collect_version_paths(base: &Path, health: &str) -> Vec<PathBuf> {
    fn visit(path: &Path, health: &str, depth: u8, result: &mut Vec<PathBuf>) {
        if depth > 2 {
            return;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let candidate = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if !kind.is_dir()
                || kind.is_symlink()
                || entry.file_name().to_string_lossy().starts_with("current")
            {
                continue;
            }
            if candidate.join(health).is_file() {
                result.push(candidate);
            } else {
                visit(&candidate, health, depth + 1, result);
            }
        }
    }
    let mut result = Vec::new();
    visit(base, health, 0, &mut result);
    result
}

#[tauri::command]
fn get_component_versions(component_id: String) -> Result<VersionInventory, String> {
    let root = frameworks_root()?;
    let definition = find_version_definition(&component_id)?;
    let current = root.join(definition.current);
    let current_target = canonical_display(&current);
    let pins = read_pins(&root);
    let pinned_path = pins.get(&component_id);
    let mut versions: Vec<_> =
        collect_version_paths(&root.join(definition.base), definition.health)
            .into_iter()
            .map(|path| {
                let path_text = canonical_display(&path).unwrap_or_else(|| display_path(&path));
                VersionEntry {
                    version: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    current: current_target
                        .as_ref()
                        .map(|value| value.eq_ignore_ascii_case(&path_text))
                        .unwrap_or(false),
                    pinned: pinned_path
                        .map(|value| value.eq_ignore_ascii_case(&path_text))
                        .unwrap_or(false),
                    healthy: path.join(definition.health).is_file(),
                    path: path_text,
                }
            })
            .collect();
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    Ok(VersionInventory {
        component_id,
        component_name: definition.name.into(),
        supports_switching: true,
        current_path: Some(display_path(&current)),
        versions,
    })
}

fn validate_version_target(
    root: &Path,
    definition: VersionDefinition,
    target: &str,
) -> Result<PathBuf, String> {
    let target_path = PathBuf::from(target);
    let canonical =
        fs::canonicalize(&target_path).map_err(|_| "目标版本目录不存在。".to_string())?;
    let allowed = collect_version_paths(&root.join(definition.base), definition.health);
    if !allowed
        .iter()
        .filter_map(|path| fs::canonicalize(path).ok())
        .any(|path| path == canonical)
    {
        return Err("目标不在已枚举版本白名单中。".into());
    }
    if !canonical.join(definition.health).is_file() {
        return Err("目标版本健康检查失败。".into());
    }
    Ok(canonical)
}

#[tauri::command]
fn switch_component_version(
    component_id: String,
    target_path: String,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let definition = find_version_definition(&component_id)?;
    let target = validate_version_target(&root, definition, &target_path)?;
    let current = root.join(definition.current);
    let backup = current.with_file_name(format!("current.greendev-backup-{}", now_millis()));
    if fs::symlink_metadata(&current).is_ok() {
        fs::rename(&current, &backup).map_err(|error| format!("备份 current 失败：{error}"))?;
    }
    let command_line = format!(
        "mklink /J \"{}\" \"{}\"",
        display_path(&current),
        display_path(&target)
    );
    let output = background_command(system_program("cmd.exe"))
        .args(["/d", "/c", &command_line])
        .current_dir(&root)
        .output();
    let success = output
        .as_ref()
        .map(|value| value.status.success())
        .unwrap_or(false)
        && current.join(definition.health).is_file();
    if !success {
        if current.exists() {
            let _ = fs::remove_dir(&current);
        }
        if backup.exists() {
            let _ = fs::rename(&backup, &current);
        }
        return Err("current 切换校验失败，已恢复原入口。".into());
    }
    if backup.exists() {
        fs::remove_dir(&backup).map_err(|error| format!("清理临时入口失败：{error}"))?;
    }
    let text = format!(
        "{} current -> {}\n健康检查: {}",
        definition.name,
        display_path(&target),
        definition.health
    );
    Ok(finish_operation(
        &root,
        "version-switch",
        &format!("切换 {} 版本", definition.name),
        started,
        true,
        Some(0),
        text,
    ))
}

#[tauri::command]
fn set_component_pin(
    component_id: String,
    target_path: Option<String>,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let definition = find_version_definition(&component_id)?;
    let mut pins = read_pins(&root);
    let message = if let Some(target) = target_path {
        let validated = validate_version_target(&root, definition, &target)?;
        pins.insert(component_id.clone(), display_path(&validated));
        format!("已固定 {} -> {}", definition.name, display_path(&validated))
    } else {
        pins.remove(&component_id);
        format!("已取消固定 {}", definition.name)
    };
    let path = pins_path(&root);
    fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
    fs::write(
        &path,
        serde_json::to_vec_pretty(&pins).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(finish_operation(
        &root,
        "version-pin",
        "更新版本固定",
        started,
        true,
        Some(0),
        message,
    ))
}

fn parse_sdkmanager_output(text: &str) -> Vec<AndroidPackage> {
    let mut section = "";
    let mut packages = HashMap::<String, AndroidPackage>::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("Installed packages:") {
            section = "installed";
            continue;
        }
        if line.starts_with("Available Packages:") || line.starts_with("Available packages:") {
            section = "available";
            continue;
        }
        if line.starts_with("Available Updates:") {
            section = "updates";
            continue;
        }
        if line.is_empty()
            || line.starts_with("Path")
            || line.starts_with('-')
            || !line.contains('|')
        {
            continue;
        }
        let columns: Vec<_> = line.split('|').map(str::trim).collect();
        if columns.len() < 3 {
            continue;
        }
        let id = columns[0];
        if !valid_package_id(id) {
            continue;
        }
        let entry = packages.entry(id.into()).or_insert(AndroidPackage {
            id: id.into(),
            version: columns[1].into(),
            description: columns[2].into(),
            installed: false,
            obsolete: false,
        });
        if section == "installed" {
            entry.installed = true;
        }
        if entry.version.is_empty() {
            entry.version = columns[1].into();
        }
    }
    let mut result: Vec<_> = packages.into_values().collect();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

fn valid_package_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() < 180
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ";._+-".contains(ch))
}

fn local_android_packages(root: &Path) -> Vec<AndroidPackage> {
    let sdk = root.join(r"Platforms\Android\Sdk");
    let mut result = Vec::new();
    for category in [
        "build-tools",
        "platforms",
        "sources",
        "cmake",
        "cmdline-tools",
    ] {
        let base = sdk.join(category);
        let Ok(entries) = fs::read_dir(base) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let id = format!("{category};{name}");
            result.push(AndroidPackage {
                id,
                version: name.clone(),
                description: format!("{category} {name}"),
                installed: true,
                obsolete: false,
            });
        }
    }
    if sdk.join(r"platform-tools\package.xml").is_file() {
        result.push(AndroidPackage {
            id: "platform-tools".into(),
            version: "installed".into(),
            description: "Android SDK Platform-Tools".into(),
            installed: true,
            obsolete: false,
        });
    }
    result
}

#[tauri::command]
fn get_android_packages() -> Result<Vec<AndroidPackage>, String> {
    let root = frameworks_root()?;
    let cache = root.join(r"Caches\GreenDevManager\android-catalog.txt");
    let mut packages = fs::read_to_string(cache)
        .map(|text| parse_sdkmanager_output(&text))
        .unwrap_or_default();
    let local = local_android_packages(&root);
    let installed: HashSet<_> = local.iter().map(|item| item.id.clone()).collect();
    for package in &mut packages {
        if installed.contains(&package.id) {
            package.installed = true;
        }
    }
    for package in local {
        if !packages.iter().any(|item| item.id == package.id) {
            packages.push(package);
        }
    }
    packages.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packages)
}

fn update_task(
    tasks: &Arc<Mutex<HashMap<String, TaskEntry>>>,
    id: &str,
    update: impl FnOnce(&mut TaskSnapshot),
) {
    if let Ok(mut guard) = tasks.lock() {
        if let Some(entry) = guard.get_mut(id) {
            let previous = entry.snapshot.stage.clone();
            update(&mut entry.snapshot);
            entry.snapshot.updated_at = now_millis();
            if entry.snapshot.stage != previous {
                entry.snapshot.timeline.push(TaskEvent {
                    at: entry.snapshot.updated_at,
                    stage: entry.snapshot.stage.clone(),
                    message: entry.snapshot.message.clone(),
                });
            }
        }
    }
}

fn task_policy(root: &Path) -> TaskPolicy {
    fs::read_to_string(root.join(r"Config\greendev\task-policy.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn try_claim_task(
    tasks: &Arc<Mutex<HashMap<String, TaskEntry>>>,
    id: &str,
    policy: &TaskPolicy,
) -> bool {
    let Ok(mut guard) = tasks.lock() else {
        return false;
    };
    let running = guard
        .values()
        .filter(|entry| {
            entry.snapshot.status == "running"
                || (entry.snapshot.status == "paused" && entry.snapshot.progress > 0)
        })
        .count();
    if running >= policy.max_concurrent.max(1) {
        return false;
    }
    let now = now_millis();
    let winner = guard
        .values()
        .filter(|entry| entry.snapshot.status == "queued" && entry.snapshot.scheduled_at <= now)
        .max_by(|a, b| {
            a.snapshot
                .priority
                .cmp(&b.snapshot.priority)
                .then_with(|| b.snapshot.started_at.cmp(&a.snapshot.started_at))
        })
        .map(|entry| entry.snapshot.id.clone());
    if winner.as_deref() != Some(id) {
        return false;
    }
    if let Some(entry) = guard.get_mut(id) {
        entry.snapshot.status = "running".into();
        entry.snapshot.stage = "starting".into();
        entry.snapshot.message = "已从队列领取，正在启动…".into();
        entry.snapshot.updated_at = now;
        entry.snapshot.queue_position = 0;
        entry.snapshot.timeline.push(TaskEvent {
            at: now,
            stage: "starting".into(),
            message: entry.snapshot.message.clone(),
        });
        true
    } else {
        false
    }
}

fn transaction_directory(root: &Path) -> PathBuf {
    root.join(r"Caches\GreenDevManager\transactions")
}

fn write_transaction(root: &Path, snapshot: &TaskSnapshot) {
    let directory = transaction_directory(root);
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let path = directory.join(format!("{}.json", snapshot.id));
    if let Ok(bytes) = serde_json::to_vec_pretty(snapshot) {
        let _ = fs::write(path, bytes);
    }
}

fn task_spec_has_sensitive_values(spec: &TaskSpec) -> bool {
    let sensitive_env = spec.envs.iter().any(|(key, _)| {
        let key = key.to_ascii_uppercase();
        ["TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "API_KEY"]
            .iter()
            .any(|marker| key.contains(marker))
    });
    let sensitive_args = spec.args.windows(2).any(|pair| {
        let flag = pair[0].to_ascii_uppercase();
        ["TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "AUTH"]
            .iter()
            .any(|marker| flag.contains(marker))
    }) || spec
        .args
        .iter()
        .any(|value| value.contains("://") && value.contains('@'));
    sensitive_env || sensitive_args
}

fn write_task_record(root: &Path, entry: &TaskEntry) {
    write_transaction(root, &entry.snapshot);
    if task_spec_has_sensitive_values(&entry.spec) {
        return;
    }
    let directory = transaction_directory(root);
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let record = PersistedTask {
        schema_version: 1,
        snapshot: entry.snapshot.clone(),
        spec: entry.spec.clone(),
    };
    if let Ok(bytes) = serde_json::to_vec_pretty(&record) {
        let target = directory.join(format!("{}.task.json", entry.snapshot.id));
        let temporary = directory.join(format!("{}.task.writing", entry.snapshot.id));
        if fs::write(&temporary, bytes).is_ok() {
            let _ = fs::remove_file(&target);
            let _ = fs::rename(temporary, target);
        }
    }
}

fn close_transaction(root: &Path, id: &str) {
    let source = transaction_directory(root).join(format!("{id}.json"));
    let recovered = transaction_directory(root).join(format!("{id}.completed.json"));
    if source.is_file() {
        let _ = fs::rename(source, recovered);
    }
    let source = transaction_directory(root).join(format!("{id}.task.json"));
    let completed = transaction_directory(root).join(format!("{id}.completed.task.json"));
    if source.is_file() {
        let _ = fs::rename(source, completed);
    }
}

fn recover_transactions(root: &Path) {
    let directory = transaction_directory(root);
    let Ok(entries) = fs::read_dir(&directory) else {
        return;
    };
    let mut recovered_items = Vec::new();
    for definition in version_definitions() {
        let current = root.join(definition.current);
        if !current.exists() {
            if let Some(parent) = current.parent() {
                let prefix = format!(
                    "{}.greendev-backup-",
                    current.file_name().unwrap_or_default().to_string_lossy()
                );
                let mut backups: Vec<_> = fs::read_dir(parent)
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
                    .collect();
                backups.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
                if let Some(backup) = backups.first() {
                    if fs::rename(backup.path(), &current).is_ok() {
                        recovered_items.push(format!("restored {} current", definition.name));
                    }
                }
            }
        }
    }
    fn report_stages(path: &Path, depth: u8, recovered: &mut Vec<String>) {
        if depth > 3 {
            return;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let candidate = entry.path();
            if !candidate.is_dir() {
                continue;
            }
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with(".greendev-")
            {
                recovered.push(format!("retained stale stage {}", display_path(&candidate)));
            } else {
                report_stages(&candidate, depth + 1, recovered);
            }
        }
    }
    for relative in ["BuildTools", "Databases", "Runtimes", "Toolchains"] {
        report_stages(&root.join(relative), 0, &mut recovered_items);
    }
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json")
            || path
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| {
                    value.ends_with(".completed.json")
                        || value.ends_with(".recovered.json")
                        || value.ends_with(".task.json")
                        || value.contains(".restarted.")
                })
                .unwrap_or(true)
        {
            continue;
        }
        let started = now_millis();
        let detail = format!(
            "检测到上次异常中断的任务事务：{}\n下载 .part 保留用于续传。\n{}",
            display_path(&path),
            if recovered_items.is_empty() {
                "没有发现待恢复入口或暂存目录。".into()
            } else {
                recovered_items.join("\n")
            }
        );
        let result = finish_operation(
            root,
            "transaction-recovery",
            "恢复未完成事务",
            started,
            true,
            Some(0),
            detail,
        );
        let recovered = path.with_extension("recovered.json");
        let _ = fs::rename(&path, recovered);
        let _ = result;
    }
}

fn restore_persisted_tasks(state: &AppState, root: &Path) -> usize {
    let directory = transaction_directory(root);
    let Ok(entries) = fs::read_dir(&directory) else {
        return 0;
    };
    let mut restored = 0;
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !name.ends_with(".task.json")
            || name.contains(".completed.")
            || name.contains(".restarted.")
        {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_slice::<PersistedTask>(&bytes) else {
            continue;
        };
        if record.schema_version != 1
            || !["queued", "running", "paused"].contains(&record.snapshot.status.as_str())
            || !["powershell.exe", "cmd.exe"].contains(&record.spec.program.as_str())
            || !is_frameworks_root(&record.spec.root)
        {
            continue;
        }
        let old_id = record.snapshot.id.clone();
        let mut spec = record.spec;
        spec.root = root.to_path_buf();
        spec.attempt = spec.attempt.max(record.snapshot.attempt).saturating_add(1);
        spec.scheduled_at = now_millis();
        spec.start_paused = record.snapshot.status == "paused";
        let restarted = path.with_file_name(format!("{old_id}.restarted.task.json"));
        if fs::rename(&path, restarted).is_err() {
            continue;
        }
        let snapshot = directory.join(format!("{old_id}.json"));
        if snapshot.is_file() {
            let _ = fs::rename(snapshot, directory.join(format!("{old_id}.restarted.json")));
        }
        launch_process_task(state, spec);
        restored += 1;
    }
    restored
}

#[cfg(windows)]
fn set_process_suspended(process_id: u32, suspended: bool) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> *mut std::ffi::c_void;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    #[link(name = "ntdll")]
    extern "system" {
        fn NtSuspendProcess(handle: *mut std::ffi::c_void) -> i32;
        fn NtResumeProcess(handle: *mut std::ffi::c_void) -> i32;
    }
    const PROCESS_SUSPEND_RESUME: u32 = 0x0800;
    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, 0, process_id);
        if handle.is_null() {
            return false;
        }
        let result = if suspended {
            NtSuspendProcess(handle)
        } else {
            NtResumeProcess(handle)
        };
        CloseHandle(handle);
        result >= 0
    }
}

#[cfg(windows)]
fn process_tree_ids(root_id: u32) -> Vec<u32> {
    #[repr(C)]
    struct ProcessEntry32 {
        size: u32,
        usage: u32,
        process_id: u32,
        default_heap: usize,
        module_id: u32,
        threads: u32,
        parent_id: u32,
        priority: i32,
        flags: u32,
        exe: [u16; 260],
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> *mut std::ffi::c_void;
        fn Process32FirstW(snapshot: *mut std::ffi::c_void, entry: *mut ProcessEntry32) -> i32;
        fn Process32NextW(snapshot: *mut std::ffi::c_void, entry: *mut ProcessEntry32) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    let mut pairs = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x00000002, 0);
        if snapshot as isize == -1 {
            return vec![root_id];
        }
        let mut entry: ProcessEntry32 = std::mem::zeroed();
        entry.size = std::mem::size_of::<ProcessEntry32>() as u32;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                pairs.push((entry.process_id, entry.parent_id));
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
    let mut result = vec![root_id];
    let mut index = 0;
    while index < result.len() {
        let parent = result[index];
        for (id, candidate_parent) in &pairs {
            if *candidate_parent == parent && !result.contains(id) {
                result.push(*id);
            }
        }
        index += 1;
    }
    result
}

#[cfg(windows)]
fn set_process_tree_suspended(process_id: u32, suspended: bool) -> bool {
    let mut ids = process_tree_ids(process_id);
    if suspended {
        ids.reverse();
    }
    let mut success = true;
    for id in ids {
        success = set_process_suspended(id, suspended) && success;
    }
    success
}

#[cfg(not(windows))]
fn set_process_tree_suspended(process_id: u32, suspended: bool) -> bool {
    set_process_suspended(process_id, suspended)
}

#[cfg(not(windows))]
fn set_process_suspended(_process_id: u32, _suspended: bool) -> bool {
    false
}

fn launch_process_task(state: &AppState, spec: TaskSpec) -> TaskSnapshot {
    let id = operation_id(&spec.kind);
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(spec.start_paused));
    let started_at = now_millis();
    let root = spec.root.clone();
    let title = spec.title.clone();
    let kind = spec.kind.clone();
    let scheduled_at = spec.scheduled_at.max(started_at);
    let snapshot = TaskSnapshot {
        id: id.clone(),
        title: title.clone(),
        kind: kind.clone(),
        status: if spec.start_paused {
            "paused".into()
        } else {
            "queued".into()
        },
        progress: 0,
        message: if spec.start_paused {
            "任务在重启后保持暂停".into()
        } else if scheduled_at > started_at {
            "等待计划时间".into()
        } else {
            "等待队列调度".into()
        },
        cancelable: true,
        pausable: true,
        retryable: false,
        stage: if spec.start_paused {
            "paused".into()
        } else {
            "queued".into()
        },
        bytes_processed: 0,
        bytes_total: 0,
        bytes_per_second: 0,
        eta_seconds: None,
        attempt: spec.attempt,
        priority: spec.priority,
        scheduled_at,
        queue_position: 1,
        timeline: vec![TaskEvent {
            at: started_at,
            stage: if spec.start_paused {
                "paused".into()
            } else {
                "queued".into()
            },
            message: if spec.start_paused {
                "已恢复任务，继续保持暂停".into()
            } else {
                "任务已进入持久队列".into()
            },
        }],
        started_at,
        updated_at: started_at,
        result: None,
    };
    state.tasks.lock().unwrap().insert(
        id.clone(),
        TaskEntry {
            snapshot: snapshot.clone(),
            cancel: Arc::clone(&cancel),
            pause: Arc::clone(&pause),
            spec: spec.clone(),
        },
    );
    if let Ok(guard) = state.tasks.lock() {
        if let Some(entry) = guard.get(&id) {
            write_task_record(&root, entry);
        }
    }
    let tasks = Arc::clone(&state.tasks);
    thread::spawn(move || {
        let started = now_millis();
        let progress_path = if spec.kind.starts_with("manifest-") {
            spec.args
                .iter()
                .position(|value| value == "-Id")
                .and_then(|index| spec.args.get(index + 1))
                .and_then(|id| {
                    read_manifest(&root).ok().and_then(|document| {
                        document.components.into_iter().find(|item| item.id == *id)
                    })
                })
                .and_then(|item| resolve_relative(&root, &item.source.archive).ok())
                .map(|path| PathBuf::from(format!("{}.part", display_path(&path))))
        } else {
            None
        };
        loop {
            if cancel.load(Ordering::Relaxed) {
                let result = finish_operation(
                    &root,
                    &kind,
                    &title,
                    started,
                    false,
                    None,
                    "任务在队列中取消。".into(),
                );
                update_task(&tasks, &id, |item| {
                    item.status = "cancelled".into();
                    item.stage = "cancelled".into();
                    item.progress = 100;
                    item.cancelable = false;
                    item.pausable = false;
                    item.result = Some(result);
                });
                if let Ok(guard) = tasks.lock() {
                    if let Some(entry) = guard.get(&id) {
                        write_task_record(&root, entry);
                    }
                }
                close_transaction(&root, &id);
                return;
            }
            if pause.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(300));
                continue;
            }
            let policy = task_policy(&root);
            if try_claim_task(&tasks, &id, &policy) {
                break;
            }
            if let Ok(mut guard) = tasks.lock() {
                let mut queued: Vec<_> = guard
                    .values()
                    .filter(|entry| entry.snapshot.status == "queued")
                    .map(|entry| {
                        (
                            entry.snapshot.id.clone(),
                            entry.snapshot.priority,
                            entry.snapshot.started_at,
                        )
                    })
                    .collect();
                queued
                    .sort_by_key(|(_, priority, started)| (std::cmp::Reverse(*priority), *started));
                for (index, (task_id, _, _)) in queued.iter().enumerate() {
                    if let Some(entry) = guard.get_mut(task_id) {
                        entry.snapshot.queue_position = index + 1;
                    }
                }
            }
            thread::sleep(Duration::from_millis(300));
        }
        update_task(&tasks, &id, |item| {
            item.progress = 3;
            item.message = "正在启动…".into();
        });
        let mut command = background_command(system_program(&spec.program));
        command
            .args(&spec.args)
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &spec.envs {
            command.env(key, value);
        }
        let spawn = command.spawn();
        let Ok(mut child) = spawn else {
            let error = spawn.err().unwrap().to_string();
            let result = finish_operation(&root, &kind, &title, started, false, None, error);
            update_task(&tasks, &id, |item| {
                item.status = "failed".into();
                item.progress = 100;
                item.cancelable = false;
                item.pausable = false;
                item.retryable = true;
                item.stage = "failed".into();
                item.result = Some(result);
            });
            if let Ok(guard) = tasks.lock() {
                if let Some(entry) = guard.get(&id) {
                    write_task_record(&root, entry);
                }
            }
            close_transaction(&root, &id);
            return;
        };
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let out_thread = thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = stdout {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        });
        let err_thread = thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = stderr {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        });
        let mut progress = 8u8;
        let mut suspended = false;
        let mut previous_bytes = 0u64;
        let mut previous_tick = now_millis();
        let status = loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                break child.wait().ok();
            }
            let requested_pause = pause.load(Ordering::Relaxed);
            if requested_pause != suspended {
                if set_process_tree_suspended(child.id(), requested_pause) {
                    suspended = requested_pause;
                }
                update_task(&tasks, &id, |item| {
                    item.status = if suspended {
                        "paused".into()
                    } else {
                        "running".into()
                    };
                    item.stage = if suspended {
                        "paused".into()
                    } else {
                        "running".into()
                    };
                    item.message = if suspended {
                        "任务已暂停，下载断点和暂存内容保持不变".into()
                    } else {
                        "任务已继续".into()
                    };
                });
                if let Ok(guard) = tasks.lock() {
                    if let Some(entry) = guard.get(&id) {
                        write_task_record(&root, entry);
                    }
                }
            }
            if suspended {
                thread::sleep(Duration::from_millis(250));
                continue;
            }
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {
                    progress = progress.saturating_add(1).min(92);
                    let bytes = progress_path
                        .as_ref()
                        .and_then(|path| fs::metadata(path).ok())
                        .map(|meta| meta.len())
                        .unwrap_or(0);
                    let tick = now_millis();
                    let elapsed = tick.saturating_sub(previous_tick).max(1);
                    let speed = bytes.saturating_sub(previous_bytes).saturating_mul(1000) / elapsed;
                    previous_bytes = bytes;
                    previous_tick = tick;
                    update_task(&tasks, &id, |item| {
                        item.progress = progress;
                        item.stage = if bytes > 0 {
                            "downloading".into()
                        } else {
                            "running".into()
                        };
                        item.bytes_processed = bytes;
                        item.bytes_per_second = speed;
                        item.bytes_total = if bytes > 0 {
                            bytes.saturating_mul(100) / u64::from(progress.max(1))
                        } else {
                            0
                        };
                        item.eta_seconds = if speed > 0 && item.bytes_total > bytes {
                            Some((item.bytes_total - bytes) / speed)
                        } else {
                            None
                        };
                        item.message = "任务运行中，可暂停或取消".into();
                    });
                    if let Ok(guard) = tasks.lock() {
                        if let Some(entry) = guard.get(&id) {
                            write_task_record(&root, entry);
                        }
                    }
                    thread::sleep(Duration::from_millis(450));
                }
                Err(_) => break child.wait().ok(),
            }
        };
        let stdout = out_thread.join().unwrap_or_default();
        let stderr = err_thread.join().unwrap_or_default();
        let mut output = String::from_utf8_lossy(&stdout).into_owned();
        if !stderr.is_empty() {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&String::from_utf8_lossy(&stderr));
        }
        let cancelled = cancel.load(Ordering::Relaxed);
        let success = !cancelled
            && status
                .as_ref()
                .map(|value| value.success())
                .unwrap_or(false);
        if success {
            if let Some(path) = &spec.cache_output {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(path, &output);
            }
        }
        if cancelled {
            output.push_str("\n任务已由用户取消。");
        }
        let result = finish_operation(
            &root,
            &kind,
            &title,
            started,
            success,
            status.and_then(|value| value.code()),
            output,
        );
        update_task(&tasks, &id, |item| {
            item.status = if cancelled {
                "cancelled".into()
            } else if success {
                "completed".into()
            } else {
                "failed".into()
            };
            item.progress = 100;
            item.stage = if cancelled {
                "cancelled".into()
            } else if success {
                "completed".into()
            } else {
                "failed".into()
            };
            item.message = if cancelled {
                "已取消".into()
            } else if success {
                "已完成".into()
            } else {
                "执行异常".into()
            };
            item.cancelable = false;
            item.pausable = false;
            item.retryable = !success && !cancelled;
            item.result = Some(result);
        });
        if let Ok(guard) = tasks.lock() {
            if let Some(entry) = guard.get(&id) {
                write_task_record(&root, entry);
            }
        }
        close_transaction(&root, &id);
    });
    snapshot
}

#[allow(clippy::too_many_arguments)]
fn start_process_task(
    state: &AppState,
    root: PathBuf,
    title: String,
    kind: String,
    program: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    cache_output: Option<PathBuf>,
) -> TaskSnapshot {
    let policy = task_policy(&root);
    launch_process_task(
        state,
        TaskSpec {
            root,
            title,
            kind,
            program,
            args,
            envs,
            cache_output,
            attempt: 1,
            priority: policy.default_priority,
            scheduled_at: now_millis(),
            start_paused: false,
        },
    )
}

#[tauri::command]
fn get_tasks(state: tauri::State<AppState>) -> Result<Vec<TaskSnapshot>, String> {
    let guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let mut tasks: Vec<_> = guard.values().map(|entry| entry.snapshot.clone()).collect();
    tasks.sort_by_key(|task| std::cmp::Reverse(task.started_at));
    Ok(tasks)
}

#[tauri::command]
fn start_android_task(
    action: String,
    packages: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    if !["list", "install", "uninstall"].contains(&action.as_str()) {
        return Err("Android 操作无效。".into());
    }
    if action != "list" && packages.is_empty() {
        return Err("请至少选择一个 Android SDK 包。".into());
    }
    if packages.iter().any(|value| !valid_package_id(value)) {
        return Err("Android SDK 包标识无效。".into());
    }
    let root = frameworks_root()?;
    let sdkmanager = root.join(r"Platforms\Android\Sdk\cmdline-tools\latest\bin\sdkmanager.bat");
    if !sdkmanager.is_file() {
        return Err("sdkmanager.bat 不存在。".into());
    }
    let mut command_line = format!("\"{}\"", display_path(&sdkmanager));
    match action.as_str() {
        "list" => command_line.push_str(" --list"),
        "install" => command_line.push_str(" --install"),
        "uninstall" => command_line.push_str(" --uninstall"),
        _ => {}
    }
    for package in &packages {
        command_line.push(' ');
        command_line.push('"');
        command_line.push_str(package);
        command_line.push('"');
    }
    let title = match action.as_str() {
        "list" => "刷新 Android SDK 目录",
        "install" => "安装 Android SDK 包",
        _ => "卸载 Android SDK 包",
    }
    .to_string();
    let cache = if action == "list" {
        Some(root.join(r"Caches\GreenDevManager\android-catalog.txt"))
    } else {
        None
    };
    let envs = vec![
        (
            "JAVA_HOME".into(),
            display_path(&root.join(r"Runtimes\Java\current")),
        ),
        (
            "ANDROID_HOME".into(),
            display_path(&root.join(r"Platforms\Android\Sdk")),
        ),
        (
            "ANDROID_SDK_ROOT".into(),
            display_path(&root.join(r"Platforms\Android\Sdk")),
        ),
    ];
    Ok(start_process_task(
        &state,
        root,
        title,
        format!("android-{action}"),
        "cmd.exe".into(),
        vec!["/d".into(), "/c".into(), command_line],
        envs,
        cache,
    ))
}

fn read_manifest(root: &Path) -> Result<ManifestDocument, String> {
    let text = fs::read_to_string(root.join(r"Config\greendev\components.json"))
        .map_err(|error| format!("读取组件清单失败：{error}"))?;
    let document: ManifestDocument =
        serde_json::from_str(&text).map_err(|error| format!("组件清单格式错误：{error}"))?;
    if !(1..=2).contains(&document.schema_version) {
        return Err(format!("不支持的清单版本：{}", document.schema_version));
    }
    Ok(document)
}

fn resolve_relative(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("清单路径必须是 Frameworks 根目录内且不含 .. 的相对路径。".into());
    }
    Ok(root.join(relative_path))
}

#[tauri::command]
fn get_manifest_components() -> Result<Vec<ManifestComponentStatus>, String> {
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let pins = read_pins(&root);
    let package_lock: Value = fs::read_to_string(root.join(r"Config\greendev\package-lock.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    let mut result = Vec::new();
    for item in &document.components {
        let install = resolve_relative(&root, &item.install_dir)?;
        let archive = resolve_relative(&root, &item.source.archive)?;
        let current = item
            .current_link
            .as_deref()
            .map(|value| resolve_relative(&root, value))
            .transpose()?;
        let installed = install.join(&item.health_path).is_file();
        let target = current
            .as_ref()
            .and_then(|path| fs::canonicalize(path).ok());
        let install_canonical = fs::canonicalize(&install).ok();
        let active = installed && target.is_some() && target == install_canonical;
        let pinned_elsewhere = pins
            .get(&item.id)
            .map(|value| fs::canonicalize(value).ok() != install_canonical)
            .unwrap_or(false);
        let locked_hash = package_lock
            .get(&item.id)
            .and_then(|value| value.get("sha256"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let checksum_ready = !item.source.sha256.is_empty() || !locked_hash.is_empty();
        let mut blockers = Vec::new();
        for dependency_id in &item.depends_on {
            match document
                .components
                .iter()
                .find(|candidate| candidate.id == *dependency_id)
            {
                Some(dependency) => {
                    let dependency_path = resolve_relative(&root, &dependency.install_dir)?;
                    if !dependency_path.join(&dependency.health_path).is_file() {
                        blockers.push(format!("缺少依赖 {dependency_id}"));
                    }
                }
                None => blockers.push(format!("清单缺少依赖定义 {dependency_id}")),
            }
        }
        if pinned_elsewhere {
            blockers.push("已固定到其他版本".into());
        }
        if !installed && !archive.is_file() && !checksum_ready {
            blockers.push("在线来源尚未锁定 SHA256，可先导入离线 ZIP".into());
        }
        let state = if !item.enabled {
            "disabled"
        } else if pinned_elsewhere {
            "pinned"
        } else if active {
            "current"
        } else if installed {
            "installed"
        } else if blockers.is_empty() {
            "available"
        } else {
            "blocked"
        };
        result.push(ManifestComponentStatus {
            id: item.id.clone(),
            name: item.name.clone(),
            desired_version: item.version.clone(),
            install_dir: display_path(&install),
            current_link: current.as_ref().map(|path| display_path(path)),
            source_url: item.source.url.clone(),
            archive_path: display_path(&archive),
            enabled: item.enabled,
            installed,
            active,
            archive_cached: archive.is_file(),
            checksum_ready,
            pinned_elsewhere,
            dependencies: item.depends_on.clone(),
            blocked_reason: blockers.join("；"),
            state: state.into(),
        });
    }
    Ok(result)
}

#[tauri::command]
fn get_install_plan(component_id: String, action: String) -> Result<InstallPlan, String> {
    if !["install", "update"].contains(&action.as_str()) {
        return Err("安装计划操作无效。".into());
    }
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let item = document
        .components
        .iter()
        .find(|value| value.id == component_id)
        .ok_or_else(|| "组件不在清单中。".to_string())?;
    let statuses = get_manifest_components()?;
    let status = statuses
        .iter()
        .find(|value| value.id == component_id)
        .ok_or_else(|| "组件状态缺失。".to_string())?;
    let mut blockers = Vec::new();
    if !status.blocked_reason.is_empty() {
        blockers.extend(status.blocked_reason.split('；').map(str::to_string));
    }
    if !item.enabled {
        blockers.push("清单项已停用".into());
    }
    let archive = resolve_relative(&root, &item.source.archive)?;
    let package_lock: Value = fs::read_to_string(root.join(r"Config\greendev\package-lock.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    let expected = if !item.source.sha256.is_empty() {
        item.source.sha256.clone()
    } else {
        package_lock
            .get(&item.id)
            .and_then(|value| value.get("sha256"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .into()
    };
    let mut steps = Vec::new();
    if status.installed {
        steps.push("验证目标版本健康文件".into());
    } else {
        steps.push(if archive.is_file() {
            "使用已缓存离线归档".into()
        } else {
            "断点续传下载归档".into()
        });
        steps.push("强制校验 SHA256".into());
        steps.push("解压到同卷暂存目录".into());
        steps.push("验证健康文件并原子落位".into());
    }
    if status.current_link.is_some() && !status.active {
        steps.push("备份 current、切换目录联接并验证；异常时回退".into());
    }
    steps.push("保留其他所有已安装版本".into());
    steps.push("写入持久操作日志".into());
    Ok(InstallPlan {
        component_id,
        action,
        steps,
        blockers: blockers.clone(),
        archive_path: display_path(&archive),
        source_url: item.source.url.clone(),
        expected_sha256: expected,
        ready: blockers.is_empty(),
    })
}

fn catalog_text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn catalog_candidates(value: Option<&Value>) -> Vec<CatalogCandidate> {
    let Some(value) = value else {
        return Vec::new();
    };
    let mut candidates: Vec<CatalogCandidate> = value
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| {
            let version = catalog_text(candidate, "version");
            if version.is_empty() {
                return None;
            }
            let sha256 = catalog_text(candidate, "sha256");
            Some(CatalogCandidate {
                id: catalog_text(candidate, "id"),
                provider: catalog_text(candidate, "provider"),
                version,
                architecture: catalog_text(candidate, "architecture"),
                channel: catalog_text(candidate, "channel"),
                url: catalog_text(candidate, "url"),
                sha256: sha256.clone(),
                archive_root: catalog_text(candidate, "archiveRoot"),
                install_dir: catalog_text(candidate, "installDir"),
                archive_path: catalog_text(candidate, "archivePath"),
                component_name: catalog_text(candidate, "componentName"),
                notes: catalog_text(candidate, "notes"),
                checksum_ready: !sha256.is_empty(),
            })
        })
        .collect();
    if candidates.is_empty() {
        let version = catalog_text(value, "version");
        if !version.is_empty() {
            let sha256 = catalog_text(value, "sha256");
            candidates.push(CatalogCandidate {
                id: catalog_text(value, "defaultCandidateId"),
                provider: "官方目录".into(),
                version,
                architecture: "x64".into(),
                channel: String::new(),
                url: catalog_text(value, "url"),
                sha256: sha256.clone(),
                archive_root: String::new(),
                install_dir: String::new(),
                archive_path: String::new(),
                component_name: String::new(),
                notes: catalog_text(value, "notes"),
                checksum_ready: !sha256.is_empty(),
            });
        }
    }
    candidates
}

#[tauri::command]
fn check_component_updates() -> Result<Vec<UpdateCandidate>, String> {
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let statuses = get_manifest_components()?;
    let pins = read_pins(&root);
    let policies: HashMap<String, String> =
        fs::read_to_string(root.join(r"Config\greendev\update-policies.json"))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
    let catalog: Value =
        fs::read_to_string(root.join(r"Caches\GreenDevManager\update-catalog.json"))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null);
    let mut result = Vec::new();
    for item in document.components {
        let status = statuses
            .iter()
            .find(|value| value.id == item.id)
            .ok_or_else(|| format!("组件状态缺失：{}", item.id))?;
        let current_version = item
            .current_link
            .as_deref()
            .and_then(|value| resolve_relative(&root, value).ok())
            .and_then(|path| canonical_display(&path))
            .and_then(|value| {
                Path::new(&value)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "未启用".into());
        let catalog_item = catalog
            .get("components")
            .and_then(|value| value.get(&item.id));
        let candidates = catalog_candidates(catalog_item);
        let catalog_available = catalog_item
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            == Some("ok")
            && !candidates.is_empty();
        let target_version = if let Some(candidate) = candidates.first() {
            candidate.version.clone()
        } else {
            item.version.clone()
        };
        let can_adopt = candidates.iter().any(|candidate| {
            candidate.version != item.version
                || (!candidate.url.is_empty() && candidate.url != item.source.url)
        });
        let install_ready = !can_adopt && status.checksum_ready;
        let release_notes = candidates
            .first()
            .map(|candidate| candidate.notes.clone())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                if status.active {
                    "当前入口已是清单目标版本。".into()
                } else {
                    "清单目标版本已就绪；执行前会预检依赖、校验值和版本固定。".into()
                }
            });
        result.push(UpdateCandidate {
            component_id: item.id.clone(),
            name: item.name,
            current_version: current_version.clone(),
            target_version: target_version.clone(),
            update_available: !current_version.contains(&target_version),
            installed: status.installed,
            active: status.active && !can_adopt,
            pinned: pins.contains_key(&item.id),
            checksum_ready: status.checksum_ready,
            catalog_available,
            install_ready,
            can_adopt,
            policy: policies
                .get(&item.id)
                .cloned()
                .unwrap_or_else(|| "stable".into()),
            release_notes,
            candidates,
        });
    }
    Ok(result)
}

#[tauri::command]
fn start_update_catalog_task(state: tauri::State<AppState>) -> Result<TaskSnapshot, String> {
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\refresh-update-catalog.ps1");
    let policy = root.join(r"Config\greendev\update-policies.json");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-PolicyPath".into(),
        display_path(&policy),
    ];
    Ok(start_process_task(
        &state,
        root,
        "刷新官方版本目录".into(),
        "update-catalog".into(),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
fn adopt_update_candidate(
    component_id: String,
    candidate_id: Option<String>,
) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let path = root.join(r"Config\greendev\components.json");
    let catalog: Value = serde_json::from_str(
        &fs::read_to_string(root.join(r"Caches\GreenDevManager\update-catalog.json"))
            .map_err(|_| "请先刷新官方版本目录。".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let catalog_item = catalog
        .get("components")
        .and_then(|value| value.get(&component_id))
        .filter(|value| value.get("status").and_then(Value::as_str) == Some("ok"))
        .ok_or_else(|| "该组件没有有效在线候选。".to_string())?;
    let candidates = catalog_candidates(Some(catalog_item));
    let candidate = candidate_id
        .as_deref()
        .and_then(|id| candidates.iter().find(|candidate| candidate.id == id))
        .or_else(|| candidates.first())
        .cloned()
        .ok_or_else(|| "该组件没有有效在线候选。".to_string())?;
    let version = candidate.version.as_str();
    let url = candidate.url.as_str();
    let sha = candidate.sha256.as_str();
    if !url.is_empty() && !url.starts_with("https://") {
        return Err("在线候选必须使用 HTTPS。".into());
    }
    if !sha.is_empty() && (sha.len() != 64 || !sha.chars().all(|value| value.is_ascii_hexdigit())) {
        return Err("在线候选 SHA256 格式无效。".into());
    }
    for (label, value) in [
        ("installDir", candidate.install_dir.as_str()),
        ("archivePath", candidate.archive_path.as_str()),
    ] {
        if !value.is_empty() {
            resolve_relative(&root, value).map_err(|error| format!("{label}: {error}"))?;
        }
    }
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let components = manifest
        .get_mut("components")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "清单缺少 components。".to_string())?;
    let item = components
        .iter_mut()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(&component_id))
        .ok_or_else(|| "组件不在清单中。".to_string())?;
    let old_version = item
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let replace_version = |text: &str| {
        if component_id == "python" {
            let old_minor = old_version.split('.').take(2).collect::<Vec<_>>().join(".");
            let new_minor = version.split('.').take(2).collect::<Vec<_>>().join(".");
            text.replace(&old_minor, &new_minor)
                .replace(&old_version, version)
        } else {
            text.replace(&old_version, version)
        }
    };
    item["version"] = json!(version);
    if !candidate.component_name.is_empty() {
        item["name"] = json!(candidate.component_name);
    }
    if !candidate.install_dir.is_empty() {
        item["installDir"] = json!(candidate.install_dir);
    } else if let Some(value) = item.get("installDir").and_then(Value::as_str) {
        item["installDir"] = json!(replace_version(value));
    }
    if !url.is_empty() {
        item["source"]["url"] = json!(url);
        if !candidate.archive_path.is_empty() {
            item["source"]["archive"] = json!(candidate.archive_path);
        } else if let Some(file_name) = url.rsplit('/').next().filter(|value| !value.is_empty()) {
            item["source"]["archive"] = json!(format!(r"downloads\packages\{file_name}"));
        }
    } else if component_id == "rust" {
        item["source"]["archive"] =
            json!(format!(r"downloads\packages\rust-{version}-standalone.zip"));
    }
    item["source"]["sha256"] = json!(sha);
    if !candidate.provider.is_empty() {
        item["source"]["provider"] = json!(candidate.provider);
    }
    item["source"]["architecture"] = json!(candidate.architecture);
    item["source"]["channel"] = json!(candidate.channel);
    let archive_root = if !candidate.archive_root.is_empty() {
        candidate.archive_root.clone()
    } else {
        match component_id.as_str() {
            "node" => format!("node-v{version}-win-x64"),
            "gradle" => format!("gradle-{version}"),
            "maven" => format!("apache-maven-{version}"),
            "mysql" => format!("mysql-{version}-winx64"),
            "python" => String::new(),
            "java" => url
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_end_matches(".zip")
                .to_string(),
            _ => item
                .get("archiveRoot")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        }
    };
    item["archiveRoot"] = json!(archive_root);
    let backup = root.join(r"Config\config-backups\manifest");
    fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
    fs::copy(
        &path,
        backup.join(format!("components-before-adopt-{started}.json.bak")),
    )
    .map_err(|error| error.to_string())?;
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "update-adopt",
        "采用在线更新候选",
        started,
        true,
        Some(0),
        format!(
            "{component_id}: {old_version} -> {version}\n来源: {} · {} · {}\nSHA256: {}\n下一步请生成安装计划。",
            candidate.provider,
            candidate.channel,
            candidate.architecture,
            if sha.is_empty() {
                "等待离线导入锁定"
            } else {
                sha
            }
        ),
    ))
}

fn visit_batch(
    id: &str,
    document: &ManifestDocument,
    selected: &HashSet<String>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    ordered: &mut Vec<String>,
    blockers: &mut Vec<String>,
) {
    if visited.contains(id) {
        return;
    }
    if !visiting.insert(id.into()) {
        blockers.push(format!("依赖环：{id}"));
        return;
    }
    let Some(item) = document.components.iter().find(|item| item.id == id) else {
        blockers.push(format!("清单缺少组件：{id}"));
        visiting.remove(id);
        return;
    };
    for dependency in &item.depends_on {
        if selected.contains(dependency) {
            visit_batch(
                dependency, document, selected, visiting, visited, ordered, blockers,
            );
        }
    }
    visiting.remove(id);
    visited.insert(id.into());
    ordered.push(id.into());
}

#[tauri::command]
fn get_batch_install_plan(component_ids: Vec<String>) -> Result<BatchInstallPlan, String> {
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let selected: HashSet<String> = component_ids.into_iter().collect();
    if selected.is_empty() {
        return Err("请选择至少一个组件。".into());
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut ordered = Vec::new();
    let mut blockers = Vec::new();
    for id in &selected {
        visit_batch(
            id,
            &document,
            &selected,
            &mut visiting,
            &mut visited,
            &mut ordered,
            &mut blockers,
        );
    }
    let mut steps = Vec::new();
    for id in &ordered {
        match get_install_plan(id.clone(), "update".into()) {
            Ok(plan) => {
                steps.push(format!("{}：{}", id, plan.steps.join(" → ")));
                for blocker in plan.blockers {
                    blockers.push(format!("{id}: {blocker}"));
                }
            }
            Err(error) => blockers.push(format!("{id}: {error}")),
        }
    }
    let ready = blockers.is_empty();
    Ok(BatchInstallPlan {
        component_ids: selected.into_iter().collect(),
        ordered_ids: ordered,
        steps,
        blockers,
        ready,
    })
}

#[tauri::command]
fn start_batch_manifest_task(
    component_ids: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    let plan = get_batch_install_plan(component_ids)?;
    if !plan.ready {
        return Err(plan.blockers.join("\n"));
    }
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\manage-component-batch.ps1");
    let manifest = root.join(r"Config\greendev\components.json");
    let ids = plan.ordered_ids.join(",");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-Ids".into(),
        ids,
        "-ManifestPath".into(),
        display_path(&manifest),
    ];
    Ok(start_process_task(
        &state,
        root,
        format!("批量更新 {} 个组件", plan.ordered_ids.len()),
        "manifest-batch".into(),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
fn rollback_component_version(component_id: String) -> Result<OperationResult, String> {
    let inventory = get_component_versions(component_id.clone())?;
    let target = inventory
        .versions
        .into_iter()
        .find(|item| !item.current && item.healthy)
        .ok_or_else(|| "没有可回退的健康版本。".to_string())?;
    switch_component_version(component_id, target.path)
}

#[tauri::command]
fn get_install_settings() -> Result<InstallSettings, String> {
    let root = frameworks_root()?;
    let path = root.join(r"Config\greendev\install-settings.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or_else(|| "安装设置格式错误。".into())
}

#[tauri::command]
fn save_install_settings(settings: InstallSettings) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if !(settings.proxy_url.is_empty()
        || settings.proxy_url.starts_with("http://")
        || settings.proxy_url.starts_with("https://")
        || settings.proxy_url.starts_with("socks5://"))
    {
        return Err("代理地址格式无效。".into());
    }
    if !advanced_ops::policy_url_allowed(&root, "allowedProxyHosts", &settings.proxy_url) {
        return Err("代理主机不在企业允许列表中。".into());
    }
    for urls in settings.mirrors.values() {
        for url in urls {
            if !(url.starts_with("https://") || url.starts_with("http://")) {
                return Err(format!("镜像 URL 格式无效：{url}"));
            }
        }
    }
    let path = root.join(r"Config\greendev\install-settings.json");
    let content =
        serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())? + "\n";
    atomic_config_write(&path, &content)?;
    Ok(finish_operation(
        &root,
        "install-settings",
        "保存安装设置",
        started,
        true,
        Some(0),
        "代理和组件镜像设置已保存。".into(),
    ))
}

#[tauri::command]
fn start_manifest_task(
    component_id: String,
    action: String,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    if !["install", "update"].contains(&action.as_str()) {
        return Err("清单操作无效。".into());
    }
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let item = document
        .components
        .iter()
        .find(|item| item.id == component_id)
        .ok_or_else(|| "组件不在清单中。".to_string())?;
    if !item.enabled || !["archive", "msi"].contains(&item.source.source_type.as_str()) {
        return Err("该清单项未启用受支持的安装来源。".into());
    }
    let script = root.join(r"Scripts\manage-component.ps1");
    let manifest = root.join(r"Config\greendev\components.json");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-Action".into(),
        action.clone(),
        "-Id".into(),
        component_id,
        "-ManifestPath".into(),
        display_path(&manifest),
    ];
    let title = format!(
        "{} {} {}",
        if action == "update" {
            "更新"
        } else {
            "安装"
        },
        item.name,
        item.version
    );
    Ok(start_process_task(
        &state,
        root,
        title,
        format!("manifest-{action}"),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
fn start_manifest_import_task(
    component_id: String,
    source_path: String,
    state: tauri::State<AppState>,
) -> Result<TaskSnapshot, String> {
    let root = frameworks_root()?;
    let document = read_manifest(&root)?;
    let item = document
        .components
        .iter()
        .find(|item| item.id == component_id)
        .ok_or_else(|| "组件不在清单中。".to_string())?;
    let source = PathBuf::from(&source_path);
    let lower = source_path.to_ascii_lowercase();
    if !source.is_file()
        || ![".zip", ".7z", ".tar.gz", ".tgz", ".tar.xz", ".msi"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
    {
        return Err("请选择存在的 ZIP、7Z、TAR 或 MSI 归档路径。".into());
    }
    let script = root.join(r"Scripts\manage-component.ps1");
    let manifest = root.join(r"Config\greendev\components.json");
    let args = vec![
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        display_path(&script),
        "-Action".into(),
        "import".into(),
        "-Id".into(),
        component_id,
        "-ManifestPath".into(),
        display_path(&manifest),
        "-ImportArchive".into(),
        display_path(&source),
    ];
    Ok(start_process_task(
        &state,
        root,
        format!("导入 {} 离线归档", item.name),
        "manifest-import".into(),
        "powershell.exe".into(),
        args,
        vec![],
        None,
    ))
}

#[tauri::command]
fn get_task(task_id: String, state: tauri::State<AppState>) -> Result<TaskSnapshot, String> {
    state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?
        .get(&task_id)
        .map(|entry| entry.snapshot.clone())
        .ok_or_else(|| "任务不存在。".into())
}

#[tauri::command]
fn cancel_task(task_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let entry = guard
        .get(&task_id)
        .ok_or_else(|| "任务不存在。".to_string())?;
    if entry.snapshot.cancelable {
        entry.cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
fn pause_task(task_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let entry = guard
        .get_mut(&task_id)
        .ok_or_else(|| "任务不存在。".to_string())?;
    if !entry.snapshot.pausable || !["queued", "running"].contains(&entry.snapshot.status.as_str())
    {
        return Err("任务当前不支持暂停。".into());
    }
    entry.pause.store(true, Ordering::Relaxed);
    if entry.snapshot.status == "queued" {
        let now = now_millis();
        entry.snapshot.status = "paused".into();
        entry.snapshot.stage = "paused".into();
        entry.snapshot.message = "任务已在队列中暂停".into();
        entry.snapshot.updated_at = now;
        entry.snapshot.queue_position = 0;
        entry.snapshot.timeline.push(TaskEvent {
            at: now,
            stage: "paused".into(),
            message: "尚未启动进程，队列位置已释放".into(),
        });
        write_task_record(&entry.spec.root, entry);
    }
    Ok(())
}

#[tauri::command]
fn resume_task(task_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let entry = guard
        .get_mut(&task_id)
        .ok_or_else(|| "任务不存在。".to_string())?;
    if entry.snapshot.status != "paused" {
        return Err("任务当前未暂停。".into());
    }
    entry.pause.store(false, Ordering::Relaxed);
    if entry.snapshot.progress == 0 {
        let now = now_millis();
        entry.snapshot.status = "queued".into();
        entry.snapshot.stage = "queued".into();
        entry.snapshot.message = "任务已返回调度队列".into();
        entry.snapshot.updated_at = now;
        entry.snapshot.queue_position = 1;
        entry.snapshot.timeline.push(TaskEvent {
            at: now,
            stage: "queued".into(),
            message: "暂停任务已恢复排队".into(),
        });
        write_task_record(&entry.spec.root, entry);
    }
    Ok(())
}

#[tauri::command]
fn retry_task(task_id: String, state: tauri::State<AppState>) -> Result<TaskSnapshot, String> {
    let spec = {
        let guard = state
            .tasks
            .lock()
            .map_err(|_| "任务状态锁异常。".to_string())?;
        let entry = guard
            .get(&task_id)
            .ok_or_else(|| "任务不存在。".to_string())?;
        if !entry.snapshot.retryable {
            return Err("任务当前没有可重试状态。".into());
        }
        let mut spec = entry.spec.clone();
        spec.attempt += 1;
        spec
    };
    Ok(launch_process_task(&state, spec))
}

#[tauri::command]
fn get_task_policy() -> Result<TaskPolicy, String> {
    let root = frameworks_root()?;
    Ok(task_policy(&root))
}

#[tauri::command]
fn save_task_policy(policy: TaskPolicy) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    if !(1..=8).contains(&policy.max_concurrent) {
        return Err("并发数应在 1–8 之间。".into());
    }
    let path = root.join(r"Config\greendev\task-policy.json");
    if path.is_file() {
        let backup = root.join(r"Config\config-backups\task-policy");
        fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        fs::copy(
            &path,
            backup.join(format!("task-policy-{started}.json.bak")),
        )
        .map_err(|e| e.to_string())?;
    }
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&policy).map_err(|e| e.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "task-policy",
        "保存任务调度策略",
        started,
        true,
        Some(0),
        format!(
            "最大并发 {}，默认优先级 {}；旧策略已归档。",
            policy.max_concurrent, policy.default_priority
        ),
    ))
}

#[tauri::command]
fn set_task_priority(
    task_id: String,
    priority: u8,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let entry = guard
        .get_mut(&task_id)
        .ok_or_else(|| "任务不存在。".to_string())?;
    if entry.snapshot.status != "queued" {
        return Err("仅排队任务可调整优先级。".into());
    }
    entry.snapshot.priority = priority;
    entry.spec.priority = priority;
    write_task_record(&entry.spec.root, entry);
    Ok(())
}

#[tauri::command]
fn reschedule_task(
    task_id: String,
    scheduled_at: u64,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut guard = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁异常。".to_string())?;
    let entry = guard
        .get_mut(&task_id)
        .ok_or_else(|| "任务不存在。".to_string())?;
    if entry.snapshot.status != "queued" {
        return Err("仅排队任务可调整计划时间。".into());
    }
    entry.snapshot.scheduled_at = scheduled_at.max(now_millis());
    entry.snapshot.message = format!(
        "计划于 {} 后执行",
        entry.snapshot.scheduled_at.saturating_sub(now_millis()) / 1000
    );
    entry.spec.scheduled_at = entry.snapshot.scheduled_at;
    write_task_record(&entry.spec.root, entry);
    Ok(())
}

#[tauri::command]
fn get_operation_logs(limit: Option<usize>) -> Result<Vec<OperationResult>, String> {
    let root = frameworks_root()?;
    let text = fs::read_to_string(root.join(r"Logs\GreenDev\operations.jsonl")).unwrap_or_default();
    let max = limit.unwrap_or(100).min(500);
    Ok(text
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str(line).ok())
        .take(max)
        .collect())
}

#[tauri::command]
fn get_diagnostics() -> Result<DiagnosticReport, String> {
    let root = frameworks_root()?;
    let mut items = Vec::new();
    let mut push_path = |id: &str, name: &str, relative: &str, file: bool| {
        let path = root.join(relative);
        let healthy = if file { path.is_file() } else { path.is_dir() };
        items.push(DiagnosticItem {
            id: id.into(),
            name: name.into(),
            healthy,
            detail: display_path(&path),
        });
    };
    push_path("root", "Frameworks 根目录", "Scripts", false);
    push_path(
        "doctor",
        "Doctor 脚本",
        r"Scripts\env-setup-output.ps1",
        true,
    );
    push_path(
        "installer",
        "清单安装脚本",
        r"Scripts\manage-component.ps1",
        true,
    );
    push_path(
        "manifest",
        "组件清单",
        r"Config\greendev\components.json",
        true,
    );
    let loader = env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join("WebView2Loader.dll"))
    });
    items.push(DiagnosticItem {
        id: "webview-loader".into(),
        name: "WebView2 Loader".into(),
        healthy: loader.as_ref().is_some_and(|path| path.is_file()),
        detail: loader
            .as_deref()
            .map(display_path)
            .unwrap_or_else(|| "当前程序目录不可用".into()),
    });
    let write_probe = root.join(r"Caches\GreenDevManager\.write-probe");
    let write_result = fs::create_dir_all(write_probe.parent().unwrap())
        .and_then(|_| fs::write(&write_probe, b"ok"))
        .and_then(|_| fs::remove_file(&write_probe));
    items.push(DiagnosticItem {
        id: "writable".into(),
        name: "根目录写入".into(),
        healthy: write_result.is_ok(),
        detail: write_result
            .err()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "Caches\\GreenDevManager 可写".into()),
    });
    let webview = background_command(system_program("reg.exe"))
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients",
            "/s",
            "/f",
            "WebView2 Runtime",
        ])
        .output()
        .ok()
        .map(|value| value.status.success())
        .unwrap_or(false)
        || background_command(system_program("reg.exe"))
            .args([
                "query",
                r"HKCU\SOFTWARE\Microsoft\EdgeUpdate\Clients",
                "/s",
                "/f",
                "WebView2 Runtime",
            ])
            .output()
            .ok()
            .map(|value| value.status.success())
            .unwrap_or(false);
    items.push(DiagnosticItem {
        id: "webview2".into(),
        name: "WebView2 Runtime".into(),
        healthy: webview,
        detail: if webview {
            "注册表检测到运行时".into()
        } else {
            "未在 EdgeUpdate 注册表项中发现独立运行时；系统 Edge 仍可能提供支持".into()
        },
    });
    match read_manifest(&root) {
        Ok(document) => items.push(DiagnosticItem {
            id: "manifest-schema".into(),
            name: "清单结构".into(),
            healthy: !document.components.is_empty(),
            detail: format!(
                "schema {} · {} 个组件",
                document.schema_version,
                document.components.len()
            ),
        }),
        Err(error) => items.push(DiagnosticItem {
            id: "manifest-schema".into(),
            name: "清单结构".into(),
            healthy: false,
            detail: error,
        }),
    }
    let log_dir = root.join(r"Logs\GreenDev");
    let log_ok = fs::create_dir_all(&log_dir).is_ok();
    items.push(DiagnosticItem {
        id: "logs".into(),
        name: "日志目录".into(),
        healthy: log_ok,
        detail: display_path(&log_dir),
    });
    items.push(DiagnosticItem {
        id: "build-mode".into(),
        name: "前端资源模式".into(),
        healthy: cfg!(feature = "custom-protocol"),
        detail: if cfg!(feature = "custom-protocol") {
            "production · 内嵌前端资源".into()
        } else {
            "development · devUrl".into()
        },
    });
    let pending = fs::read_dir(transaction_directory(&root))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                && !entry.file_name().to_string_lossy().contains(".completed.")
                && !entry.file_name().to_string_lossy().contains(".recovered.")
        })
        .count();
    items.push(DiagnosticItem {
        id: "transactions".into(),
        name: "安装事务".into(),
        healthy: pending == 0,
        detail: if pending == 0 {
            "没有待恢复事务".into()
        } else {
            format!("{pending} 个事务将在启动恢复")
        },
    });
    let healthy_count = items.iter().filter(|item| item.healthy).count();
    Ok(DiagnosticReport {
        app_version: env!("CARGO_PKG_VERSION").into(),
        generated_at: now_millis(),
        healthy_count,
        items,
    })
}

fn archive_directory(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let output = background_command(system_program("tar.exe"))
        .args([
            "-a",
            "-c",
            "-f",
            &display_path(destination),
            "-C",
            &display_path(source),
            ".",
        ])
        .output()
        .map_err(|error| format!("创建归档失败：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(())
}

fn extract_archive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let output = background_command(system_program("tar.exe"))
        .args([
            "-x",
            "-f",
            &display_path(source),
            "-C",
            &display_path(destination),
        ])
        .output()
        .map_err(|error| format!("解压归档失败：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(())
}

#[tauri::command]
fn export_diagnostic_bundle() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let stamp = now_millis();
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("diagnostic-{stamp}"));
    let destination = root
        .join("Exports")
        .join(format!("GreenDevDiagnostics-{stamp}.zip"));
    fs::create_dir_all(&stage).map_err(|error| error.to_string())?;
    let report = get_diagnostics()?;
    fs::write(
        stage.join("diagnostics.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let maintenance = get_maintenance_status()?;
    fs::write(
        stage.join("maintenance.json"),
        serde_json::to_vec_pretty(&maintenance).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    for name in ["operations.jsonl", "crash.log", "storage-history.jsonl"] {
        let source = root.join(r"Logs\GreenDev").join(name);
        if source.is_file() {
            let _ = fs::copy(source, stage.join(name));
        }
    }
    fs::write(stage.join("README.txt"), "GreenDev Manager diagnostic bundle\r\nContains runtime status and application logs; authoritative configuration contents are excluded.\r\n").map_err(|error| error.to_string())?;
    archive_directory(&stage, &destination)?;
    let _ = fs::remove_dir_all(&stage);
    Ok(finish_operation(
        &root,
        "diagnostic-export",
        "导出诊断包",
        started,
        true,
        Some(0),
        format!("诊断包: {}", display_path(&destination)),
    ))
}

fn copy_profile_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if [
            "config-backups",
            "env-backups",
            "profile-import-backups",
            "package-lock.json",
        ]
        .contains(&name.as_str())
        {
            continue;
        }
        let target = destination.join(&name);
        if entry.path().is_dir() {
            copy_profile_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn export_portable_profile() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let stamp = now_millis();
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("profile-{stamp}"));
    let destination = root
        .join("Exports")
        .join(format!("GreenDevProfile-{stamp}.zip"));
    copy_profile_tree(&root.join("Config"), &stage.join("Config"))?;
    fs::write(stage.join("profile.json"), serde_json::to_vec_pretty(&json!({"schemaVersion": 1, "createdAt": stamp, "sourceRoot": display_path(&root), "appVersion": env!("CARGO_PKG_VERSION")})).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
    archive_directory(&stage, &destination)?;
    let _ = fs::remove_dir_all(&stage);
    Ok(finish_operation(
        &root,
        "profile-export",
        "导出便携配置",
        started,
        true,
        Some(0),
        format!(
            "配置包: {}\n导入到其他盘符后运行 sync-config.bat。",
            display_path(&destination)
        ),
    ))
}

#[tauri::command]
fn import_portable_profile(source_path: String) -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let source = PathBuf::from(source_path);
    if !source.is_file()
        || source
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| !value.eq_ignore_ascii_case("zip"))
            .unwrap_or(true)
    {
        return Err("请选择存在的 GreenDevProfile ZIP。".into());
    }
    let stamp = now_millis();
    let stage = root
        .join(r"Caches\GreenDevManager")
        .join(format!("profile-import-{stamp}"));
    extract_archive(&source, &stage)?;
    let profile: Value = fs::read_to_string(stage.join("profile.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or_else(|| "配置包缺少有效 profile.json。".to_string())?;
    if profile.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("配置包 Schema 不匹配。".into());
    }
    let incoming = stage.join("Config");
    if !incoming.is_dir() {
        return Err("配置包缺少 Config 目录。".into());
    }
    let backup = root
        .join(r"Config\profile-import-backups")
        .join(stamp.to_string());
    copy_profile_tree(&root.join("Config"), &backup)?;
    copy_profile_tree(&incoming, &root.join("Config"))?;
    let _ = fs::remove_dir_all(&stage);
    let sync = sync_config_key(&root, "all").unwrap_or_else(|error| format!("同步提示：{error}"));
    Ok(finish_operation(
        &root,
        "profile-import",
        "导入便携配置",
        started,
        true,
        Some(0),
        format!(
            "来源: {}\n导入前备份: {}\n{}",
            display_path(&source),
            display_path(&backup),
            sync
        ),
    ))
}

#[tauri::command]
fn migrate_manifest_schema() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let path = root.join(r"Config\greendev\components.json");
    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let mut value: Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    let version = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if version == 2 {
        return Ok(finish_operation(
            &root,
            "manifest-migrate",
            "迁移清单 Schema",
            started,
            true,
            Some(0),
            "当前已是 Schema 2。".into(),
        ));
    }
    if version != 1 {
        return Err(format!("清单 Schema {version} 不在迁移路径中。"));
    }
    let backup = root.join(r"Config\config-backups\manifest");
    fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
    fs::copy(&path, backup.join(format!("components-{started}.json.bak")))
        .map_err(|error| error.to_string())?;
    value["schemaVersion"] = json!(2);
    atomic_config_write(
        &path,
        &(serde_json::to_string_pretty(&value).map_err(|error| error.to_string())? + "\n"),
    )?;
    Ok(finish_operation(
        &root,
        "manifest-migrate",
        "迁移清单 Schema",
        started,
        true,
        Some(0),
        "components.json 已从 Schema 1 迁移到 Schema 2，并保留原始备份。".into(),
    ))
}

#[tauri::command]
fn get_maintenance_status() -> Result<MaintenanceStatus, String> {
    let root = frameworks_root()?;
    let release_root = root.join(r"Releases\GreenDevManager");
    let mut versions: Vec<String> = fs::read_dir(&release_root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    versions.sort_by(|left, right| compare_versions(left, right));
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = versions.last().cloned().unwrap_or_else(|| current.clone());
    let pending_transactions = fs::read_dir(transaction_directory(&root))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                && !entry.file_name().to_string_lossy().contains(".completed.")
                && !entry.file_name().to_string_lossy().contains(".recovered.")
        })
        .count();
    Ok(MaintenanceStatus {
        current_version: current.clone(),
        update_available: compare_versions(&latest, &current).is_gt(),
        latest_local_version: latest,
        release_directory: display_path(&release_root),
        crash_log: display_path(&root.join(r"Logs\GreenDev\crash.log")),
        pending_transactions,
        build_mode: if cfg!(feature = "custom-protocol") {
            "production".into()
        } else {
            "development".into()
        },
    })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let escaped = display_path(path).replace('\'', "''");
    let expression = format!(
        "(Get-FileHash -LiteralPath '{}' -Algorithm SHA256).Hash",
        escaped
    );
    let output = background_command(system_program("powershell.exe"))
        .args(["-NoProfile", "-Command", &expression])
        .output()
        .map_err(|error| format!("计算 SHA256 失败：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_uppercase())
}

#[tauri::command]
fn verify_latest_release() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let status = get_maintenance_status()?;
    let directory = root
        .join(r"Releases\GreenDevManager")
        .join(&status.latest_local_version);
    let manifest_path = directory.join("release-manifest.json");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path)
            .map_err(|error| format!("读取发布清单失败：{error}"))?,
    )
    .map_err(|error| error.to_string())?;
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| "发布清单缺少 artifacts。".to_string())?;
    let mut lines = Vec::new();
    let mut valid = true;
    for artifact in artifacts {
        let name = artifact
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "发布物名称缺失。".to_string())?;
        if name.contains(['\\', '/']) {
            return Err("发布物名称无效。".into());
        }
        let expected = artifact
            .get("sha256")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_uppercase();
        let path = directory.join(name);
        let actual = sha256_file(&path).unwrap_or_default();
        let matched = !expected.is_empty() && actual == expected;
        valid &= matched;
        lines.push(format!(
            "[{}] {}",
            if matched { "OK" } else { "MISMATCH" },
            name
        ));
    }
    Ok(finish_operation(
        &root,
        "release-verify",
        "验证最新发布物",
        started,
        valid,
        Some(if valid { 0 } else { 1 }),
        format!("版本 {}\n{}", status.latest_local_version, lines.join("\n")),
    ))
}

#[tauri::command]
fn repair_current_links() -> Result<OperationResult, String> {
    let started = now_millis();
    let root = frameworks_root()?;
    let pins = read_pins(&root);
    let mut lines = Vec::new();
    let mut repaired = 0;
    for definition in version_definitions() {
        let current = root.join(definition.current);
        if current.join(definition.health).is_file() {
            lines.push(format!("[OK] {} current", definition.name));
            continue;
        }
        let mut candidates = collect_version_paths(&root.join(definition.base), definition.health);
        candidates.sort_by_key(|path| std::cmp::Reverse(display_path(path)));
        let pinned = pins
            .get(definition.id)
            .map(PathBuf::from)
            .filter(|path| path.join(definition.health).is_file());
        let Some(target) = pinned.or_else(|| candidates.into_iter().next()) else {
            lines.push(format!("[SKIP] {} 没有健康版本", definition.name));
            continue;
        };
        match switch_component_version(definition.id.into(), display_path(&target)) {
            Ok(_) => {
                repaired += 1;
                lines.push(format!(
                    "[REPAIRED] {} -> {}",
                    definition.name,
                    display_path(&target)
                ));
            }
            Err(error) => lines.push(format!("[ERROR] {}: {error}", definition.name)),
        }
    }
    Ok(finish_operation(
        &root,
        "current-repair",
        "修复 current 入口",
        started,
        true,
        Some(0),
        format!("修复 {repaired} 个入口\n{}", lines.join("\n")),
    ))
}

#[tauri::command]
fn open_dev_shell() -> Result<(), String> {
    let root = frameworks_root()?;
    let script = root.join(r"Scripts\dev-shell.bat");
    let command_line = format!("call \"{}\"", display_path(&script));
    Command::new(system_program("cmd.exe"))
        .args(["/d", "/k", &command_line])
        .current_dir(root)
        .spawn()
        .map_err(|error| format!("打开开发终端失败：{error}"))?;
    Ok(())
}

#[cfg(windows)]
fn acquire_single_instance() -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            attributes: *mut std::ffi::c_void,
            initial_owner: i32,
            name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn GetLastError() -> u32;
    }
    let name: Vec<u16> = "Local\\GreenDevManager-7FC9C4B4\0".encode_utf16().collect();
    unsafe {
        let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
        !handle.is_null() && GetLastError() != 183
    }
}

#[cfg(not(windows))]
fn acquire_single_instance() -> bool {
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if !acquire_single_instance() {
        return;
    }
    let state = AppState::default();
    if let Ok(root) = frameworks_root() {
        let _ = restore_persisted_tasks(&state, &root);
        recover_transactions(&root);
    }
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(root) = frameworks_root() {
            let directory = root.join(r"Logs\GreenDev");
            let _ = fs::create_dir_all(&directory);
            use std::io::Write;
            if let Ok(mut file) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(directory.join("crash.log"))
            {
                let _ = writeln!(file, "{} | {}", now_millis(), info);
            }
        }
    }));
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_bootstrap_status,
            select_frameworks_directory,
            initialize_frameworks_root,
            get_dashboard,
            scan_storage,
            run_doctor,
            sync_configs,
            preview_cleanup,
            apply_cleanup,
            configure_environment,
            list_environment_backups,
            restore_environment_backup,
            get_config_statuses,
            get_config_document,
            preview_config_change,
            apply_config_change,
            rollback_config,
            preview_config_backup,
            get_component_versions,
            switch_component_version,
            set_component_pin,
            get_android_packages,
            start_android_task,
            get_manifest_components,
            get_install_plan,
            get_install_settings,
            save_install_settings,
            check_component_updates,
            start_update_catalog_task,
            adopt_update_candidate,
            get_batch_install_plan,
            start_batch_manifest_task,
            rollback_component_version,
            start_manifest_task,
            start_manifest_import_task,
            get_task,
            get_tasks,
            cancel_task,
            pause_task,
            resume_task,
            retry_task,
            get_task_policy,
            save_task_policy,
            set_task_priority,
            reschedule_task,
            get_operation_logs,
            get_storage_history,
            get_diagnostics,
            export_diagnostic_bundle,
            export_portable_profile,
            import_portable_profile,
            migrate_manifest_schema,
            get_maintenance_status,
            verify_latest_release,
            repair_current_links,
            open_dev_shell,
            advanced::get_manifest_editor,
            advanced::preview_manifest_editor,
            advanced::save_manifest_editor,
            advanced::get_trust_policy,
            advanced::save_trust_policy,
            advanced::get_app_update_status,
            advanced::save_app_update_settings,
            advanced::start_app_feed_task,
            advanced::start_app_download_task,
            advanced::prepare_app_update,
            advanced::apply_prepared_app_update,
            advanced::get_profile_sets,
            advanced::save_profile_sets,
            advanced::build_profile_lock,
            advanced::diff_profile,
            advanced::export_offline_profile,
            advanced::export_incremental_profile,
            advanced::export_supply_chain_inventory,
            advanced_ops::get_recovery_center,
            advanced_ops::preview_recovery_item,
            advanced_ops::restore_recovery_item,
            advanced_ops::get_enterprise_status,
            advanced_ops::save_enterprise_policy,
            advanced_ops::start_team_sync_task,
            advanced_ops::export_audit_bundle,
            phase20_23::get_reliability_status,
            phase20_23::save_reliability_policy,
            phase20_23::archive_operation_log,
            phase20_23::run_performance_baseline,
            phase20_23::get_supply_chain_status,
            phase20_23::save_supply_chain_policy,
            phase20_23::verify_supply_chain,
            phase20_23::get_fleet_status,
            phase20_23::start_fleet_inventory_task,
            phase20_23::save_fleet_config,
            phase20_23::preview_fleet_rollout,
            phase20_23::stage_fleet_rollout,
            phase20_23::set_fleet_rollout_state,
            phase20_23::start_fleet_rollout_task,
            phase20_23::get_ecosystem_status,
            phase20_23::generate_manifest_template
        ])
        .run(tauri::generate_context!())
        .expect("error while running GreenDev Manager");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_ids_are_restricted() {
        assert!(valid_package_id("platforms;android-36"));
        assert!(!valid_package_id("platform-tools & whoami"));
    }
    #[test]
    fn sdkmanager_table_is_parsed() {
        let input = "Installed packages:\nPath | Version | Description | Location\n-------\nplatform-tools | 36.0.0 | Android SDK Platform-Tools | platform-tools\nAvailable Packages:\nplatforms;android-37 | 1 | Android SDK Platform 37 |\n";
        let values = parse_sdkmanager_output(input);
        assert_eq!(values.len(), 2);
        assert!(values
            .iter()
            .any(|item| item.id == "platform-tools" && item.installed));
    }
    #[test]
    fn fingerprints_are_stable() {
        assert_eq!(fnv_hash(b"abc"), "E71FA2190541574B");
    }
    #[test]
    fn windows_system_tools_do_not_depend_on_path() {
        for name in [
            "cmd.exe",
            "powershell.exe",
            "curl.exe",
            "reg.exe",
            "tar.exe",
        ] {
            let path = system_program(name);
            assert!(path.is_file(), "missing system tool: {}", path.display());
        }
    }
    #[cfg(windows)]
    #[test]
    fn background_powershell_task_starts_with_empty_path() {
        let console_probe = r#"Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class ConsoleProbe { [DllImport("kernel32.dll")] public static extern IntPtr GetConsoleWindow(); }'; [ConsoleProbe]::GetConsoleWindow().ToInt64()"#;
        let probe = background_command(system_program("powershell.exe"))
            .args(["-NoProfile", "-NonInteractive", "-Command", console_probe])
            .output()
            .unwrap();
        assert!(probe.status.success());
        assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "0");

        let root = env::temp_dir().join(format!("greendev-task-path-test-{}", now_millis()));
        fs::create_dir_all(&root).unwrap();
        let state = AppState::default();
        let snapshot = start_process_task(
            &state,
            root.clone(),
            "PATH fixture".into(),
            "path-fixture".into(),
            "powershell.exe".into(),
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                "Write-Output PATH_OK".into(),
            ],
            vec![("PATH".into(), String::new())],
            None,
        );
        for _ in 0..80 {
            let current = state
                .tasks
                .lock()
                .unwrap()
                .get(&snapshot.id)
                .unwrap()
                .snapshot
                .clone();
            if current.status == "completed" {
                assert!(current.result.unwrap().output.contains("PATH_OK"));
                let _ = fs::remove_dir_all(root);
                return;
            }
            assert_ne!(current.status, "failed", "task failed: {}", current.message);
            thread::sleep(Duration::from_millis(100));
        }
        panic!("background PowerShell task timed out");
    }
    #[test]
    fn semantic_versions_sort_numerically() {
        let mut values = vec!["0.7.0", "0.11.0", "1.0.0"];
        values.sort_by(|left, right| compare_versions(left, right));
        assert_eq!(values, vec!["0.7.0", "0.11.0", "1.0.0"]);
        assert!(compare_versions("1.0.0", "1.0.0-rc.1").is_gt());
        assert!(compare_versions("1.0.0-rc.2", "1.0.0-rc.1").is_gt());
    }
    #[test]
    fn bundled_manifest_is_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        let document = read_manifest(&root).expect("bundled manifest should parse");
        assert!((1..=2).contains(&document.schema_version));
        assert!(document.components.len() >= 3);
        assert!(document
            .components
            .iter()
            .all(|item| item.enabled && item.source.source_type == "archive"));
    }
    #[test]
    fn batch_plan_orders_dependencies_first() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        let document = read_manifest(&root).expect("manifest");
        let selected = HashSet::from(["java".to_string(), "gradle".to_string()]);
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut ordered = Vec::new();
        let mut blockers = Vec::new();
        visit_batch(
            "gradle",
            &document,
            &selected,
            &mut visiting,
            &mut visited,
            &mut ordered,
            &mut blockers,
        );
        assert!(blockers.is_empty());
        assert!(
            ordered.iter().position(|id| id == "java")
                < ordered.iter().position(|id| id == "gradle")
        );
    }
    #[test]
    fn storage_points_round_trip() {
        let point = StoragePoint {
            recorded_at: 1,
            total_size_bytes: 2,
            cache_size_bytes: 3,
        };
        let json = serde_json::to_string(&point).unwrap();
        let decoded: StoragePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cache_size_bytes, 3);
    }
    #[test]
    fn persistent_task_records_keep_resume_spec() {
        let root = PathBuf::from(r"D:\Frameworks");
        let snapshot = TaskSnapshot {
            id: "task-1".into(),
            title: "fixture".into(),
            kind: "fixture".into(),
            status: "queued".into(),
            progress: 0,
            message: "queued".into(),
            cancelable: true,
            pausable: true,
            retryable: false,
            stage: "queued".into(),
            bytes_processed: 0,
            bytes_total: 0,
            bytes_per_second: 0,
            eta_seconds: None,
            attempt: 1,
            priority: 50,
            scheduled_at: 1,
            queue_position: 1,
            timeline: Vec::new(),
            started_at: 1,
            updated_at: 1,
            result: None,
        };
        let record = PersistedTask {
            schema_version: 1,
            snapshot,
            spec: TaskSpec {
                root,
                title: "fixture".into(),
                kind: "fixture".into(),
                program: "powershell.exe".into(),
                args: vec!["-NoProfile".into()],
                envs: vec![("GREENDEV_TEST".into(), "1".into())],
                cache_output: None,
                attempt: 1,
                priority: 50,
                scheduled_at: 1,
                start_paused: false,
            },
        };
        let encoded = serde_json::to_vec(&record).unwrap();
        let decoded: PersistedTask = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.snapshot.status, "queued");
        assert_eq!(decoded.spec.program, "powershell.exe");
        assert_eq!(decoded.spec.envs[0].0, "GREENDEV_TEST");
    }
    #[test]
    fn persistent_tasks_exclude_sensitive_resume_values() {
        let mut spec = TaskSpec {
            root: PathBuf::from(r"D:\Frameworks"),
            title: "fixture".into(),
            kind: "fixture".into(),
            program: "powershell.exe".into(),
            args: vec!["-Token".into(), "TOKEN_VALUE".into()],
            envs: Vec::new(),
            cache_output: None,
            attempt: 1,
            priority: 50,
            scheduled_at: 1,
            start_paused: false,
        };
        assert!(task_spec_has_sensitive_values(&spec));
        spec.args = vec!["-NoProfile".into()];
        spec.envs = vec![("SERVICE_PASSWORD".into(), "PASSWORD_VALUE".into())];
        assert!(task_spec_has_sensitive_values(&spec));
    }
    #[test]
    fn backup_retention_keeps_latest_entries() {
        let root = env::temp_dir().join(format!("greendev-backup-test-{}", now_millis()));
        let directory = config_backup_directory(&root, "test");
        fs::create_dir_all(&directory).unwrap();
        for index in 0..5 {
            fs::write(directory.join(format!("{index}.bak")), index.to_string()).unwrap();
        }
        prune_config_backups(&root, "test", 3);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 3);
        assert!(directory.join("4.bak").is_file());
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn bundled_configs_pass_validation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(r"..\..\..");
        for definition in config_definitions() {
            let content =
                fs::read_to_string(root.join(definition.source)).expect("config should exist");
            let errors = validate_config_content(definition, &content);
            assert!(errors.is_empty(), "{}: {:?}", definition.id, errors);
        }
    }
    #[test]
    fn config_validation_rejects_invalid_content() {
        let definition = find_config_definition("cargo").unwrap();
        assert!(!validate_config_content(definition, "[source\nregistry =").is_empty());
        let mysql = find_config_definition("mysql").unwrap();
        assert!(validate_config_content(mysql, "[mysqld]\nport=3306\n")
            .iter()
            .any(|value| value.contains("FRAMEWORKS_HOME_FWD")));
    }
    #[test]
    fn relative_manifest_paths_are_confined() {
        let root = Path::new(r"D:\Frameworks");
        assert!(resolve_relative(root, r"downloads\package.zip").is_ok());
        assert!(resolve_relative(root, r"..\outside.zip").is_err());
        assert!(resolve_relative(root, r"C:\outside.zip").is_err());
    }
    #[test]
    fn multi_source_catalog_candidates_preserve_provider_metadata() {
        let value = json!({
            "status": "ok",
            "candidates": [
                {
                    "id": "java-temurin-17-x64",
                    "provider": "Eclipse Temurin",
                    "version": "17.0.20+8",
                    "architecture": "x64",
                    "channel": "LTS",
                    "url": "https://example.invalid/temurin.zip",
                    "sha256": "418497BE5CF585BDD2203D6486A565D66D3F5E992D5630D45104CB873FAB8122",
                    "archiveRoot": "jdk-17.0.20+8",
                    "installDir": "Runtimes\\Java\\jdk-17\\temurin-jdk-17.0.20+8",
                    "archivePath": "downloads\\packages\\temurin.zip",
                    "componentName": "Eclipse Temurin JDK",
                    "notes": "Temurin 17 LTS"
                }
            ]
        });
        let candidates = catalog_candidates(Some(&value));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "java-temurin-17-x64");
        assert_eq!(candidates[0].provider, "Eclipse Temurin");
        assert_eq!(candidates[0].architecture, "x64");
        assert!(candidates[0].checksum_ready);
        assert!(candidates[0].install_dir.contains(r"jdk-17"));
    }
    #[test]
    fn task_transaction_records_resumable_metadata() {
        let snapshot = TaskSnapshot {
            id: "task-1".into(),
            title: "test".into(),
            kind: "download".into(),
            status: "paused".into(),
            progress: 42,
            message: "paused".into(),
            cancelable: true,
            pausable: true,
            retryable: false,
            stage: "paused".into(),
            bytes_processed: 1024,
            bytes_total: 4096,
            bytes_per_second: 512,
            eta_seconds: Some(6),
            attempt: 2,
            priority: 50,
            scheduled_at: 1,
            queue_position: 0,
            timeline: vec![],
            started_at: 1,
            updated_at: 2,
            result: None,
        };
        let value = serde_json::to_value(snapshot).unwrap();
        assert_eq!(value["bytesProcessed"], 1024);
        assert_eq!(value["etaSeconds"], 6);
        assert_eq!(value["attempt"], 2);
    }
    #[test]
    fn bootstrap_root_accepts_any_directory_name() {
        let root = env::temp_dir().join(format!("custom-dev-root-{}", now_millis()));
        fs::create_dir_all(root.join("Scripts")).unwrap();
        fs::write(root.join("env-setup.bat"), "@echo off\r\n").unwrap();
        assert!(is_frameworks_root(&root));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn bootstrap_archive_entries_stay_relative() {
        assert!(safe_bootstrap_entry("Scripts/frameworks-common.ps1"));
        assert!(safe_bootstrap_entry("Config/greendev/components.json"));
        assert!(!safe_bootstrap_entry("../outside.txt"));
        assert!(!safe_bootstrap_entry(r"C:\outside.txt"));
        assert!(!safe_bootstrap_entry(r"\outside.txt"));
    }
    #[test]
    fn bootstrap_copy_preserves_tree() {
        let base = env::temp_dir().join(format!("greendev-bootstrap-copy-{}", now_millis()));
        let source = base.join("source");
        let destination = base.join("destination");
        fs::create_dir_all(source.join(r"Config\greendev")).unwrap();
        fs::write(source.join("env-setup.bat"), "@echo off\r\n").unwrap();
        fs::write(source.join(r"Config\greendev\components.json"), "{}").unwrap();
        copy_bootstrap_tree(&source, &destination).unwrap();
        assert!(destination.join("env-setup.bat").is_file());
        assert!(destination
            .join(r"Config\greendev\components.json")
            .is_file());
        let _ = fs::remove_dir_all(base);
    }
}
