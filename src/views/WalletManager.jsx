import { useState, useEffect } from "react";
import {
  Wallet,
  Plus,
  RefreshCw,
  ArrowRightLeft,
  Copy,
  Lock,
  Unlock,
  ShieldCheck,
  AlertTriangle,
  Check,
  ChevronDown,
} from "lucide-react";
import { useWalletStore } from "../stores/walletStore";
import { useChainStore } from "../stores/chainStore";
import { usePriceStore } from "../stores/priceStore";

function MnemonicBackup({ mnemonic, onConfirm }) {
  const [confirmed, setConfirmed] = useState(false);
  const words = mnemonic.split(" ");

  return (
    <div className="bg-surface rounded-lg border border-warning/40 p-6 max-w-xl mx-auto">
      <div className="flex items-center gap-2 mb-4">
        <AlertTriangle className="text-warning" size={20} />
        <h3 className="text-lg font-bold text-warning">
          Back Up Your Recovery Phrase
        </h3>
      </div>
      <p className="text-text-muted text-xs mb-4">
        Write these words down in order and store them securely. This is the{" "}
        <span className="text-warning font-semibold">only way</span> to recover
        your wallets if you forget your passphrase.
      </p>
      <div className="grid grid-cols-4 gap-2 mb-6 bg-bg rounded-lg p-4 border border-border">
        {words.map((word, i) => (
          <div
            key={i}
            className="flex items-center gap-1.5 text-sm font-mono"
          >
            <span className="text-text-dim text-[10px] w-5 text-right">
              {i + 1}.
            </span>
            <span className="text-primary">{word}</span>
          </div>
        ))}
      </div>
      <label className="flex items-center gap-2 text-xs text-text-muted mb-4 cursor-pointer">
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
          className="accent-primary"
        />
        I have written down my recovery phrase and stored it securely
      </label>
      <button
        onClick={onConfirm}
        disabled={!confirmed}
        className="w-full px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
      >
        <Check size={14} className="inline mr-1" /> I&apos;ve Backed It Up —
        Continue
      </button>
    </div>
  );
}

function VaultSetup() {
  const { createVault, loading, error, clearError } = useWalletStore();
  const [passphrase, setPassphrase] = useState("");
  const [confirm, setConfirm] = useState("");

  const handleCreate = async () => {
    clearError();
    if (passphrase.length < 8) return;
    if (passphrase !== confirm) return;
    try {
      await createVault(passphrase);
    } catch {
      /* error set in store */
    }
  };

  const valid = passphrase.length >= 8 && passphrase === confirm;

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6">
      <ShieldCheck size={48} className="text-primary" />
      <div className="text-center max-w-md">
        <h2 className="text-xl font-bold text-text mb-2">Create Your Vault</h2>
        <p className="text-text-muted text-sm mb-6">
          Set a strong passphrase to encrypt your HD wallet seed. All private
          keys are derived from this seed and never leave the Rust backend.
        </p>
        <div className="space-y-3">
          <input
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            placeholder="Passphrase (8+ characters)..."
            className="w-full px-4 py-2 bg-surface-raised border border-border rounded text-text text-sm focus:outline-none focus:border-primary"
          />
          <input
            type="password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            placeholder="Confirm passphrase..."
            className="w-full px-4 py-2 bg-surface-raised border border-border rounded text-text text-sm focus:outline-none focus:border-primary"
          />
          {error && (
            <p className="text-danger text-xs text-left">{error}</p>
          )}
          {passphrase.length > 0 && passphrase.length < 8 && (
            <p className="text-warning text-xs text-left">
              Passphrase must be at least 8 characters
            </p>
          )}
          {confirm.length > 0 && passphrase !== confirm && (
            <p className="text-danger text-xs text-left">
              Passphrases do not match
            </p>
          )}
          <button
            onClick={handleCreate}
            disabled={!valid || loading}
            className="w-full px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          >
            {loading ? "Creating vault..." : "Create Vault"}
          </button>
        </div>
      </div>
    </div>
  );
}

function VaultUnlock() {
  const { unlockVault, loading, error, clearError } = useWalletStore();
  const [passphrase, setPassphrase] = useState("");

  const handleUnlock = async () => {
    clearError();
    if (!passphrase) return;
    try {
      await unlockVault(passphrase);
    } catch {
      /* error set in store */
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6">
      <Lock size={48} className="text-text-dim" />
      <div className="text-center">
        <h2 className="text-xl font-bold text-text mb-2">Vault Locked</h2>
        <p className="text-text-muted text-sm mb-6">
          Enter your passphrase to unlock the wallet constellation
        </p>
        <div className="flex items-center gap-2 max-w-md mx-auto">
          <input
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleUnlock()}
            placeholder="Enter passphrase..."
            className="flex-1 px-4 py-2 bg-surface-raised border border-border rounded text-text text-sm focus:outline-none focus:border-primary"
          />
          <button
            onClick={handleUnlock}
            disabled={loading || !passphrase}
            className="px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors disabled:opacity-30"
          >
            {loading ? "..." : "Unlock"}
          </button>
        </div>
        {error && (
          <p className="text-danger text-xs mt-3">{error}</p>
        )}
      </div>
    </div>
  );
}

export default function WalletManager() {
  const {
    vaultLocked,
    vaultExists,
    wallets,
    mnemonic,
    loading,
    lockVault,
    deriveWallet,
    clearMnemonic,
    fetchVaultStatus,
    planConsolidation,
    consolidationPlan,
    consolidationLoading,
    consolidationError,
    clearConsolidation,
  } = useWalletStore();

  const { balances, balanceLoading, fetchAllBalances } = useChainStore();
  const { fetchPrices, toUsd } = usePriceStore();

  const [deriveChain, setDeriveChain] = useState("ethereum");
  const [chainDropdownOpen, setChainDropdownOpen] = useState(false);
  const [showConsolidation, setShowConsolidation] = useState(false);
  const [consolidationDest, setConsolidationDest] = useState("");

  useEffect(() => {
    fetchVaultStatus();
  }, [fetchVaultStatus]);

  // Fetch balances when wallets change and vault is unlocked
  useEffect(() => {
    if (!vaultLocked && wallets.length > 0) {
      const addresses = wallets.map((w) => w.address);
      fetchAllBalances(addresses);
      fetchPrices();
    }
  }, [vaultLocked, wallets, fetchAllBalances, fetchPrices]);

  const handleRefreshBalances = () => {
    if (wallets.length > 0) {
      const addresses = wallets.map((w) => w.address);
      fetchAllBalances(addresses);
    }
  };

  // Show mnemonic backup screen after vault creation
  if (mnemonic) {
    return <MnemonicBackup mnemonic={mnemonic} onConfirm={clearMnemonic} />;
  }

  // No vault yet → show setup
  if (!vaultExists) {
    return <VaultSetup />;
  }

  // Vault exists but locked
  if (vaultLocked) {
    return <VaultUnlock />;
  }

  const DERIVE_CHAINS = [
    "ethereum",
    "arbitrum",
    "optimism",
    "base",
    "polygon",
    "zksync",
  ];

  // Vault unlocked → show wallet management
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-3">
          <Wallet className="text-accent" size={28} />
          <div>
            <h2 className="text-xl font-bold text-text">
              Wallet Constellation
            </h2>
            <p className="text-text-muted text-xs">
              HD-derived wallet management •{" "}
              <span className="text-primary">{wallets.length} wallets</span>
            </p>
          </div>
        </div>
        <button
          onClick={lockVault}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-danger/10 border border-danger/30 rounded text-danger text-xs hover:bg-danger/20 transition-colors"
        >
          <Lock size={12} /> Lock Vault
        </button>
      </div>
      <div className="h-px bg-gradient-to-r from-accent to-transparent" />

      {/* Actions */}
      <div className="flex items-center gap-3">
        <div className="flex items-center">
          <button
            onClick={() => deriveWallet(deriveChain)}
            disabled={loading}
            className="flex items-center gap-2 px-3 py-1.5 bg-primary/10 border border-primary/30 rounded-l text-primary text-xs hover:bg-primary/20 transition-colors disabled:opacity-40"
          >
            <Plus size={12} /> Derive
          </button>
          <div className="relative">
            <button
              onClick={() => setChainDropdownOpen(!chainDropdownOpen)}
              className="flex items-center gap-1 px-2 py-1.5 bg-primary/10 border border-primary/30 border-l-0 rounded-r text-primary text-xs hover:bg-primary/20 transition-colors"
            >
              <span className="capitalize">{deriveChain}</span>
              <ChevronDown size={10} />
            </button>
            {chainDropdownOpen && (
              <div className="absolute right-0 top-full mt-1 bg-surface border border-border rounded shadow-lg z-10 min-w-[120px]">
                {DERIVE_CHAINS.map((chain) => (
                  <button
                    key={chain}
                    onClick={() => {
                      setDeriveChain(chain);
                      setChainDropdownOpen(false);
                    }}
                    className={`block w-full text-left px-3 py-1.5 text-xs hover:bg-primary/10 transition-colors capitalize ${
                      chain === deriveChain
                        ? "text-primary"
                        : "text-text-muted"
                    }`}
                  >
                    {chain}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
        <button
          onClick={handleRefreshBalances}
          disabled={balanceLoading}
          className="flex items-center gap-2 px-3 py-1.5 bg-accent/10 border border-accent/30 rounded text-accent text-xs hover:bg-accent/20 transition-colors disabled:opacity-40"
        >
          <RefreshCw
            size={12}
            className={balanceLoading ? "animate-spin" : ""}
          />
          {balanceLoading ? "Loading..." : "Refresh Balances"}
        </button>
        <button
          onClick={() => setShowConsolidation(true)}
          disabled={consolidationLoading || wallets.length < 2}
          className="flex items-center gap-2 px-3 py-1.5 bg-warning/10 border border-warning/30 rounded text-warning text-xs hover:bg-warning/20 transition-colors disabled:opacity-40"
        >
          <ArrowRightLeft size={12} />
          {consolidationLoading ? "Scanning..." : "Consolidate"}
        </button>
      </div>

      {/* Consolidation Modal */}
      {showConsolidation && (
        <div className="bg-surface rounded-lg border border-warning/30 p-4 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-warning flex items-center gap-2">
              <ArrowRightLeft size={14} /> Fund Consolidation
            </h3>
            <button
              onClick={() => { setShowConsolidation(false); clearConsolidation(); }}
              className="text-text-dim text-xs hover:text-text"
            >
              ✕
            </button>
          </div>
          <p className="text-text-muted text-xs">
            Scan all wallets and plan a sweep of native tokens to a destination
            address. No transactions are sent — this is a dry run.
          </p>
          <div className="flex gap-2">
            <input
              type="text"
              value={consolidationDest}
              onChange={(e) => setConsolidationDest(e.target.value)}
              placeholder="0x destination address..."
              className="flex-1 px-3 py-1.5 bg-bg border border-border rounded text-text text-xs font-mono focus:outline-none focus:border-warning"
            />
            <button
              onClick={() =>
                planConsolidation({
                  destination: consolidationDest,
                  chain: "ethereum",
                  gasPriceGwei: 10,
                })
              }
              disabled={
                consolidationLoading ||
                !consolidationDest.startsWith("0x") ||
                consolidationDest.length !== 42
              }
              className="px-3 py-1.5 bg-warning/10 border border-warning/30 rounded text-warning text-xs hover:bg-warning/20 transition-colors disabled:opacity-40"
            >
              {consolidationLoading ? "Scanning..." : "Plan Sweep"}
            </button>
          </div>
          {consolidationError && (
            <p className="text-danger text-xs">{consolidationError}</p>
          )}
          {consolidationPlan && (
            <div className="bg-bg rounded border border-border p-3 text-xs space-y-1">
              <p className="text-text">
                <span className="text-text-muted">Chain:</span>{" "}
                {consolidationPlan.chain}
              </p>
              <p className="text-text">
                <span className="text-text-muted">Sweepable wallets:</span>{" "}
                <span className="text-primary">
                  {consolidationPlan.candidates?.length ?? 0}
                </span>
              </p>
              <p className="text-text">
                <span className="text-text-muted">ERC-20 sweeps:</span>{" "}
                {consolidationPlan.total_erc20_sweeps ?? 0}
              </p>
              <p className="text-text">
                <span className="text-text-muted">Skipped (dust):</span>{" "}
                {consolidationPlan.skipped_dust ?? 0}
              </p>
              <p className="text-accent text-xs mt-1">
                Dry run complete — no transactions sent.
              </p>
            </div>
          )}
        </div>
      )}

      {/* Wallet List */}
      <div className="space-y-2">
        {wallets.length === 0 ? (
          <div className="bg-surface rounded-lg border border-border p-8 text-center">
            <Wallet size={32} className="text-text-dim mx-auto mb-3" />
            <p className="text-text-muted text-sm">
              No wallets yet. Derive your first wallet to begin.
            </p>
          </div>
        ) : (
          wallets.map((w) => (
            <WalletCard
              key={w.address}
              wallet={w}
              balances={balances[w.address] || []}
              toUsd={toUsd}
            />
          ))
        )}
      </div>
    </div>
  );
}

function WalletCard({ wallet, balances, toUsd }) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const copyAddress = (e) => {
    e.stopPropagation();
    navigator.clipboard.writeText(wallet.address);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const totalBalance = balances.reduce((sum, b) => sum + b.balance_eth, 0);
  const totalUsd = balances.reduce((sum, b) => sum + (toUsd?.(b.chain, b.balance_eth) ?? 0), 0);

  return (
    <div className="bg-surface rounded-lg border border-border hover:border-primary/30 transition-colors">
      <div
        className="p-4 flex items-center justify-between cursor-pointer"
        onClick={() => setExpanded(!expanded)}
      >
        <div>
          <div className="text-xs text-text-muted mb-1">
            {wallet.chain} •{" "}
            <span className="font-mono">{wallet.path}</span>
          </div>
          <div className="text-sm font-mono text-text flex items-center gap-2">
            {wallet.address.slice(0, 6)}...{wallet.address.slice(-4)}
            <button
              onClick={copyAddress}
              className="text-text-dim hover:text-primary transition-colors"
            >
              {copied ? (
                <Check size={12} className="text-primary" />
              ) : (
                <Copy size={12} />
              )}
            </button>
          </div>
        </div>
        <div className="text-right">
          <div className="text-sm font-bold text-primary">
            {totalBalance > 0 ? totalBalance.toFixed(6) : "—"} ETH
          </div>
          {totalUsd > 0 && (
            <div className="text-[11px] text-accent">
              ${totalUsd.toFixed(2)}
            </div>
          )}
          <div className="text-xs text-text-muted">
            {wallet.label || `Wallet #${wallet.index}`}
          </div>
        </div>
      </div>
      {expanded && balances.length > 0 && (
        <div className="border-t border-border px-4 py-3 space-y-1.5">
          {balances.map((b) => (
            <div
              key={b.chain}
              className="flex items-center justify-between text-xs"
            >
              <span className="text-text-muted capitalize">{b.chain}</span>
              <span
                className={
                  b.balance_eth > 0 ? "text-primary" : "text-text-dim"
                }
              >
                {b.balance_eth > 0
                  ? b.balance_eth.toFixed(6)
                  : "0.000000"}{" "}
                {b.chain === "polygon" ? "MATIC" : "ETH"}
                {b.balance_eth > 0 && toUsd && (
                  <span className="ml-2 text-accent">
                    (${toUsd(b.chain, b.balance_eth).toFixed(2)})
                  </span>
                )}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
