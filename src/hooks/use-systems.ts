import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export type System = {
  id: string;
  name: string;
  extensions: string;
  retroarch_core: string | null;
  standalone_binary: string | null;
};

export type NewSystem = {
  id: string;
  name: string;
  extensions: string;
  retroarchCore: string | null;
  standaloneBinary: string | null;
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

export function useCreateSystem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (system: NewSystem) => invoke<System>("create_system", system),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: systemsKey });
    },
  });
}

export function useUpdateSystem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (system: NewSystem) => invoke<System>("update_system", system),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: systemsKey });
    },
  });
}

export function useDeleteSystem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("delete_system", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: systemsKey });
    },
  });
}
