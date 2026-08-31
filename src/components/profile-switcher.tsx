import { useState } from "react";
import { RiCheckLine, RiUserLine } from "@remixicon/react";
import { useProfiles, useRefreshRaStats } from "@/hooks/use-profiles";
import { useActiveProfileId, useSetActiveProfileId } from "@/hooks/use-settings";
import { useFocusable } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { cn } from "@/lib/cn";
import { FocusableLink } from "./focusable-link";
import { Modal } from "./modal";
import { ProfileAvatar } from "./profile-avatar";

export function ProfileSwitcher() {
  const [open, setOpen] = useState(false);
  const activeProfileId = useActiveProfileId();
  const { data: profiles = [] } = useProfiles();
  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null;

  const { ref, focused } = useFocusable({ onEnterPress: () => setOpen(true) });

  return (
    <>
      <div
        ref={ref}
        className={cn(
          "flex size-8 items-center justify-center overflow-hidden rounded-full transition-bounce p-1",
          focused ? "scale-120 ring-2 ring-zinc-50 bg-zinc-700" : "bg-zinc-800 text-white",
        )}
      >
        {activeProfile ? (
          <ProfileAvatar seed={activeProfile.id} className="size-full" />
        ) : (
          <RiUserLine className="size-5" aria-hidden="true" />
        )}
      </div>
      <ProfileSwitcherModal open={open} onClose={() => setOpen(false)} />
    </>
  );
}

function ProfileSwitcherModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { data: profiles = [] } = useProfiles();
  const activeProfileId = useActiveProfileId();
  const setActiveProfileId = useSetActiveProfileId();
  // Electron's setActive refreshed RA stats server-side, in the same IPC call, as part of switching
  // profiles -- set_active_profile_id here is a plain settings write with no such side effect, so
  // that refresh is composed explicitly at the call site instead. A no-op for a profile with no
  // Web API link (see use-profiles.ts's useRefreshRaStats).
  const refreshRaStats = useRefreshRaStats();

  useActionHints(open ? [{ action: "confirm", label: "Select" }, { action: "back", label: "Close" }] : null);

  return (
    <Modal open={open} onClose={onClose} focusKey="PROFILE_SWITCHER_MODAL" className="w-128">
      <div className="bg-zinc-800 p-6 rounded-lg">
        <h2 className="text-lg font-bold font-space-grotesk mb-4">Switch Profile</h2>
        {profiles.length === 0 ? (
          <p className="text-sm text-muted-foreground">No profiles yet -- add one from Settings → Profiles.</p>
        ) : (
          <div className="space-y-1 mb-4">
            {profiles.map((profile) => (
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
            ))}
          </div>
        )}
        <FocusableLink
          to="/settings/profiles"
          className={(focused) =>
            cn(
              "block rounded px-3 py-2 text-center text-sm font-medium transition-bounce",
              focused ? "bg-zinc-700" : "text-muted-foreground",
            )
          }
        >
          Manage Profiles
        </FocusableLink>
      </div>
    </Modal>
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
    <div ref={ref} className={cn("flex items-center gap-3 rounded px-3 py-2 transition-bounce", focused ? "bg-zinc-700" : "bg-zinc-800")}>
      <ProfileAvatar seed={seed} className="size-8 shrink-0" />
      <span className="flex-1 truncate text-sm font-medium">{name}</span>
      {active && <RiCheckLine className="size-4 shrink-0 text-primary" aria-hidden="true" />}
    </div>
  );
}
