import {
  Wallet,
  Plus,
  RefreshCw,
  ArrowRightLeft,
  Copy,
  Lock,
} from "lucide-react";
import { useWalletStore } from "../stores/walletStore";

export default function WalletManager() {
  const { wallets, vaultLocked } = useWalletStore();

  if (vaultLocked) {
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
              placeholder="Enter passphrase..."
              className="flex-1 px-4 py-2 bg-surface-raised border border-border rounded text-text text-sm focus:outline-none focus:border-primary"
            />
            <button className="px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors">
              Unlock
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <Wallet className="text-accent" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Wallet Constellation</h2>
          <p className="text-text-muted text-xs">
            HD-derived wallet management and consolidation
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-accent to-transparent" />

      {/* Actions */}
      <div className="flex items-center gap-3">
        <button className="flex items-center gap-2 px-3 py-1.5 bg-primary/10 border border-primary/30 rounded text-primary text-xs hover:bg-primary/20 transition-colors">
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
            <div
              key={w.address}
              className="bg-surface rounded-lg border border-border p-4 flex items-center justify-between"
            >
              <div>
                <div className="text-xs text-text-muted mb-1">
                  {w.chain} • {w.path}
                </div>
                <div className="text-sm font-mono text-text flex items-center gap-2">
                  {w.address}
                  <button className="text-text-dim hover:text-primary transition-colors">
                    <Copy size={12} />
                  </button>
                </div>
              </div>
              <div className="text-right">
                <div className="text-sm font-bold text-primary">
                  {w.balance || "0.00"} ETH
                </div>
                <div className="text-xs text-text-muted">
                  {w.label || "Unlabeled"}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
