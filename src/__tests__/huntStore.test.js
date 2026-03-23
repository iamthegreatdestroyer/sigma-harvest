import { describe, it, expect, beforeEach } from "vitest";
import { mockInvokeResponse, clearMockResponses } from "./__mocks__/tauri";
import { useHuntStore } from "../stores/huntStore";

describe("useHuntStore", () => {
  const mockOpportunity = {
    source: "rss",
    chain: "ethereum",
    opportunity_type: "Airdrop",
    title: "Test Airdrop",
    description: "A test opportunity",
    url: "https://example.com",
    contract_address: null,
    estimated_value_usd: 50.0,
    gas_cost_estimate: 0.5,
    deadline: null,
    discovered_at: "2025-01-01T00:00:00Z",
  };

  const mockEvaluation = {
    id: "eval-001",
    title: "Test Airdrop",
    chain: "ethereum",
    source: "rss",
    harvest_score: { total: 0.85, components: {} },
    risk: { level: "low", score: 0.2 },
    sigma_score: 0.9,
    attractor_score: 0.8,
    wave_score: 0.7,
    consensus: { decision: "proceed", confidence: 0.9 },
    duplicate: false,
    similar_count: 0,
    proceed: true,
    status: "Qualified",
    estimated_value_usd: 50.0,
    gas_cost_estimate: 0.5,
    url: "https://example.com",
  };

  const mockHuntResult = {
    total_discovered: 5,
    qualified: 2,
    duplicates: 1,
    evaluations: [mockEvaluation],
    duration_ms: 1500,
  };

  const initialSources = {
    rss: { enabled: true, interval: 300 },
    dappradar: { enabled: true, interval: 600 },
    galxe: { enabled: true, interval: 600 },
    onchain: { enabled: false, interval: 60 },
    social: { enabled: false, interval: 900 },
  };

  const initialGasCeiling = {
    ethereum: 30,
    arbitrum: 0.5,
    optimism: 0.1,
    base: 0.1,
    polygon: 100,
    zksync: 0.5,
  };

  beforeEach(() => {
    clearMockResponses();
    useHuntStore.setState({
      running: false,
      logs: [],
      sources: JSON.parse(JSON.stringify(initialSources)),
      gasCeiling: { ...initialGasCeiling },
      opportunities: [],
      evaluations: [],
      huntResult: null,
      loading: false,
      error: null,
    });
  });

  // ── Initial state ──────────────────────────────────────

  it("has correct initial state", () => {
    const s = useHuntStore.getState();
    expect(s.running).toBe(false);
    expect(s.logs).toEqual([]);
    expect(s.sources.rss.enabled).toBe(true);
    expect(s.sources.onchain.enabled).toBe(false);
    expect(s.gasCeiling.ethereum).toBe(30);
    expect(s.opportunities).toEqual([]);
    expect(s.evaluations).toEqual([]);
    expect(s.huntResult).toBeNull();
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  // ── setRunning ─────────────────────────────────────────

  it("setRunning updates running state", () => {
    useHuntStore.getState().setRunning(true);
    expect(useHuntStore.getState().running).toBe(true);

    useHuntStore.getState().setRunning(false);
    expect(useHuntStore.getState().running).toBe(false);
  });

  // ── addLog / clearLogs ─────────────────────────────────

  it("addLog appends a log entry", () => {
    useHuntStore.getState().addLog("info", "hello");
    const logs = useHuntStore.getState().logs;
    expect(logs).toHaveLength(1);
    expect(logs[0].level).toBe("info");
    expect(logs[0].message).toBe("hello");
    expect(logs[0].timestamp).toBeDefined();
  });

  it("addLog caps at 500 entries", () => {
    const store = useHuntStore.getState();
    for (let i = 0; i < 510; i++) {
      store.addLog("info", `msg-${i}`);
    }
    const logs = useHuntStore.getState().logs;
    expect(logs).toHaveLength(500);
    expect(logs[logs.length - 1].message).toBe("msg-509");
  });

  it("clearLogs resets logs to empty", () => {
    useHuntStore.getState().addLog("info", "test");
    expect(useHuntStore.getState().logs).toHaveLength(1);

    useHuntStore.getState().clearLogs();
    expect(useHuntStore.getState().logs).toEqual([]);
  });

  // ── toggleSource ───────────────────────────────────────

  it("toggleSource flips enabled flag", () => {
    useHuntStore.getState().toggleSource("rss");
    expect(useHuntStore.getState().sources.rss.enabled).toBe(false);
    expect(useHuntStore.getState().sources.rss.interval).toBe(300);

    useHuntStore.getState().toggleSource("rss");
    expect(useHuntStore.getState().sources.rss.enabled).toBe(true);
  });

  it("toggleSource only affects the targeted source", () => {
    useHuntStore.getState().toggleSource("onchain");
    const sources = useHuntStore.getState().sources;
    expect(sources.onchain.enabled).toBe(true);
    expect(sources.rss.enabled).toBe(true);
    expect(sources.social.enabled).toBe(false);
  });

  // ── discoverOpportunities ──────────────────────────────

  it("discoverOpportunities returns opportunities on success", async () => {
    const mockOps = [mockOpportunity];
    mockInvokeResponse("discover_opportunities", mockOps);

    const result = await useHuntStore.getState().discoverOpportunities();
    const s = useHuntStore.getState();

    expect(result).toEqual(mockOps);
    expect(s.opportunities).toEqual(mockOps);
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  it("discoverOpportunities sends only enabled sources", async () => {
    let captured;
    mockInvokeResponse("discover_opportunities", (args) => {
      captured = args;
      return [];
    });

    await useHuntStore.getState().discoverOpportunities();

    expect(captured.sources).toEqual(
      expect.arrayContaining(["rss", "dappradar", "galxe"])
    );
    expect(captured.sources).toHaveLength(3);
    expect(captured.sources).not.toContain("onchain");
    expect(captured.sources).not.toContain("social");
    expect(captured.rss_feeds).toBeNull();
    expect(captured.dappradar_key).toBeNull();
  });

  it("discoverOpportunities sets error on failure", async () => {
    mockInvokeResponse("discover_opportunities", new Error("network down"));

    await expect(
      useHuntStore.getState().discoverOpportunities()
    ).rejects.toThrow();

    const s = useHuntStore.getState();
    expect(s.error).toBe("network down");
    expect(s.loading).toBe(false);
    expect(s.opportunities).toEqual([]);
  });

  // ── evaluateOpportunity ────────────────────────────────

  it("evaluateOpportunity appends evaluation on success", async () => {
    mockInvokeResponse("evaluate_full_pipeline", mockEvaluation);

    const result = await useHuntStore
      .getState()
      .evaluateOpportunity(mockOpportunity);
    const s = useHuntStore.getState();

    expect(result).toEqual(mockEvaluation);
    expect(s.evaluations).toHaveLength(1);
    expect(s.evaluations[0]).toEqual(mockEvaluation);
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  it("evaluateOpportunity accumulates evaluations", async () => {
    mockInvokeResponse("evaluate_full_pipeline", mockEvaluation);

    await useHuntStore.getState().evaluateOpportunity(mockOpportunity);
    await useHuntStore.getState().evaluateOpportunity(mockOpportunity);

    expect(useHuntStore.getState().evaluations).toHaveLength(2);
  });

  it("evaluateOpportunity sets error on failure", async () => {
    mockInvokeResponse("evaluate_full_pipeline", new Error("eval failed"));

    await expect(
      useHuntStore.getState().evaluateOpportunity(mockOpportunity)
    ).rejects.toThrow();

    const s = useHuntStore.getState();
    expect(s.error).toBe("eval failed");
    expect(s.loading).toBe(false);
    expect(s.evaluations).toEqual([]);
  });

  // ── runHuntCycle ───────────────────────────────────────

  it("runHuntCycle sets result and evaluations on success", async () => {
    mockInvokeResponse("run_hunt_cycle", mockHuntResult);

    const result = await useHuntStore.getState().runHuntCycle();
    const s = useHuntStore.getState();

    expect(result).toEqual(mockHuntResult);
    expect(s.huntResult).toEqual(mockHuntResult);
    expect(s.evaluations).toEqual(mockHuntResult.evaluations);
    expect(s.running).toBe(false);
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  it("runHuntCycle sends only enabled sources", async () => {
    let captured;
    mockInvokeResponse("run_hunt_cycle", (args) => {
      captured = args;
      return mockHuntResult;
    });

    await useHuntStore.getState().runHuntCycle();

    expect(captured.sources).toEqual(
      expect.arrayContaining(["rss", "dappradar", "galxe"])
    );
    expect(captured.sources).toHaveLength(3);
    expect(captured.rss_feeds).toBeNull();
    expect(captured.dappradar_key).toBeNull();
  });

  it("runHuntCycle clears running and loading on error", async () => {
    mockInvokeResponse("run_hunt_cycle", new Error("cycle failed"));

    await expect(
      useHuntStore.getState().runHuntCycle()
    ).rejects.toThrow();

    const s = useHuntStore.getState();
    expect(s.error).toBe("cycle failed");
    expect(s.running).toBe(false);
    expect(s.loading).toBe(false);
    expect(s.huntResult).toBeNull();
  });

  // ── clearEvaluations ───────────────────────────────────

  it("clearEvaluations resets evaluations and huntResult", () => {
    useHuntStore.setState({
      evaluations: [mockEvaluation],
      huntResult: mockHuntResult,
    });

    useHuntStore.getState().clearEvaluations();
    const s = useHuntStore.getState();

    expect(s.evaluations).toEqual([]);
    expect(s.huntResult).toBeNull();
  });
});
