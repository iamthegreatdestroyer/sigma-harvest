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

import { usePriceStore } from "../stores/priceStore";

describe("usePriceStore", () => {
  beforeEach(() => {
    __clearMocks();
    vi.clearAllMocks();
    usePriceStore.setState({
      prices: [],
      cached: false,
      loading: false,
      error: null,
    });
  });

  // ── Initial state ──────────────────────────────────────

  it("has correct initial state", () => {
    const state = usePriceStore.getState();
    expect(state.prices).toEqual([]);
    expect(state.cached).toBe(false);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  // ── fetchPrices ────────────────────────────────────────

  it("fetchPrices success stores prices", async () => {
    const mockResponse = {
      prices: [
        {
          id: "ethereum",
          symbol: "ETH",
          usd: 3500.42,
          usd_24h_change: 2.5,
          last_updated: "2026-01-01T00:00:00Z",
        },
        {
          id: "matic-network",
          symbol: "MATIC",
          usd: 0.85,
          usd_24h_change: -1.2,
          last_updated: "2026-01-01T00:00:00Z",
        },
      ],
      cached: false,
    };
    __mockResponse("get_token_prices", mockResponse);

    const result = await usePriceStore.getState().fetchPrices();
    const state = usePriceStore.getState();

    expect(state.prices.length).toBe(2);
    expect(state.prices[0].symbol).toBe("ETH");
    expect(state.cached).toBe(false);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
    expect(result.prices.length).toBe(2);
  });

  it("fetchPrices sets cached flag from backend", async () => {
    __mockResponse("get_token_prices", {
      prices: [
        { id: "ethereum", symbol: "ETH", usd: 3500, usd_24h_change: null, last_updated: "" },
      ],
      cached: true,
    });

    await usePriceStore.getState().fetchPrices();
    expect(usePriceStore.getState().cached).toBe(true);
  });

  it("fetchPrices failure sets error", async () => {
    __mockResponse("get_token_prices", new Error("Rate limited"));

    await usePriceStore.getState().fetchPrices();
    const state = usePriceStore.getState();
    expect(state.error).toContain("Rate limited");
    expect(state.loading).toBe(false);
    expect(state.prices).toEqual([]);
  });

  it("fetchPrices sets loading during operation", async () => {
    let captured = null;
    __mockResponse("get_token_prices", () => {
      captured = usePriceStore.getState().loading;
      return { prices: [], cached: false };
    });

    await usePriceStore.getState().fetchPrices();
    expect(captured).toBe(true);
    expect(usePriceStore.getState().loading).toBe(false);
  });

  it("fetchPrices handles empty prices array", async () => {
    __mockResponse("get_token_prices", { prices: [], cached: false });

    await usePriceStore.getState().fetchPrices();
    const state = usePriceStore.getState();
    expect(state.prices).toEqual([]);
    expect(state.error).toBeNull();
  });

  it("fetchPrices handles missing prices key", async () => {
    __mockResponse("get_token_prices", { cached: false });

    await usePriceStore.getState().fetchPrices();
    const state = usePriceStore.getState();
    expect(state.prices).toEqual([]);
  });

  // ── fetchChainPrice ────────────────────────────────────

  it("fetchChainPrice returns price for known chain", async () => {
    __mockResponse("get_chain_price_usd", 3500.0);

    const price = await usePriceStore.getState().fetchChainPrice("ethereum");
    expect(price).toBe(3500.0);
    expect(invoke).toHaveBeenCalledWith("get_chain_price_usd", {
      chain: "ethereum",
    });
  });

  it("fetchChainPrice returns null on error", async () => {
    __mockResponse("get_chain_price_usd", new Error("Unknown chain"));

    const price = await usePriceStore.getState().fetchChainPrice("solana");
    expect(price).toBeNull();
  });

  // ── getEthPrice ────────────────────────────────────────

  it("getEthPrice returns price from cached state", () => {
    usePriceStore.setState({
      prices: [
        { id: "ethereum", symbol: "ETH", usd: 3500.42 },
        { id: "matic-network", symbol: "MATIC", usd: 0.85 },
      ],
    });
    expect(usePriceStore.getState().getEthPrice()).toBe(3500.42);
  });

  it("getEthPrice returns 0 when no ETH in prices", () => {
    usePriceStore.setState({ prices: [] });
    expect(usePriceStore.getState().getEthPrice()).toBe(0);
  });

  it("getEthPrice matches by id", () => {
    usePriceStore.setState({
      prices: [{ id: "ethereum", symbol: "XXX", usd: 9999 }],
    });
    expect(usePriceStore.getState().getEthPrice()).toBe(9999);
  });

  // ── getMaticPrice ──────────────────────────────────────

  it("getMaticPrice returns price from cached state", () => {
    usePriceStore.setState({
      prices: [
        { id: "ethereum", symbol: "ETH", usd: 3500 },
        { id: "matic-network", symbol: "MATIC", usd: 0.85 },
      ],
    });
    expect(usePriceStore.getState().getMaticPrice()).toBe(0.85);
  });

  it("getMaticPrice returns 0 when no MATIC in prices", () => {
    usePriceStore.setState({ prices: [] });
    expect(usePriceStore.getState().getMaticPrice()).toBe(0);
  });

  // ── toUsd ──────────────────────────────────────────────

  it("toUsd converts ETH amount to USD", () => {
    usePriceStore.setState({
      prices: [{ id: "ethereum", symbol: "ETH", usd: 3500 }],
    });
    const usd = usePriceStore.getState().toUsd("ethereum", 1.5);
    expect(usd).toBe(5250);
  });

  it("toUsd converts MATIC amount to USD for polygon", () => {
    usePriceStore.setState({
      prices: [{ id: "matic-network", symbol: "MATIC", usd: 0.8 }],
    });
    const usd = usePriceStore.getState().toUsd("polygon", 100);
    expect(usd).toBe(80);
  });

  it("toUsd uses ETH price for L2 chains", () => {
    usePriceStore.setState({
      prices: [{ id: "ethereum", symbol: "ETH", usd: 3500 }],
    });
    expect(usePriceStore.getState().toUsd("arbitrum", 1)).toBe(3500);
    expect(usePriceStore.getState().toUsd("optimism", 2)).toBe(7000);
    expect(usePriceStore.getState().toUsd("base", 0.5)).toBe(1750);
  });

  it("toUsd returns 0 when no price available", () => {
    usePriceStore.setState({ prices: [] });
    expect(usePriceStore.getState().toUsd("ethereum", 1)).toBe(0);
  });

  it("toUsd returns 0 for zero amount", () => {
    usePriceStore.setState({
      prices: [{ id: "ethereum", symbol: "ETH", usd: 3500 }],
    });
    expect(usePriceStore.getState().toUsd("ethereum", 0)).toBe(0);
  });
});
