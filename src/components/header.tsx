import { RiSettings3Line } from "@remixicon/react";
import { FocusableLink } from "./focusable-link";
import { cn } from "@/lib/cn";
import { Logo } from "./logo";
import { Clock } from "./clock";
import { ControllerIndicator } from "./controller-indicator";
import { ProfileSwitcher } from "./profile-switcher";

export function Header() {
  return (
    <header className="flex items-center justify-between px-16 py-8">
      <Logo />

      <div className="flex items-center gap-4">
        <ControllerIndicator />
        <ProfileSwitcher />
        <FocusableLink
          to="/settings"
          className={(focused) =>
            cn(
              "flex size-8 items-center justify-center rounded-full transition-bounce",
              focused ? "scale-120 bg-zinc-800 ring-2 ring-zinc-50 text-white-950" : "bg-transparent text-white",
            )
          }
        >
          <RiSettings3Line className="size-5" aria-hidden="true" />
        </FocusableLink>
        <Clock />
      </div>
    </header>
  );
}
