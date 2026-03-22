/** Format a Wei value to ETH string */
export function formatEth(wei) {
  if (!wei) return "0.00";
  return (Number(wei) / 1e18).toFixed(4);
}

/** Truncate an address for display */
export function truncateAddress(addr, chars = 6) {
  if (!addr) return "";
  return `${addr.slice(0, chars + 2)}...${addr.slice(-chars)}`;
}

/** Format USD value */
export function formatUsd(value) {
  if (value == null) return "$0.00";
  return `$${Number(value).toFixed(2)}`;
}

/** Format a gwei value */
export function formatGwei(gwei) {
  if (gwei == null) return "—";
  return `${Number(gwei).toFixed(1)} gwei`;
}
