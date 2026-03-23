import { describe, it, expect } from "vitest";
import { CHAINS, CHAIN_LIST } from "../lib/chains";

describe("CHAINS configuration", () => {
  const EXPECTED_CHAINS = ["ethereum", "arbitrum", "optimism", "base", "polygon", "zksync"];

  it("has all 6 expected chains", () => {
    expect(Object.keys(CHAINS)).toEqual(expect.arrayContaining(EXPECTED_CHAINS));
    expect(Object.keys(CHAINS).length).toBe(6);
  });

  it("each chain has required fields", () => {
    for (const [key, chain] of Object.entries(CHAINS)) {
      expect(chain).toHaveProperty("id");
      expect(chain).toHaveProperty("name");
      expect(chain).toHaveProperty("symbol");
      expect(chain).toHaveProperty("color");
      expect(chain).toHaveProperty("rpcUrl");
      expect(chain).toHaveProperty("blockExplorer");
      expect(typeof chain.id).toBe("number");
      expect(typeof chain.name).toBe("string");
      expect(typeof chain.symbol).toBe("string");
    }
  });

  it("chain IDs are unique", () => {
    const ids = Object.values(CHAINS).map((c) => c.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("chain names are non-empty", () => {
    for (const chain of Object.values(CHAINS)) {
      expect(chain.name.length).toBeGreaterThan(0);
    }
  });

  it("rpc URLs are valid HTTP(S) URLs", () => {
    for (const chain of Object.values(CHAINS)) {
      expect(chain.rpcUrl).toMatch(/^https?:\/\//);
    }
  });

  it("block explorer URLs are valid HTTP(S) URLs", () => {
    for (const chain of Object.values(CHAINS)) {
      expect(chain.blockExplorer).toMatch(/^https?:\/\//);
    }
  });

  it("colors are valid hex colors", () => {
    for (const chain of Object.values(CHAINS)) {
      expect(chain.color).toMatch(/^#[0-9A-Fa-f]{6}$/);
    }
  });

  it("Ethereum chain ID is 1", () => {
    expect(CHAINS.ethereum.id).toBe(1);
  });

  it("Arbitrum chain ID is 42161", () => {
    expect(CHAINS.arbitrum.id).toBe(42161);
  });

  it("Optimism chain ID is 10", () => {
    expect(CHAINS.optimism.id).toBe(10);
  });

  it("Base chain ID is 8453", () => {
    expect(CHAINS.base.id).toBe(8453);
  });

  it("Polygon chain ID is 137", () => {
    expect(CHAINS.polygon.id).toBe(137);
  });

  it("zkSync chain ID is 324", () => {
    expect(CHAINS.zksync.id).toBe(324);
  });
});

describe("CHAIN_LIST", () => {
  it("is an array of 6 chains", () => {
    expect(Array.isArray(CHAIN_LIST)).toBe(true);
    expect(CHAIN_LIST.length).toBe(6);
  });

  it("contains same data as CHAINS object", () => {
    for (const chain of CHAIN_LIST) {
      const found = Object.values(CHAINS).find((c) => c.id === chain.id);
      expect(found).toBeDefined();
      expect(found.name).toBe(chain.name);
    }
  });
});
