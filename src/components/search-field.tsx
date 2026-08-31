import { useEffect, useRef, useState } from "react";
import { RiSearchLine } from "@remixicon/react";
import { pushBackHandler, useFocusable } from "@/lib/focus";
import { cn } from "@/lib/cn";

type SearchFieldProps = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
};

// Always a real <input> (unlike a text-field-row, which swaps between a plain row and an input on
// demand) since search is a live filter with no draft/commit step -- there's nothing to discard on
// Back, just real DOM focus to give up. Still needs the same virtual-focus -> real-focus handoff
// every text entry point in this app needs: d-pad navigation lands on it as a normal focusable
// row, Confirm hands it real DOM focus (which is what shows the on-screen keyboard, via its
// data-osk tag), and Back gives real focus back up, returning it to a plain virtual-focus row.
export function SearchField({ value, onChange, placeholder }: SearchFieldProps) {
  const [active, setActive] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const { ref, focused, focusSelf } = useFocusable({ onEnterPress: () => inputRef.current?.focus() });

  useEffect(() => {
    if (!active) return;
    return pushBackHandler(() => inputRef.current?.blur());
  }, [active]);

  return (
    <div
      ref={ref}
      className={cn(
        "flex w-80 items-center gap-2 rounded-full border px-4 py-2 text-sm transition-bounce",
        focused || active ? "border-white bg-gray-800" : "border-transparent bg-gray-900",
      )}
    >
      <RiSearchLine className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <input
        ref={inputRef}
        data-osk
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        onFocus={() => setActive(true)}
        onBlur={() => {
          setActive(false);
          // Norigin's autoRestoreFocus only fires when a component actually *unmounts* -- the
          // on-screen keyboard never does (it's permanently mounted in __root.tsx, just toggling
          // `focusable`), so without this, virtual focus is left stuck on whichever now-hidden
          // OSK key was last pressed instead of coming back to a visible row. Reclaiming it here
          // is what makes the very next d-pad press actually do something.
          focusSelf();
        }}
        onKeyDown={(e) => {
          // Stop these before they reach the norigin engine's own window-level keydown listener,
          // which would otherwise double-handle them (move virtual focus, or fire Enter on
          // whatever key the on-screen keyboard still has virtually focused underneath this real
          // one). "m" needs the same treatment -- it's the global menu shortcut (lib/input) and
          // that listener calls preventDefault() on it, which silently eats the letter instead of
          // typing it.
          if (e.key === "Enter" || e.key === "Escape" || e.key === "m" || e.key.startsWith("Arrow")) e.stopPropagation();
          if (e.key === "Escape") inputRef.current?.blur();
        }}
        className="min-w-0 flex-1 bg-transparent outline-none placeholder:text-muted-foreground"
      />
    </div>
  );
}
