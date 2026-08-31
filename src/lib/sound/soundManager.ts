import navUrl from "@/assets/sounds/nav.wav";
import confirmUrl from "@/assets/sounds/confirm.wav";
import backUrl from "@/assets/sounds/back.wav";
import launchUrl from "@/assets/sounds/launch.wav";
import errorUrl from "@/assets/sounds/error.wav";

export type SoundName = "nav" | "confirm" | "back" | "launch" | "error";

const SOUND_URLS: Record<SoundName, string> = {
  nav: navUrl,
  confirm: confirmUrl,
  back: backUrl,
  launch: launchUrl,
  error: errorUrl,
};

// Nav-tick needs debouncing for the obvious reason -- held-direction/fast-scrolling would
// otherwise fire it dozens of times a second and turn it into a buzz.
//
// confirm/back's debounce exists for a different, more specific reason: Modal and Drawer
// (modal.tsx, drawer.tsx) explicitly play "confirm" on their own open transition and "back" on
// their own close transition, on top of whatever the input-event stream already plays for the
// physical press that caused it (see useSoundEvents.ts) -- opening a Modal is very often the
// direct result of a confirm press (e.g. the profile switcher's avatar button), and closing one is
// very often the direct result of a back press. Without this, that one physical press produces two
// back-to-back plays of the same sound (the generic stream one, and the panel's own one reacting
// to the resulting re-render a frame later). Debouncing each against itself collapses that pair
// into one, regardless of which of the two call sites happens to fire first -- and still lets a
// panel that opens/closes from something *other* than that matching press (a main-process push,
// like the update-available dialog) play its own sound normally, since there's nothing recent to
// collapse against in that case.
const MIN_INTERVAL_MS: Partial<Record<SoundName, number>> = { nav: 45, confirm: 80, back: 80 };

let audioContext: AudioContext | null = null;
let gainNode: GainNode | null = null;
let volume = 1;
const buffers = new Map<SoundName, AudioBuffer>();
const lastPlayedAt = new Map<SoundName, number>();

function getContext(): AudioContext {
  audioContext ??= new AudioContext();
  return audioContext;
}

// Decodes every sound once -- called once on app mount (useSoundEvents, wired at the app root),
// never per-play. A sound that fails to load/decode (missing file, corrupt data) is logged and
// just never plays; nothing about the app depends on sound working.
export async function preloadSounds(): Promise<void> {
  const ctx = getContext();
  gainNode = ctx.createGain();
  gainNode.gain.value = volume;
  gainNode.connect(ctx.destination);

  await Promise.all(
    (Object.entries(SOUND_URLS) as [SoundName, string][]).map(async ([name, url]) => {
      try {
        const response = await fetch(url);
        const arrayBuffer = await response.arrayBuffer();
        buffers.set(name, await ctx.decodeAudioData(arrayBuffer));
      } catch (err) {
        console.warn(`SoundManager: failed to load sound "${name}"`, err);
      }
    }),
  );
}

export function setVolume(next: number): void {
  volume = Math.max(0, Math.min(1, next));
  if (gainNode) gainNode.gain.value = volume;
}

// Fire-and-forget: nothing awaits a sound finishing, and playback is never tracked/stopped
// individually -- a fresh AudioBufferSourceNode per call is exactly what lets overlapping sounds
// (rapid nav-ticks, or a confirm right on top of one) play cleanly instead of cutting each other
// off, which a single shared <audio>/new Audio() element can't do.
export function playSound(name: SoundName): void {
  const buffer = buffers.get(name);
  if (!buffer) return;

  const minInterval = MIN_INTERVAL_MS[name];
  if (minInterval !== undefined) {
    const now = performance.now();
    const last = lastPlayedAt.get(name) ?? -Infinity;
    if (now - last < minInterval) return;
    lastPlayedAt.set(name, now);
  }

  const ctx = getContext();
  // Browsers suspend a freshly-created AudioContext until a real user gesture -- the first
  // gamepad/keyboard press that gets this far *is* that gesture, so resume opportunistically
  // rather than requiring some separate "click to enable sound" step first.
  if (ctx.state === "suspended") void ctx.resume();

  const source = ctx.createBufferSource();
  source.buffer = buffer;
  source.connect(gainNode ?? ctx.destination);
  source.start();

  // Launching a game is itself the outcome of a confirm press on a tile -- the generic "confirm"
  // blip from the input-event stream would otherwise play on top of the launch stinger for that
  // same press (see useSoundEvents.ts/startGameLaunch.ts, both fire from the same physical press).
  // Backdating confirm's own last-played time lets the *existing* confirm debounce above absorb
  // that follow-up call instead of needing a second, separate suppression mechanism.
  if (name === "launch") lastPlayedAt.set("confirm", performance.now());
}
