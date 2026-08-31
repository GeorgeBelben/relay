import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Toaster } from "sonner";
import { BootScreen } from "@/components/boot-screen";
import { useFocusBridge } from "@/lib/focus";
import { useInputListeners } from "@/lib/input";
import { useRumbleEvents } from "@/lib/rumble";
import { useSoundEvents } from "@/lib/sound";
import type { RouterContext } from "@/router";

// Deliberately minimal -- add shared kiosk chrome here as it's built (on-screen keyboard, wallpaper
// background, hint bar, launch overlay, etc. all lived at this layer in the Electron MVP).
function RootLayout() {
  useInputListeners();
  useFocusBridge();
  useSoundEvents();
  useRumbleEvents();

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
