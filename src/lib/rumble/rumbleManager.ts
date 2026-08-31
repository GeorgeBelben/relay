export type RumbleName = "nav" | "confirm" | "error";

type Effect = { duration: number; weakMagnitude: number; strongMagnitude: number };

// Deliberately subtle ("felt more than noticed, not force-feedback-game levels of
// intensity") -- nav is the lightest possible tick, confirm a touch stronger and using both
// motors, error the same strength as confirm but played twice (see ERROR_PULSE_GAP_MS below) so
// it reads as a distinct pattern without needing to be louder or longer.
const EFFECTS: Record<RumbleName, Effect> = {
  nav: { duration: 30, weakMagnitude: 0.15, strongMagnitude: 0 },
  confirm: { duration: 60, weakMagnitude: 0.25, strongMagnitude: 0.3 },
  error: { duration: 60, weakMagnitude: 0.25, strongMagnitude: 0.3 },
};

const ERROR_PULSE_GAP_MS = 90;

// Same interval as soundManager.ts's own nav debounce (MIN_INTERVAL_MS.nav) -- this is debounced
// "alongside" that one, so held-direction/fast-scrolling doesn't turn into a buzz on the rumble
// side either.
const NAV_MIN_INTERVAL_MS = 45;

let enabled = true;
let lastNavPlayedAt = -Infinity;

// Driven from useRumbleEvents.ts reacting to the persisted "rumbleEnabled" setting, same pattern
// as soundManager.ts's setVolume.
export function setRumbleEnabled(next: boolean): void {
  enabled = next;
}

function fireEffect(pad: Gamepad, effect: Effect): void {
  const actuator = pad.vibrationActuator;
  if (!actuator) return; // some Bluetooth modes don't report one at all

  actuator
    .playEffect("dual-rumble", { duration: effect.duration, weakMagnitude: effect.weakMagnitude, strongMagnitude: effect.strongMagnitude })
    // A pad whose actuator doesn't actually support "dual-rumble" rejects here -- nothing about
    // the app depends on rumble working, so this is a silent no-op, not an error to surface.
    .catch(() => {});
}

// Fire-and-forget, mirrors soundManager.ts's playSound: every input in this app already comes
// from an undifferentiated pool of controllers (no per-player assignment yet), so there's no way
// to know *which* pad produced a given event -- this rumbles every connected pad that reports an
// actuator, the same "whoever's plugged in drives the shared UI" model the rest of input already
// uses.
export function playRumble(name: RumbleName): void {
  if (!enabled) return;

  if (name === "nav") {
    const now = performance.now();
    if (now - lastNavPlayedAt < NAV_MIN_INTERVAL_MS) return;
    lastNavPlayedAt = now;
  }

  for (const pad of navigator.getGamepads()) {
    if (!pad) continue;
    fireEffect(pad, EFFECTS[name]);
    if (name === "error") setTimeout(() => fireEffect(pad, EFFECTS.error), ERROR_PULSE_GAP_MS);
  }
}
