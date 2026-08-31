import { RiGamepadFill, RiGamepadLine } from "@remixicon/react";
import { useConnectedControllers } from "@/lib/input";
import { cn } from "@/lib/cn";

const SLOT_COUNT = 4;

// Presence only, not a join/assignment flow -- up to 4 controllers, so 4 fixed slots filled
// low-to-high by whichever gamepad indexes are currently connected. Purely informational (no
// useFocusable -- nothing here to navigate to), same as Clock sitting next to it.
export function ControllerIndicator() {
  const connected = useConnectedControllers();

  return (
    <div className="flex items-center gap-1.5" aria-label={`${connected.length} of ${SLOT_COUNT} controllers connected`}>
      {Array.from({ length: SLOT_COUNT }, (_, slot) => {
        const filled = slot < connected.length;
        const Icon = filled ? RiGamepadFill : RiGamepadLine;
        return <Icon key={slot} className={cn("size-5", filled ? "text-white" : "text-zinc-700")} aria-hidden="true" />;
      })}
    </div>
  );
}
