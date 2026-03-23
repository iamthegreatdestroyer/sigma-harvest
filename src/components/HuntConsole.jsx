import { useCallback, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Play, Square, Loader2, AlertTriangle, CheckCircle2,
  XCircle, Radio, Fuel, Trash2, Search,
} from 'lucide-react';
import { useHuntStore } from '../stores/huntStore';
import ScoreGauge from './ScoreGauge';

const SOURCE_LABELS = {
  rss: 'RSS Feeds',
  dappradar: 'DappRadar',
  galxe: 'Galxe',
  onchain: 'On-Chain',
  social: 'Social',
};

/**
 * HuntConsole — primary control panel for the ΣHARVEST discovery engine.
 * Start/stop hunts, toggle sources, view evaluation results and logs.
 */
export default function HuntConsole() {
  const running = useHuntStore(s => s.running);
  const loading = useHuntStore(s => s.loading);
  const error = useHuntStore(s => s.error);
  const logs = useHuntStore(s => s.logs);
  const sources = useHuntStore(s => s.sources);
  const opportunities = useHuntStore(s => s.opportunities);
  const evaluations = useHuntStore(s => s.evaluations);
  const huntResult = useHuntStore(s => s.huntResult);
  const gasCeiling = useHuntStore(s => s.gasCeiling);

  const toggleSource = useHuntStore(s => s.toggleSource);
  const addLog = useHuntStore(s => s.addLog);
  const clearLogs = useHuntStore(s => s.clearLogs);
  const clearEvaluations = useHuntStore(s => s.clearEvaluations);
  const runHuntCycle = useHuntStore(s => s.runHuntCycle);
  const discoverOpportunities = useHuntStore(s => s.discoverOpportunities);

  const logEndRef = useRef(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const handleRun = useCallback(async () => {
    addLog('Starting hunt cycle…');
    try {
      await runHuntCycle();
      addLog('Hunt cycle complete.');
    } catch {
      addLog('Hunt cycle failed — check error panel.');
    }
  }, [runHuntCycle, addLog]);

  const handleDiscover = useCallback(async () => {
    addLog('Discovering opportunities…');
    try {
      await discoverOpportunities();
      addLog(`Discovered ${useHuntStore.getState().opportunities.length} opportunities.`);
    } catch {
      addLog('Discovery failed.');
    }
  }, [discoverOpportunities, addLog]);

  const enabledCount = Object.values(sources).filter(Boolean).length;

  return (
    <div className="flex flex-col gap-4">
      {/* ── Header & Controls ── */}
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold text-text flex items-center gap-2">
          <Radio className="w-4 h-4 text-primary" />
          Hunt Console
        </h2>

        <div className="flex items-center gap-2">
          <button
            onClick={handleDiscover}
            disabled={loading || enabledCount === 0}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-surface-raised hover:bg-white/10 text-text-muted disabled:opacity-40 transition-colors"
          >
            <Search className="w-3 h-3" />
            Discover
          </button>

          <button
            onClick={handleRun}
            disabled={loading || running || enabledCount === 0}
            className="flex items-center gap-1 px-3 py-1 text-xs font-medium rounded bg-primary/20 text-primary hover:bg-primary/30 disabled:opacity-40 transition-colors"
          >
            {loading ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : running ? (
              <Square className="w-3 h-3" />
            ) : (
              <Play className="w-3 h-3" />
            )}
            {loading ? 'Running…' : running ? 'Running' : 'Run Cycle'}
          </button>
        </div>
      </div>

      {/* ── Error Banner ── */}
      <AnimatePresence>
        {error && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="flex items-center gap-2 px-3 py-2 text-xs rounded bg-red-500/10 border border-red-500/20 text-red-400"
          >
            <AlertTriangle className="w-3 h-3 shrink-0" />
            <span className="truncate">{error}</span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* ── Source Toggles ── */}
      <div className="flex flex-wrap gap-2">
        {Object.entries(SOURCE_LABELS).map(([key, label]) => (
          <button
            key={key}
            onClick={() => toggleSource(key)}
            className={`px-2 py-1 text-[10px] rounded border transition-colors ${
              sources[key]
                ? 'border-primary/40 bg-primary/10 text-primary'
                : 'border-white/10 bg-white/5 text-text-dim'
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* ── Gas Ceiling Display ── */}
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-[10px] text-text-muted">
        <Fuel className="w-3 h-3 text-text-dim" />
        {Object.entries(gasCeiling).map(([chain, cap]) => (
          <span key={chain}>
            {chain}: <span className="text-text">{cap}</span>
          </span>
        ))}
      </div>

      {/* ── Hunt Result Summary ── */}
      {huntResult && (
        <div className="grid grid-cols-4 gap-2 text-center">
          {[
            { label: 'Discovered', value: huntResult.total_discovered },
            { label: 'Qualified', value: huntResult.qualified },
            { label: 'Duplicates', value: huntResult.duplicates },
            { label: 'Duration', value: `${huntResult.duration_ms}ms` },
          ].map(({ label, value }) => (
            <div key={label} className="rounded bg-surface-raised/50 py-2">
              <div className="text-xs font-bold text-text">{value}</div>
              <div className="text-[10px] text-text-muted">{label}</div>
            </div>
          ))}
        </div>
      )}

      {/* ── Evaluations Table ── */}
      {evaluations.length > 0 && (
        <div className="rounded border border-white/10 overflow-hidden">
          <div className="flex items-center justify-between px-3 py-2 bg-surface-raised/30">
            <span className="text-xs font-medium text-text">
              Evaluations ({evaluations.length})
            </span>
            <button
              onClick={clearEvaluations}
              className="text-text-dim hover:text-red-400 transition-colors"
            >
              <Trash2 className="w-3 h-3" />
            </button>
          </div>

          <div className="max-h-60 overflow-y-auto">
            <table className="w-full text-[11px]">
              <thead className="bg-surface-raised/20 sticky top-0">
                <tr className="text-text-muted">
                  <th className="text-left px-3 py-1 font-medium">Title</th>
                  <th className="px-2 py-1 font-medium">Chain</th>
                  <th className="px-2 py-1 font-medium">Σ Score</th>
                  <th className="px-2 py-1 font-medium">Harvest</th>
                  <th className="px-2 py-1 font-medium">Proceed</th>
                </tr>
              </thead>
              <tbody>
                {evaluations.map((ev, i) => (
                  <tr
                    key={ev.id || i}
                    className="border-t border-white/5 hover:bg-white/5 transition-colors"
                  >
                    <td className="px-3 py-1.5 text-text truncate max-w-[180px]">
                      {ev.title}
                    </td>
                    <td className="px-2 py-1.5 text-center text-text-muted">
                      {ev.chain}
                    </td>
                    <td className="px-2 py-1.5 text-center">
                      <span className="text-primary font-bold">
                        {ev.sigma_score?.toFixed(2) ?? '—'}
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
                        <CheckCircle2 className="w-3.5 h-3.5 text-green-400 mx-auto" />
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

      {/* ── Log Panel ── */}
      <div className="rounded border border-white/10 overflow-hidden">
        <div className="flex items-center justify-between px-3 py-1.5 bg-surface-raised/30">
          <span className="text-[10px] font-medium text-text-muted">Logs</span>
          <button
            onClick={clearLogs}
            className="text-text-dim hover:text-text-muted transition-colors"
          >
            <Trash2 className="w-3 h-3" />
          </button>
        </div>
        <div className="max-h-32 overflow-y-auto px-3 py-2 font-mono text-[10px] text-text-dim space-y-0.5">
          {logs.length === 0 ? (
            <span className="text-text-dim/50">Waiting for activity…</span>
          ) : (
            logs.map((msg, i) => <div key={i}>{msg}</div>)
          )}
          <div ref={logEndRef} />
        </div>
      </div>
    </div>
  );
}
