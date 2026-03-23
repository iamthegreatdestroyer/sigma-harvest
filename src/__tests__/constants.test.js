import { describe, it, expect } from "vitest";
import {
  APP_NAME,
  APP_VERSION,
  HARVEST_SCORE_THRESHOLDS,
  DEFAULT_GAS_CEILING_GWEI,
  MAX_RETRIES,
  AUTO_LOCK_TIMEOUT_MS,
} from "../lib/constants";

describe("APP constants", () => {
  it("APP_NAME is ΣHARVEST", () => {
    expect(APP_NAME).toBe("ΣHARVEST");
  });

  it("APP_VERSION follows semver format", () => {
    expect(APP_VERSION).toMatch(/^\d+\.\d+\.\d+$/);
  });
});

describe("HARVEST_SCORE_THRESHOLDS", () => {
  it("has 4 threshold levels", () => {
    expect(Object.keys(HARVEST_SCORE_THRESHOLDS).length).toBe(4);
  });

  it("EXCELLENT is highest", () => {
    expect(HARVEST_SCORE_THRESHOLDS.EXCELLENT).toBe(80);
  });

  it("GOOD is second highest", () => {
    expect(HARVEST_SCORE_THRESHOLDS.GOOD).toBe(60);
  });

  it("FAIR is third", () => {
    expect(HARVEST_SCORE_THRESHOLDS.FAIR).toBe(40);
  });

  it("POOR is zero", () => {
    expect(HARVEST_SCORE_THRESHOLDS.POOR).toBe(0);
  });

  it("thresholds are in descending order", () => {
    expect(HARVEST_SCORE_THRESHOLDS.EXCELLENT).toBeGreaterThan(HARVEST_SCORE_THRESHOLDS.GOOD);
    expect(HARVEST_SCORE_THRESHOLDS.GOOD).toBeGreaterThan(HARVEST_SCORE_THRESHOLDS.FAIR);
    expect(HARVEST_SCORE_THRESHOLDS.FAIR).toBeGreaterThan(HARVEST_SCORE_THRESHOLDS.POOR);
  });

  it("all thresholds are between 0 and 100", () => {
    for (const val of Object.values(HARVEST_SCORE_THRESHOLDS)) {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(100);
    }
  });
});

describe("DEFAULT_GAS_CEILING_GWEI", () => {
  it("has ceilings for all 6 chains", () => {
    const chains = ["ethereum", "arbitrum", "optimism", "base", "polygon", "zksync"];
    for (const chain of chains) {
      expect(DEFAULT_GAS_CEILING_GWEI).toHaveProperty(chain);
      expect(typeof DEFAULT_GAS_CEILING_GWEI[chain]).toBe("number");
    }
  });

  it("Ethereum ceiling is reasonable (30 gwei)", () => {
    expect(DEFAULT_GAS_CEILING_GWEI.ethereum).toBe(30);
  });

  it("L2 ceilings are much lower than L1", () => {
    expect(DEFAULT_GAS_CEILING_GWEI.arbitrum).toBeLessThan(DEFAULT_GAS_CEILING_GWEI.ethereum);
    expect(DEFAULT_GAS_CEILING_GWEI.optimism).toBeLessThan(DEFAULT_GAS_CEILING_GWEI.ethereum);
    expect(DEFAULT_GAS_CEILING_GWEI.base).toBeLessThan(DEFAULT_GAS_CEILING_GWEI.ethereum);
  });

  it("all ceilings are positive", () => {
    for (const val of Object.values(DEFAULT_GAS_CEILING_GWEI)) {
      expect(val).toBeGreaterThan(0);
    }
  });
});

describe("MAX_RETRIES", () => {
  it("is 3", () => {
    expect(MAX_RETRIES).toBe(3);
  });

  it("is a positive integer", () => {
    expect(Number.isInteger(MAX_RETRIES)).toBe(true);
    expect(MAX_RETRIES).toBeGreaterThan(0);
  });
});

describe("AUTO_LOCK_TIMEOUT_MS", () => {
  it("is 15 minutes in milliseconds", () => {
    expect(AUTO_LOCK_TIMEOUT_MS).toBe(15 * 60 * 1000);
    expect(AUTO_LOCK_TIMEOUT_MS).toBe(900000);
  });

  it("is a positive number", () => {
    expect(AUTO_LOCK_TIMEOUT_MS).toBeGreaterThan(0);
  });
});
