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
} from "lucide-react";
import { useWalletStore } from "../stores/walletStore";

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
  } = useWalletStore();

  useEffect(() => {
    fetchVaultStatus();
  }, [fetchVaultStatus]);

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
        <button
          onClick={() => deriveWallet("ethereum")}
          disabled={loading}
          className="flex items-center gap-2 px-3 py-1.5 bg-primary/10 border border-primary/30 rounded text-primary text-xs hover:bg-primary/20 transition-colors disabled:opacity-40"
        >
          <Plus size={12} /> Derive Wallet
        </button>
        <button className="flex items-center gap-2 px-3 py-1.5 bg-accent/10 border border-accent/30 rounded text-accent text-xs hover:bg-accent/20 transition-colors">
          <RefreshCw size={12} /> Refresh Balances
        </button>
        <button className="flex items-center gap-2 px-3 py-1.5 bg-warning/10 border border-warning/30 rounded text-warning text-xs hover:bg-warning/20 transition-colors">
          <ArrowRightLeft size={12} /> Consolidate
        </button>
      </div>

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
            <WalletCard key={w.address} wallet={w} />
          ))
        )}
      </div>
    </div>
  );
}

function WalletCard({ wallet }) {
  const [copied, setCopied] = useState(false);

  const copyAddress = () => {
    navigator.clipboard.writeText(wallet.address);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="bg-surface rounded-lg border border-border p-4 flex items-center justify-between hover:border-primary/30 transition-colors">
      <div>
        <div className="text-xs text-text-muted mb-1">
          {wallet.chain} • <span className="font-mono">{wallet.path}</span>
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
        <div className="text-sm font-bold text-primary">— ETH</div>
        <div className="text-xs text-text-muted">
          {wallet.label || `Wallet #${wallet.index}`}
        </div>
      </div>
    </div>
  );
}
