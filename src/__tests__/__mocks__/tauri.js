/**
 * Mock for @tauri-apps/api/core
 * Provides stub invoke() for unit tests.
 */
const responses = new Map();

export function mockInvokeResponse(command, response) {
  responses.set(command, response);
}

export function clearMockResponses() {
  responses.clear();
}

export async function invoke(command, args) {
  if (responses.has(command)) {
    const resp = responses.get(command);
    if (typeof resp === "function") return resp(args);
    if (resp instanceof Error) throw resp;
    return resp;
  }
  throw new Error(`Unmocked Tauri command: ${command}`);
}
