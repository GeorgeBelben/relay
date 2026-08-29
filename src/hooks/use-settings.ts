import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export function useSetting(key: string) {
  return useQuery({
    queryKey: ["settings", key],
    queryFn: () => invoke<string | null>("get_setting", { key }),
  });
}

export function useSetSetting(key: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (value: string) => invoke<void>("set_setting", { key, value }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings", key] });
    },
  });
}
