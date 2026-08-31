// Deliberately loud -- ring utilities render via box-shadow, so there's no missing
// outline-style gotcha (Tailwind's outline-* utilities need an explicit outline-style to
// actually paint, which is easy to omit and get a silently-invisible "focused" state).
//
// FOCUS_RING_BASE is split out for the handful of consumers (game-tile.tsx) that keep
// outline-3/outline-offset-8 always present and only toggle outline-color between transparent and
// white -- animating just the color, rather than the whole outline appearing from nothing, is
// what makes that specific focus transition read smoothly. Sharing this base means the width/offset
// values themselves still can't drift from FOCUS_RING's own, even though the color toggle is
// bespoke per consumer.
export const FOCUS_RING_BASE = "outline-3 outline-offset-8";
export const FOCUS_RING = `${FOCUS_RING_BASE} outline-white`;
