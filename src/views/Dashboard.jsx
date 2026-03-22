import { LayoutDashboard, TrendingUp, Zap, Activity } from "lucide-react";
import GasTicker from "../components/GasTicker";
import HarvestFeed from "../components/HarvestFeed";

export default function Dashboard() {
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
            label: "Total Collected",
            value: "$0.00",
            icon: TrendingUp,
            color: "text-primary",
          },
          {
            label: "Active Opportunities",
            value: "0",
            icon: Zap,
            color: "text-accent",
          },
          {
            label: "Pending Claims",
            value: "0",
            icon: Activity,
            color: "text-warning",
          },
          {
            label: "Success Rate",
            value: "—",
            icon: TrendingUp,
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
      <div className="grid grid-cols-2 gap-4">
        {/* Harvest Feed */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <h3 className="text-sm font-semibold text-primary mb-3 flex items-center gap-2">
            <Zap size={14} />
            Live Harvest Feed
          </h3>
          <HarvestFeed />
        </div>

        {/* Gas Ticker */}
        <div className="bg-surface rounded-lg border border-border p-4">
          <h3 className="text-sm font-semibold text-accent mb-3 flex items-center gap-2">
            <Activity size={14} />
            Gas Prices
          </h3>
          <GasTicker />
        </div>
      </div>
    </div>
  );
}
