import { create } from "zustand";

export const useHuntStore = create((set) => ({
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
}));
