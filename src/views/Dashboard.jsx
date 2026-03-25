import { useState, useEffect, useCallback } from "react";
import { LayoutDashboard, TrendingUp, Zap, Activity, Lock, Unlock, Wallet } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import GasTicker from "../components/GasTicker";
import HarvestFeed from "../components/HarvestFeed";
import SigmaCoreWidget from "../components/SigmaCoreWidget";
import SparklineChart from "../components/SparklineChart";
import { useWalletStore } from "../stores/walletStore";

export default function Dashboard() {
  const { vaultLocked, wallets, fetchVaultStatus } = useWalletStore();
  const [appStatus, setAppStatus] = useState(null);
  const [timeSeries, setTimeSeries] = useState({ "7": [], "30": [] });
  const [timeRange, setTimeRange] = useState("7");

  const fetchTimeSeries = useCallback(async () => {
    try {
      const [week, month] = await Promise.all([
        invoke("get_time_series", { days: 7 }),
        invoke("get_time_series", { days: 30 }),
      ]);
      setTimeSeries({ "7": week, "30": month });
    } catch {
      // Time-series fetch is non-critical
    }
  }, []);

  useEffect(() => {
    fetchVaultStatus();
    invoke("get_app_status").then(setAppStatus).catch(console.error);
    fetchTimeSeries();
  }, [fetchVaultStatus, fetchTimeSeries]);
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <LayoutDashboard className="text-primary" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Command Center</h2>
          <p className="text-text-muted text-xs">
            Real-time overview of all ΣHARVEST operations
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-primary to-transparent" />

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        {[
          {
            label: "Vault Status",
            value: vaultLocked ? "Locked" : "Unlocked",
            icon: vaultLocked ? Lock : Unlock,
            color: vaultLocked ? "text-danger" : "text-primary",
          },
          {
            label: "Active Wallets",
            value: vaultLocked ? "—" : String(wallets.length),
            icon: Wallet,
            color: "text-accent",
          },
          {
            label: "Pending Claims",
            value: String(appStatus?.pending_claims ?? 0),
            icon: Activity,
            color: "text-warning",
          },
          {
            label: "Active Opportunities",
            value: String(appStatus?.active_opportunities ?? 0),
            icon: Zap,
            color: "text-primary",
          },
        ].map((stat) => (
          <div
            key={stat.label}
            className="bg-surface rounded-lg border border-border p-4"
          >
            <div className="flex items-center justify-between mb-2">
              <span className="text-text-muted text-[11px] uppercase tracking-wider">
                {stat.label}
              </span>
              <stat.icon size={14} className={stat.color} />
            </div>
            <div className={`text-2xl font-bold ${stat.color}`}>
              {stat.value}
            </div>
          </div>
        ))}
      </div>

      {/* Two-column layout */}
      <div className="grid grid-cols-3 gap-4">
        {/* Sparkline: Value Collected */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <div className="flex items-center justify-between mb-1">
            <span className="text-text-muted text-[11px] uppercase tracking-wider">
              Value Collected
            </span>
            <div className="flex gap-1">
              {["7", "30"].map((r) => (
                <button
                  key={r}
                  onClick={() => setTimeRange(r)}
                  className={`text-[10px] px-1.5 py-0.5 rounded ${
                    timeRange === r
                      ? "bg-primary/20 text-primary"
                      : "text-text-muted hover:text-text"
                  }`}
                >
                  {r}d
                </button>
              ))}
            </div>
          </div>
          <SparklineChart
            data={timeSeries[timeRange].map((p) => ({
              date: p.date,
              value: p.value_usd,
            }))}
            color="#00e5ff"
            dataKey="value"
          />
        </div>

        {/* Sparkline: Gas Spent */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <span className="text-text-muted text-[11px] uppercase tracking-wider block mb-1">
            Gas Spent
          </span>
          <SparklineChart
            data={timeSeries[timeRange].map((p) => ({
              date: p.date,
              value: p.gas_usd,
            }))}
            color="#ff6b6b"
            dataKey="value"
          />
        </div>

        {/* Sparkline: Net Profit */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <span className="text-text-muted text-[11px] uppercase tracking-wider block mb-1">
            Net Profit
          </span>
          <SparklineChart
            data={timeSeries[timeRange].map((p) => ({
              date: p.date,
              value: p.net_usd,
            }))}
            color="#51cf66"
            dataKey="value"
          />
        </div>
      </div>

      {/* Two-column layout: Feed + Gas/ΣCORE */}
      <div className="grid grid-cols-2 gap-4">
        {/* Harvest Feed */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <h3 className="text-sm font-semibold text-primary mb-3 flex items-center gap-2">
            <Zap size={14} />
            Live Harvest Feed
          </h3>
          <HarvestFeed />
        </div>

        {/* Right Column: Gas + ΣCORE */}
        <div className="space-y-4">
          {/* Gas Ticker */}
          <div className="bg-surface rounded-lg border border-border p-4">
            <h3 className="text-sm font-semibold text-accent mb-3 flex items-center gap-2">
              <Activity size={14} />
              Gas Prices
            </h3>
            <GasTicker />
          </div>

          {/* ΣCORE Nervous System */}
          <SigmaCoreWidget />
        </div>
      </div>
    </div>
  );
}
