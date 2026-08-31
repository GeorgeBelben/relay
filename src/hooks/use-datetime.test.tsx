import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import {
  useDateTimeStatus,
  useSetNtpEnabled,
  useSetTime,
  useSetTimezone,
  useTimezones,
  type DateTimeStatus,
} from "./use-datetime";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useDateTimeStatus", () => {
  it("returns the status from get_datetime_status", async () => {
    const status: DateTimeStatus = { timezone: "Europe/London", ntp_enabled: true, ntp_synchronized: true };
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_datetime_status") return Promise.resolve(status);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useDateTimeStatus(), { wrapper });

    await waitFor(() => expect(result.current.data).toEqual(status));
  });
});

describe("useTimezones", () => {
  it("returns the list from list_timezones", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_timezones") return Promise.resolve(["Europe/London", "UTC"]);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useTimezones(), { wrapper });

    await waitFor(() => expect(result.current.data).toEqual(["Europe/London", "UTC"]));
  });
});

describe("useSetTimezone / useSetNtpEnabled / useSetTime cache invalidation", () => {
  it("refetches datetime status after each mutation", async () => {
    let status: DateTimeStatus = { timezone: "UTC", ntp_enabled: true, ntp_synchronized: true };

    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_datetime_status") return Promise.resolve(status);
      if (cmd === "set_timezone") {
        status = { ...status, timezone: args!.timezone as string };
        return Promise.resolve(undefined);
      }
      if (cmd === "set_ntp_enabled") {
        status = { ...status, ntp_enabled: args!.enabled as boolean };
        return Promise.resolve(undefined);
      }
      if (cmd === "set_time") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(
      () => ({
        status: useDateTimeStatus(),
        setTimezone: useSetTimezone(),
        setNtpEnabled: useSetNtpEnabled(),
        setTime: useSetTime(),
      }),
      { wrapper: localWrapper },
    );

    await waitFor(() => expect(result.current.status.data?.timezone).toBe("UTC"));

    await act(async () => {
      await result.current.setTimezone.mutateAsync("Europe/London");
    });
    expect(invokeMock).toHaveBeenCalledWith("set_timezone", { timezone: "Europe/London" });
    await waitFor(() => expect(result.current.status.data?.timezone).toBe("Europe/London"));

    await act(async () => {
      await result.current.setNtpEnabled.mutateAsync(false);
    });
    expect(invokeMock).toHaveBeenCalledWith("set_ntp_enabled", { enabled: false });
    await waitFor(() => expect(result.current.status.data?.ntp_enabled).toBe(false));

    await act(async () => {
      await result.current.setTime.mutateAsync("2026-08-27 21:15:00");
    });
    expect(invokeMock).toHaveBeenCalledWith("set_time", { dateTime: "2026-08-27 21:15:00" });
  });
});
