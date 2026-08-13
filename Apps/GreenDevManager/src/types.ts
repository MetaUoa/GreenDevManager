export type ComponentStatus = {
  id: string;
  name: string;
  category: string;
  installed: boolean;
  healthy: boolean;
  version: string;
  executable: string;
  currentPath: string | null;
  currentTarget: string | null;
  environmentVariables: string[];
};

export type CacheEntry = {
  id: string;
  name: string;
  path: string;
  sizeBytes: number;
  protected: boolean;
};

export type Dashboard = {
  root: string;
  totalSizeBytes: number;
  cacheSizeBytes: number;
  storageReady: boolean;
  installedCount: number;
  healthyCount: number;
  components: ComponentStatus[];
  caches: CacheEntry[];
};

export type StorageMetrics = Pick<Dashboard, "totalSizeBytes" | "cacheSizeBytes" | "caches">;

export type BootstrapStatus = {
  configured: boolean;
  root: string;
  currentVersion: string;
  manifestUrl: string;
  mode?: "existing" | "fresh";
};

export type OperationResult = {
  operationId: string;
  success: boolean;
  title: string;
  summary: string;
  output: string;
  exitCode: number | null;
  kind: string;
  startedAt: number;
  finishedAt: number;
};

export type ConfigStatus = {
  id: string;
  name: string;
  sourcePath: string;
  deployedPath: string | null;
  sourceHash: string;
  deployedHash: string | null;
  state: "synced" | "drifted" | "missing" | "reference";
  detail: string;
};

export type ConfigField = {
  key: string;
  label: string;
  value: string;
  kind: string;
  help: string;
};

export type ConfigBackup = { fileName: string; createdAt: string; sizeBytes: number };

export type ConfigDocument = {
  id: string;
  name: string;
  format: string;
  sourcePath: string;
  raw: string;
  baseHash: string;
  fields: ConfigField[];
  backups: ConfigBackup[];
};

export type BackupPreview = { fileName: string; content: string; sourceHash: string };

export type ConfigPreview = {
  valid: boolean;
  errors: string[];
  diff: string;
  rendered: string;
};

export type EnvironmentBackup = {
  fileName: string;
  path: string;
  createdAt: string;
  variableCount: number;
  root: string;
};

export type VersionEntry = {
  version: string;
  path: string;
  current: boolean;
  pinned: boolean;
  healthy: boolean;
};

export type VersionInventory = {
  componentId: string;
  componentName: string;
  supportsSwitching: boolean;
  currentPath: string | null;
  versions: VersionEntry[];
};

export type AndroidPackage = {
  id: string;
  version: string;
  description: string;
  installed: boolean;
  obsolete: boolean;
};

export type ManifestComponent = {
  id: string;
  name: string;
  desiredVersion: string;
  installDir: string;
  currentLink: string | null;
  sourceUrl: string;
  archivePath: string;
  enabled: boolean;
  installed: boolean;
  active: boolean;
  archiveCached: boolean;
  checksumReady: boolean;
  pinnedElsewhere: boolean;
  dependencies: string[];
  blockedReason: string;
  state: "disabled" | "current" | "installed" | "available" | "blocked" | "pinned";
};

export type InstallPlan = {
  componentId: string;
  action: "install" | "update";
  steps: string[];
  blockers: string[];
  archivePath: string;
  sourceUrl: string;
  expectedSha256: string;
  ready: boolean;
};

export type InstallSettings = { proxyUrl: string; mirrors: Record<string, string[]> };

export type DiagnosticItem = { id: string; name: string; healthy: boolean; detail: string };
export type DiagnosticReport = { appVersion: string; generatedAt: number; healthyCount: number; items: DiagnosticItem[] };

export type StoragePoint = { recordedAt: number; totalSizeBytes: number; cacheSizeBytes: number };

export type UpdateCandidate = {
  componentId: string;
  name: string;
  currentVersion: string;
  targetVersion: string;
  updateAvailable: boolean;
  installed: boolean;
  active: boolean;
  pinned: boolean;
  checksumReady: boolean;
  catalogAvailable: boolean;
  installReady: boolean;
  canAdopt: boolean;
  policy: string;
  releaseNotes: string;
};

export type BatchInstallPlan = { componentIds: string[]; orderedIds: string[]; steps: string[]; blockers: string[]; ready: boolean };

export type MaintenanceStatus = {
  currentVersion: string;
  latestLocalVersion: string;
  updateAvailable: boolean;
  releaseDirectory: string;
  crashLog: string;
  pendingTransactions: number;
  buildMode: string;
};

export type TaskSnapshot = {
  id: string;
  title: string;
  kind: string;
  status: "queued" | "running" | "paused" | "completed" | "failed" | "cancelled";
  progress: number;
  message: string;
  cancelable: boolean;
  pausable: boolean;
  retryable: boolean;
  stage: string;
  bytesProcessed: number;
  bytesTotal: number;
  bytesPerSecond: number;
  etaSeconds: number | null;
  attempt: number;
  priority: number;
  scheduledAt: number;
  queuePosition: number;
  timeline: Array<{ at: number; stage: string; message: string }>;
  startedAt: number;
  updatedAt: number;
  result: OperationResult | null;
};
export type TaskPolicy = { maxConcurrent: number; defaultPriority: number; notifications: boolean };
export type RecoveryItem = { id: string; kind: "config" | "environment" | "transaction" | "app" | "profile"; title: string; path: string; relativePath: string; createdAt: number; sizeBytes: number; canRestore: boolean };
export type RecoveryCenter = { items: RecoveryItem[]; pendingTransactions: number };
export type EnterpriseStatus = { policy: Record<string, unknown>; checks: Array<{ id: string; healthy: boolean; detail: string }>; healthy: boolean; generatedAt: number };
export type ReliabilityStatus = {
  policy: Record<string, unknown>;
  queue: { pending: number; completed: number; restarted: number };
  logs: { activeBytes: number; archives: number; crashBytes: number };
  singleInstance: boolean;
  baseline: Record<string, unknown> | null;
  generatedAt: number;
};
export type SupplyChainStatus = {
  policy: Record<string, unknown>;
  releasePath: string;
  releaseVersion: string;
  checks: Array<{ id: string; healthy: boolean; detail: string }>;
  generatedAt: number;
};
export type FleetStatus = {
  config: Record<string, unknown>;
  nodeCount: number;
  onlineCount: number;
  errors: string[];
  rollouts: Array<{ status: string; plan: { id: string; componentId: string; version: string; nodeCount: number }; events: Array<{ at: number; state: string; detail: string }> }>;
  inventory: { schemaVersion: number; generatedAt: number; nodes: Array<{ id: string; status: "online" | "offline"; checkedAt: number; transport: string; group: string; inventory: unknown[]; error: string }> };
  generatedAt: number;
};
export type EcosystemStatus = {
  manifestSchema: string;
  example: string;
  commands: string[];
  locales: string[];
  pluginPermissions: string[];
  generatedAt: number;
};

export type ManifestEditor = { raw: string; baseHash: string; errors: string[]; path: string };
export type AppUpdateStatus = {
  currentVersion: string; latestLocalVersion: string; latestRemoteVersion: string; targetVersion: string;
  updateAvailable: boolean; prepared: boolean; localVersions: string[];
  settings: { schemaVersion: number; channel: "stable" | "beta" | "nightly" | "local"; feedUrl: string; requireSignature: boolean; autoDownload: boolean };
  feed: Record<string, unknown>;
};
export type ProfileDefinition = { id: string; name: string; components: string[]; teamTemplate: boolean; machineOverrides: Record<string, string> };
export type ProfileSets = { schemaVersion: number; activeProfile: string; profiles: ProfileDefinition[] };
export type ProfileDiff = { profileId: string; matched: boolean; rows: Array<{ id: string; state: "matched" | "drifted"; changes: string[] }> };
