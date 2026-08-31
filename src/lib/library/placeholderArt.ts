// There's no real box art until a game is matched/enriched -- a deterministic gradient per game
// (hashed from its id) at least gives shelves visual variety instead of a wall of identical gray
// boxes.
function hashString(input: string): number {
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    hash = (hash << 5) - hash + input.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

export function placeholderGradient(seed: string): string {
  const hue = hashString(seed) % 360;
  return `linear-gradient(135deg, oklch(0.45 0.12 ${hue}), oklch(0.25 0.1 ${(hue + 40) % 360}))`;
}
