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

const { invoke, __mockResponse, __clearMocks } = await import(
  "@tauri-apps/api/core"
);

import { useChainStore } from "../stores/chainStore";

describe("useChainStore", () => {
  beforeEach(() => {
    __clearMocks();
    vi.clearAllMocks();
    useChainStore.setState({
      gasPrices: [],
      balances: {},
      chainConfigs: [],
      gasLoading: false,
      balanceLoading: false,
      gasError: null,
      balanceError: null,
    });
  });

  // ── Initial state ──────────────────────────────────────

  it("has correct initial state", () => {
    const state = useChainStore.getState();
    expect(state.gasPrices).toEqual([]);
    expect(state.balances).toEqual({});
    expect(state.gasLoading).toBe(false);
    expect(state.balanceLoading).toBe(false);
  });

  // ── Gas prices ─────────────────────────────────────────

  describe("fetchGasPrices", () => {
    const mockGasPrices = [
      {
        chain: "ethereum",
        chain_id: 1,
        base_fee_gwei: 15.0,
        priority_fee_gwei: 2.0,
        total_gwei: 17.0,
      },
      {
        chain: "arbitrum",
        chain_id: 42161,
        base_fee_gwei: 0.1,
        priority_fee_gwei: 0.01,
        total_gwei: 0.11,
      },
    ];

    it("fetches gas prices and updates state", async () => {
      __mockResponse("get_gas_prices", mockGasPrices);

      await useChainStore.getState().fetchGasPrices();

      const state = useChainStore.getState();
      expect(state.gasPrices).toEqual(mockGasPrices);
      expect(state.gasLoading).toBe(false);
      expect(state.gasError).toBeNull();
      expect(invoke).toHaveBeenCalledWith("get_gas_prices");
    });

    it("handles gas price fetch error", async () => {
      __mockResponse("get_gas_prices", new Error("RPC down"));

      await useChainStore.getState().fetchGasPrices();

      const state = useChainStore.getState();
      expect(state.gasPrices).toEqual([]);
      expect(state.gasLoading).toBe(false);
      expect(state.gasError).toContain("RPC down");
    });
  });

  // ── Balances ───────────────────────────────────────────

  describe("fetchBalances", () => {
    const mockBalances = [
      {
        address: "0xABC",
        chain: "ethereum",
        chain_id: 1,
        balance_wei: "1000000000000000000",
        balance_eth: 1.0,
      },
      {
        address: "0xABC",
        chain: "arbitrum",
        chain_id: 42161,
        balance_wei: "500000000000000000",
        balance_eth: 0.5,
      },
    ];

    it("fetches balances for an address", async () => {
      __mockResponse("get_balances", mockBalances);

      await useChainStore.getState().fetchBalances("0xABC");

      const state = useChainStore.getState();
      expect(state.balances["0xABC"]).toEqual(mockBalances);
      expect(state.balanceLoading).toBe(false);
      expect(invoke).toHaveBeenCalledWith("get_balances", {
        address: "0xABC",
      });
    });

    it("handles balance fetch error", async () => {
      __mockResponse("get_balances", new Error("timeout"));

      await useChainStore.getState().fetchBalances("0xABC");

      const state = useChainStore.getState();
      expect(state.balanceLoading).toBe(false);
      expect(state.balanceError).toContain("timeout");
    });
  });

  describe("fetchAllBalances", () => {
    it("fetches balances for multiple addresses", async () => {
      __mockResponse("get_balances", (args) => [
        {
          address: args.address,
          chain: "ethereum",
          chain_id: 1,
          balance_wei: "1000000000000000000",
          balance_eth: 1.0,
        },
      ]);

      await useChainStore
        .getState()
        .fetchAllBalances(["0xAAA", "0xBBB"]);

      const state = useChainStore.getState();
      expect(state.balances["0xAAA"]).toHaveLength(1);
      expect(state.balances["0xBBB"]).toHaveLength(1);
      expect(state.balanceLoading).toBe(false);
    });
  });

  // ── Chain configs ──────────────────────────────────────

  describe("fetchChainConfigs", () => {
    it("fetches chain configs from backend", async () => {
      const mockConfigs = [
        { chain_id: 1, name: "ethereum", symbol: "ETH" },
        { chain_id: 42161, name: "arbitrum", symbol: "ARB" },
      ];
      __mockResponse("get_chain_configs", mockConfigs);

      await useChainStore.getState().fetchChainConfigs();

      const state = useChainStore.getState();
      expect(state.chainConfigs).toEqual(mockConfigs);
    });
  });

  // ── Computed helpers ───────────────────────────────────

  describe("getTotalBalance", () => {
    it("sums balances across chains for an address", () => {
      useChainStore.setState({
        balances: {
          "0xABC": [
            { chain: "ethereum", balance_eth: 1.5 },
            { chain: "arbitrum", balance_eth: 0.3 },
            { chain: "polygon", balance_eth: 0.2 },
          ],
        },
      });

      const total = useChainStore.getState().getTotalBalance("0xABC");
      expect(total).toBeCloseTo(2.0, 4);
    });

    it("returns 0 for unknown address", () => {
      const total = useChainStore.getState().getTotalBalance("0xNONE");
      expect(total).toBe(0);
    });
  });

  describe("getGasForChain", () => {
    it("finds gas price by chain name", () => {
      useChainStore.setState({
        gasPrices: [
          { chain: "ethereum", total_gwei: 17.0 },
          { chain: "arbitrum", total_gwei: 0.11 },
        ],
      });

      const ethGas = useChainStore.getState().getGasForChain("ethereum");
      expect(ethGas.total_gwei).toBe(17.0);
    });

    it("is case-insensitive", () => {
      useChainStore.setState({
        gasPrices: [{ chain: "ethereum", total_gwei: 17.0 }],
      });

      const gas = useChainStore.getState().getGasForChain("Ethereum");
      expect(gas.total_gwei).toBe(17.0);
    });

    it("returns undefined for unknown chain", () => {
      const gas = useChainStore.getState().getGasForChain("solana");
      expect(gas).toBeUndefined();
    });
  });
});
