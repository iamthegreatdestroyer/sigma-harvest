import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import SparklineChart from "../components/SparklineChart";

// Mock ResizeObserver which Recharts needs
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver = ResizeObserverMock;

describe("SparklineChart", () => {
  const sampleData = [
    { date: "2026-03-20", value: 10.5 },
    { date: "2026-03-21", value: 25.0 },
    { date: "2026-03-22", value: 15.3 },
    { date: "2026-03-23", value: 42.0 },
    { date: "2026-03-24", value: 30.0 },
  ];

  // ── Empty state ───────────────────────────────────────

  it("renders 'No data' when data is empty", () => {
    render(<SparklineChart data={[]} />);
    expect(screen.getByText("No data")).toBeTruthy();
  });

  it("renders 'No data' with default props", () => {
    render(<SparklineChart />);
    expect(screen.getByText("No data")).toBeTruthy();
  });

  // ── With data ─────────────────────────────────────────

  it("renders without crashing when data is provided", () => {
    const { container } = render(<SparklineChart data={sampleData} />);
    // Recharts renders an SVG or a container with the chart
    expect(container.firstChild).toBeTruthy();
    // Should NOT show "No data"
    expect(screen.queryByText("No data")).toBeNull();
  });

  it("accepts custom color prop", () => {
    const { container } = render(
      <SparklineChart data={sampleData} color="#ff0000" />,
    );
    expect(container.firstChild).toBeTruthy();
  });

  it("accepts custom height prop", () => {
    const { container } = render(
      <SparklineChart data={sampleData} height={80} />,
    );
    expect(container.firstChild).toBeTruthy();
  });

  it("accepts custom dataKey prop", () => {
    const customData = [
      { date: "2026-03-20", amount: 10.5 },
      { date: "2026-03-21", amount: 25.0 },
    ];
    const { container } = render(
      <SparklineChart data={customData} dataKey="amount" />,
    );
    expect(container.firstChild).toBeTruthy();
  });

  // ── Edge cases ────────────────────────────────────────

  it("renders with single data point", () => {
    const { container } = render(
      <SparklineChart data={[{ date: "2026-03-25", value: 42 }]} />,
    );
    expect(container.firstChild).toBeTruthy();
    expect(screen.queryByText("No data")).toBeNull();
  });

  it("handles zero values", () => {
    const zeroData = [
      { date: "2026-03-20", value: 0 },
      { date: "2026-03-21", value: 0 },
    ];
    const { container } = render(<SparklineChart data={zeroData} />);
    expect(container.firstChild).toBeTruthy();
  });

  it("handles negative values", () => {
    const negData = [
      { date: "2026-03-20", value: -5.0 },
      { date: "2026-03-21", value: 10.0 },
    ];
    const { container } = render(<SparklineChart data={negData} />);
    expect(container.firstChild).toBeTruthy();
  });
});
