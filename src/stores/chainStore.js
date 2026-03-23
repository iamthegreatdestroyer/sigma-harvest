import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

/**
 * Store for chain connectivity data: gas prices and wallet balances.
 * Separating from walletStore since this is RPC-dependent and refreshes independently.
 */
export const useChainStore = create((set, get) => ({
  gasPrices: [],
  balances: {}, // { [address]: AddressBalance[] }
  chainConfigs: [],
  gasLoading: false,
  balanceLoading: false,
  gasError: null,
  balanceError: null,

  /** Fetch gas prices for all supported chains. */
  fetchGasPrices: async () => {
    set({ gasLoading: true, gasError: null });
    try {
      const prices = await invoke("get_gas_prices");
      set({ gasPrices: prices, gasLoading: false });
      return prices;
    } catch (err) {
      set({ gasError: String(err), gasLoading: false });
    }
  },

  /** Fetch native balances for a single address across all chains. */
  fetchBalances: async (address) => {
    set({ balanceLoading: true, balanceError: null });
    try {
      const result = await invoke("get_balances", { address });
      set((state) => ({
        balances: { ...state.balances, [address]: result },
        balanceLoading: false,
      }));
      return result;
    } catch (err) {
      set({ balanceError: String(err), balanceLoading: false });
    }
  },

  /** Fetch balances for all given addresses. */
  fetchAllBalances: async (addresses) => {
    set({ balanceLoading: true, balanceError: null });
    try {
      const allBalances = {};
      for (const addr of addresses) {
        const result = await invoke("get_balances", { address: addr });
        allBalances[addr] = result;
      }
      set({ balances: allBalances, balanceLoading: false });
    } catch (err) {
      set({ balanceError: String(err), balanceLoading: false });
    }
  },

  /** Fetch chain configuration from backend. */
  fetchChainConfigs: async () => {
    try {
      const configs = await invoke("get_chain_configs");
      set({ chainConfigs: configs });
    } catch (err) {
      console.error("Failed to fetch chain configs:", err);
    }
  },

  /** Get total balance across all chains for an address. */
  getTotalBalance: (address) => {
    const { balances } = get();
    const addrBalances = balances[address] || [];
    return addrBalances.reduce((sum, b) => sum + b.balance_eth, 0);
  },

  /** Get a specific chain's gas price. */
  getGasForChain: (chainName) => {
    const { gasPrices } = get();
    return gasPrices.find(
      (g) => g.chain.toLowerCase() === chainName.toLowerCase(),
    );
  },
}));
