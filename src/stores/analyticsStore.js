import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

const useAnalyticsStore = create((set, get) => ({
  summary: null,
  sourceAttribution: [],
  chainBreakdown: [],
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
      const [summary, sourceAttribution, chainBreakdown] = await Promise.all([
        invoke("get_analytics_summary"),
        invoke("get_source_attribution"),
        invoke("get_chain_breakdown"),
      ]);
      set({ summary, sourceAttribution, chainBreakdown, loading: false });
    } catch (e) {
      set({ error: e.toString(), loading: false });
    }
  },
}));

export default useAnalyticsStore;
