// REL-51's "configurable window" is this one constant -- a single-user device has no real need
// for a Settings control over it, just an easy value to change here if 7 days ever feels wrong.
const NEW_BADGE_WINDOW_DAYS = 7;
const NEW_BADGE_WINDOW_MS = NEW_BADGE_WINDOW_DAYS * 24 * 60 * 60 * 1000;

// addedAt is games.created_at (see use-library.ts's LibraryGame), as a unix timestamp -- set once
// when a rom is first scanned in, never touched by a later rescan, so this naturally stops being
// true on its own once the window passes rather than needing any kind of manual dismissal.
export function isRecentlyAdded(addedAt: number): boolean {
  return Date.now() - addedAt * 1000 < NEW_BADGE_WINDOW_MS;
}
