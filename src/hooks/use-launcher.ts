import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useMutation, useQueryClient } from "@tanstack/react-query";

export type LauncherStatus =
  | { state: "idle" }
  | { state: "launching" }
  | { state: "running" }
  | { state: "exited" }
  | { state: "crashed"; exit_code: number | null; signal: number | null }
  | { state: "error"; message: string };

export type LogLine = {
  stream: "stdout" | "stderr";
  line: string;
  timestamp: number;
};

// Pulls the current status on mount -- a launch that finishes fast (an emulator that immediately
// errors out) can complete before this hook's listener attaches, so relying on the push event
// alone would leave a caller stuck showing "idle" forever -- then stays live via the
// "launcher:status" push (commands::emulator::launch_game) for everything after.
export function useLauncherStatus() {
  const [status, setStatus] = useState<LauncherStatus>({ state: "idle" });

  useEffect(() => {
    let cancelled = false;

    invoke<LauncherStatus>("get_launcher_status").then((initial) => {
      if (!cancelled) setStatus(initial);
    });

    const unlisten = listen<LauncherStatus>("launcher:status", (event) => {
      setStatus(event.payload);
    });

    return () => {
      cancelled = true;
      unlisten.then((f) => f());
    };
  }, []);

  return status;
}

// Accumulates stdout/stderr lines pushed during the current (or most recent) launch. Cleared on
// the next "launching" transition so a fresh launch doesn't show a stale previous run's output.
export function useLauncherLog() {
  const [lines, setLines] = useState<LogLine[]>([]);

  useEffect(() => {
    const unlistenLog = listen<LogLine>("launcher:log", (event) => {
      setLines((prev) => [...prev, event.payload]);
    });
    const unlistenStatus = listen<LauncherStatus>("launcher:status", (event) => {
      if (event.payload.state === "launching") setLines([]);
    });

    return () => {
      unlistenLog.then((f) => f());
      unlistenStatus.then((f) => f());
    };
  }, []);

  return lines;
}

export function useLaunchGame() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (gameId: string) => invoke<void>("launch_game", { gameId }),
    onSuccess: () => {
      // A completed launch may have changed nothing about the games list, but RA/playtime
      // metadata eventually will -- invalidating here keeps this consistent with the other
      // mutations rather than requiring a caller to remember to do it themselves.
      queryClient.invalidateQueries({ queryKey: ["games"] });
    },
  });
}

export function useKillGame() {
  return useMutation({
    mutationFn: () => invoke<void>("kill_game"),
  });
}

// Both fire-and-forget over RetroArch's UDP command port (REL-23) -- a silent no-op rather than
// an error for a standalone-emulator game (Dolphin/PCSX2/yabause-qt have no such interface), so
// callers don't need to check which kind of game is running before calling.
export function usePauseToggleGame() {
  return useMutation({
    mutationFn: () => invoke<void>("pause_toggle_game"),
  });
}

export function useSaveStateGame() {
  return useMutation({
    mutationFn: () => invoke<void>("save_state_game"),
  });
}
