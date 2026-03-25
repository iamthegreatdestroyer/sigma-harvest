import { useState, useEffect, useRef } from "react";
import {
  Settings as SettingsIcon,
  Save,
  Download,
  Upload,
  Eye,
  EyeOff,
  RotateCcw,
  CheckCircle,
} from "lucide-react";
import { useSettingsStore } from "../stores/settingsStore";

const CHAINS = ["ethereum", "arbitrum", "optimism", "base", "polygon", "zksync"];
const SOURCES = ["rss", "dappradar", "galxe", "onchain", "social"];
const LOCK_OPTIONS = [
  { label: "5 minutes", value: 5 },
  { label: "15 minutes", value: 15 },
  { label: "30 minutes", value: 30 },
  { label: "1 hour", value: 60 },
  { label: "Never", value: 0 },
];

export default function Settings() {
  const {
    autoLockMinutes,
    notificationsEnabled,
    notificationThreshold,
    rpcOverrides,
    gasCeilings,
    apiKeys,
    discoveryIntervals,
    loading,
    error,
    dirty,
    loadSettings,
    saveAll,
    exportSettings,
    importSettings,
    setAutoLockMinutes,
    setNotificationsEnabled,
    setNotificationThreshold,
    setRpcOverride,
    setGasCeiling,
    setApiKey,
    setDiscoveryInterval,
    clearError,
  } = useSettingsStore();

  const [showApiKeys, setShowApiKeys] = useState({});
  const [saved, setSaved] = useState(false);
  const fileInputRef = useRef(null);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const handleSave = async () => {
    await saveAll();
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleExport = () => {
    const json = exportSettings();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "sigma-harvest-settings.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const ok = importSettings(ev.target.result);
      if (!ok) alert("Invalid settings file");
    };
    reader.readAsText(file);
    e.target.value = "";
  };

  const toggleKeyVisibility = (service) =>
    setShowApiKeys((prev) => ({ ...prev, [service]: !prev[service] }));

  return (
    <div className="space-y-6 max-w-3xl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <SettingsIcon className="text-primary" size={28} />
          <div>
            <h2 className="text-xl font-bold text-text">Settings</h2>
            <p className="text-text-muted text-xs">
              Configure ΣHARVEST preferences
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-surface-raised border border-border rounded text-text-muted hover:text-text transition-colors"
          >
            <Download size={12} /> Export
          </button>
          <button
            onClick={() => fileInputRef.current?.click()}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-surface-raised border border-border rounded text-text-muted hover:text-text transition-colors"
          >
            <Upload size={12} /> Import
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".json"
            className="hidden"
            onChange={handleImport}
          />
          <button
            onClick={handleSave}
            disabled={loading || !dirty}
            className={`flex items-center gap-1.5 px-4 py-1.5 text-xs rounded font-medium transition-colors ${
              dirty
                ? "bg-primary text-bg hover:opacity-90"
                : "bg-surface-raised border border-border text-text-dim"
            }`}
          >
            {saved ? (
              <><CheckCircle size={12} /> Saved</>
            ) : (
              <><Save size={12} /> Save All</>
            )}
          </button>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-primary to-transparent" />

      {error && (
        <div className="p-3 rounded bg-danger/10 border border-danger/30 text-danger text-xs flex justify-between">
          <span>{error}</span>
          <button onClick={clearError} className="underline">Dismiss</button>
        </div>
      )}

      {/* Security Section */}
      <Section title="Security">
        <FieldRow label="Auto-Lock Timeout" description="Lock vault after idle period">
          <select
            value={autoLockMinutes}
            onChange={(e) => setAutoLockMinutes(parseInt(e.target.value, 10))}
            className="bg-surface-raised border border-border rounded px-3 py-1.5 text-xs text-text"
          >
            {LOCK_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
        </FieldRow>
      </Section>

      {/* Notifications Section */}
      <Section title="Notifications">
        <FieldRow label="Desktop Notifications" description="Alert on high-score opportunities and claim results">
          <Toggle checked={notificationsEnabled} onChange={setNotificationsEnabled} />
        </FieldRow>
        <FieldRow label="Score Threshold" description="Minimum σ-score to trigger notification">
          <input
            type="number"
            min={0}
            max={100}
            value={notificationThreshold}
            onChange={(e) => setNotificationThreshold(parseInt(e.target.value, 10) || 0)}
            className="w-20 bg-surface-raised border border-border rounded px-3 py-1.5 text-xs text-text text-right"
          />
        </FieldRow>
      </Section>

      {/* API Keys Section */}
      <Section title="API Keys">
        {Object.entries(apiKeys).map(([service, key]) => (
          <FieldRow key={service} label={service.charAt(0).toUpperCase() + service.slice(1)} description={`API key for ${service}`}>
            <div className="flex items-center gap-1">
              <input
                type={showApiKeys[service] ? "text" : "password"}
                value={key}
                onChange={(e) => setApiKey(service, e.target.value)}
                placeholder="Enter API key..."
                className="w-64 bg-surface-raised border border-border rounded px-3 py-1.5 text-xs text-text font-mono"
              />
              <button
                onClick={() => toggleKeyVisibility(service)}
                className="p-1.5 text-text-muted hover:text-text"
              >
                {showApiKeys[service] ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>
          </FieldRow>
        ))}
      </Section>

      {/* Gas Ceilings Section */}
      <Section title="Gas Ceilings (gwei)">
        <div className="grid grid-cols-3 gap-3">
          {CHAINS.map((chain) => (
            <div key={chain} className="flex items-center justify-between bg-surface-raised rounded p-2.5 border border-border">
              <span className="text-xs text-text-muted capitalize">{chain}</span>
              <input
                type="number"
                min={0}
                step={0.1}
                value={gasCeilings[chain] ?? 0}
                onChange={(e) => setGasCeiling(chain, parseFloat(e.target.value) || 0)}
                className="w-20 bg-bg border border-border rounded px-2 py-1 text-xs text-text text-right"
              />
            </div>
          ))}
        </div>
      </Section>

      {/* RPC Overrides Section */}
      <Section title="RPC Endpoint Overrides">
        {CHAINS.map((chain) => (
          <FieldRow key={chain} label={chain.charAt(0).toUpperCase() + chain.slice(1)} description="Custom RPC URL (leave empty for default)">
            <input
              type="text"
              value={rpcOverrides[chain] || ""}
              onChange={(e) => setRpcOverride(chain, e.target.value)}
              placeholder="https://..."
              className="w-80 bg-surface-raised border border-border rounded px-3 py-1.5 text-xs text-text font-mono"
            />
          </FieldRow>
        ))}
      </Section>

      {/* Discovery Intervals Section */}
      <Section title="Discovery Intervals (seconds)">
        <div className="grid grid-cols-3 gap-3">
          {SOURCES.map((source) => (
            <div key={source} className="flex items-center justify-between bg-surface-raised rounded p-2.5 border border-border">
              <span className="text-xs text-text-muted capitalize">{source}</span>
              <input
                type="number"
                min={10}
                step={10}
                value={discoveryIntervals[source] ?? 300}
                onChange={(e) => setDiscoveryInterval(source, parseInt(e.target.value, 10) || 60)}
                className="w-20 bg-bg border border-border rounded px-2 py-1 text-xs text-text text-right"
              />
            </div>
          ))}
        </div>
      </Section>
    </div>
  );
}

/* ── Helper Components ──────────────────────────────────── */

function Section({ title, children }) {
  return (
    <div className="bg-surface rounded-lg border border-border p-5">
      <h3 className="text-sm font-semibold text-primary mb-4">{title}</h3>
      <div className="space-y-3">{children}</div>
    </div>
  );
}

function FieldRow({ label, description, children }) {
  return (
    <div className="flex items-center justify-between py-1">
      <div>
        <div className="text-xs font-medium text-text">{label}</div>
        {description && <div className="text-[10px] text-text-dim">{description}</div>}
      </div>
      {children}
    </div>
  );
}

function Toggle({ checked, onChange }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-5 rounded-full transition-colors ${
        checked ? "bg-primary" : "bg-border"
      }`}
    >
      <div
        className={`absolute top-0.5 w-4 h-4 rounded-full bg-text transition-transform ${
          checked ? "translate-x-5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}
