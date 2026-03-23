import { useEffect, useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Brain, Cpu, Database, Activity, Zap, Users, FlaskConical, RefreshCw, Sparkles } from 'lucide-react';
import useSigmaStore from '../stores/sigmaStore';

/**
 * ΣCORE Nervous System interactive dashboard widget.
 * Displays live metrics and exposes controls for the swarm,
 * dynamics engine, memory, and pipeline evaluation.
 */
export default function SigmaCoreWidget() {
  const {
    status, swarmSummary, lastEvaluation, loading,
    fetchStatus, fetchSwarmSummary,
    evolveSwarm, setMutationRate, toggleDynamics,
    mutationRate,
  } = useSigmaStore();

  const [localMutation, setLocalMutation] = useState(mutationRate);
  const [evolving, setEvolving] = useState(false);

  useEffect(() => {
    fetchStatus();
    fetchSwarmSummary();
    const interval = setInterval(() => {
      fetchStatus();
      fetchSwarmSummary();
    }, 10000);
    return () => clearInterval(interval);
  }, [fetchStatus, fetchSwarmSummary]);

  const getAttractorPercent = useSigmaStore(s => s.getAttractorPercent);
  const getCompressionPercent = useSigmaStore(s => s.getCompressionPercent);
  const getMemoryDisplay = useSigmaStore(s => s.getMemoryDisplay);

  const handleEvolve = useCallback(async () => {
    setEvolving(true);
    await evolveSwarm();
    setEvolving(false);
  }, [evolveSwarm]);

  const handleMutationCommit = useCallback(async () => {
    await setMutationRate(localMutation);
  }, [localMutation, setMutationRate]);

  const handleDynamicsToggle = useCallback(async () => {
    if (status) {
      await toggleDynamics(!status.dynamics_enabled);
    }
  }, [status, toggleDynamics]);

  if (loading && !status) {
    return (
      <div className="rounded-xl border border-white/10 bg-white/5 p-4">
        <div className="flex items-center gap-2 text-white/40">
          <Brain className="w-4 h-4 animate-pulse" />
          <span className="text-sm">Initializing ΣCORE...</span>
        </div>
      </div>
    );
  }

  const metrics = [
    {
      label: 'Memory',
      value: status ? `${status.memory_entries}` : '0',
      sub: getMemoryDisplay(),
      icon: Database,
      color: 'text-indigo-400',
    },
    {
      label: 'Agents',
      value: status ? `${status.active_agents}` : '0',
      sub: swarmSummary ? `Gen ${swarmSummary.avg_generation.toFixed(1)}` : 'Gen 0',
      icon: Users,
      color: 'text-emerald-400',
    },
    {
      label: 'Attractor',
      value: getAttractorPercent(),
      sub: 'Strength',
      icon: Zap,
      color: 'text-amber-400',
    },
    {
      label: 'Compression',
      value: getCompressionPercent(),
      sub: 'Ratio',
      icon: Cpu,
      color: 'text-cyan-400',
    },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="rounded-xl border border-white/10 bg-white/5 p-4"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Brain className="w-4 h-4 text-indigo-400" />
          <span className="text-sm font-medium text-white/80">ΣCORE</span>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={handleDynamicsToggle}
            className="flex items-center gap-1 text-xs transition-colors hover:text-white/80"
            title={status?.dynamics_enabled ? 'Dynamics ON — click to disable' : 'Dynamics OFF — click to enable'}
          >
            <Activity className={`w-3 h-3 ${status?.dynamics_enabled ? 'text-emerald-400' : 'text-white/30'}`} />
            <span className={status?.dynamics_enabled ? 'text-emerald-400' : 'text-white/30'}>
              {status?.dynamics_enabled ? 'Live' : 'Off'}
            </span>
          </button>
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-2 gap-3">
        {metrics.map((m) => (
          <div key={m.label} className="flex items-start gap-2">
            <m.icon className={`w-3.5 h-3.5 mt-0.5 ${m.color}`} />
            <div>
              <div className="text-sm font-mono text-white/90">{m.value}</div>
              <div className="text-xs text-white/40">{m.sub}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Swarm Success Rate Bar */}
      {swarmSummary && swarmSummary.total_successes + swarmSummary.total_failures > 0 && (
        <div className="mt-3 pt-3 border-t border-white/5">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-white/40">Swarm Success</span>
            <span className="text-xs font-mono text-white/60">
              {(swarmSummary.overall_success_rate * 100).toFixed(0)}%
            </span>
          </div>
          <div className="h-1 rounded-full bg-white/10 overflow-hidden">
            <motion.div
              className="h-full rounded-full bg-emerald-500"
              initial={{ width: 0 }}
              animate={{ width: `${swarmSummary.overall_success_rate * 100}%` }}
              transition={{ duration: 0.6 }}
            />
          </div>
        </div>
      )}

      {/* Controls */}
      <div className="mt-3 pt-3 border-t border-white/5 space-y-2">
        {/* Mutation Rate Slider */}
        <div className="flex items-center gap-2">
          <FlaskConical className="w-3 h-3 text-purple-400 shrink-0" />
          <input
            type="range"
            min="0"
            max="100"
            value={Math.round(localMutation * 100)}
            onChange={e => setLocalMutation(e.target.value / 100)}
            onMouseUp={handleMutationCommit}
            onTouchEnd={handleMutationCommit}
            className="flex-1 h-1 accent-purple-500 bg-white/10 rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5
                       [&::-webkit-slider-thumb]:h-2.5 [&::-webkit-slider-thumb]:rounded-full
                       [&::-webkit-slider-thumb]:bg-purple-400"
            title="Mutation rate"
          />
          <span className="text-xs font-mono text-white/50 w-8 text-right">
            {(localMutation * 100).toFixed(0)}%
          </span>
        </div>

        {/* Evolve Button */}
        <button
          onClick={handleEvolve}
          disabled={evolving}
          className="w-full flex items-center justify-center gap-1.5 py-1 px-2 rounded-lg
                     bg-white/5 border border-white/10 text-xs text-white/60
                     hover:bg-white/10 hover:text-white/80 transition-all
                     disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {evolving ? (
            <RefreshCw className="w-3 h-3 animate-spin" />
          ) : (
            <Sparkles className="w-3 h-3" />
          )}
          {evolving ? 'Evolving...' : 'Evolve Swarm'}
        </button>
      </div>

      {/* Last Evaluation */}
      <AnimatePresence>
        {lastEvaluation && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="mt-3 pt-3 border-t border-white/5"
          >
            <div className="flex items-center justify-between">
              <span className="text-xs text-white/40">Last Σ-Score</span>
              <span className={`text-sm font-mono ${
                lastEvaluation.sigma_score >= 0.7 ? 'text-emerald-400' :
                lastEvaluation.sigma_score >= 0.5 ? 'text-amber-400' : 'text-red-400'
              }`}>
                {lastEvaluation.sigma_score.toFixed(3)}
              </span>
            </div>
            <div className="flex items-center gap-2 mt-1">
              <span className={`text-[10px] px-1.5 py-0.5 rounded ${
                lastEvaluation.proceed ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'
              }`}>
                {lastEvaluation.proceed ? 'PROCEED' : 'HOLD'}
              </span>
              {lastEvaluation.duplicate && (
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-400">
                  DUPLICATE
                </span>
              )}
              <span className="text-[10px] text-white/30 ml-auto">
                {lastEvaluation.similar_count} similar
              </span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
