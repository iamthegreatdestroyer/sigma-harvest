import { useState, useEffect } from "react";
import {
  LayoutDashboard,
  Crosshair,
  Wallet,
  Search,
  BarChart3,
  Settings,
  Lock,
  Unlock,
  Activity,
} from "lucide-react";
import Dashboard from "./views/Dashboard";
import HuntConsole from "./views/HuntConsole";
import WalletManager from "./views/WalletManager";
import OpportunityInspector from "./views/OpportunityInspector";
import AnalyticsBay from "./views/AnalyticsBay";
import CommandPalette from "./components/CommandPalette";
import { useAppStore } from "./stores/appStore";
import { useWalletStore } from "./stores/walletStore";

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
];

const VIEW_MAP = {
  dashboard: Dashboard,
  hunt: HuntConsole,
  wallets: WalletManager,
  inspect: OpportunityInspector,
  analytics: AnalyticsBay,
};

export default function App() {
  const {
    activeView,
    setActiveView,
    commandPaletteOpen,
    setCommandPaletteOpen,
  } = useAppStore();
  const { vaultLocked, fetchVaultStatus } = useWalletStore();

  useEffect(() => {
    fetchVaultStatus();
  }, [fetchVaultStatus]);
  const [sidebarExpanded, setSidebarExpanded] = useState(true);

  useEffect(() => {
    const handleKeyDown = (e) => {
      // Ctrl+K → Command palette
      if (e.ctrlKey && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      }
      // Alt+1-5 → Navigate views
      if (e.altKey && e.key >= "1" && e.key <= "5") {
        e.preventDefault();
        setActiveView(NAV_ITEMS[parseInt(e.key) - 1].id);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [commandPaletteOpen, setActiveView, setCommandPaletteOpen]);

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
    </div>
  );
}
