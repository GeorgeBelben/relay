import { createEmitter } from "./emitter";
import { startGamepadListener } from "./gamepad";
import { startKeyboardListener } from "./keyboard";
import { useInputMethodStore } from "./store";
import type { NavEvent } from "./types";

// Module-level singleton: there is exactly one nav-event stream for the whole app, so a
// React context would only add indirection without buying anything.
export const navEvents = createEmitter<NavEvent>();

// A second, sound-only stream -- deliberately separate from navEvents above. Gamepad's
// onEvent already carries every direction/action it detects, so it feeds both streams identically;
// keyboard's onEvent only ever carries Back/Menu (arrows/Enter are handled natively by the focus
// engine, see keyboard.ts), so this one also gets keyboard's onAnyRecognizedKey, which fires for
// arrows/Enter too without those ever driving navigation through this emitter. The result: a
// SoundManager subscribing here hears identical events for a given action regardless of which
// input method produced it, without touching how navigation itself is driven.
export const soundEvents = createEmitter<NavEvent>();

// A third stream, gamepad-only unlike soundEvents above -- rumble needs actual haptic
// hardware to buzz, which a keyboard press has none of, so this deliberately isn't fed from
// startKeyboardListener the way soundEvents is.
export const rumbleEvents = createEmitter<NavEvent>();

export function startInputListeners(): () => void {
  const setLastInputMethod = useInputMethodStore.getState().setLastInputMethod;

  const stopKeyboard = startKeyboardListener(
    (event) => navEvents.emit(event),
    () => setLastInputMethod("keyboard"),
    (event) => soundEvents.emit(event),
  );
  const stopGamepad = startGamepadListener(
    (event) => {
      navEvents.emit(event);
      soundEvents.emit(event);
      rumbleEvents.emit(event);
    },
    () => setLastInputMethod("gamepad"),
  );

  return () => {
    stopKeyboard();
    stopGamepad();
  };
}
