import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

const DEFAULTS = {
  autoLockMinutes: 15,
  notificationsEnabled: true,
  notificationThreshold: 60,
  rpcOverrides: {},
  gasCeilings: {
    ethereum: 30,
    arbitrum: 0.5,
    optimism: 0.1,
    base: 0.1,
    polygon: 100,
    zksync: 0.5,
  },
  apiKeys: {
    dappradar: "",
    twitter: "",
    coingecko: "",
  },
  discoveryIntervals: {
    rss: 300,
    dappradar: 600,
    galxe: 600,
    onchain: 60,
    social: 900,
  },
};

export const useSettingsStore = create((set, get) => ({
  ...DEFAULTS,
  loading: false,
  error: null,
  dirty: false,

  /** Load all settings from the Tauri backend config table. */
  loadSettings: async () => {
    set({ loading: true, error: null });
    try {
      const pairs = await invoke("get_all_config");
      const updates = {};
      for (const [key, value] of pairs) {
        if (key === "settings_autoLockMinutes") updates.autoLockMinutes = parseInt(value, 10) || DEFAULTS.autoLockMinutes;
        else if (key === "settings_notificationsEnabled") updates.notificationsEnabled = value === "true";
        else if (key === "settings_notificationThreshold") updates.notificationThreshold = parseInt(value, 10) || DEFAULTS.notificationThreshold;
        else if (key === "settings_rpcOverrides") try { updates.rpcOverrides = JSON.parse(value); } catch { /* skip */ }
        else if (key === "settings_gasCeilings") try { updates.gasCeilings = JSON.parse(value); } catch { /* skip */ }
        else if (key === "settings_apiKeys") try { updates.apiKeys = JSON.parse(value); } catch { /* skip */ }
        else if (key === "settings_discoveryIntervals") try { updates.discoveryIntervals = JSON.parse(value); } catch { /* skip */ }
      }
      set({ ...updates, loading: false, dirty: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  /** Persist a single setting to the backend. */
  saveSetting: async (key, value) => {
    const serialized = typeof value === "object" ? JSON.stringify(value) : String(value);
    try {
      await invoke("set_config", { key: `settings_${key}`, value: serialized });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  /** Persist all current settings to the backend. */
  saveAll: async () => {
    const state = get();
    const entries = {
      autoLockMinutes: state.autoLockMinutes,
      notificationsEnabled: state.notificationsEnabled,
      notificationThreshold: state.notificationThreshold,
      rpcOverrides: state.rpcOverrides,
      gasCeilings: state.gasCeilings,
      apiKeys: state.apiKeys,
      discoveryIntervals: state.discoveryIntervals,
    };
    set({ loading: true, error: null });
    try {
      for (const [key, value] of Object.entries(entries)) {
        const serialized = typeof value === "object" ? JSON.stringify(value) : String(value);
        await invoke("set_config", { key: `settings_${key}`, value: serialized });
      }
      set({ loading: false, dirty: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  /** Export settings snapshot as a JSON string. */
  exportSettings: () => {
    const state = get();
    return JSON.stringify({
      autoLockMinutes: state.autoLockMinutes,
      notificationsEnabled: state.notificationsEnabled,
      notificationThreshold: state.notificationThreshold,
      rpcOverrides: state.rpcOverrides,
      gasCeilings: state.gasCeilings,
      apiKeys: state.apiKeys,
      discoveryIntervals: state.discoveryIntervals,
    }, null, 2);
  },

  /** Import settings from a JSON string. */
  importSettings: (jsonStr) => {
    try {
      const data = JSON.parse(jsonStr);
      const updates = {};
      if (data.autoLockMinutes != null) updates.autoLockMinutes = data.autoLockMinutes;
      if (data.notificationsEnabled != null) updates.notificationsEnabled = data.notificationsEnabled;
      if (data.notificationThreshold != null) updates.notificationThreshold = data.notificationThreshold;
      if (data.rpcOverrides) updates.rpcOverrides = data.rpcOverrides;
      if (data.gasCeilings) updates.gasCeilings = data.gasCeilings;
      if (data.apiKeys) updates.apiKeys = data.apiKeys;
      if (data.discoveryIntervals) updates.discoveryIntervals = data.discoveryIntervals;
      set({ ...updates, dirty: true });
      return true;
    } catch {
      return false;
    }
  },

  setAutoLockMinutes: (minutes) => set({ autoLockMinutes: minutes, dirty: true }),
  setNotificationsEnabled: (enabled) => set({ notificationsEnabled: enabled, dirty: true }),
  setNotificationThreshold: (threshold) => set({ notificationThreshold: threshold, dirty: true }),
  setRpcOverride: (chain, url) =>
    set((s) => ({ rpcOverrides: { ...s.rpcOverrides, [chain]: url }, dirty: true })),
  setGasCeiling: (chain, value) =>
    set((s) => ({ gasCeilings: { ...s.gasCeilings, [chain]: value }, dirty: true })),
  setApiKey: (service, key) =>
    set((s) => ({ apiKeys: { ...s.apiKeys, [service]: key }, dirty: true })),
  setDiscoveryInterval: (source, seconds) =>
    set((s) => ({ discoveryIntervals: { ...s.discoveryIntervals, [source]: seconds }, dirty: true })),
  clearError: () => set({ error: null }),
}));
