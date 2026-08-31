import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { RiCheckLine } from "@remixicon/react";
import { useProfiles, useRefreshRaStats } from "@/hooks/use-profiles";
import { useActiveProfileId, useSetActiveProfileId } from "@/hooks/use-settings";
import { useBackHandler, useFocusable } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { useSystemMenuStore } from "@/lib/system-menu";
import { cn } from "@/lib/cn";
import { Modal } from "./modal";
import { List, ListRow } from "./list";
import { ProfileAvatar } from "./profile-avatar";

type View = "menu" | "profile";

// Neither /settings nor /settings/profiles has been ported from the Electron MVP yet (same gap
// the old header settings icon and ProfileSwitcherModal's "Manage Profiles" link already had) --
// typed as plain `string`, not inline literals, so TanStack Router's route-literal check doesn't
// reject them outright; same escape hatch FocusableLink's own `to: string` prop already relies on.
const SETTINGS_PATH: string = "/settings";
const MANAGE_PROFILES_PATH: string = "/settings/profiles";

// Browsing-mode counterpart to QuickMenu (REL-137's in-game menu) -- opened by the "home" action
// (see lib/system-menu/useSystemMenuListener) while nothing's playing. Deliberately a separate
// component and store from QuickMenu, not a phase-branch of it (REL-138) -- and replaces both the
// header's old settings icon and the profile-switcher avatar as focus targets (see header.tsx /
// profile-switcher.tsx, now purely informational).
export function SystemMenu() {
  const open = useSystemMenuStore((state) => state.open);
  const closeMenu = useSystemMenuStore((state) => state.closeMenu);
  const [view, setView] = useState<View>("menu");

  const close = () => {
    closeMenu();
    // Deliberately not reset immediately -- same as QuickMenu/GameCardDrawer's own
    // closeAndReset, so the menu doesn't visibly flash back to its default view while still
    // animating shut.
    setTimeout(() => setView("menu"), 200);
  };

  return (
    <Modal open={open} onClose={close} focusKey="SYSTEM_MENU" className="w-[28rem]">
      {view === "menu" && (
        <SystemMenuActions onChangeProfile={() => setView("profile")} onClose={close} />
      )}
      {view === "profile" && <ProfileListView onBack={() => setView("menu")} onClose={close} />}
    </Modal>
  );
}

function SystemMenuActions({
  onChangeProfile,
  onClose,
}: {
  onChangeProfile: () => void;
  onClose: () => void;
}) {
  const navigate = useNavigate();

  return (
    <>
      <h2 className="px-4 pb-2 text-base font-semibold">Relay</h2>
      <List>
        <ListRow label="Change Profile" onSelect={onChangeProfile} />
        <ListRow
          label="Settings"
          onSelect={() => {
            // Close first -- this menu is root-mounted (see __root.tsx) and stays mounted across
            // route changes the same way QuickMenu does, so navigating away without closing would
            // leave it sitting open on top of the destination route.
            onClose();
            navigate({ to: SETTINGS_PATH });
          }}
        />
      </List>
    </>
  );
}

// Content mirrors the old header ProfileSwitcher's modal (now retired -- see profile-switcher.tsx)
// as a view inside this menu instead of its own separate modal.
function ProfileListView({ onBack, onClose }: { onBack: () => void; onClose: () => void }) {
  // Back returns to the menu view rather than closing outright -- same pattern as
  // AchievementsView/BoxArtPicker's own back-handler registration.
  useBackHandler(onBack);
  const navigate = useNavigate();

  const { data: profiles = [] } = useProfiles();
  const activeProfileId = useActiveProfileId();
  const setActiveProfileId = useSetActiveProfileId();
  // Electron's setActive refreshed RA stats server-side, in the same IPC call, as part of switching
  // profiles -- set_active_profile_id here is a plain settings write with no such side effect, so
  // that refresh is composed explicitly at the call site instead. A no-op for a profile with no
  // Web API link (see use-profiles.ts's useRefreshRaStats).
  const refreshRaStats = useRefreshRaStats();

  useActionHints([
    { action: "confirm", label: "Select" },
    { action: "back", label: "Back" },
  ]);

  return (
    <>
      <h2 className="px-4 pb-2 text-base font-semibold">Switch Profile</h2>
      <List>
        {profiles.length === 0 ? (
          <p className="px-4 py-2 text-sm text-muted-foreground">
            No profiles yet -- add one from Settings → Profiles.
          </p>
        ) : (
          profiles.map((profile) => (
            <ProfileOptionRow
              key={profile.id}
              name={profile.name}
              seed={profile.id}
              active={profile.id === activeProfileId}
              onSelect={() => {
                if (profile.id !== activeProfileId) {
                  setActiveProfileId.mutate(profile.id);
                  refreshRaStats.mutate(profile.id);
                }
                onClose();
              }}
            />
          ))
        )}
        <ListRow
          label="Manage Profiles"
          onSelect={() => {
            onClose();
            navigate({ to: MANAGE_PROFILES_PATH });
          }}
        />
      </List>
    </>
  );
}

function ProfileOptionRow({
  name,
  seed,
  active,
  onSelect,
}: {
  name: string;
  seed: string;
  active: boolean;
  onSelect: () => void;
}) {
  const { ref, focused } = useFocusable({ onEnterPress: onSelect });

  return (
    <div
      ref={ref}
      className={cn(
        "flex items-center gap-3 rounded px-3 py-2 transition-bounce",
        focused ? "bg-gray-700" : "bg-gray-800",
      )}
    >
      <ProfileAvatar seed={seed} className="size-8 shrink-0" />
      <span className="flex-1 truncate text-sm font-medium">{name}</span>
      {active && <RiCheckLine className="size-4 shrink-0 text-primary" aria-hidden="true" />}
    </div>
  );
}
