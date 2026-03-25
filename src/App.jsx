import { useState, useEffect } from "react";
import {
  LayoutDashboard,
  Crosshair,
  Wallet,
  Search,
  BarChart3,
  Settings as SettingsIcon,
  Lock,
  Unlock,
  Activity,
  HelpCircle,
} from "lucide-react";
import Dashboard from "./views/Dashboard";
import HuntConsole from "./views/HuntConsole";
import WalletManager from "./views/WalletManager";
import OpportunityInspector from "./views/OpportunityInspector";
import AnalyticsBay from "./views/AnalyticsBay";
import SettingsView from "./views/Settings";
import CommandPalette from "./components/CommandPalette";
import { useAppStore } from "./stores/appStore";
import { useWalletStore } from "./stores/walletStore";
import { useSettingsStore } from "./stores/settingsStore";
import { useHuntStore } from "./stores/huntStore";
import { initNotifications } from "./lib/notifications";

const NAV_ITEMS = [
  {
    id: "dashboard",
    label: "Command Center",
    icon: LayoutDashboard,
    shortcut: "Alt+1",
  },
  { id: "hunt", label: "Hunt Console", icon: Crosshair, shortcut: "Alt+2" },
  { id: "wallets", label: "Wallet Manager", icon: Wallet, shortcut: "Alt+3" },
  {
    id: "inspect",
    label: "Opportunity Inspector",
    icon: Search,
    shortcut: "Alt+4",
  },
  {
    id: "analytics",
    label: "Analytics Bay",
    icon: BarChart3,
    shortcut: "Alt+5",
  },
  {
    id: "settings",
    label: "Settings",
    icon: SettingsIcon,
    shortcut: "Alt+6",
  },
];

const VIEW_MAP = {
  dashboard: Dashboard,
  hunt: HuntConsole,
  wallets: WalletManager,
  inspect: OpportunityInspector,
  analytics: AnalyticsBay,
  settings: SettingsView,
};

export default function App() {
  const {
    activeView,
    setActiveView,
    commandPaletteOpen,
    setCommandPaletteOpen,
  } = useAppStore();
  const { vaultLocked, fetchVaultStatus, lockVault } = useWalletStore();
  const { autoLockMinutes, loadSettings: loadSettingsFromStore } = useSettingsStore();
  const { running: huntRunning, setRunning: setHuntRunning } = useHuntStore();

  useEffect(() => {
    fetchVaultStatus();
    loadSettingsFromStore();
    initNotifications();
  }, [fetchVaultStatus, loadSettingsFromStore]);
  const [sidebarExpanded, setSidebarExpanded] = useState(true);
  const [helpOpen, setHelpOpen] = useState(false);

  // ── Auto-Lock Timer ─────────────────────────────────────────
  useEffect(() => {
    if (vaultLocked || autoLockMinutes <= 0) return;
    let timer;
    const resetTimer = () => {
      clearTimeout(timer);
      timer = setTimeout(() => lockVault(), autoLockMinutes * 60_000);
    };
    const events = ["mousedown", "keydown", "mousemove", "scroll", "touchstart"];
    events.forEach((e) => window.addEventListener(e, resetTimer, { passive: true }));
    resetTimer();
    return () => {
      clearTimeout(timer);
      events.forEach((e) => window.removeEventListener(e, resetTimer));
    };
  }, [vaultLocked, autoLockMinutes, lockVault]);

  // ── Keyboard Shortcuts ──────────────────────────────────────
  useEffect(() => {
    const handleKeyDown = (e) => {
      // Ctrl+K → Command palette
      if (e.ctrlKey && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      }
      // Alt+1-6 → Navigate views
      if (e.altKey && e.key >= "1" && e.key <= "6") {
        e.preventDefault();
        const idx = parseInt(e.key) - 1;
        if (NAV_ITEMS[idx]) setActiveView(NAV_ITEMS[idx].id);
      }
      // Ctrl+H → Toggle hunt
      if (e.ctrlKey && e.key === "h") {
        e.preventDefault();
        setHuntRunning(!huntRunning);
      }
      // Ctrl+L → Lock vault
      if (e.ctrlKey && e.key === "l") {
        e.preventDefault();
        lockVault();
      }
      // ? → Help overlay (only if no input focused)
      if (e.key === "?" && !["INPUT", "TEXTAREA", "SELECT"].includes(e.target.tagName)) {
        e.preventDefault();
        setHelpOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [commandPaletteOpen, setActiveView, setCommandPaletteOpen, huntRunning, setHuntRunning, lockVault]);

  const ActiveView = VIEW_MAP[activeView] || Dashboard;

  return (
    <div className="flex flex-col h-screen bg-bg text-text font-mono">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-3 bg-surface border-b border-border">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-primary to-accent flex items-center justify-center text-bg font-black text-xl">
            Σ
          </div>
          <div>
            <h1 className="text-primary font-bold tracking-wider text-lg">
              ΣHARVEST
            </h1>
            <p className="text-text-dim text-[10px] tracking-widest uppercase">
              Web3 Autonomous Collection System
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setCommandPaletteOpen(true)}
            className="px-3 py-1.5 text-xs bg-surface-raised border border-border rounded text-text-muted hover:text-text transition-colors"
          >
            ⌘K
          </button>
          <div className="px-2.5 py-1 rounded bg-surface-raised border border-border text-[11px] text-warning">
            PERSONAL USE
          </div>
          <div className="px-2.5 py-1 rounded bg-surface-raised border border-border text-[11px] text-primary">
            v0.1.0
          </div>
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <nav
          className={`${sidebarExpanded ? "w-56" : "w-14"} bg-surface border-r border-border flex flex-col transition-all duration-200 shrink-0`}
        >
          <button
            onClick={() => setSidebarExpanded(!sidebarExpanded)}
            className="p-3 text-text-muted text-xs border-b border-border hover:text-text transition-colors text-left"
          >
            {sidebarExpanded ? "◀ Collapse" : "▶"}
          </button>
          <div className="flex-1 py-2 overflow-y-auto">
            {NAV_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = activeView === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveView(item.id)}
                  className={`flex items-center gap-3 w-full px-4 py-2.5 text-left text-xs transition-all duration-150 border-l-[3px] ${
                    isActive
                      ? "bg-surface-raised text-text border-l-primary"
                      : "border-l-transparent text-text-muted hover:text-text hover:bg-surface-raised/50"
                  }`}
                >
                  <Icon size={16} className="shrink-0" />
                  {sidebarExpanded && (
                    <div className="overflow-hidden">
                      <div className="truncate">{item.label}</div>
                      <div className="text-[10px] text-text-dim">
                        {item.shortcut}
                      </div>
                    </div>
                  )}
                </button>
              );
            })}
          </div>
          {/* Sidebar footer */}
          <div className="p-3 border-t border-border">
            <div className="flex items-center gap-2 text-text-muted text-[10px]">
              {vaultLocked ? (
                <><Lock size={12} className="text-danger" />{sidebarExpanded && <span className="text-danger">Vault Locked</span>}</>
              ) : (
                <><Unlock size={12} className="text-primary" />{sidebarExpanded && <span className="text-primary">Vault Open</span>}</>
              )}
            </div>
          </div>
        </nav>

        {/* Main Content */}
        <main className="flex-1 overflow-auto p-8">
          <ActiveView />
        </main>
      </div>

      {/* Footer */}
      <footer className="flex items-center justify-between px-6 py-2 bg-surface border-t border-border text-[11px] text-text-dim">
        <span>ΣHARVEST v0.1.0 — iamthegreatdestroyer</span>
        <span>Ctrl+K for commands</span>
      </footer>

      {/* Command Palette */}
      <CommandPalette />

      {/* Help Overlay */}
      {helpOpen && (
        <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center" onClick={() => setHelpOpen(false)}>
          <div className="bg-surface rounded-lg border border-border p-6 w-[420px] max-h-[80vh] overflow-auto" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-bold text-primary flex items-center gap-2">
                <HelpCircle size={16} />
                Keyboard Shortcuts
              </h3>
              <button onClick={() => setHelpOpen(false)} className="text-text-muted hover:text-text text-xs">ESC</button>
            </div>
            <div className="space-y-2 text-xs">
              {[
                ["Ctrl+K", "Open command palette"],
                ["Alt+1", "Command Center"],
                ["Alt+2", "Hunt Console"],
                ["Alt+3", "Wallet Manager"],
                ["Alt+4", "Opportunity Inspector"],
                ["Alt+5", "Analytics Bay"],
                ["Alt+6", "Settings"],
                ["Ctrl+H", "Toggle hunt"],
                ["Ctrl+L", "Lock vault"],
                ["?", "Toggle this help"],
              ].map(([key, desc]) => (
                <div key={key} className="flex justify-between py-1 border-b border-border/50">
                  <kbd className="px-2 py-0.5 bg-surface-raised rounded text-primary font-mono">{key}</kbd>
                  <span className="text-text-muted">{desc}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
