import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionGranted = false;

/**
 * Initialize notification permissions.
 * Call once at app startup.
 */
export async function initNotifications() {
  try {
    permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const result = await requestPermission();
      permissionGranted = result === "granted";
    }
  } catch {
    // Not running in Tauri context (e.g., tests) — silently ignore
    permissionGranted = false;
  }
  return permissionGranted;
}

/**
 * Send a desktop notification if permissions are granted and notifications are enabled.
 * @param {string} title - Notification title
 * @param {string} body - Notification body text
 * @param {boolean} enabled - Whether notifications are enabled in settings
 */
export function notify(title, body, enabled = true) {
  if (!enabled || !permissionGranted) return;
  try {
    sendNotification({ title, body });
  } catch {
    // Silently fail — don't crash the app over notifications
  }
}

/**
 * Notify about a high-score opportunity discovery.
 * @param {object} opportunity - The discovered opportunity
 * @param {number} threshold - Minimum score to trigger notification
 * @param {boolean} enabled - Whether notifications are enabled
 */
export function notifyHighScore(opportunity, threshold, enabled) {
  const score = opportunity?.harvest_score ?? opportunity?.sigma_score ?? 0;
  if (score < threshold) return;
  notify(
    `🎯 High-Score Opportunity (${score})`,
    `${opportunity.title ?? "Unknown"} on ${opportunity.chain ?? "unknown chain"}`,
    enabled,
  );
}

/**
 * Notify about a claim result.
 * @param {"success"|"failure"} status
 * @param {string} title - Opportunity title
 * @param {string} detail - Additional detail (value, error, etc.)
 * @param {boolean} enabled - Whether notifications are enabled
 */
export function notifyClaim(status, title, detail, enabled) {
  if (status === "success") {
    notify(`✅ Claim Succeeded`, `${title}: ${detail}`, enabled);
  } else {
    notify(`❌ Claim Failed`, `${title}: ${detail}`, enabled);
  }
}
