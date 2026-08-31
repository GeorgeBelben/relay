import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { BootScreen } from "./boot-screen";
import type { ScanStatus } from "@/hooks/use-ingest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

type Listener = (event: { payload: ScanStatus }) => void;
let capturedListener: Listener | undefined;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_event: string, handler: Listener) => {
    capturedListener = handler;
    return Promise.resolve(vi.fn());
  },
}));

const toastErrorMock = vi.fn();
vi.mock("sonner", () => ({
  toast: { error: (...args: unknown[]) => toastErrorMock(...args) },
}));

function renderBootScreen() {
  const queryClient = new QueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      <BootScreen>
        <p>App content</p>
      </BootScreen>
    </QueryClientProvider>,
  );
}

describe("BootScreen", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    toastErrorMock.mockReset();
    capturedListener = undefined;
  });

  it("triggers a rescan on mount and holds the app behind a logo/loader until done", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_scan_status") return Promise.resolve({ state: "idle" });
      if (cmd === "rescan_library") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    renderBootScreen();

    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("rescan_library"));
    expect(screen.getByText("Starting up…")).toBeInTheDocument();
    expect(screen.queryByText("App content")).not.toBeInTheDocument();

    capturedListener!({ payload: { state: "scanning-files" } });
    await waitFor(() => expect(screen.getByText("Scanning ROM library…")).toBeInTheDocument());
    expect(screen.queryByText("App content")).not.toBeInTheDocument();

    capturedListener!({ payload: { state: "enriching-art", current: 3, total: 10 } });
    await waitFor(() => expect(screen.getByText("Fetching artwork… (3/10)")).toBeInTheDocument());

    capturedListener!({ payload: { state: "done" } });
    await waitFor(() => expect(screen.getByText("App content")).toBeInTheDocument());
    expect(screen.queryByText("Fetching artwork… (3/10)")).not.toBeInTheDocument();
  });

  it("falls through to the app on scan error, with a toast", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_scan_status") return Promise.resolve({ state: "idle" });
      if (cmd === "rescan_library") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    renderBootScreen();
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("rescan_library"));

    capturedListener!({ payload: { state: "error", message: "disk unreadable" } });

    await waitFor(() => expect(screen.getByText("App content")).toBeInTheDocument());
    expect(toastErrorMock).toHaveBeenCalledWith("Library scan failed: disk unreadable");
  });

  it("does not trigger a second rescan once already triggered", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_scan_status") return Promise.resolve({ state: "idle" });
      if (cmd === "rescan_library") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    renderBootScreen();
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("rescan_library"));

    capturedListener!({ payload: { state: "scanning-files" } });
    await waitFor(() => expect(screen.getByText("Scanning ROM library…")).toBeInTheDocument());

    expect(invokeMock.mock.calls.filter(([cmd]) => cmd === "rescan_library")).toHaveLength(1);
  });
});
