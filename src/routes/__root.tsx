import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { Toaster } from "sonner";
import { BootScreen } from "@/components/boot-screen";
import { HintBar } from "@/components/hint-bar";
import { QuickMenu } from "@/components/quick-menu";
import { useFocusBridge } from "@/lib/focus";
import { useInputListeners } from "@/lib/input";
import { useRumbleEvents } from "@/lib/rumble";
import { useSoundEvents } from "@/lib/sound";
import { useLauncherListener, useQuickMenuListener } from "@/lib/launch";
import type { RouterContext } from "@/router";

// Deliberately minimal -- add shared kiosk chrome here as it's built (on-screen keyboard, wallpaper
// background, etc. all lived at this layer in the Electron MVP). useLauncherListener is
// wired up already (drives the launch-phase store off the "launcher:status" push) even though
// LaunchOverlay itself -- the component that actually renders the logo/error phases hasn't been
// ported yet, so a launch currently has no visible loading UI beyond the launch sound; a future
// LaunchOverlay drop-in will pick up this store with no further wiring. QuickMenu (REL-23) is
// further along -- it renders itself already, gated on the same store's quickMenuOpen. Several
// routes and components already call useActionHints (routes/index.tsx, games.tsx,
// systems/$systemId.tsx, profile-switcher.tsx) -- those hints just had nowhere to render until
// HintBar was mounted here.
function RootLayout() {
  useInputListeners();
  useFocusBridge();
  useSoundEvents();
  useRumbleEvents();
  useLauncherListener();
  useQuickMenuListener();

  return (
    <div className="h-svh w-full flex flex-col">
      <BootScreen>
        <Outlet />
      </BootScreen>
      <HintBar />
      <QuickMenu />
      <Toaster theme="dark" />
    </div>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});
