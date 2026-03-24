import { useEffect } from "react";
import {
  BarChart3,
  TrendingUp,
  DollarSign,
  Fuel,
  Loader2,
  AlertCircle,
  RefreshCw,
} from "lucide-react";
import {
  PieChart,
  Pie,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  Cell,
  ResponsiveContainer,
  Legend,
} from "recharts";
import useAnalyticsStore from "../stores/analyticsStore";

const SOURCE_COLORS = [
  "#6366f1",
  "#22d3ee",
  "#f59e0b",
  "#10b981",
  "#ef4444",
  "#8b5cf6",
  "#ec4899",
];
const CHAIN_COLORS = [
  "#6366f1",
  "#22d3ee",
  "#f59e0b",
  "#10b981",
  "#ef4444",
  "#8b5cf6",
];

function fmt(v) {
  if (v == null) return "$0.00";
  return `$${Number(v).toFixed(2)}`;
}

export default function AnalyticsBay() {
  const {
    summary,
    sourceAttribution,
    chainBreakdown,
    loading,
    error,
    fetchAll,
  } = useAnalyticsStore();

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  const totalHarvested = summary?.total_value_collected_usd ?? 0;
  const gasSpent = summary?.total_gas_spent_usd ?? 0;
  const netProfit = totalHarvested - gasSpent;
  const claimCount = summary?.total_claims ?? 0;
  const successRate = claimCount > 0 ? (summary?.successful_claims ?? 0) / claimCount : 0;

  const metrics = [
    {
      label: "Total Harvested",
      value: fmt(totalHarvested),
      icon: DollarSign,
      color: "text-primary",
    },
    {
      label: "Gas Spent",
      value: fmt(gasSpent),
      icon: Fuel,
      color: "text-warning",
    },
    {
      label: "Net Profit",
      value: fmt(netProfit),
      icon: TrendingUp,
      color: netProfit >= 0 ? "text-accent" : "text-danger",
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <BarChart3 className="text-accent" size={28} />
          <div>
            <h2 className="text-xl font-bold text-text">Analytics Bay</h2>
            <p className="text-text-muted text-xs">
              Performance metrics &amp; ROI tracking
            </p>
          </div>
        </div>
        <button
          onClick={fetchAll}
          disabled={loading}
          className="p-2 rounded-lg bg-surface border border-border text-text-muted hover:text-text transition-colors disabled:opacity-50"
          title="Refresh analytics"
        >
          <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
        </button>
      </div>
      <div className="h-px bg-gradient-to-r from-accent to-transparent" />

      {/* Error Banner */}
      {error && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-danger/10 border border-danger/30 text-danger text-sm">
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {/* Loading Overlay */}
      {loading && !summary && (
        <div className="flex items-center justify-center py-16">
          <Loader2 size={32} className="animate-spin text-accent" />
          <span className="ml-3 text-text-muted text-sm">
            Loading analytics...
          </span>
        </div>
      )}

      {/* Metric Cards */}
      <div className="grid grid-cols-3 gap-4">
        {metrics.map((m) => (
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

      {/* Secondary Stats */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-surface rounded-lg border border-border p-4">
          <span className="text-text-muted text-[11px] uppercase tracking-wider">
            Total Claims
          </span>
          <div className="text-2xl font-bold text-text mt-1">{claimCount}</div>
        </div>
        <div className="bg-surface rounded-lg border border-border p-4">
          <span className="text-text-muted text-[11px] uppercase tracking-wider">
            Success Rate
          </span>
          <div className="text-2xl font-bold text-text mt-1">
            {(successRate * 100).toFixed(1)}%
          </div>
        </div>
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-2 gap-4">
        {/* Source Attribution Pie */}
        <div className="bg-surface rounded-lg border border-border p-6">
          <h3 className="text-sm font-semibold text-text-muted mb-4">
            Source Attribution
          </h3>
          {sourceAttribution.length > 0 ? (
            <ResponsiveContainer width="100%" height={240}>
              <PieChart>
                <Pie
                  data={sourceAttribution}
                  dataKey="claim_count"
                  nameKey="source"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  label={({ source, percent }) =>
                    `${source} ${(percent * 100).toFixed(0)}%`
                  }
                >
                  {sourceAttribution.map((_, i) => (
                    <Cell
                      key={i}
                      fill={SOURCE_COLORS[i % SOURCE_COLORS.length]}
                    />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    backgroundColor: "var(--color-surface)",
                    border: "1px solid var(--color-border)",
                    borderRadius: "8px",
                    color: "var(--color-text)",
                  }}
                />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-60 flex items-center justify-center border border-dashed border-border rounded">
              <p className="text-text-dim text-sm">
                No source data yet
              </p>
            </div>
          )}
        </div>

        {/* Chain Breakdown Bar */}
        <div className="bg-surface rounded-lg border border-border p-6">
          <h3 className="text-sm font-semibold text-text-muted mb-4">
            Chain Breakdown
          </h3>
          {chainBreakdown.length > 0 ? (
            <ResponsiveContainer width="100%" height={240}>
              <BarChart data={chainBreakdown}>
                <XAxis
                  dataKey="chain"
                  tick={{ fill: "var(--color-text-muted)", fontSize: 11 }}
                  axisLine={{ stroke: "var(--color-border)" }}
                />
                <YAxis
                  tick={{ fill: "var(--color-text-muted)", fontSize: 11 }}
                  axisLine={{ stroke: "var(--color-border)" }}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "var(--color-surface)",
                    border: "1px solid var(--color-border)",
                    borderRadius: "8px",
                    color: "var(--color-text)",
                  }}
                />
                <Bar dataKey="claim_count" name="Opportunities" radius={[4, 4, 0, 0]}>
                  {chainBreakdown.map((_, i) => (
                    <Cell
                      key={i}
                      fill={CHAIN_COLORS[i % CHAIN_COLORS.length]}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-60 flex items-center justify-center border border-dashed border-border rounded">
              <p className="text-text-dim text-sm">
                No chain data yet
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Export */}
      <div className="flex justify-end">
        <button
          disabled={!summary}
          className="px-3 py-1.5 bg-surface-raised border border-border rounded text-xs text-text-muted hover:text-text transition-colors disabled:opacity-50"
        >
          Export CSV
        </button>
      </div>
    </div>
  );
}
