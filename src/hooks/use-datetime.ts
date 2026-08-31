import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Mirrors src-tauri/src/system/datetime.rs's DateTimeStatus.
export type DateTimeStatus = {
  timezone: string;
  ntp_enabled: boolean;
  ntp_synchronized: boolean;
};

const STATUS_KEY = ["datetime", "status"] as const;

export function useDateTimeStatus() {
  return useQuery({
    queryKey: STATUS_KEY,
    queryFn: () => invoke<DateTimeStatus>("get_datetime_status"),
  });
}

// Never changes at runtime (the OS's own tzdata) -- a long staleTime avoids re-fetching the full
// IANA list on every mount of the timezone picker.
export function useTimezones() {
  return useQuery({
    queryKey: ["datetime", "timezones"],
    queryFn: () => invoke<string[]>("list_timezones"),
    staleTime: Infinity,
  });
}

function useInvalidateDateTimeStatus() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: STATUS_KEY });
}

export function useSetTimezone() {
  const invalidate = useInvalidateDateTimeStatus();
  return useMutation({
    mutationFn: (timezone: string) => invoke<void>("set_timezone", { timezone }),
    onSuccess: invalidate,
  });
}

export function useSetNtpEnabled() {
  const invalidate = useInvalidateDateTimeStatus();
  return useMutation({
    mutationFn: (enabled: boolean) => invoke<void>("set_ntp_enabled", { enabled }),
    onSuccess: invalidate,
  });
}

// dateTime must be a format `timedatectl set-time` accepts, e.g. "2026-08-27 21:15:00". Only
// meaningful while NTP is off (see set_time's own doc comment) -- the frontend only shows this
// control in that state.
export function useSetTime() {
  const invalidate = useInvalidateDateTimeStatus();
  return useMutation({
    mutationFn: (dateTime: string) => invoke<void>("set_time", { dateTime }),
    onSuccess: invalidate,
  });
}
