import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

const useSigmaStore = create((set, get) => ({
  // ── State ────────────────────────────────────────────────
  status: null,
  swarmSummary: null,
  lastEvaluation: null,
  memoryLabels: [],
  queryResults: [],
  hurstResult: null,
  mutationRate: 0.1,
  loading: false,
  error: null,

  // ── Read-Only Actions ────────────────────────────────────

  /** Fetch ΣCORE nervous system status. */
  fetchStatus: async () => {
    set({ loading: true, error: null });
    try {
      const status = await invoke('get_sigma_status');
      set({ status, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  /** Fetch swarm performance summary. */
  fetchSwarmSummary: async () => {
    set({ loading: true, error: null });
    try {
      const swarmSummary = await invoke('get_swarm_summary');
      set({ swarmSummary, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  /** Calculate wave score for an opportunity. */
  getWaveScore: async (gasPressure, communitySignal, deadlineUrgency, valueEstimate) => {
    try {
      return await invoke('sigma_wave_score', {
        gasPressure, communitySignal, deadlineUrgency, valueEstimate,
      });
    } catch (err) {
      set({ error: String(err) });
      return 0.0;
    }
  },

  /** Fetch memory entry labels. */
  fetchMemoryLabels: async () => {
    try {
      const memoryLabels = await invoke('sigma_memory_labels');
      set({ memoryLabels });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  // ── Mutation Actions ─────────────────────────────────────

  /** Run the full ΣCORE pipeline: encode → dedup → vote → wave → score. */
  evaluateOpportunity: async ({
    chain, opportunityType, riskLevel, label, tags = [],
    gasPressure = 0.5, communitySignal = 0.5,
    deadlineUrgency = 0.5, valueEstimate = 50,
  }) => {
    set({ loading: true, error: null });
    try {
      const evaluation = await invoke('sigma_evaluate_opportunity', {
        chain, opportunityType, riskLevel, label, tags,
        gasPressure, communitySignal, deadlineUrgency, valueEstimate,
      });
      set({ lastEvaluation: evaluation, loading: false });
      return evaluation;
    } catch (err) {
      set({ error: String(err), loading: false });
      return null;
    }
  },

  /** Record outcome feedback (success/failure) for learning. */
  recordOutcome: async ({ label, chain, opportunityType, riskLevel, success }) => {
    try {
      const swarmSummary = await invoke('sigma_record_outcome', {
        label, chain, opportunityType, riskLevel, success,
      });
      set({ swarmSummary });
      return swarmSummary;
    } catch (err) {
      set({ error: String(err) });
      return null;
    }
  },

  /** Query memory for similar opportunities. */
  queryMemory: async ({ chain, opportunityType, riskLevel, k = 5 }) => {
    try {
      const queryResults = await invoke('sigma_memory_query', {
        chain, opportunityType, riskLevel, k,
      });
      set({ queryResults });
      return queryResults;
    } catch (err) {
      set({ error: String(err) });
      return [];
    }
  },

  /** Reinforce a memory entry (strengthen attractor pull). */
  reinforceMemory: async (label) => {
    try {
      await invoke('sigma_memory_reinforce', { label });
      get().fetchStatus();
    } catch (err) {
      set({ error: String(err) });
    }
  },

  /** Evict stale memory entries. */
  evictStaleMemory: async (maxAgeSecs = 86400, minReinforcement = 1) => {
    try {
      const evicted = await invoke('sigma_memory_evict', { maxAgeSecs, minReinforcement });
      get().fetchStatus();
      get().fetchMemoryLabels();
      return evicted;
    } catch (err) {
      set({ error: String(err) });
      return 0;
    }
  },

  /** Run swarm consensus vote on an opportunity. */
  swarmVote: async ({ chain, opportunityType, riskLevel }) => {
    try {
      return await invoke('sigma_swarm_vote', { chain, opportunityType, riskLevel });
    } catch (err) {
      set({ error: String(err) });
      return null;
    }
  },

  /** Trigger evolutionary step on the swarm. */
  evolveSwarm: async () => {
    try {
      const swarmSummary = await invoke('sigma_swarm_evolve');
      set({ swarmSummary });
      return swarmSummary;
    } catch (err) {
      set({ error: String(err) });
      return null;
    }
  },

  /** Set swarm mutation rate (0.0 to 1.0). */
  setMutationRate: async (rate) => {
    try {
      const clamped = await invoke('sigma_swarm_set_mutation_rate', { rate });
      set({ mutationRate: clamped });
      return clamped;
    } catch (err) {
      set({ error: String(err) });
      return get().mutationRate;
    }
  },

  /** Push a log entry to compression pipeline. */
  pushLog: async (source, level, message) => {
    try {
      await invoke('sigma_compression_push', { source, level, message });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  /** Search compressed logs. */
  searchLogs: async (query, topK = 5) => {
    try {
      return await invoke('sigma_compression_search', { query, topK });
    } catch (err) {
      set({ error: String(err) });
      return [];
    }
  },

  /** Run Hurst exponent analysis on a time series. */
  analyzeHurst: async (series) => {
    try {
      const hurstResult = await invoke('sigma_hurst_analysis', { series });
      set({ hurstResult });
      return hurstResult;
    } catch (err) {
      set({ error: String(err) });
      return null;
    }
  },

  /** Toggle dynamics engine on/off. */
  toggleDynamics: async (enabled) => {
    try {
      await invoke('sigma_toggle_dynamics', { enabled });
      get().fetchStatus();
    } catch (err) {
      set({ error: String(err) });
    }
  },

  // ── Computed Helpers ─────────────────────────────────────

  /** Get attractor strength as a percentage string. */
  getAttractorPercent: () => {
    const s = get().status;
    if (!s) return '0%';
    return `${(s.attractor_strength * 100).toFixed(1)}%`;
  },

  /** Get compression ratio as a percentage string. */
  getCompressionPercent: () => {
    const s = get().status;
    if (!s) return '100%';
    return `${(s.compression_ratio * 100).toFixed(1)}%`;
  },

  /** Get formatted memory usage. */
  getMemoryDisplay: () => {
    const s = get().status;
    if (!s) return '0 B';
    const bytes = s.memory_bytes;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  },

  /** Get ΣSCORE as a color-coded class name. */
  getSigmaScoreColor: () => {
    const e = get().lastEvaluation;
    if (!e) return 'text-white/40';
    if (e.sigma_score >= 0.7) return 'text-emerald-400';
    if (e.sigma_score >= 0.5) return 'text-amber-400';
    return 'text-red-400';
  },
}));

export default useSigmaStore;
