import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export const useWalletStore = create((set, get) => ({
  vaultLocked: true,
  vaultExists: false,
  wallets: [],
  selectedWallet: null,
  mnemonic: null, // Shown only once during creation
  loading: false,
  error: null,

  /** Fetch vault status from backend on app start. */
  fetchVaultStatus: async () => {
    try {
      const status = await invoke("get_vault_status");
      set({
        vaultLocked: status.locked,
        vaultExists: status.wallet_count > 0 || !status.locked,
      });
      // Also check has_vault for fresh installs:
      const exists = await invoke("has_vault");
      set({ vaultExists: exists });
    } catch (err) {
      console.error("Failed to fetch vault status:", err);
    }
  },

  /** Create a brand-new vault. Returns mnemonic (show once). */
  createVault: async (passphrase) => {
    set({ loading: true, error: null, mnemonic: null });
    try {
      const result = await invoke("create_wallet", { passphrase });
      const wallets = await invoke("list_wallets");
      set({
        mnemonic: result.mnemonic,
        wallets,
        vaultLocked: false,
        vaultExists: true,
        loading: false,
      });
      return result;
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  /** Unlock the vault with a passphrase. */
  unlockVault: async (passphrase) => {
    set({ loading: true, error: null });
    try {
      await invoke("unlock_vault", { passphrase });
      const wallets = await invoke("list_wallets");
      set({ vaultLocked: false, wallets, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  /** Lock the vault. */
  lockVault: async () => {
    try {
      await invoke("lock_vault");
      set({ vaultLocked: true, wallets: [], mnemonic: null });
    } catch (err) {
      console.error("Failed to lock vault:", err);
    }
  },

  /** Derive a new wallet on a given chain. */
  deriveWallet: async (chain = "ethereum") => {
    set({ loading: true, error: null });
    try {
      const wallet = await invoke("derive_next_wallet", { chain });
      set((state) => ({
        wallets: [...state.wallets, wallet],
        loading: false,
      }));
      return wallet;
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  /** Clear the displayed mnemonic (user has backed it up). */
  clearMnemonic: () => set({ mnemonic: null }),
  setSelectedWallet: (wallet) => set({ selectedWallet: wallet }),
  clearError: () => set({ error: null }),
}));
