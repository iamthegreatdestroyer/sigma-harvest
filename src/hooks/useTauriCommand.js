import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback } from "react";

/**
 * Hook to invoke a Tauri backend command.
 * Returns [data, loading, error, execute].
 */
export function useTauriCommand(command) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const execute = useCallback(
    async (args = {}) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke(command, args);
        setData(result);
        return result;
      } catch (err) {
        setError(err);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [command]
  );

  return [data, loading, error, execute];
}
