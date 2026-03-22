/** Supported chain configurations */
export const CHAINS = {
  ethereum: {
    id: 1,
    name: "Ethereum",
    symbol: "ETH",
    color: "#627EEA",
    rpcUrl: "https://eth.llamarpc.com",
    blockExplorer: "https://etherscan.io",
  },
  arbitrum: {
    id: 42161,
    name: "Arbitrum",
    symbol: "ARB",
    color: "#28A0F0",
    rpcUrl: "https://arb1.arbitrum.io/rpc",
    blockExplorer: "https://arbiscan.io",
  },
  optimism: {
    id: 10,
    name: "Optimism",
    symbol: "OP",
    color: "#FF0420",
    rpcUrl: "https://mainnet.optimism.io",
    blockExplorer: "https://optimistic.etherscan.io",
  },
  base: {
    id: 8453,
    name: "Base",
    symbol: "BASE",
    color: "#0052FF",
    rpcUrl: "https://mainnet.base.org",
    blockExplorer: "https://basescan.org",
  },
  polygon: {
    id: 137,
    name: "Polygon",
    symbol: "MATIC",
    color: "#8247E5",
    rpcUrl: "https://polygon-rpc.com",
    blockExplorer: "https://polygonscan.com",
  },
  zksync: {
    id: 324,
    name: "zkSync Era",
    symbol: "ZK",
    color: "#8C8DFC",
    rpcUrl: "https://mainnet.era.zksync.io",
    blockExplorer: "https://explorer.zksync.io",
  },
};

export const CHAIN_LIST = Object.values(CHAINS);
