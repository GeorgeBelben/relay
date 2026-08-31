import { useEffect, type ReactNode } from "react";
import { FocusContext, useFocusable } from "@/lib/focus";

type GridProps = {
  focusKey: string;
  autoFocus?: boolean;
  children: ReactNode;
};

// Like Carousel, but wraps into a grid instead of scrolling horizontally. Used for the Home
// consoles section.
export function Grid({ focusKey, autoFocus, children }: GridProps) {
  const { ref, focusKey: resolvedFocusKey, focusSelf } = useFocusable({
    focusKey,
    trackChildren: true,
    saveLastFocusedChild: true,
  });

  useEffect(() => {
    if (autoFocus) focusSelf();
  }, [autoFocus, focusSelf]);

  return (
    <FocusContext.Provider value={resolvedFocusKey}>
      <div className="space-y-2">
        <div ref={ref} className="grid grid-cols-5 justify-center gap-6 px-16 pb-1">
          {children}
        </div>
      </div>
    </FocusContext.Provider>
  );
}
