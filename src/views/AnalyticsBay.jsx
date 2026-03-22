import { BarChart3, TrendingUp, DollarSign, Fuel } from "lucide-react";

export default function AnalyticsBay() {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <BarChart3 className="text-accent" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Analytics Bay</h2>
          <p className="text-text-muted text-xs">
            Performance metrics and ROI tracking
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-accent to-transparent" />

      {/* Metric Cards */}
      <div className="grid grid-cols-3 gap-4">
        {[
          {
            label: "Total Value Collected",
            value: "$0.00",
            icon: DollarSign,
            color: "text-primary",
          },
          {
            label: "Gas Spent",
            value: "$0.00",
            icon: Fuel,
            color: "text-warning",
          },
          {
            label: "Net ROI",
            value: "—",
            icon: TrendingUp,
            color: "text-accent",
          },
        ].map((m) => (
          <div
            key={m.label}
            className="bg-surface rounded-lg border border-border p-5"
          >
            <div className="flex items-center justify-between mb-3">
              <span className="text-text-muted text-[11px] uppercase tracking-wider">
                {m.label}
              </span>
              <m.icon size={16} className={m.color} />
            </div>
            <div className={`text-3xl font-bold ${m.color}`}>{m.value}</div>
          </div>
        ))}
      </div>

      {/* Chart Placeholder */}
      <div className="bg-surface rounded-lg border border-border p-6">
        <h3 className="text-sm font-semibold text-text-muted mb-4">
          Collection Over Time
        </h3>
        <div className="h-64 flex items-center justify-center border border-dashed border-border rounded">
          <p className="text-text-dim text-sm">
            Charts will populate as claims are executed
          </p>
        </div>
      </div>

      {/* Export */}
      <div className="flex justify-end">
        <button className="px-3 py-1.5 bg-surface-raised border border-border rounded text-xs text-text-muted hover:text-text transition-colors">
          Export CSV
        </button>
      </div>
    </div>
  );
}
