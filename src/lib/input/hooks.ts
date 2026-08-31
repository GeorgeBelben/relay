import { useEffect, useState } from "react";
import { getGamepads } from "./getGamepads";
import { navEvents, rumbleEvents, soundEvents, startInputListeners } from "./nav";
import { useInputMethodStore } from "./store";
import type { NavEvent } from "./types";

// Mounted once at the app root -- starts the keyboard/gamepad polling for the lifetime of the app.
export function useInputListeners() {
  useEffect(() => startInputListeners(), []);
}

export function useNavEvent(listener: (event: NavEvent) => void) {
  useEffect(() => navEvents.subscribe(listener), [listener]);
}

// See nav.ts's soundEvents for why this is a separate stream from useNavEvent above.
export function useSoundEvent(listener: (event: NavEvent) => void) {
  useEffect(() => soundEvents.subscribe(listener), [listener]);
}

// See nav.ts's rumbleEvents for why this is gamepad-only, unlike useSoundEvent above.
export function useRumbleEvent(listener: (event: NavEvent) => void) {
  useEffect(() => rumbleEvents.subscribe(listener), [listener]);
}

export function useLastInputMethod() {
  return useInputMethodStore((state) => state.lastInputMethod);
}

function readConnectedIndexes(): number[] {
  return [...getGamepads()]
    .filter((pad): pad is Gamepad => pad !== null)
    .map((pad) => pad.index)
    .sort((a, b) => a - b);
}

// Just presence, not identity -- no player/profile assignment. Slots are the sorted gamepad
// indexes, low to high, so "controller 1/2/3/4" stays stable and predictable rather than tracking
// which physical pad is "assigned" to which slot.
//
// gamepadconnected/disconnected cover hot-plug, but Chromium doesn't fire gamepadconnected
// retroactively for a pad that was already connected before this hook's listener attaches (e.g.
// on a fresh page load with a controller already plugged in) until that pad sends its first
// input -- so this also reads the current state directly on mount to catch that case immediately
// rather than waiting on a button press.
export function useConnectedControllers(): number[] {
  const [connected, setConnected] = useState<number[]>(readConnectedIndexes);

  useEffect(() => {
    const refresh = () => setConnected(readConnectedIndexes());
    refresh();
    window.addEventListener("gamepadconnected", refresh);
    window.addEventListener("gamepaddisconnected", refresh);
    return () => {
      window.removeEventListener("gamepadconnected", refresh);
      window.removeEventListener("gamepaddisconnected", refresh);
    };
  }, []);

  return connected;
}
