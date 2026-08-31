import { getGamepads } from "./getGamepads";
import type { NavAction, NavDirection, NavEvent } from "./types";

// Standard W3C Gamepad API "standard" mapping button indices -- what 8BitDo controllers (and
// Xbox/generic Xinput pads generally) report themselves as.
export const BUTTON_ACTION_MAP: Record<number, NavAction> = {
  0: "confirm", // A / Cross
  1: "back", // B / Circle
  3: "menu", // Y / Triangle / X (north face button)
};

export const BUTTON_DIRECTION_MAP: Record<number, NavDirection> = {
  12: "up",
  13: "down",
  14: "left",
  15: "right",
};

const STICK_DEADZONE = 0.5;
const REPEAT_INITIAL_DELAY_MS = 400;
const REPEAT_INTERVAL_MS = 120;

export function stickToDirection(x: number, y: number): NavDirection | null {
  if (Math.abs(x) < STICK_DEADZONE && Math.abs(y) < STICK_DEADZONE) return null;
  return Math.abs(x) > Math.abs(y) ? (x > 0 ? "right" : "left") : y > 0 ? "down" : "up";
}

function readDirection(pad: Gamepad): NavDirection | null {
  for (const [index, direction] of Object.entries(BUTTON_DIRECTION_MAP)) {
    if (pad.buttons[Number(index)]?.pressed) return direction;
  }
  return stickToDirection(pad.axes[0] ?? 0, pad.axes[1] ?? 0);
}

export function startGamepadListener(onEvent: (event: NavEvent) => void, onUsed: () => void): () => void {
  const heldActions = new Set<NavAction>();
  let activeDirection: { direction: NavDirection; since: number; lastRepeat: number } | null = null;
  let frame: number;

  function poll(now: number) {
    let usedThisFrame = false;
    let direction: NavDirection | null = null;

    for (const pad of getGamepads()) {
      if (!pad) continue;

      for (const [index, action] of Object.entries(BUTTON_ACTION_MAP)) {
        const pressed = pad.buttons[Number(index)]?.pressed ?? false;
        if (pressed && !heldActions.has(action)) {
          heldActions.add(action);
          usedThisFrame = true;
          onEvent({ type: "action", action });
        } else if (!pressed) {
          heldActions.delete(action);
        }
      }

      direction ??= readDirection(pad);
    }

    if (direction) {
      usedThisFrame = true;
      if (!activeDirection || activeDirection.direction !== direction) {
        activeDirection = { direction, since: now, lastRepeat: now };
        onEvent({ type: "direction", direction });
      } else if (now - activeDirection.since > REPEAT_INITIAL_DELAY_MS && now - activeDirection.lastRepeat > REPEAT_INTERVAL_MS) {
        activeDirection.lastRepeat = now;
        onEvent({ type: "direction", direction });
      }
    } else {
      activeDirection = null;
    }

    if (usedThisFrame) onUsed();
    frame = requestAnimationFrame(poll);
  }

  frame = requestAnimationFrame(poll);
  return () => cancelAnimationFrame(frame);
}
