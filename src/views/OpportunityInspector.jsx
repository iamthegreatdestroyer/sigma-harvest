import { useState, useMemo } from "react";
import {
  Search,
  ExternalLink,
  AlertTriangle,
  CheckCircle,
  Shield,
  XCircle,
  ChevronRight,
  Activity,
} from "lucide-react";
import ScoreGauge from "../components/ScoreGauge";
import { useHuntStore } from "../stores/huntStore";

const RISK_COLORS = {
  Low: "text-green-400",
  Medium: "text-yellow-400",
  High: "text-orange-400",
  Critical: "text-red-400",
};

export default function OpportunityInspector() {
  const { evaluations } = useHuntStore();
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState(null);

  const filtered = useMemo(() => {
    if (!search) return evaluations;
    const q = search.toLowerCase();
    return evaluations.filter(
      (e) =>
        e.title?.toLowerCase().includes(q) ||
        e.chain?.toLowerCase().includes(q) ||
        e.source?.toLowerCase().includes(q)
    );
  }, [evaluations, search]);

  const selected = useMemo(
    () => evaluations.find((e) => e.id === selectedId) || null,
    [evaluations, selectedId]
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3 mb-2">
        <Search className="text-primary" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Opportunity Inspector</h2>
          <p className="text-text-muted text-xs">
            Deep-dive analysis of discovered opportunities
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-primary to-transparent" />

      {/* Search */}
      <div className="flex items-center gap-2">
        <div className="flex-1 relative">
          <Search
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
          />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search opportunities by title, chain, or source..."
            className="w-full pl-9 pr-4 py-2 bg-surface border border-border rounded text-sm text-text focus:outline-none focus:border-primary"
          />
        </div>
        <span className="text-xs text-text-dim whitespace-nowrap">
          {filtered.length} result{filtered.length !== 1 ? "s" : ""}
        </span>
      </div>

      <div className="grid grid-cols-3 gap-4" style={{ minHeight: 400 }}>
        {/* List panel */}
        <div className="col-span-1 bg-surface rounded-lg border border-border overflow-hidden flex flex-col">
          <div className="px-3 py-2 border-b border-border text-[10px] text-text-muted uppercase tracking-wider">
            Evaluated Opportunities
          </div>
          <div className="flex-1 overflow-y-auto">
            {filtered.length === 0 ? (
              <div className="p-6 text-center text-text-dim text-xs">
                {evaluations.length === 0
                  ? "Run a hunt cycle to populate evaluations"
                  : "No matches"}
              </div>
            ) : (
              filtered.map((ev) => (
                <button
                  key={ev.id}
                  onClick={() => setSelectedId(ev.id)}
                  className={`w-full text-left px-3 py-2.5 border-b border-white/5 hover:bg-white/5 transition-colors ${
                    selectedId === ev.id ? "bg-primary/10 border-l-2 border-l-primary" : ""
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-text truncate block max-w-[160px]">
                      {ev.title}
                    </span>
                    <ChevronRight size={12} className="text-text-dim shrink-0" />
                  </div>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-[10px] text-text-muted">{ev.chain}</span>
                    <span className="text-[10px] text-primary font-bold">
                      Σ{ev.sigma_score?.toFixed(2)}
                    </span>
                    {ev.proceed ? (
                      <CheckCircle size={10} className="text-green-400" />
                    ) : (
                      <XCircle size={10} className="text-red-400" />
                    )}
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        {/* Detail panel */}
        <div className="col-span-2 bg-surface rounded-lg border border-border overflow-y-auto">
          {!selected ? (
            <div className="flex flex-col items-center justify-center h-full p-12 text-center">
              <Search size={40} className="text-text-dim mb-4" />
              <p className="text-text-muted text-sm mb-2">
                Select an opportunity from the list
              </p>
              <p className="text-text-dim text-xs">
                The inspector shows scoring, risk breakdown, and evaluation details
              </p>
            </div>
          ) : (
            <div className="p-5 space-y-5">
              {/* Title + meta */}
              <div>
                <div className="flex items-start justify-between">
                  <h3 className="text-lg font-bold text-text">{selected.title}</h3>
                  {selected.url && (
                    <a
                      href={selected.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary hover:text-primary/80 transition-colors"
                    >
                      <ExternalLink size={16} />
                    </a>
                  )}
                </div>
                <div className="flex items-center gap-3 mt-1 text-xs text-text-muted">
                  <span>{selected.chain}</span>
                  <span>·</span>
                  <span>Source: {selected.source}</span>
                  {selected.estimated_value_usd != null && (
                    <>
                      <span>·</span>
                      <span className="text-primary">
                        ~${selected.estimated_value_usd.toFixed(2)}
                      </span>
                    </>
                  )}
                </div>
              </div>

              {/* Status badge */}
              <div className="flex items-center gap-2">
                {selected.proceed ? (
                  <span className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-green-500/10 border border-green-500/20 text-green-400">
                    <CheckCircle size={12} /> Qualified
                  </span>
                ) : (
                  <span className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-red-500/10 border border-red-500/20 text-red-400">
                    <XCircle size={12} /> Not Qualified
                  </span>
                )}
                {selected.duplicate && (
                  <span className="px-2 py-1 text-xs rounded bg-yellow-500/10 border border-yellow-500/20 text-yellow-400">
                    Duplicate
                  </span>
                )}
              </div>

              {/* Score cards */}
              <div className="grid grid-cols-4 gap-3">
                <ScoreCard
                  label="Sigma Score"
                  value={selected.sigma_score?.toFixed(2) ?? "—"}
                  color="text-primary"
                />
                <ScoreCard
                  label="Harvest"
                  value={selected.harvest_score?.total_score ?? "—"}
                  color="text-accent"
                />
                <ScoreCard
                  label="Wave"
                  value={selected.wave_score?.toFixed(2) ?? "—"}
                  color="text-warning"
                />
                <ScoreCard
                  label="Attractor"
                  value={selected.attractor_score?.toFixed(2) ?? "—"}
                  color="text-text"
                />
              </div>

              {/* Risk Assessment */}
              {selected.risk && (
                <div className="bg-surface-raised/30 rounded-lg border border-white/5 p-4 space-y-3">
                  <div className="flex items-center gap-2">
                    <Shield size={14} className="text-text-muted" />
                    <span className="text-xs font-semibold text-text">Risk Assessment</span>
                    <span
                      className={`ml-auto text-xs font-bold ${
                        RISK_COLORS[selected.risk.level] ?? "text-text-muted"
                      }`}
                    >
                      {selected.risk.level}
                    </span>
                  </div>
                  {selected.risk.flags?.length > 0 && (
                    <div className="flex flex-wrap gap-1.5">
                      {selected.risk.flags.map((flag, i) => (
                        <span
                          key={i}
                          className="flex items-center gap-1 px-2 py-0.5 text-[10px] rounded bg-red-500/10 border border-red-500/15 text-red-300"
                        >
                          <AlertTriangle size={9} />
                          {flag}
                        </span>
                      ))}
                    </div>
                  )}
                  <div className="text-[10px] text-text-dim">
                    Risk score: {selected.risk.risk_score}/100
                  </div>
                </div>
              )}

              {/* Harvest Breakdown */}
              {selected.harvest_score?.breakdown && (
                <div className="bg-surface-raised/30 rounded-lg border border-white/5 p-4 space-y-2">
                  <div className="flex items-center gap-2 mb-2">
                    <Activity size={14} className="text-text-muted" />
                    <span className="text-xs font-semibold text-text">Harvest Breakdown</span>
                  </div>
                  <div className="grid grid-cols-2 gap-x-6 gap-y-1 text-[11px]">
                    {Object.entries(selected.harvest_score.breakdown).map(
                      ([key, val]) => (
                        <div key={key} className="flex justify-between">
                          <span className="text-text-muted capitalize">
                            {key.replace(/_/g, " ")}
                          </span>
                          <span className="text-text font-medium">{val}</span>
                        </div>
                      )
                    )}
                  </div>
                </div>
              )}

              {/* Consensus */}
              {selected.consensus && (
                <div className="bg-surface-raised/30 rounded-lg border border-white/5 p-4 space-y-1">
                  <span className="text-xs font-semibold text-text">Swarm Consensus</span>
                  <div className="grid grid-cols-3 gap-3 text-[11px] mt-2">
                    <div>
                      <span className="text-text-muted">Score</span>
                      <div className="text-text font-bold">
                        {selected.consensus.score?.toFixed(2)}
                      </div>
                    </div>
                    <div>
                      <span className="text-text-muted">Confidence</span>
                      <div className="text-text font-bold">
                        {selected.consensus.confidence?.toFixed(2)}
                      </div>
                    </div>
                    <div>
                      <span className="text-text-muted">Proceed</span>
                      <div
                        className={
                          selected.consensus.proceed ? "text-green-400" : "text-red-400"
                        }
                      >
                        {selected.consensus.proceed ? "YES" : "NO"}
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ScoreCard({ label, value, color }) {
  return (
    <div className="bg-surface-raised/30 rounded border border-white/5 p-3 text-center">
      <div className={`text-xl font-bold ${color}`}>{value}</div>
      <div className="text-[10px] text-text-muted mt-0.5">{label}</div>
    </div>
  );
}
