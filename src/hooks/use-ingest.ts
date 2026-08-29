import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useMutation, useQueryClient } from "@tanstack/react-query";

export type ScanStatus =
  | { state: "idle" }
  | { state: "scanning-files" }
  | { state: "enriching-art"; current: number; total: number }
  | { state: "done" }
  | { state: "error"; message: string };

// Pulls the current status on mount -- a scan that finishes fast (nothing new since last boot)
// can complete before this hook's listener attaches, so relying on the push event alone would
// leave a caller stuck showing "idle" forever -- then stays live via the "scanner:status" push
// (commands::ingestion::rescan_library) for everything after.
export function useScanStatus() {
  const [status, setStatus] = useState<ScanStatus>({ state: "idle" });
  const queryClient = useQueryClient();

  useEffect(() => {
    let cancelled = false;

    invoke<ScanStatus>("get_scan_status").then((initial) => {
      if (!cancelled) setStatus(initial);
    });

    const unlisten = listen<ScanStatus>("scanner:status", (event) => {
      setStatus(event.payload);
      if (event.payload.state === "done" || event.payload.state === "error") {
        queryClient.invalidateQueries({ queryKey: ["games"] });
        queryClient.invalidateQueries({ queryKey: ["roms"] });
      }
    });

    return () => {
      cancelled = true;
      unlisten.then((f) => f());
    };
  }, [queryClient]);

  return status;
}

export function useRescan() {
  return useMutation({
    mutationFn: () => invoke<void>("rescan_library"),
  });
}
