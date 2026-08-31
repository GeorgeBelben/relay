import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Toaster } from "sonner";
import { BootScreen } from "@/components/boot-screen";
import { useFocusBridge } from "@/lib/focus";
import { useInputListeners } from "@/lib/input";
import { useRumbleEvents } from "@/lib/rumble";
import { useSoundEvents } from "@/lib/sound";
import { useLauncherListener } from "@/lib/launch";
import type { RouterContext } from "@/router";

// Deliberately minimal -- add shared kiosk chrome here as it's built (on-screen keyboard, wallpaper
// background, hint bar, etc. all lived at this layer in the Electron MVP). useLauncherListener is
// wired up already (drives the launch-phase store off the "launcher:status" push) even though
// LaunchOverlay itself -- the component that actually renders that phase -- hasn't been ported
// yet, so a launch currently has no visible loading UI beyond the launch sound; a future
// LaunchOverlay drop-in will pick up this store with no further wiring.
function RootLayout() {
  useInputListeners();
  useFocusBridge();
  useSoundEvents();
  useRumbleEvents();
  useLauncherListener();

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
