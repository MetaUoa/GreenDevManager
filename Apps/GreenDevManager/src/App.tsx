import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity, ArchiveRestore, Boxes, Check, ChevronRight, CircleAlert, Database,
  Code2, Download, FileArchive, FileClock, FolderCog, FolderOpen, Gauge, HardDrive, Layers3, ListTodo, LoaderCircle, Moon,
  MonitorCog, PackageCheck, Pin, PinOff, Play, RefreshCw, RotateCcw, Save, Search, Settings2,
  ShieldCheck, Smartphone, Sparkles, Square, Sun, TerminalSquare, Trash2, Upload, Wrench, X
} from "lucide-react";
import type {
  AndroidPackage, BackupPreview, BatchInstallPlan, CacheEntry, ComponentStatus, ConfigDocument, ConfigPreview, ConfigStatus, Dashboard,
  DiagnosticReport, EnvironmentBackup, InstallPlan, InstallSettings, MaintenanceStatus, ManifestComponent, OperationResult,
  AppUpdateStatus, BootstrapStatus, EcosystemStatus, EnterpriseStatus, FleetStatus, ManifestEditor, ProfileDiff, ProfileSets, RecoveryCenter, ReliabilityStatus, StorageMetrics, StoragePoint, SupplyChainStatus, TaskPolicy, TaskSnapshot, UpdateCandidate, VersionInventory
} from "./types";

type View = "overview" | "components" | "android" | "manifest" | "catalog" | "updater" | "profiles" | "recovery" | "stability" | "supply" | "fleet" | "developer" | "enterprise" | "environment" | "cache" | "config" | "diagnostics" | "logs";
type ConfirmState = { title: string; detail: string; label: string; destructive?: boolean; disabled?: boolean; action: () => void };

const configurable = ["java", "node", "gradle", "maven", "android", "rust", "python", "c", "acpi", "mysql"];
const versioned = new Set(["java", "node", "python", "gradle", "maven", "rust", "mysql"]);
const isDesktop = Boolean(window.__TAURI_INTERNALS__);

const mockDashboard: Dashboard = {
  root: "D:\\Frameworks", totalSizeBytes: 0, cacheSizeBytes: 0, storageReady: false,
  installedCount: 3, healthyCount: 3,
  components: [
    ["java", "Java", "运行时", "zulu21"], ["node", "Node.js / npm", "运行时", "v24.18.0"],
    ["android", "Android SDK", "平台", "SDK"]
  ].map(([id, name, category, version]) => ({ id, name, category, version, installed: true, healthy: true,
    executable: `D:\\Frameworks\\${id}`, currentPath: versioned.has(id) ? `D:\\Frameworks\\${id}\\current` : null,
    currentTarget: versioned.has(id) ? `D:\\Frameworks\\${id}\\${version}` : null, environmentVariables: [] })),
  caches: []
};

const navItems: Array<{ id: View; label: string; icon: typeof Gauge }> = [
  { id: "overview", label: "总览", icon: Gauge },
  { id: "components", label: "版本管理", icon: Boxes },
  { id: "android", label: "Android SDK", icon: Smartphone },
  { id: "manifest", label: "安装与更新", icon: Download },
  { id: "catalog", label: "清单与插件", icon: Code2 },
  { id: "updater", label: "应用更新", icon: RefreshCw },
  { id: "profiles", label: "环境档案", icon: Layers3 },
  { id: "recovery", label: "恢复中心", icon: ArchiveRestore },
  { id: "stability", label: "稳定性中心", icon: Activity },
  { id: "supply", label: "供应链安全", icon: ShieldCheck },
  { id: "fleet", label: "远程节点", icon: MonitorCog },
  { id: "developer", label: "开发者生态", icon: Code2 },
  { id: "enterprise", label: "团队与合规", icon: ShieldCheck },
  { id: "environment", label: "环境配置", icon: MonitorCog },
  { id: "cache", label: "缓存管理", icon: HardDrive },
  { id: "config", label: "配置中心", icon: FolderCog },
  { id: "diagnostics", label: "诊断与发布", icon: Wrench },
  { id: "logs", label: "操作日志", icon: FileClock }
];

function formatBytes(bytes: number) {
  if (bytes === 0) return "—";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index >= 3 ? 2 : 1)} ${units[index]}`;
}

function compareVersions(left: string, right: string) {
  const parse = (value: string) => {
    const [core, prerelease = ""] = value.replace(/^v/, "").split("-", 2);
    return { core: core.split(".").map(part => Number(part) || 0), prerelease: prerelease.split(".").filter(Boolean) };
  };
  const a = parse(left); const b = parse(right);
  for (let index = 0; index < Math.max(a.core.length, b.core.length); index++) {
    const difference = (a.core[index] ?? 0) - (b.core[index] ?? 0); if (difference) return Math.sign(difference);
  }
  if (!a.prerelease.length || !b.prerelease.length) return a.prerelease.length === b.prerelease.length ? 0 : a.prerelease.length ? -1 : 1;
  for (let index = 0; index < Math.max(a.prerelease.length, b.prerelease.length); index++) {
    const leftPart = a.prerelease[index]; const rightPart = b.prerelease[index];
    if (leftPart === undefined || rightPart === undefined) return leftPart === rightPart ? 0 : leftPart === undefined ? -1 : 1;
    if (leftPart === rightPart) continue;
    const leftNumber = /^\d+$/.test(leftPart) ? Number(leftPart) : null; const rightNumber = /^\d+$/.test(rightPart) ? Number(rightPart) : null;
    if (leftNumber !== null && rightNumber !== null) return Math.sign(leftNumber - rightNumber);
    if (leftNumber !== null || rightNumber !== null) return leftNumber !== null ? -1 : 1;
    return leftPart.localeCompare(rightPart);
  }
  return 0;
}

function fallbackResult(title: string, output: string, success = true): OperationResult {
  const now = Date.now();
  return { operationId: `preview-${now}`, success, title, summary: output.split("\n")[0], output, exitCode: success ? 0 : null, kind: "preview", startedAt: now, finishedAt: now };
}

function App() {
  const [view, setView] = useState<View>(() => (localStorage.getItem("greendev-view") as View) || "overview");
  const [theme, setTheme] = useState<"light" | "dark">(() => localStorage.getItem("greendev-theme") === "dark" ? "dark" : "light");
  const [globalQuery, setGlobalQuery] = useState("");
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [logs, setLogs] = useState<OperationResult[]>([]);
  const [result, setResult] = useState<OperationResult | null>(null);
  const [configs, setConfigs] = useState<ConfigStatus[]>([]);
  const [backups, setBackups] = useState<EnvironmentBackup[]>([]);
  const [androidPackages, setAndroidPackages] = useState<AndroidPackage[]>([]);
  const [manifest, setManifest] = useState<ManifestComponent[]>([]);
  const [installSettings, setInstallSettings] = useState<InstallSettings>({ proxyUrl: "", mirrors: {} });
  const [diagnostics, setDiagnostics] = useState<DiagnosticReport | null>(null);
  const [maintenance, setMaintenance] = useState<MaintenanceStatus | null>(null);
  const [updates, setUpdates] = useState<UpdateCandidate[]>([]);
  const [storageHistory, setStorageHistory] = useState<StoragePoint[]>([]);
  const [tasks, setTasks] = useState<TaskSnapshot[]>([]);
  const [taskCenterOpen, setTaskCenterOpen] = useState(false);
  const [profileImportOpen, setProfileImportOpen] = useState(false);
  const [environmentOpen, setEnvironmentOpen] = useState(false);
  const [selectedComponents, setSelectedComponents] = useState<Set<string>>(new Set(configurable));
  const [cleanupPreview, setCleanupPreview] = useState<OperationResult | null>(null);
  const [cleanupLevel, setCleanupLevel] = useState<"safe" | "normal">("normal");
  const [includeWrapper, setIncludeWrapper] = useState(false);
  const [versionInventory, setVersionInventory] = useState<VersionInventory | null>(null);
  const [configDocument, setConfigDocument] = useState<ConfigDocument | null>(null);
  const [importTarget, setImportTarget] = useState<ManifestComponent | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [activeTask, setActiveTask] = useState<TaskSnapshot | null>(null);
  const [confirm, setConfirm] = useState<ConfirmState | null>(null);
  const [manifestEditor, setManifestEditor] = useState<ManifestEditor | null>(isDesktop ? null : { raw: JSON.stringify({ schemaVersion: 2, components: [] }, null, 2), baseHash: "preview", errors: [], path: "Config\\greendev\\components.json" });
  const [trustPolicy, setTrustPolicy] = useState<Record<string, unknown>>({});
  const [appUpdate, setAppUpdate] = useState<AppUpdateStatus | null>(isDesktop ? null : { currentVersion: "1.0.1", latestLocalVersion: "1.0.1", latestRemoteVersion: "", targetVersion: "1.0.1", updateAvailable: false, prepared: false, localVersions: ["1.0.1"], settings: { schemaVersion: 1, channel: "stable", feedUrl: "https://github.com/MetaUoa/GreenDevManager/releases/latest/download/update-feed.json", requireSignature: false, autoDownload: false }, feed: {} });
  const [profileSets, setProfileSets] = useState<ProfileSets | null>(isDesktop ? null : { schemaVersion: 1, activeProfile: "default", profiles: [{ id: "default", name: "默认开发环境", components: [], teamTemplate: false, machineOverrides: {} }] });
  const [profileDiff, setProfileDiff] = useState<ProfileDiff | null>(null);
  const [taskPolicy, setTaskPolicy] = useState<TaskPolicy>({ maxConcurrent: 2, defaultPriority: 50, notifications: true });
  const [recoveryCenter, setRecoveryCenter] = useState<RecoveryCenter>({ items: [], pendingTransactions: 0 });
  const [enterprise, setEnterprise] = useState<EnterpriseStatus | null>(null);
  const [reliability, setReliability] = useState<ReliabilityStatus | null>(null);
  const [supplyChain, setSupplyChain] = useState<SupplyChainStatus | null>(null);
  const [fleet, setFleet] = useState<FleetStatus | null>(null);
  const [ecosystem, setEcosystem] = useState<EcosystemStatus | null>(null);
  const [bootstrap, setBootstrap] = useState<BootstrapStatus | null>(isDesktop ? null : { configured: true, root: mockDashboard.root, currentVersion: "1.0.1", manifestUrl: "" });
  const [bootstrapBusy, setBootstrapBusy] = useState(false);
  const [bootstrapError, setBootstrapError] = useState("");

  async function loadDashboard() {
    setBusy("refresh");
    try {
      const data = isDesktop ? await invoke<Dashboard>("get_dashboard") : mockDashboard;
      setDashboard(data);
      setBusy(null);
      if (isDesktop) {
        void invoke<StorageMetrics>("scan_storage").then(storage => { setDashboard(current => current ? { ...current, ...storage, storageReady: true } : current); return invoke<StoragePoint[]>("get_storage_history", { limit: 30 }); }).then(setStorageHistory);
      }
    } catch (error) {
      setResult(fallbackResult("读取环境失败", String(error), false));
      setBusy(null);
    }
  }

  async function loadSupportingData() {
    if (!isDesktop) return;
    const [savedLogs, configStates, envBackups, sdkPackages, manifestItems, settings, report, updateItems, taskItems, history, maintenanceStatus, editor, trust, appStatus, profiles, queuePolicy, recovery, enterpriseStatus, reliabilityStatus, supplyStatus, fleetStatus, ecosystemStatus] = await Promise.all([
      invoke<OperationResult[]>("get_operation_logs", { limit: 150 }),
      invoke<ConfigStatus[]>("get_config_statuses"),
      invoke<EnvironmentBackup[]>("list_environment_backups"),
      invoke<AndroidPackage[]>("get_android_packages"),
      invoke<ManifestComponent[]>("get_manifest_components"),
      invoke<InstallSettings>("get_install_settings"),
      invoke<DiagnosticReport>("get_diagnostics"),
      invoke<UpdateCandidate[]>("check_component_updates"),
      invoke<TaskSnapshot[]>("get_tasks"),
      invoke<StoragePoint[]>("get_storage_history", { limit: 30 }),
      invoke<MaintenanceStatus>("get_maintenance_status"),
      invoke<ManifestEditor>("get_manifest_editor"),
      invoke<Record<string, unknown>>("get_trust_policy"),
      invoke<AppUpdateStatus>("get_app_update_status"),
      invoke<ProfileSets>("get_profile_sets"),
      invoke<TaskPolicy>("get_task_policy"),
      invoke<RecoveryCenter>("get_recovery_center"),
      invoke<EnterpriseStatus>("get_enterprise_status"),
      invoke<ReliabilityStatus>("get_reliability_status"),
      invoke<SupplyChainStatus>("get_supply_chain_status"),
      invoke<FleetStatus>("get_fleet_status"),
      invoke<EcosystemStatus>("get_ecosystem_status")
    ]);
    setLogs(savedLogs); setConfigs(configStates); setBackups(envBackups); setAndroidPackages(sdkPackages); setManifest(manifestItems); setInstallSettings(settings); setDiagnostics(report); setUpdates(updateItems); setTasks(taskItems); setStorageHistory(history); setMaintenance(maintenanceStatus); setManifestEditor(editor); setTrustPolicy(trust); setAppUpdate(appStatus); setProfileSets(profiles); setTaskPolicy(queuePolicy); setRecoveryCenter(recovery); setEnterprise(enterpriseStatus); setReliability(reliabilityStatus); setSupplyChain(supplyStatus); setFleet(fleetStatus); setEcosystem(ecosystemStatus);
  }

  useEffect(() => {
    window.scrollTo(0, 0);
    if (!isDesktop) { void loadDashboard(); return; }
    void invoke<BootstrapStatus>("get_bootstrap_status").then(status => {
      setBootstrap(status);
      if (status.configured) {
        void loadDashboard();
        void loadSupportingData().catch(error => setResult(fallbackResult("读取管理数据失败", String(error), false)));
      }
    }).catch(error => { setBootstrap({ configured: false, root: "", currentVersion: "", manifestUrl: "" }); setBootstrapError(String(error)); });
  }, []);

  useEffect(() => { localStorage.setItem("greendev-view", view); window.scrollTo(0, 0); }, [view]);
  useEffect(() => { localStorage.setItem("greendev-theme", theme); document.documentElement.dataset.theme = theme; }, [theme]);
  useEffect(() => { const navigate = (event: KeyboardEvent) => { if (event.ctrlKey && event.key.toLowerCase() === "k") { event.preventDefault(); document.querySelector<HTMLInputElement>(".global-search input")?.focus(); return; } if (!event.ctrlKey || !["PageDown", "PageUp"].includes(event.key)) return; event.preventDefault(); setView(current => { const index = navItems.findIndex(item => item.id === current); const delta = event.key === "PageDown" ? 1 : -1; return navItems[(index + delta + navItems.length) % navItems.length].id; }); }; window.addEventListener("keydown", navigate); return () => window.removeEventListener("keydown", navigate); }, []);

  async function refreshAfterChange() {
    await loadDashboard();
    await loadSupportingData();
  }

  async function runOperation(key: string, command: string, args: Record<string, unknown> = {}, refresh = false) {
    setBusy(key);
    try {
      const operation = isDesktop ? await invoke<OperationResult>(command, args) : fallbackResult("界面预览", "桌面构建会在此返回结构化操作结果。");
      setLogs(previous => [operation, ...previous.filter(item => item.operationId !== operation.operationId)]);
      setResult(operation);
      if (refresh) await refreshAfterChange();
      return operation;
    } catch (error) {
      const operation = fallbackResult("操作失败", String(error), false);
      setLogs(previous => [operation, ...previous]); setResult(operation); return operation;
    } finally { setBusy(null); }
  }

  async function startManagedTask(command: string, args: Record<string, unknown>) {
    try {
      let task = isDesktop ? await invoke<TaskSnapshot>(command, args) : { id: "preview", title: "界面预览任务", kind: "preview", status: "completed" as const, progress: 100, message: "完成", cancelable: false, pausable: false, retryable: false, stage: "completed", bytesProcessed: 0, bytesTotal: 0, bytesPerSecond: 0, etaSeconds: null, attempt: 1, priority: 50, scheduledAt: Date.now(), queuePosition: 0, timeline: [], startedAt: Date.now(), updatedAt: Date.now(), result: fallbackResult("界面预览任务", "后台任务输出") };
      setActiveTask(task); setTasks(previous => [task, ...previous.filter(item => item.id !== task.id)]);
      while (isDesktop && (task.status === "queued" || task.status === "running" || task.status === "paused")) {
        await new Promise(resolve => window.setTimeout(resolve, 650));
        task = await invoke<TaskSnapshot>("get_task", { taskId: task.id });
        setActiveTask(task); setTasks(previous => [task, ...previous.filter(item => item.id !== task.id)]);
      }
      if (task.result) {
        setLogs(previous => [task.result!, ...previous.filter(item => item.operationId !== task.result!.operationId)]);
        setResult(task.result);
        if (taskPolicy.notifications && "Notification" in window && Notification.permission === "granted") new Notification(task.title, { body: task.message });
      }
      await refreshAfterChange();
    } catch (error) {
      setResult(fallbackResult("后台任务失败", String(error), false));
    } finally { setActiveTask(null); }
  }

  async function openVersions(componentId: string) {
    setBusy(`versions-${componentId}`);
    try { setVersionInventory(isDesktop ? await invoke("get_component_versions", { componentId }) : null); }
    catch (error) { setResult(fallbackResult("读取版本失败", String(error), false)); }
    finally { setBusy(null); }
  }

  async function openConfigEditor(id: string) {
    setBusy(`config-${id}`);
    try { setConfigDocument(isDesktop ? await invoke<ConfigDocument>("get_config_document", { id }) : null); }
    catch (error) { setResult(fallbackResult("读取配置失败", String(error), false)); }
    finally { setBusy(null); }
  }

  async function planManifestAction(item: ManifestComponent, action: "install" | "update") {
    try {
      const plan = isDesktop ? await invoke<InstallPlan>("get_install_plan", { componentId: item.id, action }) : { componentId: item.id, action, steps: ["界面预览"], blockers: [], archivePath: item.archivePath, sourceUrl: item.sourceUrl, expectedSha256: "", ready: true };
      const detail = [`目标版本: ${item.desiredVersion}`, `安装位置: ${item.installDir}`, `归档: ${plan.archivePath}`, `SHA256: ${plan.expectedSha256 || "等待离线导入锁定"}`, "", "执行步骤:", ...plan.steps.map((step, index) => `${index + 1}. ${step}`), ...(plan.blockers.length ? ["", "阻塞项:", ...plan.blockers.map(value => `- ${value}`)] : [])].join("\n");
      setConfirm({ title: `${action === "update" ? "更新" : "安装"} ${item.name}`, detail, label: plan.ready ? "开始执行" : "存在阻塞项", disabled: !plan.ready, action: () => { if (plan.ready) void startManagedTask("start_manifest_task", { componentId: item.id, action }); } });
    } catch (error) { setResult(fallbackResult("生成安装计划失败", String(error), false)); }
  }

  async function refreshVersionInventory(componentId: string) {
    if (isDesktop) setVersionInventory(await invoke("get_component_versions", { componentId }));
  }

  async function previewCleanup() {
    setBusy("cleanup-preview");
    try {
      const operation = isDesktop ? await invoke<OperationResult>("preview_cleanup", { level: cleanupLevel, includeWrapper }) : fallbackResult("缓存清理预览", "没有发现可清理项。");
      setCleanupPreview(operation); setLogs(previous => [operation, ...previous]);
    } catch (error) { setResult(fallbackResult("预览失败", String(error), false)); }
    finally { setBusy(null); }
  }

  async function planBatch(componentIds: string[]) {
    try {
      const plan = await invoke<BatchInstallPlan>("get_batch_install_plan", { componentIds });
      const detail = [`执行顺序: ${plan.orderedIds.join(" → ")}`, "", ...plan.steps, ...(plan.blockers.length ? ["", "阻塞项:", ...plan.blockers.map(value => `- ${value}`)] : [])].join("\n");
      setConfirm({ title: `批量更新 ${plan.orderedIds.length} 个组件`, detail, label: plan.ready ? "按顺序执行" : "存在阻塞项", disabled: !plan.ready, action: () => void startManagedTask("start_batch_manifest_task", { componentIds: plan.orderedIds }) });
    } catch (error) { setResult(fallbackResult("批量计划失败", String(error), false)); }
  }

  async function openShell() {
    setBusy("shell");
    try { if (isDesktop) await invoke("open_dev_shell"); }
    catch (error) { setResult(fallbackResult("打开终端失败", String(error), false)); }
    finally { setBusy(null); }
  }

  async function browseBootstrapDirectory() {
    setBootstrapError("");
    try { return isDesktop ? await invoke<string | null>("select_frameworks_directory") : null; }
    catch (error) { setBootstrapError(String(error)); return null; }
  }

  async function initializeBootstrap(path: string, mode: "existing" | "fresh") {
    setBootstrapBusy(true); setBootstrapError("");
    try {
      const status = await invoke<BootstrapStatus>("initialize_frameworks_root", { path, mode });
      setBootstrap(status);
      await Promise.all([loadDashboard(), loadSupportingData()]);
      setView(mode === "fresh" ? "manifest" : "overview");
    } catch (error) { setBootstrapError(String(error)); }
    finally { setBootstrapBusy(false); }
  }

  const healthPercent = dashboard ? Math.round(dashboard.healthyCount / Math.max(1, dashboard.components.length) * 100) : 0;
  const groupedComponents = useMemo(() => {
    const groups = new Map<string, ComponentStatus[]>();
    dashboard?.components.forEach(component => groups.set(component.category, [...(groups.get(component.category) ?? []), component]));
    return groups;
  }, [dashboard]);
  const currentTitle = navItems.find(item => item.id === view)?.label ?? "总览";
  const searchResults = useMemo(() => {
    const query = globalQuery.trim().toLowerCase(); if (!query) return [];
    const pages = navItems.map(item => ({ key: `page-${item.id}`, label: item.label, detail: "页面", view: item.id }));
    const components = (dashboard?.components ?? []).map(item => ({ key: `component-${item.id}`, label: item.name, detail: `${item.version} · ${item.category}`, view: "components" as View }));
    const configItems = configs.map(item => ({ key: `config-${item.id}`, label: item.name, detail: item.sourcePath, view: "config" as View }));
    return [...pages, ...components, ...configItems].filter(item => `${item.label} ${item.detail}`.toLowerCase().includes(query)).slice(0, 8);
  }, [globalQuery, dashboard, configs]);

  if (!bootstrap) return <BootstrapLoading />;
  if (!bootstrap.configured) return <BootstrapWizard busy={bootstrapBusy} error={bootstrapError} onBrowse={browseBootstrapDirectory} onInitialize={initializeBootstrap} />;

  return <><a className="skip-link" href="#main-content">跳到主要内容</a><div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark"><Layers3 size={21} /></div><div><strong>GreenDev</strong><span>Manager</span></div></div>
      <nav aria-label="主要导航">{navItems.map(item => { const Icon = item.icon; return <button aria-current={view === item.id ? "page" : undefined} className={view === item.id ? "nav-item active" : "nav-item"} onClick={() => setView(item.id)} key={item.id}><Icon size={18} /><span>{item.label}</span></button>; })}</nav>
      <div className="sidebar-foot"><div className="root-label">环境根目录</div><code>{dashboard?.root ?? "正在发现…"}</code><div className="portable"><ShieldCheck size={14} />绿色模式 · Phase 23</div></div>
    </aside>
    <main id="main-content" tabIndex={-1}>
      <header className="topbar"><div><p className="eyebrow">WINDOWS DEVELOPMENT ENVIRONMENT</p><h1>{currentTitle}</h1></div><GlobalSearch query={globalQuery} setQuery={setGlobalQuery} results={searchResults} onSelect={target => { setView(target); setGlobalQuery(""); }} /><div className="top-actions"><div className={healthPercent === 100 ? "health-chip ok" : "health-chip warning"}><span className="status-dot" />{healthPercent === 100 ? "环境健康" : "需要检查"}</div><button className="icon-button task-button" aria-label="任务中心" onClick={() => setTaskCenterOpen(true)}><ListTodo size={18} />{tasks.some(item => item.status === "running" || item.status === "queued") && <span />}</button><button className="icon-button" aria-label="切换主题" onClick={() => setTheme(value => value === "light" ? "dark" : "light")}>{theme === "light" ? <Moon size={17} /> : <Sun size={17} />}</button><button className="icon-button" aria-label="刷新" onClick={() => void refreshAfterChange()} disabled={busy !== null}><RefreshCw size={18} className={busy === "refresh" ? "spin" : ""} /></button></div></header>
      <div className="content">{!dashboard ? <Loading /> : <>
        {view === "overview" && <Overview dashboard={dashboard} storageHistory={storageHistory} busy={busy} onDoctor={deep => void runOperation(deep ? "doctor-deep" : "doctor", "run_doctor", { deep })} onSync={() => void runOperation("sync", "sync_configs", {}, true)} onConfigure={() => setEnvironmentOpen(true)} onShell={() => void openShell()} />}
        {view === "components" && <Components groups={groupedComponents} busy={busy} onVersions={id => void openVersions(id)} />}
        {view === "android" && <AndroidView packages={androidPackages} taskActive={Boolean(activeTask)} onRefresh={() => void startManagedTask("start_android_task", { action: "list", packages: [] })} onAction={(action, packages) => setConfirm({ title: action === "install" ? "安装 Android SDK 包" : "卸载 Android SDK 包", detail: packages.join("\n"), label: action === "install" ? "开始安装" : "确认卸载", destructive: action === "uninstall", action: () => void startManagedTask("start_android_task", { action, packages }) })} />}
        {view === "manifest" && <ManifestView items={manifest} updates={updates} taskActive={Boolean(activeTask)} onAction={(item, action) => void planManifestAction(item, action)} onImport={setImportTarget} onSettings={() => setSettingsOpen(true)} onBatch={ids => void planBatch(ids)} onRollback={id => void runOperation("rollback-version", "rollback_component_version", { componentId: id }, true)} onRefreshCatalog={() => void startManagedTask("start_update_catalog_task", {})} onAdopt={id => void runOperation("update-adopt", "adopt_update_candidate", { componentId: id }, true)} />}
        {view === "catalog" && <CatalogView editor={manifestEditor} trustPolicy={trustPolicy} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} />}
        {view === "updater" && <UpdaterView status={appUpdate} taskActive={Boolean(activeTask)} onRefresh={() => void startManagedTask("start_app_feed_task", {})} onDownload={version => void startManagedTask("start_app_download_task", { version })} onApply={() => void invoke("apply_prepared_app_update")} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} onPrepare={version => void runOperation("app-update-prepare", "prepare_app_update", { version }, true)} onSbom={() => void runOperation("supply-chain", "export_supply_chain_inventory")} />}
        {view === "profiles" && <ProfilesView value={profileSets} diff={profileDiff} components={manifest} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} onLock={id => void runOperation("profile-lock", "build_profile_lock", { profileId: id }, true)} onDiff={id => void invoke<ProfileDiff>("diff_profile", { profileId: id }).then(setProfileDiff).catch(error => setResult(fallbackResult("档案差异失败", String(error), false)))} onOffline={id => void runOperation("profile-offline", "export_offline_profile", { profileId: id })} onIncremental={id => void runOperation("profile-incremental", "export_incremental_profile", { profileId: id })} />}
        {view === "recovery" && <RecoveryView center={recoveryCenter} onRestored={operation => { setResult(operation); void refreshAfterChange(); }} />}
        {view === "stability" && <StabilityView status={reliability} onRun={() => void runOperation("performance", "run_performance_baseline", {}, true)} onArchive={() => void runOperation("log-archive", "archive_operation_log", {}, true)} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} />}
        {view === "supply" && <SupplyChainView status={supplyChain} onVerify={() => void runOperation("supply-verify", "verify_supply_chain", {}, true)} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} />}
        {view === "fleet" && <FleetView status={fleet} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} onInventory={() => void startManagedTask("start_fleet_inventory_task", {})} onTask={(id, action) => void startManagedTask("start_fleet_rollout_task", { id, action })} />}
        {view === "developer" && <DeveloperView status={ecosystem} onGenerate={id => void runOperation("manifest-sdk", "generate_manifest_template", { componentId: id }, true)} />}
        {view === "enterprise" && <EnterpriseView status={enterprise} onSaved={operation => { setResult(operation); void refreshAfterChange(); }} onSync={action => void startManagedTask("start_team_sync_task", { action })} onAudit={() => void runOperation("audit-export", "export_audit_bundle")} />}
        {view === "environment" && <Environment dashboard={dashboard} backups={backups} onConfigure={() => setEnvironmentOpen(true)} onRestore={backup => setConfirm({ title: "恢复用户环境备份", detail: `${backup.createdAt}\n${backup.path}\n变量数量: ${backup.variableCount}\n恢复前会自动创建新的安全备份。`, label: "备份当前值并恢复", action: () => void runOperation("restore", "restore_environment_backup", { fileName: backup.fileName }, true) })} />}
        {view === "cache" && <CacheView caches={dashboard.caches} storageReady={dashboard.storageReady} level={cleanupLevel} setLevel={setCleanupLevel} includeWrapper={includeWrapper} setIncludeWrapper={setIncludeWrapper} busy={busy} onPreview={() => void previewCleanup()} />}
        {view === "config" && <ConfigView statuses={configs} busy={busy} onSync={() => void runOperation("sync", "sync_configs", {}, true)} onEdit={id => void openConfigEditor(id)} />}
        {view === "diagnostics" && <DiagnosticsView report={diagnostics} maintenance={maintenance} onRefresh={() => void refreshAfterChange()} onExportDiagnostics={() => void runOperation("diagnostic-export", "export_diagnostic_bundle")} onRepair={() => void runOperation("current-repair", "repair_current_links", {}, true)} onExportProfile={() => void runOperation("profile-export", "export_portable_profile")} onImportProfile={() => setProfileImportOpen(true)} onMigrate={() => void runOperation("manifest-migrate", "migrate_manifest_schema", {}, true)} onVerifyRelease={() => void runOperation("release-verify", "verify_latest_release")} />}
        {view === "logs" && <LogsView logs={logs} onSelect={setResult} />}
      </>}</div>
    </main>
    {environmentOpen && <EnvironmentDialog dashboard={dashboard} selected={selectedComponents} setSelected={setSelectedComponents} busy={busy === "configure"} onClose={() => setEnvironmentOpen(false)} onApply={() => void runOperation("configure", "configure_environment", { components: [...selectedComponents] }, true).then(value => { if (value.success) setEnvironmentOpen(false); })} />}
    {versionInventory && <VersionDialog inventory={versionInventory} busy={busy} onClose={() => setVersionInventory(null)} onSwitch={(path) => setConfirm({ title: `切换 ${versionInventory.componentName} current`, detail: `新入口: ${path}\n原入口会临时备份，健康检查通过后再完成切换。已安装版本均保留。`, label: "确认切换", action: () => void runOperation("version-switch", "switch_component_version", { componentId: versionInventory.componentId, targetPath: path }, true).then(() => refreshVersionInventory(versionInventory.componentId)) })} onPin={(path) => void runOperation("version-pin", "set_component_pin", { componentId: versionInventory.componentId, targetPath: path }, false).then(() => refreshVersionInventory(versionInventory.componentId))} />}
    {configDocument && <ConfigEditorDialog document={configDocument} onClose={() => setConfigDocument(null)} onSaved={operation => { setConfigDocument(null); setLogs(previous => [operation, ...previous]); setResult(operation); void refreshAfterChange(); }} />}
    {importTarget && <ImportArchiveDialog item={importTarget} onClose={() => setImportTarget(null)} onImport={path => { const target = importTarget; setImportTarget(null); void startManagedTask("start_manifest_import_task", { componentId: target.id, sourcePath: path }); }} />}
    {settingsOpen && <InstallSettingsDialog settings={installSettings} onClose={() => setSettingsOpen(false)} onSaved={operation => { setSettingsOpen(false); setLogs(previous => [operation, ...previous]); setResult(operation); void refreshAfterChange(); }} />}
    {cleanupPreview && <ConfirmDialog title="确认清理缓存" detail={cleanupPreview.output} confirmLabel="执行清理" destructive onClose={() => setCleanupPreview(null)} onConfirm={() => { setCleanupPreview(null); void runOperation("cleanup", "apply_cleanup", { level: cleanupLevel, includeWrapper }, true); }} />}
    {confirm && <ConfirmDialog title={confirm.title} detail={confirm.detail} confirmLabel={confirm.label} destructive={confirm.destructive} disabled={confirm.disabled} onClose={() => setConfirm(null)} onConfirm={() => { const action = confirm.action; setConfirm(null); action(); }} />}
    {activeTask && <TaskDialog task={activeTask} onCancel={() => void invoke("cancel_task", { taskId: activeTask.id })} onPause={() => void invoke(activeTask.status === "paused" ? "resume_task" : "pause_task", { taskId: activeTask.id })} />}
    {taskCenterOpen && <TaskCenterDialog tasks={tasks} policy={taskPolicy} onPolicy={policy => void runOperation("task-policy", "save_task_policy", { policy }, true)} onClose={() => setTaskCenterOpen(false)} onSelect={task => { if (task.result) setResult(task.result); }} onCancel={id => void invoke("cancel_task", { taskId: id })} onPause={task => void invoke(task.status === "paused" ? "resume_task" : "pause_task", { taskId: task.id })} onRetry={id => void startManagedTask("retry_task", { taskId: id })} onPriority={(id, priority) => void invoke("set_task_priority", { taskId: id, priority }).then(refreshAfterChange)} onSchedule={(id, scheduledAt) => void invoke("reschedule_task", { taskId: id, scheduledAt }).then(refreshAfterChange)} />}
    {profileImportOpen && <ProfileImportDialog onClose={() => setProfileImportOpen(false)} onImport={path => { setProfileImportOpen(false); void runOperation("profile-import", "import_portable_profile", { sourcePath: path }, true); }} />}
    {result && <ResultDialog result={result} onClose={() => setResult(null)} />}
  </div><div className="sr-only" role="status" aria-live="polite">{busy ? "操作进行中" : "就绪"}</div></>;
}

function BootstrapLoading() {
  return <main className="bootstrap-shell"><section className="bootstrap-card compact"><div className="brand-mark"><Layers3 size={24} /></div><LoaderCircle className="spin" size={24} /><h1>正在发现开发环境</h1><p>检查已保存目录、环境变量和程序所在目录。</p></section></main>;
}

function BootstrapWizard({ busy, error, onBrowse, onInitialize }: { busy: boolean; error: string; onBrowse: () => Promise<string | null>; onInitialize: (path: string, mode: "existing" | "fresh") => Promise<void> }) {
  const [path, setPath] = useState("");
  const [mode, setMode] = useState<"existing" | "fresh">("fresh");
  async function browse() { const value = await onBrowse(); if (value) setPath(value); }
  return <main className="bootstrap-shell"><section className="bootstrap-card"><div className="bootstrap-brand"><div className="brand-mark"><Layers3 size={23} /></div><div><strong>GreenDev Manager</strong><span>FIRST RUN SETUP</span></div></div><div className="bootstrap-copy"><span className="section-kicker"><Sparkles size={14} />首次启动</span><h1>选择你的开发环境目录</h1><p>根目录不限定盘符和名称。你可以接入已有环境，或选择空目录并从正式发布源下载一套全新的配置、脚本和 CLI。</p></div><div className="bootstrap-modes"><button className={mode === "fresh" ? "bootstrap-mode selected" : "bootstrap-mode"} onClick={() => setMode("fresh")}><Download size={21} /><span><strong>全新下载配置</strong><small>要求空目录；下载初始化包并校验 SHA-256</small></span>{mode === "fresh" && <Check size={17} />}</button><button className={mode === "existing" ? "bootstrap-mode selected" : "bootstrap-mode"} onClick={() => setMode("existing")}><ArchiveRestore size={21} /><span><strong>接入现有环境</strong><small>验证 Scripts、Config 和 env-setup.bat</small></span>{mode === "existing" && <Check size={17} />}</button></div><div className="bootstrap-path"><label><span>环境根目录</span><div><input value={path} onChange={event => setPath(event.target.value)} placeholder="请选择或新建目录" disabled={busy} /><button className="secondary-button" onClick={() => void browse()} disabled={busy}><FolderOpen size={16} />浏览</button></div></label></div>{error && <div className="bootstrap-error"><CircleAlert size={17} /><span>{error}</span></div>}<div className="bootstrap-footer"><span>{mode === "fresh" ? "已有文件保持不动；非空目录不会执行初始化。" : "接入后会保存根目录，下次启动自动加载。"}</span><button className="primary-button" disabled={!path.trim() || busy} onClick={() => void onInitialize(path.trim(), mode)}>{busy ? <LoaderCircle className="spin" size={17} /> : mode === "fresh" ? <Download size={17} /> : <FolderOpen size={17} />}{busy ? "正在初始化…" : mode === "fresh" ? "下载并初始化" : "使用此目录"}</button></div></section></main>;
}

function GlobalSearch({ query, setQuery, results, onSelect }: { query: string; setQuery: (value: string) => void; results: Array<{ key: string; label: string; detail: string; view: View }>; onSelect: (view: View) => void }) {
  return <div className="global-search"><label><Search size={15} /><input value={query} onChange={event => setQuery(event.target.value)} placeholder="搜索组件、配置和页面…" /></label>{query && <div className="global-results">{results.length ? results.map(item => <button key={item.key} onClick={() => onSelect(item.view)}><strong>{item.label}</strong><span>{item.detail}</span></button>) : <div>没有匹配项</div>}</div>}</div>;
}

function Loading() { return <div className="loading"><LoaderCircle className="spin" size={28} /><span>正在读取组件清单…</span></div>; }

function Overview({ dashboard, storageHistory, busy, onDoctor, onSync, onConfigure, onShell }: { dashboard: Dashboard; storageHistory: StoragePoint[]; busy: string | null; onDoctor: (deep: boolean) => void; onSync: () => void; onConfigure: () => void; onShell: () => void }) {
  const health = Math.round(dashboard.healthyCount / Math.max(1, dashboard.components.length) * 100);
  const trendMax = Math.max(...storageHistory.map(item => item.totalSizeBytes), 1);
  return <div className="page-stack"><section className="hero-panel"><div className="hero-copy"><span className="section-kicker"><Sparkles size={14} />Phase 1–19 已接入</span><h2>所有工具，一处掌控。</h2><p>持久队列、统一恢复、CLI 共核、团队仓库、策略合规与多机器复现统一留痕和回滚。</p></div><div className="health-ring" style={{ "--health": `${health * 3.6}deg` } as React.CSSProperties}><div><strong>{health}%</strong><span>健康度</span></div></div></section>
    <section className="metric-grid"><Metric icon={PackageCheck} label="已安装组件" value={`${dashboard.installedCount}`} note={`共 ${dashboard.components.length} 项`} /><Metric icon={ShieldCheck} label="健康组件" value={`${dashboard.healthyCount}`} note="入口文件检查" good /><Metric icon={HardDrive} label="环境占用" value={dashboard.storageReady ? formatBytes(dashboard.totalSizeBytes) : "扫描中"} note="后台容量统计" /><Metric icon={Database} label="集中缓存" value={dashboard.storageReady ? formatBytes(dashboard.cacheSizeBytes) : "扫描中"} note="Maven 仓库受保护" /></section>
    <section className="section-block"><div className="section-heading"><div><h2>快捷操作</h2><p>操作结果采用结构化记录并写入持久日志。</p></div></div><div className="action-grid"><Action icon={Activity} title="运行 Doctor" detail="检查入口、组件和配置漂移" onClick={() => onDoctor(false)} loading={busy === "doctor"} primary /><Action icon={MonitorCog} title="一键配置" detail="选择组件并备份用户环境" onClick={onConfigure} loading={busy === "configure"} /><Action icon={FolderCog} title="同步配置" detail="部署 Config 权威副本" onClick={onSync} loading={busy === "sync"} /><Action icon={TerminalSquare} title="开发终端" detail="加载临时环境并打开 CMD" onClick={onShell} loading={busy === "shell"} /></div></section>
    {storageHistory.length > 1 && <section className="section-block"><div className="section-heading"><div><h2>磁盘占用趋势</h2><p>最近 {storageHistory.length} 次后台扫描，绿色为环境总体、浅色为集中缓存。</p></div><code>{formatBytes(storageHistory.at(-1)?.totalSizeBytes ?? 0)}</code></div><div className="storage-trend">{storageHistory.map(point => <div key={point.recordedAt} title={`${new Date(point.recordedAt).toLocaleString()} · ${formatBytes(point.totalSizeBytes)}`}><span style={{ height: `${Math.max(6, point.totalSizeBytes / trendMax * 100)}%` }} /><i style={{ height: `${Math.max(2, point.cacheSizeBytes / trendMax * 100)}%` }} /></div>)}</div></section>}
    <section className="section-block"><div className="section-heading"><div><h2>组件概览</h2><p>快速扫描不会递归遍历大型 SDK。</p></div><button className="text-button" onClick={() => onDoctor(true)}>深度版本检查 <ChevronRight size={16} /></button></div><div className="compact-list">{dashboard.components.slice(0, 7).map(component => <div className="compact-row" key={component.id}><ComponentGlyph id={component.id} /><div><strong>{component.name}</strong><span>{component.category}</span></div><code>{component.version}</code><Status healthy={component.healthy} /></div>)}</div></section></div>;
}

function Metric({ icon: Icon, label, value, note, good }: { icon: typeof Gauge; label: string; value: string; note: string; good?: boolean }) { return <div className="metric-card"><div className={good ? "metric-icon good" : "metric-icon"}><Icon size={19} /></div><span>{label}</span><strong>{value}</strong><small>{note}</small></div>; }
function Action({ icon: Icon, title, detail, onClick, loading, primary }: { icon: typeof Gauge; title: string; detail: string; onClick: () => void; loading: boolean; primary?: boolean }) { return <button className={primary ? "action-card primary" : "action-card"} onClick={onClick} disabled={loading}><span className="action-icon">{loading ? <LoaderCircle className="spin" size={20} /> : <Icon size={20} />}</span><span><strong>{title}</strong><small>{detail}</small></span><ChevronRight size={18} /></button>; }

function Components({ groups, busy, onVersions }: { groups: Map<string, ComponentStatus[]>; busy: string | null; onVersions: (id: string) => void }) {
  return <div className="page-stack"><div className="notice"><ShieldCheck size={18} /><div><strong>版本切换策略</strong><p>只切换经过健康检查的 current 目录联接；失败自动回退，版本固定写入 Config，旧版本始终保留。</p></div></div>{[...groups.entries()].map(([category, components]) => <section className="section-block" key={category}><div className="section-heading"><div><h2>{category}</h2><p>{components.length} 个组件</p></div></div><div className="component-grid">{components.map(component => <article className="component-card" key={component.id}><div className="component-top"><ComponentGlyph id={component.id} /><Status healthy={component.healthy} /></div><h3>{component.name}</h3><div className="version-line"><span>当前版本</span><code>{component.version}</code></div><div className="path-line" title={component.currentTarget ?? component.executable}>{component.currentTarget ?? component.executable}</div><button className="secondary-button" disabled={!versioned.has(component.id) || busy === `versions-${component.id}`} onClick={() => onVersions(component.id)}>{busy === `versions-${component.id}` ? <LoaderCircle className="spin" size={15} /> : <Boxes size={15} />}{versioned.has(component.id) ? "管理已安装版本" : "固定入口"}</button></article>)}</div></section>)}</div>;
}

function AndroidView({ packages, taskActive, onRefresh, onAction }: { packages: AndroidPackage[]; taskActive: boolean; onRefresh: () => void; onAction: (action: "install" | "uninstall", packages: string[]) => void }) {
  const [query, setQuery] = useState(""); const [selected, setSelected] = useState<Set<string>>(new Set());
  const filtered = packages.filter(item => `${item.id} ${item.description}`.toLowerCase().includes(query.toLowerCase()));
  function toggle(id: string) { const next = new Set(selected); next.has(id) ? next.delete(id) : next.add(id); setSelected(next); }
  const selectedItems = packages.filter(item => selected.has(item.id));
  return <div className="page-stack"><section className="section-block"><div className="section-heading"><div><h2>SDK 包目录</h2><p>本地清单即时显示；刷新目录通过 sdkmanager 后台执行。</p></div><button className="primary-button" onClick={onRefresh} disabled={taskActive}><RefreshCw size={16} />刷新在线目录</button></div><div className="toolbar-row"><label className="search-box"><Search size={15} /><input value={query} onChange={event => setQuery(event.target.value)} placeholder="筛选 platforms、build-tools、cmake…" /></label><span>{packages.filter(item => item.installed).length} 已安装 / {packages.length} 项</span></div><div className="package-list">{filtered.map(item => <label className={selected.has(item.id) ? "package-row selected" : "package-row"} key={item.id}><input type="checkbox" checked={selected.has(item.id)} onChange={() => toggle(item.id)} /><span><strong>{item.id}</strong><small>{item.description}</small></span><code>{item.version}</code><Status healthy={item.installed} label={item.installed ? "已安装" : "可安装"} /></label>)}</div></section><section className="settings-strip"><div><strong>已选择 {selected.size} 项</strong><span>安装和卸载前均显示精确包标识；任务支持取消。</span></div><button className="secondary-button" disabled={!selectedItems.some(item => !item.installed) || taskActive} onClick={() => onAction("install", selectedItems.filter(item => !item.installed).map(item => item.id))}><Download size={15} />安装未安装项</button><button className="danger-outline" disabled={!selectedItems.some(item => item.installed) || taskActive} onClick={() => onAction("uninstall", selectedItems.filter(item => item.installed).map(item => item.id))}><Trash2 size={15} />卸载已安装项</button></section></div>;
}

function ManifestView({ items, updates, taskActive, onAction, onImport, onSettings, onBatch, onRollback, onRefreshCatalog, onAdopt }: { items: ManifestComponent[]; updates: UpdateCandidate[]; taskActive: boolean; onAction: (item: ManifestComponent, action: "install" | "update") => void; onImport: (item: ManifestComponent) => void; onSettings: () => void; onBatch: (ids: string[]) => void; onRollback: (id: string) => void; onRefreshCatalog: () => void; onAdopt: (id: string) => void }) {
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const stateLabel: Record<ManifestComponent["state"], string> = { disabled: "已停用", current: "当前版本", installed: "已安装", available: "可安装", blocked: "待解锁", pinned: "已固定" };
  function toggle(id: string) { const next = new Set(selected); next.has(id) ? next.delete(id) : next.add(id); setSelected(next); }
  return <div className="page-stack"><div className="notice"><PackageCheck size={18} /><div><strong>声明式升级中心</strong><p>按 LTS/Stable 策略刷新官方目录，采用候选后再执行依赖排序、SHA256 预检和失败回退。旧版本始终保留。</p></div><div className="button-row"><button className="secondary-button" disabled={taskActive} onClick={onRefreshCatalog}><RefreshCw size={15} />联网刷新</button><button className="secondary-button" onClick={onSettings}><Settings2 size={15} />镜像与代理</button></div></div><section className="settings-strip"><div><strong>已选择 {selected.size} 个更新</strong><span>执行顺序由组件依赖自动计算。</span></div><button className="primary-button" disabled={!selected.size || taskActive} onClick={() => onBatch([...selected])}><Layers3 size={15} />批量更新计划</button></section><section className="manifest-grid">{items.map(item => { const update = updates.find(value => value.componentId === item.id); return <article className={selected.has(item.id) ? "manifest-card selected" : "manifest-card"} key={item.id}><div className="component-top"><label className="manifest-select"><input type="checkbox" checked={selected.has(item.id)} disabled={!update?.updateAvailable || update?.canAdopt || item.pinnedElsewhere} onChange={() => toggle(item.id)} /><ComponentGlyph id={item.id} /></label><span className={`phase-state ${item.state}`}>{update?.canAdopt ? "发现新版" : stateLabel[item.state]}</span></div><h3>{item.name}</h3><div className="manifest-version"><span>{update?.currentVersion ?? "当前"} →</span><strong>{update?.targetVersion ?? item.desiredVersion}</strong></div><dl><dt>策略</dt><dd>{update?.policy.toUpperCase() ?? "STABLE"}</dd><dt>目录</dt><dd>{update?.catalogAvailable ? "官方目录已刷新" : "使用本地清单"}</dd><dt>校验</dt><dd>{update?.canAdopt ? "采用候选后预检" : item.checksumReady ? "SHA256 已锁定" : "等待离线导入"}</dd><dt>依赖</dt><dd>{item.dependencies.join("、") || "无"}</dd></dl><p className="release-note">{update?.releaseNotes}</p>{item.blockedReason && !update?.canAdopt && <div className="card-warning">{item.blockedReason}</div>}<div className="card-actions three"><button className="secondary-button" disabled={taskActive} onClick={() => onImport(item)}><Upload size={14} />导入</button><button className="secondary-button" disabled={taskActive || !item.active} onClick={() => onRollback(item.id)}><RotateCcw size={14} />回退</button>{update?.canAdopt ? <button className="primary-button" disabled={taskActive || item.pinnedElsewhere} onClick={() => onAdopt(item.id)}><Check size={14} />采用</button> : <button className="primary-button" disabled={!item.enabled || taskActive || item.active || item.pinnedElsewhere} onClick={() => onAction(item, item.installed ? "update" : "install")}><Download size={14} />{item.active ? "当前" : "计划"}</button>}</div></article>; })}</section></div>;
}

function Environment({ dashboard, backups, onConfigure, onRestore }: { dashboard: Dashboard; backups: EnvironmentBackup[]; onConfigure: () => void; onRestore: (backup: EnvironmentBackup) => void }) {
  return <div className="page-stack"><div className="two-column"><section className="section-block grow"><div className="section-heading"><div><h2>用户环境</h2><p>选择已有组件，写入用户级变量和 PATH。</p></div><button className="primary-button" onClick={onConfigure}><MonitorCog size={17} />开始配置</button></div><div className="env-list">{dashboard.components.filter(item => configurable.includes(item.id)).map(item => <div className="env-row" key={item.id}><ComponentGlyph id={item.id} /><div><strong>{item.name}</strong><span>{item.environmentVariables.length ? item.environmentVariables.join(" · ") : "PATH"}</span></div><Status healthy={item.installed} label={item.installed ? "可配置" : "缺失"} /></div>)}</div></section><aside className="info-panel"><ShieldCheck size={24} /><h3>应用与恢复均先备份</h3><p>快照保存在 <code>Config\env-backups</code>，恢复后新终端生效。</p><div className="info-rule" /><strong>保持不变</strong><ul><li>系统级环境变量</li><li>工具安装目录</li><li>Windows 服务</li></ul></aside></div><section className="section-block"><div className="section-heading"><div><h2>环境变量备份</h2><p>显示历史快照，并支持先备份当前状态后恢复。</p></div></div><div className="backup-list">{backups.map(backup => <div className="backup-row" key={backup.fileName}><ArchiveRestore size={18} /><div><strong>{backup.createdAt || backup.fileName}</strong><span>{backup.variableCount} 个变量 · {backup.root}</span></div><code>{backup.fileName}</code><button className="secondary-button" onClick={() => onRestore(backup)}>恢复</button></div>)}</div></section></div>;
}

function CacheView({ caches, storageReady, level, setLevel, includeWrapper, setIncludeWrapper, busy, onPreview }: { caches: CacheEntry[]; storageReady: boolean; level: "safe" | "normal"; setLevel: (value: "safe" | "normal") => void; includeWrapper: boolean; setIncludeWrapper: (value: boolean) => void; busy: string | null; onPreview: () => void }) {
  const max = Math.max(...caches.map(cache => cache.sizeBytes), 1);
  return <div className="page-stack"><section className="section-block"><div className="section-heading"><div><h2>缓存占用</h2><p>{storageReady ? "后台容量扫描已完成。" : "后台正在统计容量，不影响其他页面操作。"}</p></div><button className="primary-button" onClick={onPreview} disabled={busy === "cleanup-preview"}>{busy === "cleanup-preview" ? <LoaderCircle className="spin" size={17} /> : <Trash2 size={17} />}生成清理预览</button></div><div className="cache-list">{caches.map(cache => <div className="cache-row" key={cache.id}><div className="cache-name"><strong>{cache.name}</strong><span>{cache.protected ? "受保护" : cache.path}</span></div><div className="cache-track"><span style={{ width: `${storageReady ? Math.max(2, cache.sizeBytes / max * 100) : 0}%` }} /></div><code>{storageReady ? formatBytes(cache.sizeBytes) : "扫描中"}</code></div>)}</div></section><section className="settings-strip"><div><strong>清理级别</strong><span>normal 包含可重建的依赖缓存</span></div><div className="segmented"><button className={level === "safe" ? "selected" : ""} onClick={() => setLevel("safe")}>Safe</button><button className={level === "normal" ? "selected" : ""} onClick={() => setLevel("normal")}>Normal</button></div><label className="check-control"><input type="checkbox" checked={includeWrapper} onChange={event => setIncludeWrapper(event.target.checked)} />包含 Gradle Wrapper</label></section></div>;
}

function ConfigView({ statuses, busy, onSync, onEdit }: { statuses: ConfigStatus[]; busy: string | null; onSync: () => void; onEdit: (id: string) => void }) {
  const labels: Record<ConfigStatus["state"], string> = { synced: "已同步", drifted: "有漂移", missing: "缺失", reference: "环境加载" };
  return <div className="page-stack"><section className="section-block"><div className="section-heading"><div><h2>权威配置</h2><p>表单与源码双模式；保存前校验并展示差异，写入后自动同步。</p></div><button className="primary-button" onClick={onSync} disabled={busy === "sync"}>{busy === "sync" ? <LoaderCircle className="spin" size={17} /> : <RefreshCw size={17} />}同步全部</button></div><div className="config-list">{statuses.map(item => <button className="config-row config-detailed config-button" key={item.id} onClick={() => onEdit(item.id)} disabled={busy === `config-${item.id}`}><div className="config-icon">{busy === `config-${item.id}` ? <LoaderCircle className="spin" size={18} /> : <Settings2 size={18} />}</div><div><strong>{item.name}</strong><span>{item.detail}</span></div><div className="hash-pair"><code title={item.sourcePath}>源 {item.sourceHash.slice(0, 8)}</code><code title={item.deployedPath ?? "环境变量"}>目标 {item.deployedHash?.slice(0, 8) ?? "—"}</code></div><span className={`sync-state ${item.state}`}>{item.state === "synced" || item.state === "reference" ? <ShieldCheck size={14} /> : <CircleAlert size={14} />}{labels[item.state]}</span></button>)}</div></section><div className="notice"><ShieldCheck size={18} /><div><strong>安全写入</strong><p>原内容归档到 Config\config-backups；临时文件验证后替换。同步异常会恢复权威副本。</p></div></div></div>;
}

function DiagnosticsView({ report, maintenance, onRefresh, onExportDiagnostics, onRepair, onExportProfile, onImportProfile, onMigrate, onVerifyRelease }: { report: DiagnosticReport | null; maintenance: MaintenanceStatus | null; onRefresh: () => void; onExportDiagnostics: () => void; onRepair: () => void; onExportProfile: () => void; onImportProfile: () => void; onMigrate: () => void; onVerifyRelease: () => void }) {
  if (!report) return <Loading />;
  return <div className="page-stack"><section className="section-block"><div className="section-heading"><div><h2>运行诊断</h2><p>GreenDev Manager {report.appVersion} · {report.healthyCount}/{report.items.length} 项正常 · {maintenance?.buildMode ?? "—"}</p></div><div className="button-row"><button className="secondary-button" onClick={onExportDiagnostics}><FileArchive size={15} />导出诊断包</button><button className="primary-button" onClick={onRefresh}><RefreshCw size={16} />重新诊断</button></div></div><div className="diagnostic-grid">{report.items.map(item => <div className={item.healthy ? "diagnostic-card healthy" : "diagnostic-card warning"} key={item.id}>{item.healthy ? <Check size={17} /> : <CircleAlert size={17} />}<div><strong>{item.name}</strong><span>{item.detail}</span></div></div>)}</div></section><section className="section-block"><div className="section-heading"><div><h2>恢复与迁移</h2><p>待恢复事务 {maintenance?.pendingTransactions ?? 0} 个；本地发布最新版本 {maintenance?.latestLocalVersion ?? "—"}。</p></div><button className="secondary-button" onClick={onRepair}><Wrench size={15} />修复 current</button></div><div className="maintenance-grid"><button onClick={onExportProfile}><Upload size={18} /><strong>导出便携配置</strong><span>生成可跨盘符导入的 Profile ZIP</span></button><button onClick={onImportProfile}><Download size={18} /><strong>导入便携配置</strong><span>导入前备份当前 Config 并自动同步</span></button><button onClick={onMigrate}><RefreshCw size={18} /><strong>检查 Manifest Schema</strong><span>确保 Schema 2，并为旧清单保留迁移备份</span></button></div></section><section className="section-block"><div className="section-heading"><div><h2>发布自动化</h2><p>一次生成 NSIS、绿色 ZIP、Release Notes、发布清单、更新 Feed 和 SHA256；可传入证书指纹签名。</p></div><button className="secondary-button" onClick={onVerifyRelease}><ShieldCheck size={15} />验证最新发布物</button></div><div className="command-card"><code>powershell -ExecutionPolicy Bypass -File Apps\GreenDevManager\release.ps1 [-Channel stable] [-SignThumbprint CERT_THUMBPRINT]</code></div><div className="command-card"><code>powershell -ExecutionPolicy Bypass -File Apps\GreenDevManager\e2e-test.ps1</code></div></section></div>;
}

function CatalogView({ editor, trustPolicy, onSaved }: { editor: ManifestEditor | null; trustPolicy: Record<string, unknown>; onSaved: (value: OperationResult) => void }) {
  const [raw, setRaw] = useState(editor?.raw ?? ""); const [trust, setTrust] = useState(JSON.stringify(trustPolicy, null, 2)); const [errors, setErrors] = useState<string[]>(editor?.errors ?? []); const [busy, setBusy] = useState(false);
  useEffect(() => { setRaw(editor?.raw ?? ""); setErrors(editor?.errors ?? []); }, [editor]); useEffect(() => setTrust(JSON.stringify(trustPolicy, null, 2)), [trustPolicy]);
  const parsed = useMemo(() => { try { return JSON.parse(raw) as { components?: Array<{ id: string; name: string; version: string; source?: { archive?: string; type?: string }; dependsOn?: string[] }> }; } catch { return {}; } }, [raw]);
  async function preview() { setBusy(true); try { const result = await invoke<{ valid: boolean; errors: string[] }>("preview_manifest_editor", { raw }); setErrors(result.errors); } catch (error) { setErrors([String(error)]); } finally { setBusy(false); } }
  async function save() { setBusy(true); try { onSaved(await invoke<OperationResult>("save_manifest_editor", { raw, expectedHash: editor?.baseHash ?? "" })); } catch (error) { setErrors([String(error)]); } finally { setBusy(false); } }
  async function saveTrust() { setBusy(true); try { onSaved(await invoke<OperationResult>("save_trust_policy", { policy: JSON.parse(trust) })); } catch (error) { setErrors([String(error)]); } finally { setBusy(false); } }
  function addComponent() { try { const value = JSON.parse(raw); value.schemaVersion = 2; value.components ??= []; value.components.push({ id: `custom-${value.components.length + 1}`, name: "自定义组件", version: "1.0.0", enabled: true, dependsOn: [], installDir: "Toolchains\\Custom\\custom-1.0.0", currentLink: "Toolchains\\Custom\\current", healthPath: "bin\\tool.exe", archiveRoot: "", source: { type: "archive", url: "", archive: "downloads\\packages\\custom-1.0.0.zip", sha256: "" } }); setRaw(JSON.stringify(value, null, 2)); } catch (error) { setErrors([String(error)]); } }
  return <div className="page-stack"><div className="notice"><ShieldCheck size={18} /><div><strong>Manifest Schema 2 与最小权限插件目录</strong><p>支持 ZIP、7Z、TAR.GZ/TGZ/TAR.XZ 和 MSI；路径、依赖、哈希与目录权限在保存前统一校验。</p></div></div><section className="section-block"><div className="section-heading"><div><h2>可视化组件清单</h2><p>{editor?.path ?? "加载中"} · {parsed.components?.length ?? 0} 个定义</p></div><div className="button-row"><button className="secondary-button" onClick={addComponent}><PackageCheck size={15} />添加组件</button><button className="secondary-button" disabled={busy} onClick={() => void preview()}><Search size={15} />校验</button><button className="primary-button" disabled={busy || errors.length > 0} onClick={() => void save()}><Save size={15} />备份并保存</button></div></div><div className="definition-strip">{parsed.components?.map(item => <div key={item.id}><ComponentGlyph id={item.id} /><span><strong>{item.name}</strong><small>{item.version} · {item.source?.type ?? "archive"} · {item.dependsOn?.join("+") || "无依赖"}</small></span></div>)}</div><textarea className="settings-editor manifest-editor" spellCheck={false} value={raw} onChange={event => { setRaw(event.target.value); setErrors(["内容有改动，请先校验。"]); }} />{errors.length > 0 && <div className="inline-error">{errors.join("\n")}</div>}</section><section className="section-block"><div className="section-heading"><div><h2>可信目录与插件隔离</h2><p>目录签名、网络/进程权限和允许写入根目录集中管理。</p></div><button className="primary-button" disabled={busy} onClick={() => void saveTrust()}><Save size={15} />保存策略</button></div><textarea className="settings-editor policy-editor" value={trust} spellCheck={false} onChange={event => setTrust(event.target.value)} /></section></div>;
}

function UpdaterView({ status, taskActive, onRefresh, onDownload, onApply, onSaved, onPrepare, onSbom }: { status: AppUpdateStatus | null; taskActive: boolean; onRefresh: () => void; onDownload: (version: string) => void; onApply: () => void; onSaved: (value: OperationResult) => void; onPrepare: (version: string) => void; onSbom: () => void }) {
  const [settings, setSettings] = useState(status?.settings); useEffect(() => setSettings(status?.settings), [status]);
  async function save() { if (!settings) return; try { onSaved(await invoke<OperationResult>("save_app_update_settings", { settings })); } catch (error) { onSaved(fallbackResult("保存更新设置失败", String(error), false)); } }
  const remoteNeedsDownload = Boolean(status?.updateAvailable && status.latestRemoteVersion && !status.localVersions.includes(status.latestRemoteVersion));
  const online = Boolean(settings?.feedUrl.trim());
  return <div className="page-stack"><section className="hero-panel update-hero"><div className="hero-copy"><span className="section-kicker"><RefreshCw size={14} />应用自更新</span><h2>{status?.currentVersion ?? "—"} <span className="version-arrow">→</span> {status?.targetVersion ?? "—"}</h2><p>通道源、发布清单、SHA256 与代码签名策略完成验证后写入更新事务；替换前保存当前程序，启动健康检查异常时自动回退。</p></div><div className="button-column"><div className={status?.updateAvailable ? "update-badge available" : "update-badge"}>{status?.prepared ? "已准备" : status?.updateAvailable ? "可更新" : "最新"}</div>{remoteNeedsDownload && <button className="secondary-button" disabled={taskActive} onClick={() => onDownload(status!.latestRemoteVersion)}><Download size={14} />下载候选</button>}{status?.prepared && <button className="primary-button" onClick={onApply}><RefreshCw size={14} />重启并应用</button>}</div></section><section className="section-block"><div className="section-heading"><div><h2>更新通道</h2><p>当前来源：{online ? "远程 Feed（联网）" : "本地 Releases（不联网）"}。Stable / Beta / Nightly / Local 分通道管理。</p></div><div className="button-row"><button className="secondary-button" disabled={taskActive} onClick={onRefresh}><RefreshCw size={15} />{online ? "联网刷新" : "刷新本地发布"}</button><button className="primary-button" onClick={() => void save()}><Save size={15} />保存</button></div></div>{settings && <div className="update-settings"><label><span>通道</span><select value={settings.channel} onChange={event => setSettings({ ...settings, channel: event.target.value as AppUpdateStatus["settings"]["channel"] })}><option value="stable">Stable</option><option value="beta">Beta</option><option value="nightly">Nightly</option><option value="local">Local</option></select></label><label className="grow"><span>Feed URL</span><input value={settings.feedUrl} placeholder="留空使用本地发布目录；联网需填写完整 HTTPS 地址" onChange={event => setSettings({ ...settings, feedUrl: event.target.value })} /></label><label className="check-control"><input type="checkbox" checked={settings.requireSignature} onChange={event => setSettings({ ...settings, requireSignature: event.target.checked })} />强制代码签名</label></div>}<div className="release-versions">{status?.localVersions.map(version => { const eligible = compareVersions(version, status.currentVersion) > 0; return <div key={version}><FileArchive size={17} /><strong>{version}</strong><span>{version === status.currentVersion ? "当前" : eligible ? "可升级" : "历史版本"}</span><button className="secondary-button" disabled={!eligible} onClick={() => onPrepare(version)}>验证并准备</button></div>; })}</div></section><section className="settings-strip"><div><strong>供应链清单</strong><span>导出 CycloneDX 1.5、组件哈希和本地漏洞通告。</span></div><button className="secondary-button" onClick={onSbom}><FileArchive size={15} />导出 SBOM</button></section></div>;
}

function ProfilesView({ value, diff, components, onSaved, onLock, onDiff, onOffline, onIncremental }: { value: ProfileSets | null; diff: ProfileDiff | null; components: ManifestComponent[]; onSaved: (value: OperationResult) => void; onLock: (id: string) => void; onDiff: (id: string) => void; onOffline: (id: string) => void; onIncremental: (id: string) => void }) {
  const [profiles, setProfiles] = useState(value); useEffect(() => setProfiles(value), [value]); if (!profiles) return <Loading />;
  function toggle(profileIndex: number, id: string) { if (!profiles) return; const next: ProfileSets = structuredClone(profiles); const list = next.profiles[profileIndex].components; next.profiles[profileIndex].components = list.includes(id) ? list.filter(item => item !== id) : [...list, id]; setProfiles(next); }
  async function save() { try { onSaved(await invoke<OperationResult>("save_profile_sets", { profiles })); } catch (error) { onSaved(fallbackResult("保存 Profile 失败", String(error), false)); } }
  return <div className="page-stack"><div className="notice"><Layers3 size={18} /><div><strong>可复现的多机器环境</strong><p>Profile 保存组件集合和机器覆盖项；Lock 固定版本、哈希、依赖与 current 目标，导出介质自动排除凭据特征。</p></div><button className="primary-button" onClick={() => void save()}><Save size={15} />保存档案</button></div><section className="profile-grid">{profiles.profiles.map((profile, index) => <article className={profiles.activeProfile === profile.id ? "profile-card active" : "profile-card"} key={profile.id}><div className="component-top"><ComponentGlyph id={profile.id} /><span className="phase-state current">{profile.teamTemplate ? "团队模板" : "本机档案"}</span></div><input className="profile-name" value={profile.name} onChange={event => { const next = structuredClone(profiles); next.profiles[index].name = event.target.value; setProfiles(next); }} /><div className="profile-components">{components.map(item => <label key={item.id}><input type="checkbox" checked={profile.components.includes(item.id)} onChange={() => toggle(index, item.id)} />{item.name}</label>)}</div><div className="profile-actions"><button className="secondary-button" onClick={() => { setProfiles({ ...profiles, activeProfile: profile.id }); }}><Check size={14} />设为活动</button><button className="secondary-button" onClick={() => onLock(profile.id)}><Pin size={14} />生成 Lock</button><button className="secondary-button" onClick={() => onDiff(profile.id)}><Search size={14} />差异</button><button className="secondary-button" onClick={() => onIncremental(profile.id)}><FileArchive size={14} />增量介质</button><button className="primary-button" onClick={() => onOffline(profile.id)}><FileArchive size={14} />完整介质</button></div></article>)}</section>{diff && <section className="section-block"><div className="section-heading"><div><h2>{diff.profileId} 环境差异</h2><p>{diff.matched ? "当前机器与锁文件一致。" : "检测到版本、入口、哈希或安装状态漂移。"}</p></div></div><div className="diff-list">{diff.rows.map(row => <div key={row.id}><Status healthy={row.state === "matched"} label={row.state === "matched" ? "一致" : "漂移"} /><strong>{row.id}</strong><span>{row.changes.join("、") || "无差异"}</span></div>)}</div></section>}</div>;
}

function RecoveryView({ center, onRestored }: { center: RecoveryCenter; onRestored: (value: OperationResult) => void }) {
  const [query, setQuery] = useState(""); const [preview, setPreview] = useState<{ item: { id: string; title: string; path: string }; sha256: string; preview: string } | null>(null); const [busy, setBusy] = useState(false);
  const items = center.items.filter(item => `${item.title} ${item.kind} ${item.relativePath}`.toLowerCase().includes(query.toLowerCase()));
  async function show(id: string) { setBusy(true); try { setPreview(await invoke("preview_recovery_item", { id })); } finally { setBusy(false); } }
  async function restore() { if (!preview || !globalThis.confirm(`恢复 ${preview.item.title}？当前状态会先备份。`)) return; setBusy(true); try { onRestored(await invoke<OperationResult>("restore_recovery_item", { id: preview.item.id })); setPreview(null); } catch (error) { onRestored(fallbackResult("恢复失败", String(error), false)); } finally { setBusy(false); } }
  return <div className="page-stack"><div className="notice"><ArchiveRestore size={18} /><div><strong>统一恢复中心</strong><p>集中显示配置、环境变量、Profile、应用程序和任务事务；每次恢复仍逐项预览与确认。</p></div><span className="phase-state current">{center.pendingTransactions} 个待恢复事务</span></div><section className="section-block"><div className="section-heading"><div><h2>恢复点</h2><p>共 {center.items.length} 项，按时间倒序排列。</p></div><label className="search-box"><Search size={14} /><input value={query} onChange={event => setQuery(event.target.value)} placeholder="筛选类型或路径" /></label></div><div className="recovery-list">{items.slice(0, 100).map(item => <button key={item.id} onClick={() => void show(item.id)}><span className={`recovery-kind ${item.kind}`}>{item.kind}</span><div><strong>{item.title}</strong><small>{item.relativePath}</small></div><code>{formatBytes(item.sizeBytes)}</code><span>{new Date(item.createdAt).toLocaleString()}</span><ChevronRight size={15} /></button>)}</div></section>{preview && <section className="section-block"><div className="section-heading"><div><h2>{preview.item.title}</h2><p>{preview.item.path}</p></div><button className="primary-button" disabled={busy} onClick={() => void restore()}><ArchiveRestore size={15} />确认恢复</button></div><code className="recovery-hash">SHA256 {preview.sha256 || "目录恢复点"}</code><pre className="recovery-preview">{preview.preview}</pre></section>}</div>;
}

function EnterpriseView({ status, onSaved, onSync, onAudit }: { status: EnterpriseStatus | null; onSaved: (value: OperationResult) => void; onSync: (action: "preview" | "apply") => void; onAudit: () => void }) {
  const [raw, setRaw] = useState(JSON.stringify(status?.policy ?? {}, null, 2)); const [error, setError] = useState(""); useEffect(() => setRaw(JSON.stringify(status?.policy ?? {}, null, 2)), [status]);
  async function save() { setError(""); try { onSaved(await invoke<OperationResult>("save_enterprise_policy", { policy: JSON.parse(raw) })); } catch (value) { setError(String(value)); } }
  return <div className="page-stack"><section className={status?.healthy ? "hero-panel enterprise-hero healthy" : "hero-panel enterprise-hero"}><div className="hero-copy"><span className="section-kicker"><ShieldCheck size={14} />团队与企业管理</span><h2>{status?.healthy ? "策略合规" : "存在待处理策略"}</h2><p>机器组、签名要求、团队 Profile 仓库、只读字段和审计记录统一管理。</p></div><div className="update-badge">{status?.checks.filter(item => item.healthy).length ?? 0}/{status?.checks.length ?? 0}</div></section><section className="section-block"><div className="section-heading"><div><h2>合规扫描</h2><p>策略只影响管理器配置，不改变已安装旧版本。</p></div><button className="secondary-button" onClick={onAudit}><FileArchive size={15} />导出审计包</button></div><div className="diagnostic-grid">{status?.checks.map(item => <div className={item.healthy ? "diagnostic-card healthy" : "diagnostic-card warning"} key={item.id}>{item.healthy ? <Check size={17} /> : <CircleAlert size={17} />}<div><strong>{item.id}</strong><span>{item.detail}</span></div></div>)}</div></section><section className="section-block"><div className="section-heading"><div><h2>企业策略 JSON</h2><p>支持 directory / http / git 团队仓库和 appUpdate / trustPolicy / profiles 字段锁定。</p></div><button className="primary-button" onClick={() => void save()}><Save size={15} />保存策略</button></div><textarea className="settings-editor policy-editor" spellCheck={false} value={raw} onChange={event => setRaw(event.target.value)} />{error && <div className="inline-error">{error}</div>}</section><section className="settings-strip"><div><strong>团队 Profile 仓库</strong><span>先预览新增和变更；应用前归档当前 profile-sets.json。</span></div><div className="button-row"><button className="secondary-button" onClick={() => onSync("preview")}><Search size={15} />预览同步</button><button className="primary-button" onClick={() => onSync("apply")}><Download size={15} />备份并应用</button></div></section></div>;
}

function StabilityView({ status, onRun, onArchive, onSaved }: { status: ReliabilityStatus | null; onRun: () => void; onArchive: () => void; onSaved: (value: OperationResult) => void }) {
  const [raw, setRaw] = useState(JSON.stringify(status?.policy ?? {}, null, 2)); const [error, setError] = useState(""); useEffect(() => setRaw(JSON.stringify(status?.policy ?? {}, null, 2)), [status]);
  async function save() { setError(""); try { onSaved(await invoke<OperationResult>("save_reliability_policy", { policy: JSON.parse(raw) })); } catch (value) { setError(String(value)); } }
  const measurements = (status?.baseline?.measurementsMs ?? {}) as Record<string, number>;
  return <div className="page-stack"><section className="hero-panel enterprise-hero healthy"><div className="hero-copy"><span className="section-kicker"><Activity size={14} />Phase 20 · 稳定性收敛</span><h2>可恢复、可度量、只归档</h2><p>任务执行规格随事务落盘，进程重启后自动重排队；单实例、性能预算和日志容量统一观察。</p></div><div className="update-badge">P20</div></section><section className="metric-grid"><Metric icon={ListTodo} label="待恢复任务" value={`${status?.queue.pending ?? 0}`} note={`${status?.queue.restarted ?? 0} 次重启恢复`} /><Metric icon={FileClock} label="活动日志" value={formatBytes(status?.logs.activeBytes ?? 0)} note={`${status?.logs.archives ?? 0} 份历史归档`} /><Metric icon={ShieldCheck} label="单实例" value={status?.singleInstance ? "启用" : "检查"} note="Windows 命名互斥锁" good /><Metric icon={Gauge} label="总览基线" value={measurements.dashboard === undefined ? "未运行" : `${measurements.dashboard} ms`} note="本机可重复测量" /></section><section className="settings-strip"><div><strong>可靠性操作</strong><span>归档不会移除历史日志；基线写入 Caches。</span></div><div className="button-row"><button className="secondary-button" onClick={onArchive}><FileArchive size={15} />立即归档日志</button><button className="primary-button" onClick={onRun}><Gauge size={15} />运行性能基线</button></div></section><section className="section-block"><div className="section-heading"><div><h2>可靠性策略</h2><p>稳定 / Beta / Nightly 验证矩阵与性能预算。</p></div><button className="primary-button" onClick={() => void save()}><Save size={15} />保存策略</button></div><textarea className="settings-editor policy-editor" spellCheck={false} value={raw} onChange={event => setRaw(event.target.value)} />{error && <div className="inline-error">{error}</div>}</section></div>;
}

function SupplyChainView({ status, onVerify, onSaved }: { status: SupplyChainStatus | null; onVerify: () => void; onSaved: (value: OperationResult) => void }) {
  const [raw, setRaw] = useState(JSON.stringify(status?.policy ?? {}, null, 2)); const [error, setError] = useState(""); useEffect(() => setRaw(JSON.stringify(status?.policy ?? {}, null, 2)), [status]);
  async function save() { setError(""); try { onSaved(await invoke<OperationResult>("save_supply_chain_policy", { policy: JSON.parse(raw) })); } catch (value) { setError(String(value)); } }
  const healthy = status?.checks.filter(item => item.healthy).length ?? 0;
  return <div className="page-stack"><section className="hero-panel update-hero"><div className="hero-copy"><span className="section-kicker"><ShieldCheck size={14} />Phase 21 · 签名与供应链</span><h2>{status?.releaseVersion || "本地发布"}</h2><p>Authenticode、RSA-PSS 分离签名、信任/吊销列表、构建来源证明和 CycloneDX 清单形成完整验证链。</p></div><div className="update-badge">{healthy}/{status?.checks.length ?? 0}</div></section><section className="section-block"><div className="section-heading"><div><h2>发布验证链</h2><p>{status?.releasePath || "尚未生成发布目录"}</p></div><button className="primary-button" onClick={onVerify}><ShieldCheck size={15} />验证当前发布</button></div><div className="diagnostic-grid">{status?.checks.map(item => <div className={item.healthy ? "diagnostic-card healthy" : "diagnostic-card warning"} key={item.id}>{item.healthy ? <Check size={17} /> : <CircleAlert size={17} />}<div><strong>{item.detail}</strong><span>{item.id}</span></div></div>)}</div></section><section className="section-block"><div className="section-heading"><div><h2>供应链策略</h2><p>可信指纹、吊销指纹、轮换周期与许可证允许/拒绝清单。</p></div><button className="primary-button" onClick={() => void save()}><Save size={15} />保存策略</button></div><textarea className="settings-editor policy-editor" spellCheck={false} value={raw} onChange={event => setRaw(event.target.value)} />{error && <div className="inline-error">{error}</div>}</section></div>;
}

function FleetView({ status, onSaved, onInventory, onTask }: { status: FleetStatus | null; onSaved: (value: OperationResult) => void; onInventory: () => void; onTask: (id: string, action: "apply" | "rollback") => void }) {
  const [raw, setRaw] = useState(JSON.stringify(status?.config ?? {}, null, 2)); const [componentId, setComponentId] = useState("java"); const [version, setVersion] = useState(""); const [group, setGroup] = useState(""); const [plan, setPlan] = useState<Record<string, unknown> | null>(null); const [error, setError] = useState(""); useEffect(() => setRaw(JSON.stringify(status?.config ?? {}, null, 2)), [status]);
  async function save() { setError(""); try { onSaved(await invoke<OperationResult>("save_fleet_config", { config: JSON.parse(raw) })); } catch (value) { setError(String(value)); } }
  async function preview() { setError(""); try { setPlan(await invoke<Record<string, unknown>>("preview_fleet_rollout", { request: { componentId, version, group, tags: [], batchPercent: 20 } })); } catch (value) { setError(String(value)); } }
  async function stage() { if (!plan) return; try { onSaved(await invoke<OperationResult>("stage_fleet_rollout", { plan })); setPlan(null); } catch (value) { setError(String(value)); } }
  async function transition(id: string, action: "approve" | "pause" | "resume" | "rollback") { try { onSaved(await invoke<OperationResult>("set_fleet_rollout_state", { id, action })); } catch (value) { setError(String(value)); } }
  return <div className="page-stack"><section className="hero-panel enterprise-hero"><div className="hero-copy"><span className="section-kicker"><MonitorCog size={14} />Phase 22 · 远程机器</span><h2>{status?.onlineCount ?? 0}/{status?.nodeCount ?? 0} 节点在线</h2><p>节点按标签和机器组筛选，发布按批次、维护窗口、审批点和失败回退生成可审计事务。</p></div><div className="button-column"><div className="update-badge">P22</div><button className="secondary-button" onClick={onInventory}><RefreshCw size={14} />刷新只读清单</button></div></section><section className="section-block"><div className="section-heading"><div><h2>分批发布计划</h2><p>预览与暂存不会执行远程变更，审批后由 Agent/WinRM 接手。</p></div></div><div className="rollout-form"><label>组件<input value={componentId} onChange={event => setComponentId(event.target.value)} /></label><label>版本<input value={version} onChange={event => setVersion(event.target.value)} placeholder="目标版本" /></label><label>机器组<input value={group} onChange={event => setGroup(event.target.value)} placeholder="全部" /></label><button className="secondary-button" onClick={() => void preview()}><Search size={15} />生成预览</button></div>{plan && <div className="rollout-preview"><pre>{JSON.stringify(plan, null, 2)}</pre><button className="primary-button" onClick={() => void stage()}><FileArchive size={15} />暂存并等待审批</button></div>}</section>{Boolean(status?.rollouts.length) && <section className="section-block"><div className="section-heading"><div><h2>发布事务</h2><p>状态转换和逐节点结果持续追加，修改前记录保留。</p></div></div><div className="rollout-list">{status?.rollouts.map(item => <div key={item.plan.id}><span className={`task-state ${item.status}`}>{item.status}</span><div><strong>{item.plan.componentId} {item.plan.version}</strong><small>{item.plan.nodeCount} 节点 · {item.events.length} 事件 · {item.plan.id}</small></div><div className="button-row">{item.status === "awaiting-approval" && <button className="primary-button" onClick={() => void transition(item.plan.id, "approve")}>审批</button>}{item.status === "approved" && <><button className="primary-button" onClick={() => onTask(item.plan.id, "apply")}>执行</button><button className="secondary-button" onClick={() => void transition(item.plan.id, "pause")}>暂停</button></>}{item.status === "paused" && <button className="secondary-button" onClick={() => void transition(item.plan.id, "resume")}>继续</button>}{["completed", "failed"].includes(item.status) && <button className="danger-outline" onClick={() => void transition(item.plan.id, "rollback")}>请求回滚</button>}{item.status === "rollback-requested" && <button className="danger-outline" onClick={() => onTask(item.plan.id, "rollback")}>执行回滚</button>}</div></div>)}</div></section>}<section className="section-block"><div className="section-heading"><div><h2>节点注册表</h2><p>支持 local / winrm / agent；配置仅保存 credentialRef。最近清单：{status?.inventory.generatedAt ? new Date(status.inventory.generatedAt).toLocaleString() : "未采集"}</p></div><button className="primary-button" onClick={() => void save()}><Save size={15} />保存节点</button></div><textarea className="settings-editor policy-editor" spellCheck={false} value={raw} onChange={event => setRaw(event.target.value)} />{[...(status?.errors ?? []), ...(error ? [error] : [])].length > 0 && <div className="inline-error">{[...(status?.errors ?? []), ...(error ? [error] : [])].join("\n")}</div>}</section></div>;
}

function DeveloperView({ status, onGenerate }: { status: EcosystemStatus | null; onGenerate: (id: string) => void }) {
  const [id, setId] = useState("custom-tool");
  return <div className="page-stack"><div className="notice"><Code2 size={18} /><div><strong>Phase 23 · Manifest SDK 与插件契约</strong><p>模板生成、Schema 校验、权限审查、CLI 补全与中英文资源约定共用同一套本地规范。</p></div></div><section className="section-block"><div className="section-heading"><div><h2>生成组件模板</h2><p>输出到 Config\greendev\examples，不修改当前 components.json。</p></div></div><div className="rollout-form"><label>组件 ID<input value={id} onChange={event => setId(event.target.value)} /></label><button className="primary-button" disabled={!/^[A-Za-z0-9_-]+$/.test(id)} onClick={() => onGenerate(id)}><Code2 size={15} />生成 Manifest</button></div></section><section className="platform-grid"><article className="section-block"><h2>Schema 与示例</h2><code>{status?.manifestSchema}</code><code>{status?.example}</code></article><article className="section-block"><h2>插件最小权限</h2>{status?.pluginPermissions.map(item => <span className="phase-state current" key={item}>{item}</span>)}</article><article className="section-block"><h2>CLI 与自动化</h2>{status?.commands.map(item => <code key={item}>{item}</code>)}</article><article className="section-block"><h2>键盘与语言</h2><p><kbd>Ctrl</kbd> + <kbd>K</kbd> 命令搜索；<kbd>Ctrl</kbd> + <kbd>PageDown</kbd> 切换页面。</p><p>{status?.locales.join(" · ")}</p></article></section></div>;
}

function LogsView({ logs, onSelect }: { logs: OperationResult[]; onSelect: (log: OperationResult) => void }) {
  return <section className="section-block"><div className="section-heading"><div><h2>持久操作日志</h2><p>最近 {logs.length} 条；来源 Logs\GreenDev\operations.jsonl。</p></div></div>{logs.length === 0 ? <div className="empty-state"><FileClock size={32} /><strong>还没有操作记录</strong><span>运行 Doctor 或维护任务后会显示在这里。</span></div> : <div className="log-list">{logs.map(log => <button className="log-row" onClick={() => onSelect(log)} key={log.operationId}><span className={log.success ? "log-status success" : "log-status failure"}>{log.success ? <Check size={15} /> : <CircleAlert size={15} />}</span><div><strong>{log.title}</strong><span>{log.summary}</span></div><code>{new Date(log.finishedAt).toLocaleString()}</code><ChevronRight size={17} /></button>)}</div>}</section>;
}

function ConfigEditorDialog({ document, onClose, onSaved }: { document: ConfigDocument; onClose: () => void; onSaved: (operation: OperationResult) => void }) {
  const [mode, setMode] = useState<"form" | "source">(document.fields.length ? "form" : "source");
  const [raw, setRaw] = useState(document.raw);
  const [fields, setFields] = useState<Record<string, string>>(Object.fromEntries(document.fields.map(item => [item.key, item.value])));
  const [preview, setPreview] = useState<ConfigPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [backupQuery, setBackupQuery] = useState("");
  const [backupPreview, setBackupPreview] = useState<BackupPreview | null>(null);
  const args = () => mode === "source" ? { id: document.id, raw, fields: null } : { id: document.id, raw: null, fields };
  async function previewChange() {
    setBusy(true); setError("");
    try { setPreview(isDesktop ? await invoke<ConfigPreview>("preview_config_change", args()) : { valid: true, errors: [], diff: "+ preview", rendered: raw }); }
    catch (value) { setError(String(value)); }
    finally { setBusy(false); }
  }
  async function saveChange() {
    setBusy(true); setError("");
    try { const operation = isDesktop ? await invoke<OperationResult>("apply_config_change", { ...args(), expectedHash: document.baseHash }) : fallbackResult("保存配置", "界面预览"); onSaved(operation); }
    catch (value) { setError(String(value)); }
    finally { setBusy(false); }
  }
  async function rollback(fileName: string) {
    if (!globalThis.confirm(`恢复 ${fileName}？当前配置会先生成安全备份。`)) return;
    setBusy(true);
    try { const operation = await invoke<OperationResult>("rollback_config", { id: document.id, fileName }); onSaved(operation); }
    catch (value) { setError(String(value)); }
    finally { setBusy(false); }
  }
  async function showBackup(fileName: string) { setBusy(true); try { setBackupPreview(await invoke<BackupPreview>("preview_config_backup", { id: document.id, fileName })); } catch (value) { setError(String(value)); } finally { setBusy(false); } }
  const visibleBackups = document.backups.filter(item => item.fileName.toLowerCase().includes(backupQuery.toLowerCase()));
  return <div className="modal-backdrop"><section className="modal editor-modal"><div className="modal-header"><div><span className="section-kicker"><Code2 size={14} />安全配置编辑</span><h2>{document.name}</h2><p>{document.sourcePath} · 指纹 {document.baseHash.slice(0, 8)}</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="editor-toolbar"><div className="segmented"><button className={mode === "form" ? "selected" : ""} disabled={!document.fields.length} onClick={() => { setMode("form"); setPreview(null); }}>表单</button><button className={mode === "source" ? "selected" : ""} onClick={() => { setMode("source"); setPreview(null); }}>源码</button></div><span>{document.format} · 保存时检查外部修改</span></div><div className="editor-body">{mode === "form" ? <div className="form-fields">{document.fields.map(item => <label className="form-field" key={item.key}><span><strong>{item.label}</strong><small>{item.help}</small></span>{item.kind === "boolean" ? <select value={fields[item.key]} onChange={event => { setFields({ ...fields, [item.key]: event.target.value }); setPreview(null); }}><option value="true">true</option><option value="false">false</option></select> : <input type={item.kind === "number" ? "number" : "text"} value={fields[item.key] ?? ""} onChange={event => { setFields({ ...fields, [item.key]: event.target.value }); setPreview(null); }} />}</label>)}</div> : <div className="source-shell"><div>{raw.split("\n").map((_, index) => <span key={index}>{index + 1}</span>)}</div><textarea className="source-editor" spellCheck={false} value={raw} onChange={event => { setRaw(event.target.value); setPreview(null); }} /></div>}{preview && <div className={preview.valid ? "preview-panel valid" : "preview-panel invalid"}><strong>{preview.valid ? "校验通过" : "校验异常"}</strong>{preview.errors.map(value => <div key={value}>{value}</div>)}<pre>{preview.diff}</pre></div>}{error && <div className="inline-error">{error}</div>}{document.backups.length > 0 && <details className="backup-details"><summary>历史备份（保留最近 30 份）</summary><label className="search-box backup-search"><Search size={14} /><input value={backupQuery} onChange={event => setBackupQuery(event.target.value)} placeholder="按备份名称筛选" /></label>{visibleBackups.map(item => <div className="backup-preview-row" key={item.fileName}><button onClick={() => void showBackup(item.fileName)} disabled={busy}><Search size={13} /><span>{item.fileName}</span><code>{formatBytes(item.sizeBytes)}</code></button><button className="icon-button" title="恢复此备份" onClick={() => void rollback(item.fileName)}><RotateCcw size={13} /></button></div>)}{backupPreview && <pre className="backup-source">{backupPreview.content}</pre>}</details>}</div><div className="modal-footer"><span>源文件变更时保存会中止并提示重新载入</span><div><button className="secondary-button" onClick={() => void previewChange()} disabled={busy}>{busy ? <LoaderCircle className="spin" size={15} /> : <Search size={15} />}校验与差异</button><button className="primary-button" onClick={() => void saveChange()} disabled={busy || !preview?.valid}><Save size={15} />备份、保存并同步</button></div></div></section></div>;
}

function ImportArchiveDialog({ item, onClose, onImport }: { item: ManifestComponent; onClose: () => void; onImport: (path: string) => void }) {
  const [path, setPath] = useState("");
  const supported = [".zip", ".7z", ".tar.gz", ".tgz", ".tar.xz", ".msi"].some(suffix => path.trim().toLowerCase().endsWith(suffix));
  return <div className="modal-backdrop"><section className="modal"><div className="modal-header"><div><span className="section-kicker"><Upload size={14} />离线安装</span><h2>导入 {item.name} 归档</h2><p>支持 ZIP / 7Z / TAR / MSI；导入后计算 SHA256 并写入本机包锁。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="simple-form"><label><span>归档完整路径</span><input value={path} onChange={event => setPath(event.target.value)} placeholder="D:\Downloads\component.zip" /></label><div className="notice"><ShieldCheck size={16} /><div><strong>目标缓存</strong><p>{item.archivePath}</p></div></div></div><div className="modal-footer"><span>原始文件保持不变</span><div><button className="secondary-button" onClick={onClose}>取消</button><button className="primary-button" disabled={!supported} onClick={() => onImport(path.trim())}><Upload size={15} />复制并锁定</button></div></div></section></div>;
}

function InstallSettingsDialog({ settings, onClose, onSaved }: { settings: InstallSettings; onClose: () => void; onSaved: (operation: OperationResult) => void }) {
  const [proxyUrl, setProxyUrl] = useState(settings.proxyUrl);
  const [mirrors, setMirrors] = useState(JSON.stringify(settings.mirrors, null, 2));
  const [error, setError] = useState(""); const [busy, setBusy] = useState(false);
  async function save() {
    setBusy(true); setError("");
    try {
      const parsed = JSON.parse(mirrors) as Record<string, string[]>;
      const operation = isDesktop ? await invoke<OperationResult>("save_install_settings", { settings: { proxyUrl, mirrors: parsed } }) : fallbackResult("保存安装设置", "界面预览");
      onSaved(operation);
    } catch (value) { setError(String(value)); }
    finally { setBusy(false); }
  }
  return <div className="modal-backdrop"><section className="modal large"><div className="modal-header"><div><span className="section-kicker"><Settings2 size={14} />下载传输</span><h2>镜像与代理</h2><p>每个组件的镜像数组优先于清单官方来源。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="simple-form"><label><span>代理 URL（可留空）</span><input value={proxyUrl} onChange={event => setProxyUrl(event.target.value)} placeholder="http://127.0.0.1:7890" /></label><label><span>组件镜像 JSON</span><textarea className="settings-editor" spellCheck={false} value={mirrors} onChange={event => setMirrors(event.target.value)} /></label>{error && <div className="inline-error">{error}</div>}</div><div className="modal-footer"><span>下载使用 curl 断点续传和失败重试</span><div><button className="secondary-button" onClick={onClose}>取消</button><button className="primary-button" onClick={() => void save()} disabled={busy}>{busy ? <LoaderCircle className="spin" size={15} /> : <Save size={15} />}保存设置</button></div></div></section></div>;
}

function EnvironmentDialog({ dashboard, selected, setSelected, busy, onClose, onApply }: { dashboard: Dashboard | null; selected: Set<string>; setSelected: (value: Set<string>) => void; busy: boolean; onClose: () => void; onApply: () => void }) {
  const components = dashboard?.components.filter(item => configurable.includes(item.id)) ?? [];
  function toggle(id: string) { const next = new Set(selected); next.has(id) ? next.delete(id) : next.add(id); setSelected(next); }
  return <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) onClose(); }}><section className="modal large"><div className="modal-header"><div><span className="section-kicker"><ShieldCheck size={14} />用户级配置</span><h2>选择开发组件</h2><p>应用前自动保存完整用户环境快照。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="selection-grid">{components.map(component => <label className={selected.has(component.id) ? "selection-card selected" : "selection-card"} key={component.id}><input type="checkbox" checked={selected.has(component.id)} onChange={() => toggle(component.id)} /><ComponentGlyph id={component.id} /><span><strong>{component.name}</strong><small>{component.environmentVariables.join(" · ") || "PATH"}</small></span><span className="selection-check"><Check size={15} /></span></label>)}</div><div className="modal-footer"><span>已选择 {selected.size} / {components.length}</span><div><button className="secondary-button" onClick={onClose}>取消</button><button className="primary-button" disabled={selected.size === 0 || busy} onClick={onApply}>{busy ? <LoaderCircle className="spin" size={17} /> : <Play size={17} />}备份并应用</button></div></div></section></div>;
}

function VersionDialog({ inventory, busy, onClose, onSwitch, onPin }: { inventory: VersionInventory; busy: string | null; onClose: () => void; onSwitch: (path: string) => void; onPin: (path: string | null) => void }) {
  return <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) onClose(); }}><section className="modal large"><div className="modal-header"><div><span className="section-kicker"><Boxes size={14} />已安装版本</span><h2>{inventory.componentName}</h2><p>切换只更新 {inventory.currentPath}，不会移除任何版本。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="version-list">{inventory.versions.map(item => <div className={item.current ? "version-row current" : "version-row"} key={item.path}><ComponentGlyph id={inventory.componentId} /><div><strong>{item.version}</strong><span title={item.path}>{item.path}</span></div>{item.current && <span className="phase-state current">current</span>}{item.pinned && <span className="phase-state pinned"><Pin size={11} />已固定</span>}<button className="icon-button" title={item.pinned ? "取消固定" : "固定版本"} onClick={() => onPin(item.pinned ? null : item.path)}>{item.pinned ? <PinOff size={15} /> : <Pin size={15} />}</button><button className="secondary-button" disabled={item.current || !item.healthy || busy === "version-switch"} onClick={() => onSwitch(item.path)}>切换</button></div>)}</div><div className="modal-footer"><span>共 {inventory.versions.length} 个健康版本</span><button className="primary-button" onClick={onClose}>完成</button></div></section></div>;
}

function ConfirmDialog({ title, detail, onClose, onConfirm, confirmLabel, destructive, disabled }: { title: string; detail: string; onClose: () => void; onConfirm: () => void; confirmLabel: string; destructive?: boolean; disabled?: boolean }) {
  return <div className="modal-backdrop"><section className="modal"><div className="modal-header"><div><h2>{title}</h2><p>请核对以下执行计划。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="modal-output"><pre>{detail}</pre></div><div className="modal-footer"><span>操作将写入持久日志</span><div><button className="secondary-button" onClick={onClose}>取消</button><button className={destructive ? "danger-button" : "primary-button"} disabled={disabled} onClick={onConfirm}>{destructive ? <Trash2 size={16} /> : <Play size={16} />}{confirmLabel}</button></div></div></section></div>;
}

function TaskDialog({ task, onCancel, onPause }: { task: TaskSnapshot; onCancel: () => void; onPause: () => void }) {
  return <div className="modal-backdrop"><section className="modal task-modal"><div className="modal-header"><div><span className="section-kicker"><Activity size={14} />后台任务 · 第 {task.attempt} 次</span><h2>{task.title}</h2><p>{task.message}</p></div>{task.cancelable && <button className="close-button" title="取消任务" onClick={onCancel}><X size={17} /></button>}</div><div className="task-content"><div className="task-progress"><span style={{ width: `${task.progress}%` }} /></div><div className="task-meta"><strong>{task.progress}% · {task.stage}</strong><code>{task.bytesPerSecond ? `${formatBytes(task.bytesProcessed)} · ${formatBytes(task.bytesPerSecond)}/s` : task.id}</code></div></div><div className="modal-footer"><span>{task.etaSeconds ? `预计剩余 ${task.etaSeconds}s` : "事务阶段持续写入磁盘。"}</span><div>{task.pausable && <button className="secondary-button" onClick={onPause}>{task.status === "paused" ? <Play size={14} /> : <Square size={14} />}{task.status === "paused" ? "继续" : "暂停"}</button>}{task.cancelable && <button className="danger-outline" onClick={onCancel}><X size={14} />取消任务</button>}</div></div></section></div>;
}

function TaskCenterDialog({ tasks, policy, onPolicy, onClose, onSelect, onCancel, onPause, onRetry, onPriority, onSchedule }: { tasks: TaskSnapshot[]; policy: TaskPolicy; onPolicy: (value: TaskPolicy) => void; onClose: () => void; onSelect: (task: TaskSnapshot) => void; onCancel: (id: string) => void; onPause: (task: TaskSnapshot) => void; onRetry: (id: string) => void; onPriority: (id: string, priority: number) => void; onSchedule: (id: string, scheduledAt: number) => void }) {
  const [draft, setDraft] = useState(policy); useEffect(() => setDraft(policy), [policy]);
  return <div className="modal-backdrop"><section className="modal large"><div className="modal-header"><div><span className="section-kicker"><ListTodo size={14} />全局任务中心</span><h2>下载与维护任务</h2><p>持久队列、并发限制、优先级、计划时间、暂停/继续和事务时间线。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="queue-policy"><label>并发数 <input type="number" min="1" max="8" value={draft.maxConcurrent} onChange={event => setDraft({ ...draft, maxConcurrent: Number(event.target.value) })} /></label><label>默认优先级 <input type="number" min="0" max="100" value={draft.defaultPriority} onChange={event => setDraft({ ...draft, defaultPriority: Number(event.target.value) })} /></label><label className="check-control"><input type="checkbox" checked={draft.notifications} onChange={event => setDraft({ ...draft, notifications: event.target.checked })} />完成通知</label><button className="secondary-button" onClick={() => onPolicy(draft)}>保存调度策略</button></div><div className="task-list">{tasks.length ? tasks.map(task => <div className="task-row" key={task.id}><span className={`task-state ${task.status}`}>{task.status === "queued" ? `#${task.queuePosition}` : `${task.progress}%`}</span><div><strong>{task.title}</strong><small>{task.stage} · 优先级 {task.priority} · 第 {task.attempt} 次 · {task.scheduledAt > Date.now() ? `计划 ${new Date(task.scheduledAt).toLocaleTimeString()}` : task.bytesPerSecond ? `${formatBytes(task.bytesPerSecond)}/s` : new Date(task.startedAt).toLocaleString()}</small><div className="mini-progress"><span style={{ width: `${task.progress}%` }} /></div><details className="task-timeline"><summary>{task.timeline.length} 个阶段</summary>{task.timeline.map((event, index) => <span key={`${event.at}-${index}`}>{new Date(event.at).toLocaleTimeString()} · {event.stage} · {event.message}</span>)}</details></div>{task.status === "queued" && <select aria-label="任务优先级" value={task.priority} onChange={event => onPriority(task.id, Number(event.target.value))}><option value="90">高</option><option value="50">普通</option><option value="10">低</option></select>}{task.status === "queued" && <button className="secondary-button" onClick={() => onSchedule(task.id, Date.now() + 15 * 60_000)}>+15 分钟</button>}{task.result && <button className="secondary-button" onClick={() => onSelect(task)}>结果</button>}{task.pausable && <button className="secondary-button" onClick={() => onPause(task)}>{task.status === "paused" ? "继续" : "暂停"}</button>}{task.retryable && <button className="secondary-button" onClick={() => onRetry(task.id)}>重试</button>}{task.cancelable && <button className="danger-outline" onClick={() => onCancel(task.id)}>取消</button>}</div>) : <div className="empty-state"><ListTodo size={30} /><strong>暂无后台任务</strong></div>}</div><div className="modal-footer"><span>事务时间线在异常退出后仍可由恢复中心检查。</span><button className="primary-button" onClick={onClose}>完成</button></div></section></div>;
}

function ProfileImportDialog({ onClose, onImport }: { onClose: () => void; onImport: (path: string) => void }) {
  const [path, setPath] = useState("");
  return <div className="modal-backdrop"><section className="modal"><div className="modal-header"><div><span className="section-kicker"><ArchiveRestore size={14} />跨盘符迁移</span><h2>导入 GreenDev Profile</h2><p>当前 Config 会先完整备份，再导入权威配置并同步到活动工具。</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="simple-form"><label><span>GreenDevProfile ZIP 完整路径</span><input value={path} onChange={event => setPath(event.target.value)} placeholder="E:\Backup\GreenDevProfile-xxx.zip" /></label></div><div className="modal-footer"><span>安装目录和旧版本保持原样</span><div><button className="secondary-button" onClick={onClose}>取消</button><button className="primary-button" disabled={!path.trim().toLowerCase().endsWith(".zip")} onClick={() => onImport(path.trim())}><Download size={15} />备份并导入</button></div></div></section></div>;
}

function ResultDialog({ result, onClose }: { result: OperationResult; onClose: () => void }) {
  return <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) onClose(); }}><section className="modal"><div className="modal-header"><div><span className={result.success ? "result-label success" : "result-label failure"}>{result.success ? <Check size={14} /> : <CircleAlert size={14} />}{result.success ? "操作成功" : "操作异常"}</span><h2>{result.title}</h2><p>{result.summary}</p></div><button className="close-button" onClick={onClose}>×</button></div><div className="modal-output"><pre>{result.output || "没有输出。"}</pre></div><div className="modal-footer"><span>{result.exitCode === null ? result.operationId : `退出代码 ${result.exitCode} · ${result.operationId}`}</span><button className="primary-button" onClick={onClose}>完成</button></div></section></div>;
}

function ComponentGlyph({ id }: { id: string }) { const labels: Record<string, string> = { java: "J", node: "N", python: "Py", gradle: "G", maven: "M", android: "A", rust: "Rs", c: "C", acpi: "AC", mysql: "My", ghidra: "Gh" }; return <span className={`component-glyph glyph-${id}`}>{labels[id] ?? id.slice(0, 2)}</span>; }
function Status({ healthy, label }: { healthy: boolean; label?: string }) { return <span className={healthy ? "status healthy" : "status unhealthy"}><span />{label ?? (healthy ? "正常" : "异常")}</span>; }

export default App;
