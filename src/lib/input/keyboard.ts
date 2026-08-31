import type { NavEvent } from "./types";

const KEY_MAP: Record<string, NavEvent> = {
  ArrowUp: { type: "direction", direction: "up" },
  ArrowDown: { type: "direction", direction: "down" },
  ArrowLeft: { type: "direction", direction: "left" },
  ArrowRight: { type: "direction", direction: "right" },
  Enter: { type: "action", action: "confirm" },
  Escape: { type: "action", action: "back" },
  m: { type: "action", action: "menu" },
};

export function mapKeyToNavEvent(key: string): NavEvent | null {
  return KEY_MAP[key] ?? null;
}

// Arrows and Enter are handled natively by norigin-spatial-navigation's own keydown listener
// (see lib/focus/init.ts) -- it owns focus movement and enter-press directly, and calling
// preventDefault/emitting our own event for those keys too would double-handle every press.
// "Back" and "menu" have no equivalent in that library, so Escape/M are ours alone to own.
const OWNED_BY_THIS_LISTENER = new Set(["Escape", "m"]);

// onAnyRecognizedKey: fires for *every* recognized key, including arrows/Enter, which
// this listener otherwise leaves entirely to the focus engine's own native keydown handling (see
// OWNED_BY_THIS_LISTENER below). It exists purely so something like a sound layer can hear about
// keyboard-driven navigation/confirm too -- unlike onEvent, it must never be wired to anything
// that actually *drives* navigation (nav.ts's navEvents/bridge.ts), since arrows/Enter are already
// being handled natively; emitting them there too would double-navigate.
export function startKeyboardListener(
  onEvent: (event: NavEvent) => void,
  onUsed: () => void,
  onAnyRecognizedKey?: (event: NavEvent) => void,
): () => void {
  const handleKeyDown = (e: KeyboardEvent) => {
    // Tab carries a native browser focus behavior that would fight our own focus system --
    // suppressed here so it's dead everywhere from day one, not just wherever the focus
    // engine happens to be mounted.
    if (e.key === "Tab") {
      e.preventDefault();
      return;
    }

    const navEvent = mapKeyToNavEvent(e.key);
    if (!navEvent) return;

    onUsed();
    onAnyRecognizedKey?.(navEvent);

    if (!OWNED_BY_THIS_LISTENER.has(e.key)) return;

    e.preventDefault();
    onEvent(navEvent);
  };

  window.addEventListener("keydown", handleKeyDown);
  return () => window.removeEventListener("keydown", handleKeyDown);
}
