import { GitBranch } from "lucide-react";

export default function WalletTree({ wallets = [] }) {
  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2 text-xs text-text-muted mb-2">
        <GitBranch size={12} />
        <span>HD Derivation Tree</span>
      </div>
      {wallets.length === 0 ? (
        <p className="text-text-dim text-xs italic pl-4">No wallets derived</p>
      ) : (
        wallets.map((w, i) => (
          <div key={w.address} className="flex items-center gap-2 pl-4 text-xs">
            <span className="text-text-dim">
              {i === wallets.length - 1 ? "└─" : "├─"}
            </span>
            <span className="text-text-muted font-mono">{w.path}</span>
            <span className="text-text truncate">{w.address}</span>
          </div>
        ))
      )}
    </div>
  );
}
