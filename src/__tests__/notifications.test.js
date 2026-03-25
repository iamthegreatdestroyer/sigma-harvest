import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  resetNotificationMock,
  setPermissionGranted,
  getLastNotification,
} from "./__mocks__/notification";
import {
  initNotifications,
  notify,
  notifyHighScore,
  notifyClaim,
} from "../lib/notifications";

describe("notifications", () => {
  beforeEach(() => {
    resetNotificationMock();
  });

  // ── initNotifications ─────────────────────────────────

  it("initializes and returns true when permission granted", async () => {
    setPermissionGranted(true);
    const result = await initNotifications();
    expect(result).toBe(true);
  });

  it("requests permission when not already granted", async () => {
    setPermissionGranted(false);
    const result = await initNotifications();
    expect(result).toBe(false);
  });

  // ── notify ────────────────────────────────────────────

  it("sends notification when enabled and permitted", async () => {
    await initNotifications();
    notify("Title", "Body", true);
    expect(getLastNotification()).toEqual({ title: "Title", body: "Body" });
  });

  it("does not send when enabled is false", async () => {
    await initNotifications();
    notify("Title", "Body", false);
    expect(getLastNotification()).toBeNull();
  });

  it("does not send when permission not granted", async () => {
    setPermissionGranted(false);
    await initNotifications();
    notify("Title", "Body", true);
    expect(getLastNotification()).toBeNull();
  });

  it("defaults enabled to true", async () => {
    await initNotifications();
    notify("Title", "Body");
    expect(getLastNotification()).toEqual({ title: "Title", body: "Body" });
  });

  // ── notifyHighScore ───────────────────────────────────

  it("sends notification for high harvest_score", async () => {
    await initNotifications();
    notifyHighScore({ harvest_score: 85, title: "Alpha Drop", chain: "ethereum" }, 70, true);
    const n = getLastNotification();
    expect(n).not.toBeNull();
    expect(n.title).toContain("85");
    expect(n.body).toContain("Alpha Drop");
    expect(n.body).toContain("ethereum");
  });

  it("sends notification for high sigma_score", async () => {
    await initNotifications();
    notifyHighScore({ sigma_score: 90, title: "Beta", chain: "arbitrum" }, 70, true);
    expect(getLastNotification()).not.toBeNull();
  });

  it("does not notify below threshold", async () => {
    await initNotifications();
    notifyHighScore({ harvest_score: 50, title: "Low", chain: "ethereum" }, 70, true);
    expect(getLastNotification()).toBeNull();
  });

  it("does not notify when disabled", async () => {
    await initNotifications();
    notifyHighScore({ harvest_score: 99, title: "High", chain: "base" }, 70, false);
    expect(getLastNotification()).toBeNull();
  });

  it("handles null opportunity gracefully", async () => {
    await initNotifications();
    notifyHighScore(null, 70, true);
    expect(getLastNotification()).toBeNull();
  });

  it("handles missing fields gracefully", async () => {
    await initNotifications();
    notifyHighScore({ harvest_score: 80 }, 70, true);
    const n = getLastNotification();
    expect(n.body).toContain("Unknown");
    expect(n.body).toContain("unknown chain");
  });

  // ── notifyClaim ───────────────────────────────────────

  it("sends success notification", async () => {
    await initNotifications();
    notifyClaim("success", "Test Claim", "$50.00", true);
    const n = getLastNotification();
    expect(n.title).toContain("Succeeded");
    expect(n.body).toContain("Test Claim");
    expect(n.body).toContain("$50.00");
  });

  it("sends failure notification", async () => {
    await initNotifications();
    notifyClaim("failure", "Test Claim", "Out of gas", true);
    const n = getLastNotification();
    expect(n.title).toContain("Failed");
    expect(n.body).toContain("Out of gas");
  });

  it("does not notify claims when disabled", async () => {
    await initNotifications();
    notifyClaim("success", "X", "Y", false);
    expect(getLastNotification()).toBeNull();
  });
});
