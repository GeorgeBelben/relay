import { useEffect } from "react";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Toaster } from "sonner";
import { BootScreen } from "@/components/boot-screen";
import type { RouterContext } from "@/router";

// The window starts hidden (tauri.conf.json) so there's no white flash before the webview has
// anything to paint -- reveal it once the boot screen's first frame is actually on screen, not
// before. Two rAFs: the first fires once the DOM update from mount has been committed, the
// second once that's actually been painted.
function useRevealWindowOnFirstPaint() {
  useEffect(() => {
    let raf1 = 0;
    let raf2 = 0;
    raf1 = requestAnimationFrame(() => {
      raf2 = requestAnimationFrame(() => {
        getCurrentWindow().show();
      });
    });
    return () => {
      cancelAnimationFrame(raf1);
      cancelAnimationFrame(raf2);
    };
  }, []);
}

// Deliberately minimal -- add shared kiosk chrome here as it's built (on-screen keyboard,
// focus/input listeners, etc. all lived at this layer in the Electron MVP).
function RootLayout() {
  useRevealWindowOnFirstPaint();

  return (
    <div className="h-svh w-full flex flex-col">
      <BootScreen>
        <Outlet />
      </BootScreen>
      <Toaster theme="dark" />
      {import.meta.env.DEV && <TanStackRouterDevtools />}
    </div>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});
