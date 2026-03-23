import { describe, it, expect, beforeEach, vi } from "vitest";
import { mockInvokeResponse, clearMockResponses } from "./__mocks__/tauri";
import useSigmaStore from "../stores/sigmaStore";

describe("useSigmaStore", () => {
  const mockStatus = {
    memory_entries: 42,
    memory_bytes: 1344,
    active_agents: 8,
    attractor_strength: 0.75,
    compression_ratio: 0.35,
    dynamics_enabled: true,
  };

  const mockSwarmSummary = {
    total_agents: 8,
    active_agents: 8,
    total_successes: 15,
    total_failures: 5,
    overall_success_rate: 0.75,
    avg_generation: 1.5,
  };

  beforeEach(() => {
    clearMockResponses();
    useSigmaStore.setState({
      status: null,
      swarmSummary: null,
      lastEvaluation: null,
      memoryLabels: [],
      queryResults: [],
      hurstResult: null,
      mutationRate: 0.1,
      loading: false,
      error: null,
    });
  });

  // ── Initial state ──────────────────────────────────────

  it("has correct initial state", () => {
    const state = useSigmaStore.getState();
    expect(state.status).toBeNull();
    expect(state.swarmSummary).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  // ── fetchStatus ────────────────────────────────────────

  it("fetchStatus sets status on success", async () => {
    mockInvokeResponse("get_sigma_status", mockStatus);
    await useSigmaStore.getState().fetchStatus();

    const state = useSigmaStore.getState();
    expect(state.status).toEqual(mockStatus);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("fetchStatus sets error on failure", async () => {
    mockInvokeResponse("get_sigma_status", new Error("ΣCORE offline"));
    await useSigmaStore.getState().fetchStatus();

    const state = useSigmaStore.getState();
    expect(state.status).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.error).toContain("ΣCORE offline");
  });

  // ── fetchSwarmSummary ──────────────────────────────────

  it("fetchSwarmSummary sets summary on success", async () => {
    mockInvokeResponse("get_swarm_summary", mockSwarmSummary);
    await useSigmaStore.getState().fetchSwarmSummary();

    const state = useSigmaStore.getState();
    expect(state.swarmSummary).toEqual(mockSwarmSummary);
    expect(state.loading).toBe(false);
  });

  it("fetchSwarmSummary sets error on failure", async () => {
    mockInvokeResponse("get_swarm_summary", new Error("swarm error"));
    await useSigmaStore.getState().fetchSwarmSummary();

    const state = useSigmaStore.getState();
    expect(state.swarmSummary).toBeNull();
    expect(state.error).toContain("swarm error");
  });

  // ── getWaveScore ───────────────────────────────────────

  it("getWaveScore returns score from backend", async () => {
    mockInvokeResponse("sigma_wave_score", 0.82);
    const score = await useSigmaStore.getState().getWaveScore(0.1, 0.8, 0.9, 0.5);
    expect(score).toBe(0.82);
  });

  it("getWaveScore returns 0.0 on error", async () => {
    mockInvokeResponse("sigma_wave_score", new Error("fail"));
    const score = await useSigmaStore.getState().getWaveScore(0.1, 0.8, 0.9, 0.5);
    expect(score).toBe(0.0);
  });

  // ── Computed helpers ───────────────────────────────────

  it("getAttractorPercent returns 0% when no status", () => {
    const result = useSigmaStore.getState().getAttractorPercent();
    expect(result).toBe("0%");
  });

  it("getAttractorPercent formats correctly", () => {
    useSigmaStore.setState({ status: mockStatus });
    const result = useSigmaStore.getState().getAttractorPercent();
    expect(result).toBe("75.0%");
  });

  it("getCompressionPercent returns 100% when no status", () => {
    const result = useSigmaStore.getState().getCompressionPercent();
    expect(result).toBe("100%");
  });

  it("getCompressionPercent formats correctly", () => {
    useSigmaStore.setState({ status: mockStatus });
    const result = useSigmaStore.getState().getCompressionPercent();
    expect(result).toBe("35.0%");
  });

  it("getMemoryDisplay shows bytes", () => {
    useSigmaStore.setState({ status: { ...mockStatus, memory_bytes: 512 } });
    expect(useSigmaStore.getState().getMemoryDisplay()).toBe("512 B");
  });

  it("getMemoryDisplay shows KB", () => {
    useSigmaStore.setState({ status: { ...mockStatus, memory_bytes: 2048 } });
    expect(useSigmaStore.getState().getMemoryDisplay()).toBe("2.0 KB");
  });

  it("getMemoryDisplay shows MB", () => {
    useSigmaStore.setState({ status: { ...mockStatus, memory_bytes: 1048576 } });
    expect(useSigmaStore.getState().getMemoryDisplay()).toBe("1.0 MB");
  });

  it("getMemoryDisplay returns 0 B when no status", () => {
    expect(useSigmaStore.getState().getMemoryDisplay()).toBe("0 B");
  });

  // ── Status field access ────────────────────────────────

  it("exposes all status fields correctly", async () => {
    mockInvokeResponse("get_sigma_status", mockStatus);
    await useSigmaStore.getState().fetchStatus();

    const { status } = useSigmaStore.getState();
    expect(status.memory_entries).toBe(42);
    expect(status.memory_bytes).toBe(1344);
    expect(status.active_agents).toBe(8);
    expect(status.attractor_strength).toBe(0.75);
    expect(status.compression_ratio).toBe(0.35);
    expect(status.dynamics_enabled).toBe(true);
  });

  it("exposes all swarm fields correctly", async () => {
    mockInvokeResponse("get_swarm_summary", mockSwarmSummary);
    await useSigmaStore.getState().fetchSwarmSummary();

    const { swarmSummary } = useSigmaStore.getState();
    expect(swarmSummary.total_agents).toBe(8);
    expect(swarmSummary.active_agents).toBe(8);
    expect(swarmSummary.total_successes).toBe(15);
    expect(swarmSummary.total_failures).toBe(5);
    expect(swarmSummary.overall_success_rate).toBe(0.75);
    expect(swarmSummary.avg_generation).toBe(1.5);
  });

  // ── evaluateOpportunity ────────────────────────────────

  const mockEvaluation = {
    label: "test-opp-1",
    sigma_score: 0.72,
    attractor_score: 0.6,
    consensus: { score: 0.5, votes_for: 6, votes_against: 1, abstentions: 1, proceed: true },
    wave_score: 0.8,
    duplicate: false,
    similar_count: 2,
    proceed: true,
  };

  it("evaluateOpportunity stores evaluation on success", async () => {
    mockInvokeResponse("sigma_evaluate_opportunity", mockEvaluation);
    const result = await useSigmaStore.getState().evaluateOpportunity({
      chain: "ethereum", opportunityType: "airdrop", riskLevel: "low",
      label: "test-opp-1",
    });

    expect(result).toEqual(mockEvaluation);
    expect(useSigmaStore.getState().lastEvaluation).toEqual(mockEvaluation);
    expect(useSigmaStore.getState().loading).toBe(false);
  });

  it("evaluateOpportunity returns null on error", async () => {
    mockInvokeResponse("sigma_evaluate_opportunity", new Error("eval failed"));
    const result = await useSigmaStore.getState().evaluateOpportunity({
      chain: "ethereum", opportunityType: "airdrop", riskLevel: "low",
      label: "test-opp-1",
    });

    expect(result).toBeNull();
    expect(useSigmaStore.getState().error).toContain("eval failed");
  });

  // ── recordOutcome ──────────────────────────────────────

  it("recordOutcome updates swarm summary", async () => {
    const updatedSummary = { ...mockSwarmSummary, total_successes: 16 };
    mockInvokeResponse("sigma_record_outcome", updatedSummary);
    const result = await useSigmaStore.getState().recordOutcome({
      label: "test-opp-1", chain: "ethereum",
      opportunityType: "airdrop", riskLevel: "low", success: true,
    });

    expect(result.total_successes).toBe(16);
    expect(useSigmaStore.getState().swarmSummary).toEqual(updatedSummary);
  });

  it("recordOutcome returns null on error", async () => {
    mockInvokeResponse("sigma_record_outcome", new Error("outcome fail"));
    const result = await useSigmaStore.getState().recordOutcome({
      label: "x", chain: "ethereum",
      opportunityType: "airdrop", riskLevel: "low", success: false,
    });

    expect(result).toBeNull();
    expect(useSigmaStore.getState().error).toContain("outcome fail");
  });

  // ── queryMemory ────────────────────────────────────────

  it("queryMemory stores results on success", async () => {
    const mockResults = [
      { label: "arb-opp", similarity: 0.89, tags: ["layer2"], reinforcement: 3 },
    ];
    mockInvokeResponse("sigma_memory_query", mockResults);
    const results = await useSigmaStore.getState().queryMemory({
      chain: "arbitrum", opportunityType: "airdrop", riskLevel: "medium",
    });

    expect(results).toEqual(mockResults);
    expect(useSigmaStore.getState().queryResults).toEqual(mockResults);
  });

  it("queryMemory returns empty on error", async () => {
    mockInvokeResponse("sigma_memory_query", new Error("query fail"));
    const results = await useSigmaStore.getState().queryMemory({
      chain: "x", opportunityType: "x", riskLevel: "x",
    });

    expect(results).toEqual([]);
  });

  // ── fetchMemoryLabels ──────────────────────────────────

  it("fetchMemoryLabels sets labels", async () => {
    mockInvokeResponse("sigma_memory_labels", ["eth-airdrop", "arb-quest"]);
    await useSigmaStore.getState().fetchMemoryLabels();
    expect(useSigmaStore.getState().memoryLabels).toEqual(["eth-airdrop", "arb-quest"]);
  });

  // ── evolveSwarm ────────────────────────────────────────

  it("evolveSwarm updates swarm summary", async () => {
    const evolved = { ...mockSwarmSummary, avg_generation: 2.5 };
    mockInvokeResponse("sigma_swarm_evolve", evolved);
    const result = await useSigmaStore.getState().evolveSwarm();

    expect(result.avg_generation).toBe(2.5);
    expect(useSigmaStore.getState().swarmSummary).toEqual(evolved);
  });

  it("evolveSwarm returns null on error", async () => {
    mockInvokeResponse("sigma_swarm_evolve", new Error("evolve fail"));
    const result = await useSigmaStore.getState().evolveSwarm();
    expect(result).toBeNull();
  });

  // ── setMutationRate ────────────────────────────────────

  it("setMutationRate updates local state", async () => {
    mockInvokeResponse("sigma_swarm_set_mutation_rate", 0.25);
    const rate = await useSigmaStore.getState().setMutationRate(0.25);
    expect(rate).toBe(0.25);
    expect(useSigmaStore.getState().mutationRate).toBe(0.25);
  });

  it("setMutationRate returns old rate on error", async () => {
    mockInvokeResponse("sigma_swarm_set_mutation_rate", new Error("fail"));
    const rate = await useSigmaStore.getState().setMutationRate(0.9);
    expect(rate).toBe(0.1); // default
  });

  // ── analyzeHurst ───────────────────────────────────────

  it("analyzeHurst stores result", async () => {
    const mockHurst = { exponent: 0.73, regime: "persistent" };
    mockInvokeResponse("sigma_hurst_analysis", mockHurst);
    const result = await useSigmaStore.getState().analyzeHurst([1, 2, 3, 4, 5]);

    expect(result).toEqual(mockHurst);
    expect(useSigmaStore.getState().hurstResult).toEqual(mockHurst);
  });

  it("analyzeHurst returns null on error", async () => {
    mockInvokeResponse("sigma_hurst_analysis", new Error("too short"));
    const result = await useSigmaStore.getState().analyzeHurst([1]);
    expect(result).toBeNull();
  });

  // ── toggleDynamics ─────────────────────────────────────

  it("toggleDynamics calls backend and refreshes status", async () => {
    mockInvokeResponse("sigma_toggle_dynamics", true);
    mockInvokeResponse("get_sigma_status", mockStatus);
    await useSigmaStore.getState().toggleDynamics(true);
    // Should trigger fetchStatus which sets status
    expect(useSigmaStore.getState().status).toEqual(mockStatus);
  });

  // ── swarmVote ──────────────────────────────────────────

  it("swarmVote returns consensus result", async () => {
    const mockConsensus = { score: 0.6, votes_for: 7, votes_against: 1, abstentions: 0, proceed: true };
    mockInvokeResponse("sigma_swarm_vote", mockConsensus);
    const result = await useSigmaStore.getState().swarmVote({
      chain: "ethereum", opportunityType: "airdrop", riskLevel: "low",
    });
    expect(result.proceed).toBe(true);
    expect(result.votes_for).toBe(7);
  });

  it("swarmVote returns null on error", async () => {
    mockInvokeResponse("sigma_swarm_vote", new Error("vote fail"));
    const result = await useSigmaStore.getState().swarmVote({
      chain: "x", opportunityType: "x", riskLevel: "x",
    });
    expect(result).toBeNull();
  });

  // ── getSigmaScoreColor ─────────────────────────────────

  it("getSigmaScoreColor returns grey when no evaluation", () => {
    expect(useSigmaStore.getState().getSigmaScoreColor()).toBe("text-white/40");
  });

  it("getSigmaScoreColor returns green for high score", () => {
    useSigmaStore.setState({ lastEvaluation: { ...mockEvaluation, sigma_score: 0.8 } });
    expect(useSigmaStore.getState().getSigmaScoreColor()).toBe("text-emerald-400");
  });

  it("getSigmaScoreColor returns amber for medium score", () => {
    useSigmaStore.setState({ lastEvaluation: { ...mockEvaluation, sigma_score: 0.55 } });
    expect(useSigmaStore.getState().getSigmaScoreColor()).toBe("text-amber-400");
  });

  it("getSigmaScoreColor returns red for low score", () => {
    useSigmaStore.setState({ lastEvaluation: { ...mockEvaluation, sigma_score: 0.3 } });
    expect(useSigmaStore.getState().getSigmaScoreColor()).toBe("text-red-400");
  });
});
