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

// Dynamic import AFTER mock is set up
const { invoke, __mockResponse, __clearMocks } = await import("@tauri-apps/api/core");

// We need to re-import the store fresh for each test to reset state
// Use zustand's vanilla store pattern
import { useWalletStore } from "../stores/walletStore";

describe("useWalletStore", () => {
  beforeEach(() => {
    __clearMocks();
    vi.clearAllMocks();
    // Reset store state
    useWalletStore.setState({
      vaultLocked: true,
      vaultExists: false,
      wallets: [],
      selectedWallet: null,
      mnemonic: null,
      loading: false,
      error: null,
    });
  });

  // ── Initial state ──────────────────────────────────────

  it("has correct initial state", () => {
    const state = useWalletStore.getState();
    expect(state.vaultLocked).toBe(true);
    expect(state.vaultExists).toBe(false);
    expect(state.wallets).toEqual([]);
    expect(state.selectedWallet).toBeNull();
    expect(state.mnemonic).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  // ── fetchVaultStatus ───────────────────────────────────

  it("fetchVaultStatus updates locked state", async () => {
    __mockResponse("get_vault_status", { locked: false, wallet_count: 2, last_unlock: "2026-01-01" });
    __mockResponse("has_vault", true);

    await useWalletStore.getState().fetchVaultStatus();
    const state = useWalletStore.getState();
    expect(state.vaultExists).toBe(true);
  });

  it("fetchVaultStatus handles error gracefully", async () => {
    __mockResponse("get_vault_status", new Error("backend crash"));

    // Should not throw
    await useWalletStore.getState().fetchVaultStatus();
    const state = useWalletStore.getState();
    // Still in default state
    expect(state.vaultLocked).toBe(true);
  });

  // ── createVault ────────────────────────────────────────

  it("createVault success stores mnemonic and wallets", async () => {
    __mockResponse("create_wallet", {
      mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      first_address: "0xABC123",
    });
    __mockResponse("list_wallets", [
      { id: "w1", derivation_path: "m/44'/60'/0'/0/0", public_address: "0xABC123", chain: "ethereum", label: null },
    ]);

    const result = await useWalletStore.getState().createVault("test1234");
    const state = useWalletStore.getState();

    expect(result.mnemonic).toContain("abandon");
    expect(state.mnemonic).toContain("abandon");
    expect(state.vaultLocked).toBe(false);
    expect(state.vaultExists).toBe(true);
    expect(state.wallets.length).toBe(1);
    expect(state.loading).toBe(false);
  });

  it("createVault failure sets error", async () => {
    __mockResponse("create_wallet", new Error("too short"));

    await expect(useWalletStore.getState().createVault("short")).rejects.toThrow("too short");
    const state = useWalletStore.getState();
    expect(state.error).toBe("Error: too short");
    expect(state.loading).toBe(false);
    expect(state.mnemonic).toBeNull();
  });

  it("createVault sets loading during operation", async () => {
    let captured = null;
    __mockResponse("create_wallet", (args) => {
      captured = useWalletStore.getState().loading;
      return { mnemonic: "test test test", first_address: "0x123" };
    });
    __mockResponse("list_wallets", []);

    await useWalletStore.getState().createVault("test1234");
    expect(captured).toBe(true); // loading was true during invoke
    expect(useWalletStore.getState().loading).toBe(false); // done
  });

  // ── unlockVault ────────────────────────────────────────

  it("unlockVault success updates state", async () => {
    __mockResponse("unlock_vault", undefined);
    __mockResponse("list_wallets", [
      { id: "w1", public_address: "0xABC", chain: "ethereum" },
      { id: "w2", public_address: "0xDEF", chain: "arbitrum" },
    ]);

    await useWalletStore.getState().unlockVault("test1234");
    const state = useWalletStore.getState();
    expect(state.vaultLocked).toBe(false);
    expect(state.wallets.length).toBe(2);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("unlockVault failure sets error", async () => {
    __mockResponse("unlock_vault", new Error("Invalid passphrase"));

    await expect(useWalletStore.getState().unlockVault("wrong")).rejects.toThrow();
    const state = useWalletStore.getState();
    expect(state.error).toContain("Invalid passphrase");
    expect(state.vaultLocked).toBe(true);
  });

  // ── lockVault ──────────────────────────────────────────

  it("lockVault clears wallets and mnemonic", async () => {
    // Set up unlocked state
    useWalletStore.setState({
      vaultLocked: false,
      wallets: [{ id: "w1" }],
      mnemonic: "some phrase",
    });
    __mockResponse("lock_vault", undefined);

    await useWalletStore.getState().lockVault();
    const state = useWalletStore.getState();
    expect(state.vaultLocked).toBe(true);
    expect(state.wallets).toEqual([]);
    expect(state.mnemonic).toBeNull();
  });

  it("lockVault handles error gracefully", async () => {
    __mockResponse("lock_vault", new Error("unexpected"));
    // Should not throw
    await useWalletStore.getState().lockVault();
  });

  // ── deriveWallet ───────────────────────────────────────

  it("deriveWallet appends to wallets array", async () => {
    useWalletStore.setState({
      wallets: [{ id: "w1", public_address: "0xAAA" }],
    });
    __mockResponse("derive_next_wallet", {
      id: "w2",
      public_address: "0xBBB",
      chain: "ethereum",
      index: 1,
    });

    const wallet = await useWalletStore.getState().deriveWallet("ethereum");
    const state = useWalletStore.getState();
    expect(state.wallets.length).toBe(2);
    expect(wallet.public_address).toBe("0xBBB");
    expect(state.loading).toBe(false);
  });

  it("deriveWallet defaults to ethereum chain", async () => {
    __mockResponse("derive_next_wallet", { id: "w1", chain: "ethereum" });

    await useWalletStore.getState().deriveWallet();
    expect(invoke).toHaveBeenCalledWith("derive_next_wallet", { chain: "ethereum" });
  });

  it("deriveWallet failure sets error", async () => {
    __mockResponse("derive_next_wallet", new Error("Vault is locked"));

    await expect(useWalletStore.getState().deriveWallet()).rejects.toThrow();
    expect(useWalletStore.getState().error).toContain("locked");
  });

  // ── Utility actions ────────────────────────────────────

  it("clearMnemonic nullifies mnemonic", () => {
    useWalletStore.setState({ mnemonic: "secret phrase" });
    useWalletStore.getState().clearMnemonic();
    expect(useWalletStore.getState().mnemonic).toBeNull();
  });

  it("setSelectedWallet updates selection", () => {
    const wallet = { id: "w1", public_address: "0xAAA" };
    useWalletStore.getState().setSelectedWallet(wallet);
    expect(useWalletStore.getState().selectedWallet).toEqual(wallet);
  });

  it("clearError nullifies error", () => {
    useWalletStore.setState({ error: "some error" });
    useWalletStore.getState().clearError();
    expect(useWalletStore.getState().error).toBeNull();
  });
});
