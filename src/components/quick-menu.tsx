import { useEffect, useRef, useState } from "react";
import { Modal } from "./modal";
import { List, ListRow } from "./list";
import { AchievementsView } from "./achievements-view";
import { useLaunchStore } from "@/lib/launch/store";
import { useKillGame, usePauseToggleGame, useSaveStateGame } from "@/hooks/use-launcher";
import { useSystem } from "@/hooks/use-systems";
import { useActiveProfileId } from "@/hooks/use-settings";
import { useProfile } from "@/hooks/use-profiles";
import type { LibraryGame } from "@/hooks/use-library";

type View = "menu" | "achievements";

// Home-button quick menu (REL-23): opened by useQuickMenuListener while a game is playing.
// Mounted once at the app root (see __root.tsx) -- reads the active game straight off
// useLaunchStore rather than taking it as a prop, same reasoning as BootScreen reading its own
// scan status.
export function QuickMenu() {
  const game = useLaunchStore((state) => state.game);
  const open = useLaunchStore((state) => state.quickMenuOpen);
  const closeQuickMenu = useLaunchStore((state) => state.closeQuickMenu);

  const [view, setView] = useState<View>("menu");

  const pauseToggle = usePauseToggleGame();
  // Exactly one PAUSE_TOGGLE per real open<->close transition -- RetroArch's command is a toggle
  // with no separate pause/unpause (see retroarch_command.rs), so firing it more than once per
  // transition (e.g. on every render) would desync from the emulator's actual paused state. Mirrors
  // modal.tsx's own "wasOpen" ref pattern for its open/close sound, for the same reason: only a
  // real flip should trigger the side effect, not a re-render with the same `open` value.
  const wasOpen = useRef(open);
  useEffect(() => {
    if (wasOpen.current !== open) pauseToggle.mutate();
    wasOpen.current = open;
    // pauseToggle is a fresh useMutation object every render -- only `open` should ever
    // retrigger this.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const close = () => {
    closeQuickMenu();
    // Deliberately not reset immediately -- same as GameCardDrawer's closeAndReset, so the menu
    // doesn't visibly flash back to its default view while still animating shut.
    setTimeout(() => setView("menu"), 200);
  };

  // No game yet (shouldn't normally render before a launch, but keeps this safe to mount
  // unconditionally at the root rather than needing a guard at the call site).
  if (!game) return null;

  return (
    <Modal open={open} onClose={close} focusKey="QUICK_MENU" className="w-[28rem]">
      {view === "menu" && (
        <QuickMenuActions
          game={game}
          onAchievements={() => setView("achievements")}
          onClose={close}
        />
      )}
      {view === "achievements" && <AchievementsView game={game} onBack={() => setView("menu")} />}
    </Modal>
  );
}

function QuickMenuActions({
  game,
  onAchievements,
  onClose,
}: {
  game: LibraryGame;
  onAchievements: () => void;
  onClose: () => void;
}) {
  const system = useSystem(game.system_id);
  // Only a RetroArch-core system has a save-state command to send at all (see
  // retroarch_command.rs) -- a standalone emulator (Dolphin/PCSX2/yabause-qt) has no such
  // interface, so the row doesn't offer something that would silently do nothing.
  const canSaveState = Boolean(system.data?.retroarch_core);

  const activeProfileId = useActiveProfileId();
  const activeProfile = useProfile(activeProfileId);
  // "if enabled": only offer the Achievements tab for a profile actually linked to
  // RetroAchievements -- an unlinked profile has nothing to show there (see AchievementsView's own
  // null-progress case for the further "not matched to an RA game" case this doesn't cover).
  const achievementsEnabled = Boolean(
    activeProfile.data?.has_web_api_link || activeProfile.data?.has_connect_link,
  );

  const saveState = useSaveStateGame();
  const killGame = useKillGame();

  return (
    <>
      <h2 className="truncate px-4 pb-2 text-base font-semibold">{game.title}</h2>
      <List>
        {achievementsEnabled && <ListRow label="Achievements" onSelect={onAchievements} />}
        {canSaveState && <ListRow label="Save State" onSelect={() => saveState.mutate()} />}
        <ListRow
          label="Quit to Relay"
          onSelect={() => {
            // Optimistic close -- the real "exited" launcher-status push (once the killed
            // process actually exits) resets the launch store fully too (see
            // useLauncherListener.ts's dismiss()), but there's no reason to wait for that
            // round-trip just to close this menu.
            onClose();
            killGame.mutate();
          }}
        />
      </List>
    </>
  );
}
