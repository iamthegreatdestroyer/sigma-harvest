import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

const useAnalyticsStore = create((set, get) => ({
  summary: null,
  sourceAttribution: [],
  chainBreakdown: [],
  timeSeries: [],
  loading: false,
  error: null,

  fetchSummary: async () => {
    set({ loading: true, error: null });
    try {
      const summary = await invoke("get_analytics_summary");
      set({ summary, loading: false });
    } catch (e) {
      set({ error: e.toString(), loading: false });
    }
  },

  fetchSourceAttribution: async () => {
    set({ loading: true, error: null });
    try {
      const sourceAttribution = await invoke("get_source_attribution");
      set({ sourceAttribution, loading: false });
    } catch (e) {
      set({ error: e.toString(), loading: false });
    }
  },

  fetchChainBreakdown: async () => {
    set({ loading: true, error: null });
    try {
      const chainBreakdown = await invoke("get_chain_breakdown");
      set({ chainBreakdown, loading: false });
    } catch (e) {
      set({ error: e.toString(), loading: false });
    }
  },

  fetchAll: async () => {
    set({ loading: true, error: null });
    try {
      const [summary, sourceAttribution, chainBreakdown, timeSeries] = await Promise.all([
        invoke("get_analytics_summary"),
        invoke("get_source_attribution"),
        invoke("get_chain_breakdown"),
        invoke("get_time_series", { days: 30 }),
      ]);
      set({ summary, sourceAttribution, chainBreakdown, timeSeries, loading: false });
    } catch (e) {
      set({ error: e.toString(), loading: false });
    }
  },

  fetchTimeSeries: async (days = 30) => {
    try {
      const timeSeries = await invoke("get_time_series", { days });
      set({ timeSeries });
    } catch (e) {
      set({ error: e.toString() });
    }
  },
}));

export default useAnalyticsStore;
