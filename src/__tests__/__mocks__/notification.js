/**
 * Mock for @tauri-apps/plugin-notification
 * Provides stub functions for unit tests.
 */
let _permissionGranted = true;
let _lastNotification = null;

export function setPermissionGranted(val) {
  _permissionGranted = val;
}

export function getLastNotification() {
  return _lastNotification;
}

export function resetNotificationMock() {
  _permissionGranted = true;
  _lastNotification = null;
}

export async function isPermissionGranted() {
  return _permissionGranted;
}

export async function requestPermission() {
  return _permissionGranted ? "granted" : "denied";
}

export function sendNotification(options) {
  _lastNotification = options;
}
