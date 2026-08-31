import { Avatar, Style } from "@dicebear/core";
import thumbsDefinition from "@dicebear/styles/thumbs.json";

// Rendered fully offline, deterministically from a seed (profile id) -- this is core chrome (the
// header switcher shows on every screen), so it can't depend on a network call to DiceBear's
// hosted API the way a web app might. Style wrapped once and reused across avatars, per
// @dicebear/core's own README recommendation (passing a raw definition per-Avatar is deprecated).
const thumbsStyle = new Style(thumbsDefinition);

// Same seed always produces the same SVG, and a handful of profiles' avatars can each render in
// more than one place at once (header trigger + switcher modal row) -- cheap to cache, not worth
// recomputing.
const cache = new Map<string, string>();

export function getAvatarDataUri(seed: string): string {
  const cached = cache.get(seed);
  if (cached) return cached;

  const dataUri = new Avatar(thumbsStyle, { seed, size: 64 }).toDataUri();
  cache.set(seed, dataUri);
  return dataUri;
}
