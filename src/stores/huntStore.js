import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { notifyHighScore, notifyClaim } from "../lib/notifications";
import { useSettingsStore } from "./settingsStore";

/** Read the DappRadar API key from settings, return null if empty. */
function getDappRadarKey() {
  const key = useSettingsStore.getState().apiKeys?.dappradar;
  return key || null;
}

export const useHuntStore = create((set, get) => ({
  running: false,
  logs: [],
  sources: {
    rss: { enabled: true, interval: 300 },
    dappradar: { enabled: true, interval: 600 },
    galxe: { enabled: true, interval: 600 },
    onchain: { enabled: false, interval: 60 },
    social: { enabled: false, interval: 900 },
  },
  gasCeiling: { ethereum: 30, arbitrum: 0.5, optimism: 0.1, base: 0.1, polygon: 100, zksync: 0.5 },
  opportunities: [],
  evaluations: [],
  huntResult: null,
  loading: false,
  error: null,

  setRunning: (running) => set({ running }),
  addLog: (level, message) =>
    set((state) => ({
      logs: [
        ...state.logs.slice(-499),
        { level, message, timestamp: new Date().toLocaleTimeString() },
      ],
    })),
  clearLogs: () => set({ logs: [] }),
  toggleSource: (source) =>
    set((state) => ({
      sources: {
        ...state.sources,
        [source]: { ...state.sources[source], enabled: !state.sources[source].enabled },
      },
    })),

  discoverOpportunities: async () => {
    set({ loading: true, error: null });
    try {
      const { sources } = get();
      const enabledSources = Object.entries(sources)
        .filter(([, v]) => v.enabled)
        .map(([k]) => k);
      const result = await invoke("discover_opportunities", {
        sources: enabledSources,
        rss_feeds: null,
        dappradar_key: getDappRadarKey(),
      });
      set({ opportunities: result, loading: false });
      // Notify for high-score opportunities
      for (const opp of result) {
        notifyHighScore(opp, 70, true);
      }
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ loading: false, error: msg });
      throw e;
    }
  },

  evaluateOpportunity: async (opportunity) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke("evaluate_full_pipeline", { opportunity });
      set((state) => ({
        evaluations: [...state.evaluations, result],
        loading: false,
      }));
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ loading: false, error: msg });
      throw e;
    }
  },

  runHuntCycle: async () => {
    set({ loading: true, error: null, running: true });
    try {
      const { sources } = get();
      const enabledSources = Object.entries(sources)
        .filter(([, v]) => v.enabled)
        .map(([k]) => k);
      const result = await invoke("run_hunt_cycle", {
        sources: enabledSources,
        rss_feeds: null,
        dappradar_key: getDappRadarKey(),
      });
      set({ huntResult: result, evaluations: result.evaluations, loading: false, running: false });
      // Notify for high-score evaluated opportunities
      for (const ev of result.evaluations ?? []) {
        notifyHighScore(ev, 70, true);
      }
      // Notify claim results if present
      for (const claim of result.claims ?? []) {
        const status = claim.status === "Confirmed" ? "success" : "failure";
        const detail = status === "success"
          ? `$${claim.value_received_usd?.toFixed(2) ?? "0.00"}`
          : claim.error ?? "Unknown error";
        notifyClaim(status, claim.title ?? "Claim", detail, true);
      }
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ loading: false, error: msg, running: false });
      throw e;
    }
  },

  setGasCeiling: (chain, value) =>
    set((state) => ({
      gasCeiling: { ...state.gasCeiling, [chain]: value },
    })),

  clearEvaluations: () => set({ evaluations: [], huntResult: null }),
}));
