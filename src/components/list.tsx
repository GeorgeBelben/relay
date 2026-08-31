import type { ReactNode } from "react";
import { useNavigate } from "@tanstack/react-router";
import { RiArrowRightSLine, RiCheckLine } from "@remixicon/react";
import { scrollFocusedIntoView, useFocusable } from "@/lib/focus";
import { cn } from "@/lib/cn";

// iOS-style grouped list -- rounded corners on the group, a divider between each row.
export function List({ children }: { children: ReactNode }) {
  return <div className="space-y-1">{children}</div>;
}

type ListRowBase = {
  label: string;
  focusKey?: string;
};

type ListRowActivation = { to: string; onSelect?: never } | { onSelect: () => void; to?: never } | { to?: never; onSelect?: never };

type ListRowAccessory =
  | { accessory?: "chevron"; value?: string }
  | { accessory: "checkbox"; checked: boolean }
  | { accessory: "switch"; checked: boolean };

export type ListRowProps = ListRowBase & ListRowActivation & ListRowAccessory;

// Whole row is the focusable/activatable unit -- matches iOS's tap-anywhere-on-row behavior.
// Pass `to` to navigate, `onSelect` for anything else (toggling a checkbox, opening a picker), or
// neither for a purely informational row (e.g. "Version 0.1.1") -- that one never registers as
// focusable at all.
export function ListRow(props: ListRowProps) {
  const { label, focusKey } = props;
  const navigate = useNavigate();

  const interactive = Boolean(props.to || props.onSelect);
  const handleSelect = () => {
    if (props.to) navigate({ to: props.to });
    else props.onSelect?.();
  };

  const { ref, focused } = useFocusable({
    focusKey,
    focusable: interactive,
    onEnterPress: interactive ? handleSelect : undefined,
    onFocus: () => scrollFocusedIntoView(ref.current),
  });

  return (
    <div
      ref={ref}
      className={cn("flex items-center justify-between gap-3 px-4 py-3 text-sm font-medium rounded transition-bounce", {
        "bg-gray-700 my-1 py-5": focused,
        "bg-gray-800": !focused,
      })}
    >
      <span>{label}</span>
      <div className="flex items-center gap-2 text-muted-foreground">
        {props.accessory === "checkbox" ? (
          <Checkbox checked={props.checked} />
        ) : props.accessory === "switch" ? (
          <Switch checked={props.checked} />
        ) : (
          <>
            {props.value && <span>{props.value}</span>}
            {interactive && <RiArrowRightSLine className="h-4 w-4" aria-hidden="true" />}
          </>
        )}
      </div>
    </div>
  );
}

// Single-select-from-a-list indicator (e.g. a timezone picker, "which alternate box art is
// this") -- picking a different row changes which one is checked, but a row is never toggled off
// by re-selecting it, so this stays a checkmark rather than the on/off Switch below.
function Checkbox({ checked }: { checked: boolean }) {
  return (
    <div
      className={cn(
        "flex h-5 w-5 items-center justify-center rounded border",
        checked ? "border-primary bg-primary" : "border-border bg-transparent",
      )}
    >
      {checked && <RiCheckLine className="h-3.5 w-3.5 text-primary-foreground" aria-hidden="true" />}
    </div>
  );
}

// iOS-style toggle for a genuine on/off setting (NTP, controller rumble) -- unlike Checkbox
// above, selecting one of these *is* the flip, so a sliding pill reads correctly where a
// checkmark wouldn't. transition-bounce (main.css) is the same overshoot easing every other
// focus/hover transition in the app already uses.
function Switch({ checked }: { checked: boolean }) {
  return (
    <div className={cn("flex h-5 w-9 shrink-0 items-center rounded-full p-0.5 transition-bounce", checked ? "bg-gray-500" : "bg-gray-600")}>
      <div className={cn("h-4 w-4 rounded-full bg-white transition-bounce", checked && "translate-x-4")} />
    </div>
  );
}
