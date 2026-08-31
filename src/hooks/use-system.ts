import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery } from "@tanstack/react-query";

export function useUsername(): string | undefined {
  const { data } = useQuery({ queryKey: ["system", "username"], queryFn: () => invoke<string>("get_username") });
  return data;
}

// The kiosk systemd unit runs this app as the only thing on screen -- quitting is what actually
// drops back to the console's login/CLI, not a browser-style "close tab" (commands::system::quit).
export function useQuit() {
  return useMutation({
    mutationFn: () => invoke<void>("quit"),
  });
}
