import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type DaemonConnectionStatus = "checking" | "connected" | "disconnected";

export function useDaemonConnection(): DaemonConnectionStatus {
  const [status, setStatus] = useState<DaemonConnectionStatus>("checking");

  useEffect(() => {
    invoke<string>("check_daemon_connection")
      .then(() => setStatus("connected"))
      .catch(() => setStatus("disconnected"));
  }, []);

  return status;
}
