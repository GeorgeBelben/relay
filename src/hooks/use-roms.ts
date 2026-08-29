import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export type Rom = {
  id: string;
  system_id: string;
  path: string;
  crc32: string | null;
  size_bytes: number | null;
  discs: string | null;
  status: string;
  created_at: number;
  updated_at: number;
};

export type NewRom = {
  systemId: string;
  path: string;
  crc32: string | null;
  sizeBytes: number | null;
  discs: string | null;
};

const romsKey = ["roms"] as const;

export function useRoms() {
  return useQuery({
    queryKey: romsKey,
    queryFn: () => invoke<Rom[]>("list_roms"),
  });
}

export function useRom(id: string) {
  return useQuery({
    queryKey: [...romsKey, id],
    queryFn: () => invoke<Rom | null>("get_rom", { id }),
  });
}

export function useCreateRom() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (rom: NewRom) => invoke<Rom>("create_rom", rom),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: romsKey });
    },
  });
}

export function useUpdateRom() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (rom: NewRom & { id: string }) => invoke<Rom>("update_rom", rom),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: romsKey });
    },
  });
}

export function useDeleteRom() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("delete_rom", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: romsKey });
    },
  });
}
