import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";

// The systems Relay supports (NES, SNES, etc.) are a fixed set defined in Rust code
// (src-tauri/src/systems.rs), not user-editable data -- these are read-only queries, no
// create/update/delete.
export type System = {
  id: string;
  name: string;
  extensions: string[];
  retroarch_core: string | null;
  standalone_binary: string | null;
};

const systemsKey = ["systems"] as const;

export function useSystems() {
  return useQuery({
    queryKey: systemsKey,
    queryFn: () => invoke<System[]>("list_systems"),
  });
}

export function useSystem(id: string) {
  return useQuery({
    queryKey: [...systemsKey, id],
    queryFn: () => invoke<System | null>("get_system", { id }),
  });
}
