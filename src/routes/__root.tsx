import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { Toaster } from "sonner";
import { BootScreen } from "@/components/boot-screen";
import { HintBar } from "@/components/hint-bar";
import { LaunchOverlay } from "@/components/launch-overlay";
import { QuickMenu } from "@/components/quick-menu";
import { SystemMenu } from "@/components/system-menu";
import { useFocusBridge } from "@/lib/focus";
import { useInputListeners } from "@/lib/input";
import { useRumbleEvents } from "@/lib/rumble";
import { useSoundEvents } from "@/lib/sound";
import { useLauncherListener, useQuickMenuListener } from "@/lib/launch";
import { useSystemMenuListener } from "@/lib/system-menu";
import type { RouterContext } from "@/router";

// Deliberately minimal -- add shared kiosk chrome here as it's built (on-screen keyboard, wallpaper
// background, etc. all lived at this layer in the Electron MVP). useLauncherListener drives the
// launch-phase store off the "launcher:status" push; LaunchOverlay renders whatever phase it lands
// on (fade to black + pulsing logo while launching, a fade-back-out once "playing", or an error
// message) -- see launch-overlay.tsx. QuickMenu (REL-137) is the in-game counterpart -- it renders
// itself already, gated on the same store's quickMenuOpen, and owns "menu" while a game is
// playing. SystemMenu (REL-138) is the browsing-mode counterpart to that -- a separate
// component/store, opened by the distinct "home" action instead, and replaces both the header's
// old settings icon and the profile-switcher avatar as focus targets (see header.tsx). Several
// routes and components already call useActionHints (routes/index.tsx, games.tsx,
// systems/$systemId.tsx, system-menu.tsx) -- those hints just had nowhere to render until
// HintBar was mounted here.
function RootLayout() {
  useInputListeners();
  useFocusBridge();
  useSoundEvents();
  useRumbleEvents();
  useLauncherListener();
  useQuickMenuListener();
  useSystemMenuListener();

  return (
    <div className="h-svh w-full flex flex-col">
      <BootScreen>
        <Outlet />
      </BootScreen>
      <HintBar />
      <LaunchOverlay />
      <QuickMenu />
      <SystemMenu />
      <Toaster theme="dark" />
    </div>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});
