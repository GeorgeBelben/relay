import type { CSSProperties, ReactNode } from "react";
import { useNavigate } from "@tanstack/react-router";
import { scrollFocusedIntoView, useFocusable } from "@/lib/focus";
import { cn } from "@/lib/cn";

type FocusableLinkProps = {
  to: string;
  focusKey?: string;
  className?: string | ((focused: boolean) => string);
  style?: CSSProperties;
  // Set by Carousel when this link is one of its slides -- see game-tile.tsx for why.
  onTileFocus?: () => void;
  children: ReactNode;
};

// Wraps the useFocusable + navigate boilerplate for a single navigable row/tile so screens don't
// hand-roll it per element. Pass a static className, or a function of `focused` for elements that
// need more than the default ring (e.g. swapping background color when focused).
export function FocusableLink({ to, focusKey, className, style, onTileFocus, children }: FocusableLinkProps) {
  const navigate = useNavigate();
  const { ref, focused } = useFocusable({
    focusKey,
    onEnterPress: () => navigate({ to }),
    onFocus: () => {
      onTileFocus?.();
      scrollFocusedIntoView(ref.current);
    },
  });

  const resolvedClassName = typeof className === "function" ? className(focused) : className;

  return (
    <div ref={ref} className={cn(resolvedClassName)} style={style}>
      {children}
    </div>
  );
}
