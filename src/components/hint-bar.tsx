import { useCurrentHints } from "@/lib/hints";
import { useLastInputMethod } from "@/lib/input";
import { useControllerType } from "@/hooks/use-settings";
import { getIconUrl } from "@/lib/icons/iconSet";

// Always-on-screen, bottom-right -- see lib/hints/useActionHints for how screens/modals
// contribute what shows up here. z-[70]: has to sit above every modal/drawer/overlay (Modal's
// backdrop/viewport are z-50, the highest currently in this repo), or "always-on-screen" isn't
// actually true -- an opaque overlay above this in stacking order hides the hint bar completely,
// not just dims it the way Modal's translucent backdrop happens to leave it legible.
export function HintBar() {
  const hints = useCurrentHints();
  const inputMethod = useLastInputMethod() ?? "gamepad";
  const controllerType = useControllerType();

  if (hints.length === 0) return null;

  return (
    <div className="fixed inset-x-0 py-6 px-16 bottom-0 z-[70] flex items-center justify-end gap-4 bg-linear-to-t from-[#111111] via-[#111111]/70 to-[#111111]/0">
      {hints.map((hint) => (
        <div key={hint.action} className="flex items-center gap-1.5">
          <img
            src={getIconUrl(inputMethod, controllerType, hint.action)}
            alt=""
            className="h-6 w-6"
          />
          <span className="text-sm text-white font-space-grotesk font-medium">{hint.label}</span>
        </div>
      ))}
    </div>
  );
}
