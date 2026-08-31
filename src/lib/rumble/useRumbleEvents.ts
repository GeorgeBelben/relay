import { useEffect } from "react";
import { useRumbleEvent } from "@/lib/input";
import { useRumbleEnabled } from "@/hooks/use-settings";
import { playRumble, setRumbleEnabled } from "./rumbleManager";

// Mounted once at the app root (see __root.tsx), alongside useSoundEvents -- the rumble layer.
// Subscribes to input's rumbleEvents (gamepad-only -- see nav.ts's own comment on why that's a
// separate stream from soundEvents).
export function useRumbleEvents() {
  const enabled = useRumbleEnabled();
  useEffect(() => {
    setRumbleEnabled(enabled);
  }, [enabled]);

  useRumbleEvent((event) => {
    if (event.type === "direction") {
      playRumble("nav");
    } else if (event.action === "confirm") {
      playRumble("confirm");
    }
    // back/menu have no rumble of their own (scope: nav tick + confirm here, error wired
    // separately from useLauncherListener.ts's own playSound("error") call site).
  });
}
