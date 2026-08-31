// WebKitGTK doesn't implement the Gamepad API on every build -- `navigator.getGamepads` can be
// `undefined` outright (not just "no pads connected"), which the Electron original never had to
// account for since Chromium always has it. Every caller already tolerates null entries in the
// returned array; this just keeps that same shape when the API doesn't exist at all, instead of
// throwing on the very first poll and silently killing gamepad input for the rest of the session.
export function getGamepads(): (Gamepad | null)[] {
  return typeof navigator.getGamepads === "function" ? navigator.getGamepads() : [];
}
