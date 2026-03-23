import { describe, it, expect } from "vitest";
import { formatEth, truncateAddress, formatUsd, formatGwei } from "../lib/formatters";

describe("formatEth", () => {
  it("converts wei to ETH with 4 decimals", () => {
    expect(formatEth(1e18)).toBe("1.0000");
  });

  it("handles zero", () => {
    expect(formatEth(0)).toBe("0.00");
  });

  it("handles null/undefined", () => {
    expect(formatEth(null)).toBe("0.00");
    expect(formatEth(undefined)).toBe("0.00");
  });

  it("handles fractional wei", () => {
    expect(formatEth(5e17)).toBe("0.5000");
  });

  it("handles large values", () => {
    expect(formatEth(1000e18)).toBe("1000.0000");
  });

  it("handles string numbers", () => {
    expect(formatEth("1000000000000000000")).toBe("1.0000");
  });
});

describe("truncateAddress", () => {
  const ADDR = "0x1234567890abcdef1234567890abcdef12345678";

  it("truncates with default chars=6", () => {
    const result = truncateAddress(ADDR);
    expect(result).toBe("0x123456...345678");
    expect(result.length).toBeLessThan(ADDR.length);
  });

  it("truncates with custom chars", () => {
    const result = truncateAddress(ADDR, 4);
    expect(result).toBe("0x1234...5678");
  });

  it("handles null/undefined", () => {
    expect(truncateAddress(null)).toBe("");
    expect(truncateAddress(undefined)).toBe("");
  });

  it("handles empty string", () => {
    expect(truncateAddress("")).toBe("");
  });

  it("preserves 0x prefix", () => {
    const result = truncateAddress(ADDR);
    expect(result.startsWith("0x")).toBe(true);
  });

  it("contains ellipsis", () => {
    expect(truncateAddress(ADDR)).toContain("...");
  });
});

describe("formatUsd", () => {
  it("formats basic USD value", () => {
    expect(formatUsd(1234.5)).toBe("$1234.50");
  });

  it("handles zero", () => {
    expect(formatUsd(0)).toBe("$0.00");
  });

  it("handles null/undefined", () => {
    expect(formatUsd(null)).toBe("$0.00");
    expect(formatUsd(undefined)).toBe("$0.00");
  });

  it("formats small values", () => {
    expect(formatUsd(0.01)).toBe("$0.01");
  });

  it("rounds to 2 decimal places", () => {
    expect(formatUsd(1.999)).toBe("$2.00");
  });

  it("handles negative values", () => {
    expect(formatUsd(-5.5)).toBe("$-5.50");
  });
});

describe("formatGwei", () => {
  it("formats gwei value", () => {
    expect(formatGwei(15.3)).toBe("15.3 gwei");
  });

  it("handles null/undefined", () => {
    expect(formatGwei(null)).toBe("—");
    expect(formatGwei(undefined)).toBe("—");
  });

  it("formats zero", () => {
    expect(formatGwei(0)).toBe("0.0 gwei");
  });

  it("rounds to 1 decimal", () => {
    expect(formatGwei(15.34)).toBe("15.3 gwei");
  });

  it("handles very small gwei", () => {
    expect(formatGwei(0.01)).toBe("0.0 gwei");
  });
});
