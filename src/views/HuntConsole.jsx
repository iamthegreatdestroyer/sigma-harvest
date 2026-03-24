import { Crosshair, Play, Pause, Square, Terminal, Search, Fuel, Loader2, Trash2, CheckCircle, XCircle } from "lucide-react";
import { useHuntStore } from "../stores/huntStore";
import { useCallback, useRef, useEffect } from "react";
import ScoreGauge from "../components/ScoreGauge";

const SOURCE_LABELS = {
  rss: "RSS Feeds",
  dappradar: "DappRadar",
  galxe: "Galxe",
  onchain: "On-Chain",
  social: "Social",
};

export default function HuntConsole() {
  const {
    running,
    loading,
    logs,
    sources,
    gasCeiling,
    evaluations,
    huntResult,
    error,
    addLog,
    clearLogs,
    toggleSource,
    runHuntCycle,
    discoverOpportunities,
    clearEvaluations,
  } = useHuntStore();

  const logEndRef = useRef(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const handleRun = useCallback(async () => {
    addLog("info", "Starting hunt cycle...");
    try {
      const result = await runHuntCycle();
      addLog("success", `Hunt complete: ${result.qualified} qualified of ${result.total_discovered} discovered (${result.duration_ms}ms)`);
    } catch {
      addLog("error", "Hunt cycle failed — check error panel.");
    }
  }, [runHuntCycle, addLog]);

  const handleDiscover = useCallback(async () => {
    addLog("info", "Discovering opportunities...");
    try {
      const result = await discoverOpportunities();
      addLog("success", `Discovered ${result.length} raw opportunities.`);
    } catch {
      addLog("error", "Discovery failed.");
    }
  }, [discoverOpportunities, addLog]);

  const enabledCount = Object.values(sources).filter((v) => v.enabled).length;

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <Crosshair className="text-warning" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Hunt Console</h2>
          <p className="text-text-muted text-xs">
            Control the discovery and claim pipeline
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-warning to-transparent" />

      {/* Error Banner */}
      {error && (
        <div className="flex items-center gap-2 px-3 py-2 text-xs rounded bg-red-500/10 border border-red-500/20 text-red-400">
          <span className="truncate">{error}</span>
        </div>
      )}

      {/* Controls */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleDiscover}
          disabled={loading || enabledCount === 0}
          className="flex items-center gap-2 px-4 py-2 bg-surface border border-border rounded text-text-muted text-sm hover:bg-white/10 transition-colors disabled:opacity-40"
        >
          <Search size={14} /> Discover
        </button>
        <button
          onClick={handleRun}
          disabled={loading || running || enabledCount === 0}
          className="flex items-center gap-2 px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors disabled:opacity-40"
        >
          {loading ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <Play size={14} />
          )}
          {loading ? "Running..." : "Run Cycle"}
        </button>
        <div className="ml-auto flex items-center gap-2 text-xs">
          <div
            className={`w-2 h-2 rounded-full ${running ? "bg-primary animate-pulse" : "bg-text-dim"}`}
          />
          <span className="text-text-muted">
            {running ? "Pipeline Running" : "Pipeline Idle"}
          </span>
        </div>
      </div>

      {/* Source Toggles */}
      <div className="flex flex-wrap gap-2">
        {Object.entries(SOURCE_LABELS).map(([key, label]) => (
          <button
            key={key}
            onClick={() => toggleSource(key)}
            className={`px-3 py-1.5 text-xs rounded border transition-colors ${
              sources[key]?.enabled
                ? "border-primary/40 bg-primary/10 text-primary"
                : "border-white/10 bg-white/5 text-text-dim"
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Gas Ceiling Display */}
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-[10px] text-text-muted items-center">
        <Fuel className="w-3 h-3 text-text-dim" />
        {Object.entries(gasCeiling).map(([chain, cap]) => (
          <span key={chain}>
            {chain}: <span className="text-text">{cap} gwei</span>
          </span>
        ))}
      </div>

      {/* Hunt Result Summary */}
      {huntResult && (
        <div className="grid grid-cols-4 gap-2 text-center">
          {[
            { label: "Discovered", value: huntResult.total_discovered },
            { label: "Qualified", value: huntResult.qualified },
            { label: "Duplicates", value: huntResult.duplicates },
            { label: "Duration", value: `${huntResult.duration_ms}ms` },
          ].map(({ label, value }) => (
            <div key={label} className="rounded bg-surface-raised/50 border border-border py-3">
              <div className="text-lg font-bold text-text">{value}</div>
              <div className="text-[10px] text-text-muted">{label}</div>
            </div>
          ))}
        </div>
      )}

      {/* Evaluations Table */}
      {evaluations.length > 0 && (
        <div className="bg-surface rounded-lg border border-border overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2 border-b border-border">
            <span className="text-xs font-medium text-text">
              Evaluations ({evaluations.length})
            </span>
            <button
              onClick={clearEvaluations}
              className="text-text-dim hover:text-red-400 transition-colors"
            >
              <Trash2 size={12} />
            </button>
          </div>
          <div className="max-h-72 overflow-y-auto">
            <table className="w-full text-[11px]">
              <thead className="bg-surface-raised/20 sticky top-0">
                <tr className="text-text-muted">
                  <th className="text-left px-3 py-1.5 font-medium">Title</th>
                  <th className="px-2 py-1.5 font-medium">Chain</th>
                  <th className="px-2 py-1.5 font-medium">Sigma</th>
                  <th className="px-2 py-1.5 font-medium">Harvest</th>
                  <th className="px-2 py-1.5 font-medium">Proceed</th>
                </tr>
              </thead>
              <tbody>
                {evaluations.map((ev, i) => (
                  <tr
                    key={ev.id || i}
                    className="border-t border-white/5 hover:bg-white/5 transition-colors"
                  >
                    <td className="px-3 py-1.5 text-text truncate max-w-[200px]">
                      {ev.title}
                    </td>
                    <td className="px-2 py-1.5 text-center text-text-muted">
                      {ev.chain}
                    </td>
                    <td className="px-2 py-1.5 text-center">
                      <span className="text-primary font-bold">
                        {ev.sigma_score?.toFixed(2) ?? "—"}
                      </span>
                    </td>
                    <td className="px-2 py-1.5 flex justify-center">
                      {ev.harvest_score ? (
                        <ScoreGauge score={ev.harvest_score.total_score} size="sm" />
                      ) : (
                        <span className="text-text-dim">—</span>
                      )}
                    </td>
                    <td className="px-2 py-1.5 text-center">
                      {ev.proceed ? (
                        <CheckCircle className="w-3.5 h-3.5 text-green-400 mx-auto" />
                      ) : (
                        <XCircle className="w-3.5 h-3.5 text-red-400 mx-auto" />
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Log Stream */}
      <div className="bg-surface rounded-lg border border-border">
        <div className="flex items-center justify-between px-4 py-2 border-b border-border">
          <div className="flex items-center gap-2">
            <Terminal size={14} className="text-text-muted" />
            <span className="text-xs text-text-muted uppercase tracking-wider">
              Agent Log Stream
            </span>
          </div>
          <button
            onClick={clearLogs}
            className="text-text-dim hover:text-text-muted transition-colors"
          >
            <Trash2 size={12} />
          </button>
        </div>
        <div className="h-64 overflow-y-auto p-4 font-mono text-xs space-y-1">
          {logs.length === 0 ? (
            <div className="text-text-dim italic">
              No activity yet. Start a hunt to begin.
            </div>
          ) : (
            logs.map((log, i) => (
              <div
                key={i}
                className={`${
                  log.level === "error"
                    ? "text-danger"
                    : log.level === "warn"
                      ? "text-warning"
                      : log.level === "success"
                        ? "text-primary"
                        : "text-text-muted"
                }`}
              >
                <span className="text-text-dim">[{log.timestamp}]</span>{" "}
                {log.message}
              </div>
            ))
          )}
          <div ref={logEndRef} />
        </div>
      </div>
    </div>
  );
}
