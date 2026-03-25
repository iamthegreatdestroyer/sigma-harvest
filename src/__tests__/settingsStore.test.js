import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock @tauri-apps/api/core before importing the store
vi.mock("@tauri-apps/api/core", () => {
  const responses = new Map();
  return {
    invoke: vi.fn(async (cmd, args) => {
      if (responses.has(cmd)) {
        const resp = responses.get(cmd);
        if (resp instanceof Error) throw resp;
        if (typeof resp === "function") return resp(args);
        return resp;
      }
      throw new Error(`Unmocked: ${cmd}`);
    }),
    __mockResponse: (cmd, val) => responses.set(cmd, val),
    __clearMocks: () => responses.clear(),
  };
});

const { invoke, __mockResponse, __clearMocks } = await import("@tauri-apps/api/core");
import { useSettingsStore } from "../stores/settingsStore";

describe("useSettingsStore", () => {
  beforeEach(() => {
    __clearMocks();
    useSettingsStore.setState({
      autoLockMinutes: 15,
      notificationsEnabled: true,
      notificationThreshold: 60,
      rpcOverrides: {},
      gasCeilings: { ethereum: 30, arbitrum: 0.5, optimism: 0.1, base: 0.1, polygon: 100, zksync: 0.5 },
      apiKeys: { dappradar: "", twitter: "", coingecko: "" },
      discoveryIntervals: { rss: 300, dappradar: 600, galxe: 600, onchain: 60, social: 900 },
      loading: false,
      error: null,
      dirty: false,
    });
  });

  it("has correct defaults", () => {
    const state = useSettingsStore.getState();
    expect(state.autoLockMinutes).toBe(15);
    expect(state.notificationsEnabled).toBe(true);
    expect(state.notificationThreshold).toBe(60);
    expect(state.gasCeilings.ethereum).toBe(30);
    expect(state.dirty).toBe(false);
  });

  it("setAutoLockMinutes updates value and marks dirty", () => {
    useSettingsStore.getState().setAutoLockMinutes(30);
    const state = useSettingsStore.getState();
    expect(state.autoLockMinutes).toBe(30);
    expect(state.dirty).toBe(true);
  });

  it("setNotificationsEnabled toggles", () => {
    useSettingsStore.getState().setNotificationsEnabled(false);
    expect(useSettingsStore.getState().notificationsEnabled).toBe(false);
    expect(useSettingsStore.getState().dirty).toBe(true);
  });

  it("setGasCeiling updates specific chain", () => {
    useSettingsStore.getState().setGasCeiling("arbitrum", 1.0);
    const state = useSettingsStore.getState();
    expect(state.gasCeilings.arbitrum).toBe(1.0);
    expect(state.gasCeilings.ethereum).toBe(30); // unchanged
    expect(state.dirty).toBe(true);
  });

  it("setApiKey updates specific service", () => {
    useSettingsStore.getState().setApiKey("dappradar", "abc123");
    const state = useSettingsStore.getState();
    expect(state.apiKeys.dappradar).toBe("abc123");
    expect(state.apiKeys.twitter).toBe(""); // unchanged
  });

  it("setRpcOverride updates specific chain", () => {
    useSettingsStore.getState().setRpcOverride("ethereum", "https://my-rpc.com");
    expect(useSettingsStore.getState().rpcOverrides.ethereum).toBe("https://my-rpc.com");
  });

  it("setDiscoveryInterval updates source", () => {
    useSettingsStore.getState().setDiscoveryInterval("rss", 120);
    expect(useSettingsStore.getState().discoveryIntervals.rss).toBe(120);
  });

  it("setNotificationThreshold works", () => {
    useSettingsStore.getState().setNotificationThreshold(80);
    expect(useSettingsStore.getState().notificationThreshold).toBe(80);
  });

  // ── loadSettings ─────────────────────────────────────────

  it("loadSettings populates from backend", async () => {
    __mockResponse("get_all_config", [
      ["settings_autoLockMinutes", "30"],
      ["settings_notificationsEnabled", "false"],
      ["settings_notificationThreshold", "75"],
      ["settings_gasCeilings", JSON.stringify({ ethereum: 50 })],
      ["settings_apiKeys", JSON.stringify({ dappradar: "key123" })],
    ]);
    await useSettingsStore.getState().loadSettings();
    const state = useSettingsStore.getState();
    expect(state.autoLockMinutes).toBe(30);
    expect(state.notificationsEnabled).toBe(false);
    expect(state.notificationThreshold).toBe(75);
    expect(state.gasCeilings.ethereum).toBe(50);
    expect(state.apiKeys.dappradar).toBe("key123");
    expect(state.loading).toBe(false);
    expect(state.dirty).toBe(false);
  });

  it("loadSettings handles empty config", async () => {
    __mockResponse("get_all_config", []);
    await useSettingsStore.getState().loadSettings();
    const state = useSettingsStore.getState();
    expect(state.autoLockMinutes).toBe(15); // default
    expect(state.loading).toBe(false);
  });

  it("loadSettings handles error", async () => {
    __mockResponse("get_all_config", new Error("DB error"));
    await useSettingsStore.getState().loadSettings();
    const state = useSettingsStore.getState();
    expect(state.error).toContain("DB error");
    expect(state.loading).toBe(false);
  });

  // ── saveAll ──────────────────────────────────────────────

  it("saveAll calls set_config for each setting", async () => {
    __mockResponse("set_config", undefined);
    useSettingsStore.getState().setAutoLockMinutes(5);
    await useSettingsStore.getState().saveAll();
    expect(invoke).toHaveBeenCalledWith("set_config", expect.objectContaining({
      key: "settings_autoLockMinutes",
      value: "5",
    }));
    expect(useSettingsStore.getState().dirty).toBe(false);
  });

  // ── export / import ──────────────────────────────────────

  it("exportSettings returns JSON string", () => {
    const json = useSettingsStore.getState().exportSettings();
    const parsed = JSON.parse(json);
    expect(parsed.autoLockMinutes).toBe(15);
    expect(parsed.gasCeilings).toBeDefined();
    expect(parsed.apiKeys).toBeDefined();
  });

  it("importSettings updates state from JSON", () => {
    const json = JSON.stringify({
      autoLockMinutes: 60,
      notificationsEnabled: false,
      gasCeilings: { ethereum: 100 },
    });
    const ok = useSettingsStore.getState().importSettings(json);
    expect(ok).toBe(true);
    const state = useSettingsStore.getState();
    expect(state.autoLockMinutes).toBe(60);
    expect(state.notificationsEnabled).toBe(false);
    expect(state.gasCeilings.ethereum).toBe(100);
    expect(state.dirty).toBe(true);
  });

  it("importSettings rejects invalid JSON", () => {
    const ok = useSettingsStore.getState().importSettings("not json");
    expect(ok).toBe(false);
  });

  it("clearError clears error", () => {
    useSettingsStore.setState({ error: "something" });
    useSettingsStore.getState().clearError();
    expect(useSettingsStore.getState().error).toBeNull();
  });
});
