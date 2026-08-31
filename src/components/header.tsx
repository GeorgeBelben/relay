import { Logo } from "./logo";
import { Clock } from "./clock";
import { ControllerIndicator } from "./controller-indicator";
import { ProfileSwitcher } from "./profile-switcher";

// Nothing here is a focus target (see profile-switcher.tsx) -- Settings and profile switching
// both moved to the home-button System Menu (REL-138), reachable from anywhere, not just screens
// that render this header.
export function Header() {
  return (
    <header className="flex items-center justify-between px-16 py-8">
      <Logo className="w-8" />

      <div className="flex items-center gap-4">
        <ControllerIndicator />
        <ProfileSwitcher />
        <Clock />
      </div>
    </header>
  );
}
