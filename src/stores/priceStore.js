import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

/**
 * Store for token price data from CoinGecko.
 * Cached for 5 minutes on the backend; frontend can refresh anytime.
 */
export const usePriceStore = create((set, get) => ({
  prices: [], // TokenPrice[]
  cached: false,
  loading: false,
  error: null,

  /** Fetch native token prices (ETH, MATIC) from CoinGecko via backend. */
  fetchPrices: async () => {
    set({ loading: true, error: null });
    try {
      const response = await invoke("get_token_prices");
      set({
        prices: response.prices || [],
        cached: response.cached || false,
        loading: false,
      });
      return response;
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  /** Get USD price for a specific chain's native token. */
  fetchChainPrice: async (chain) => {
    try {
      return await invoke("get_chain_price_usd", { chain });
    } catch (e) {
      console.error(`Failed to fetch price for ${chain}:`, e);
      return null;
    }
  },

  /** Get the ETH price from the cached prices array. */
  getEthPrice: () => {
    const { prices } = get();
    const eth = prices.find((p) => p.symbol === "ETH" || p.id === "ethereum");
    return eth?.usd ?? 0;
  },

  /** Get the MATIC price from the cached prices array. */
  getMaticPrice: () => {
    const { prices } = get();
    const matic = prices.find(
      (p) => p.symbol === "MATIC" || p.id === "matic-network",
    );
    return matic?.usd ?? 0;
  },

  /** Convert a native balance (ETH or MATIC) to USD. */
  toUsd: (chain, ethAmount) => {
    const { prices } = get();
    // Most chains use ETH for gas
    const isPolygon =
      chain?.toLowerCase() === "polygon" || chain?.toLowerCase() === "matic";
    const token = isPolygon
      ? prices.find((p) => p.id === "matic-network")
      : prices.find((p) => p.id === "ethereum");

    return token ? ethAmount * token.usd : 0;
  },
}));
