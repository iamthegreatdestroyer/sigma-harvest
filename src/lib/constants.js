export const APP_NAME = "ΣHARVEST";
export const APP_VERSION = "0.1.0";

export const HARVEST_SCORE_THRESHOLDS = {
  EXCELLENT: 80,
  GOOD: 60,
  FAIR: 40,
  POOR: 0,
};

export const DEFAULT_GAS_CEILING_GWEI = {
  ethereum: 30,
  arbitrum: 0.5,
  optimism: 0.1,
  base: 0.1,
  polygon: 100,
  zksync: 0.5,
};

export const MAX_RETRIES = 3;
export const AUTO_LOCK_TIMEOUT_MS = 15 * 60 * 1000; // 15 minutes
