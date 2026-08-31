import { useEffect } from "react";
import { useSoundEvent } from "@/lib/input";
import { useSoundVolume } from "@/hooks/use-settings";
import { playSound, preloadSounds, setVolume } from "./soundManager";

// Mounted once at the app root (see __root.tsx), alongside useInputListeners/useFocusBridge --
// the UI sound layer. Subscribes to soundEvents specifically, not navEvents: the latter
// is what actually *drives* spatial navigation (see nav.ts's own comment on the split), and
// double-subscribing here would just be redundant, not wrong, but soundEvents is the one that
// carries identical events for gamepad and keyboard alike.
export function useSoundEvents() {
  useEffect(() => {
    preloadSounds();
  }, []);

  const volume = useSoundVolume();
  useEffect(() => {
    setVolume(volume / 100);
  }, [volume]);

  useSoundEvent((event) => {
    if (event.type === "direction") {
      playSound("nav");
    } else if (event.action === "confirm") {
      playSound("confirm");
    } else if (event.action === "back") {
      playSound("back");
    }
    // "menu" has no sound of its own (scope: navigation, confirm, back, launch, error).
  });
}
