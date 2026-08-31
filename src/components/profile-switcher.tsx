import { RiUserLine } from "@remixicon/react";
import { useProfiles } from "@/hooks/use-profiles";
import { useActiveProfileId } from "@/hooks/use-settings";
import { ProfileAvatar } from "./profile-avatar";

// Purely informational -- like Clock/ControllerIndicator next to it, not a focus target (see
// header.tsx: nothing in the header is selectable anymore). Switching profiles now happens
// through the home-button System Menu (REL-138)'s Change Profile row instead of navigating here.
export function ProfileSwitcher() {
  const activeProfileId = useActiveProfileId();
  const { data: profiles = [] } = useProfiles();
  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null;

  return (
    <div className="flex size-8 items-center justify-center overflow-hidden rounded-full bg-gray-800 p-1 text-white">
      {activeProfile ? (
        <ProfileAvatar seed={activeProfile.id} className="size-full" />
      ) : (
        <RiUserLine className="size-5" aria-hidden="true" />
      )}
    </div>
  );
}
