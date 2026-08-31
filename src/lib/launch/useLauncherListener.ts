import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { playSound } from "@/lib/sound";
import { playRumble } from "@/lib/rumble";
import { useLaunchStore } from "./store";
import type { LauncherStatus } from "@/hooks/use-launcher";

// Always show the fade-to-logo beat for at least this long, even if the emulator spawns near-
// instantly -- otherwise a fast launch reads as a jarring flash-cut instead of a deliberate
// transition.
const MIN_LOGO_DURATION_MS = 1500;

// Mounted once at the app root (see __root.tsx). Owns the "launcher:status" push
// (commands::emulator::launch_game) -> LaunchPhase translation -- the game-tile onEnterPress
// handlers that call start(game) don't need to know any of this, they just kick off the launch and
// this hook drives what the overlay shows as it actually resolves.
export function useLauncherListener() {
  const phase = useLaunchStore((state) => state.phase);
  const setPhase = useLaunchStore((state) => state.setPhase);
  const dismiss = useLaunchStore((state) => state.dismiss);

  // Both read inside the status handler below via refs, not state -- they're only ever
  // consulted, never rendered, and keeping them out of the effect's dependency array means the
  // event subscription itself is only ever set up once, not torn down and rebuilt on every phase
  // change.
  const phaseRef = useRef(phase);
  const logoStartedAt = useRef<number | null>(null);
  useEffect(() => {
    phaseRef.current = phase;
    if (phase === "logo") logoStartedAt.current = Date.now();
  }, [phase]);

  useEffect(() => {
    const unlisten = listen<LauncherStatus>("launcher:status", (event) => {
      const status = event.payload;

      // "crashed" has no Electron equivalent (the old launcher only ever reported a plain
      // "error") -- Rust's process-monitoring distinguishes a clean exit from one that died with a
      // nonzero code/signal. Treated the same as "error" here: without this branch, a crash would
      // never match "error" *or* "exited" below and would leave the logo overlay stuck up forever.
      if (status.state === "error" || status.state === "crashed") {
        playSound("error");
        playRumble("error");
        const message = status.state === "error" ? status.message : `Emulator exited unexpectedly (code ${status.exit_code ?? "?"})`;
        setPhase("error", message);
        return;
      }

      if (status.state === "running") {
        const elapsed = Date.now() - (logoStartedAt.current ?? 0);
        const remaining = Math.max(0, MIN_LOGO_DURATION_MS - elapsed);
        setTimeout(dismiss, remaining);
        return;
      }

      // "exited": the emulator's already closed -- if the overlay is still up (a launch that
      // failed silently before reaching "running", or exited faster than the minimum logo beat),
      // there's nothing left to wait for. Leaves an already-shown error alone rather than
      // clobbering it -- the player needs to actually see and dismiss that, not have it vanish
      // out from under them.
      if (status.state === "exited" && phaseRef.current !== "error") {
        dismiss();
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [setPhase, dismiss]);
}
