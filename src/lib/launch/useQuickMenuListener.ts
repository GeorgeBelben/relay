import { useNavEvent } from "@/lib/input";
import { useLaunchStore } from "./store";

// Mounted once at the app root (see __root.tsx), alongside useLauncherListener. Home-button-style
// quick menu (REL-23): the "menu" action (gamepad Y/Triangle/X north face button, or M on
// keyboard -- see lib/input/keyboard.ts) toggles the quick menu open/closed, but only matters
// while a game is actually playing. Everywhere else, "menu" still belongs to whichever tile is
// focused (see focus/bridge.ts's handleMenu() dispatch, which explicitly steps aside for this
// listener while phase is "playing" -- see its own comment for why that guard is needed).
//
// Whether this listener's events even reach the webview while an emulator has focus/input
// exclusivity is unverified -- flagged as an open question alongside this feature, the same way
// cage's single-client windowing was until it was tested against the real device.
export function useQuickMenuListener() {
  const toggleQuickMenu = useLaunchStore((state) => state.toggleQuickMenu);

  useNavEvent((event) => {
    if (event.type !== "action" || event.action !== "menu") return;
    if (useLaunchStore.getState().phase !== "playing") return;
    toggleQuickMenu();
  });
}
