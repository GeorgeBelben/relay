import { Children, cloneElement, isValidElement, useEffect, type ReactElement, type ReactNode } from "react";
import useEmblaCarousel from "embla-carousel-react";
import { FOCUS_RING, FocusContext, useFocusable } from "@/lib/focus";

type CarouselProps = {
  focusKey: string;
  autoFocus?: boolean;
  children: ReactNode;
};

type TileFocusProps = { onTileFocus?: () => void };

// Nintendo Switch home-screen style: fixed-size tiles (not a fraction of the row's width), the
// selected one grows, and there's always a sliver of the next tile peeking at the edge to hint
// there's more. Only one carousel exists today (Home's recents row), so these live here rather
// than being threaded through as props -- becomes one if a second carousel needs a different size.
export const CAROUSEL_TILE_WIDTH = "w-48";
export const CAROUSEL_TILE_WIDTH_FOCUSED = `w-70 rounded-2xl ${FOCUS_RING}`;

// Like Grid, but a real Embla carousel -- snap-to-slide motion, instead of an abrupt edge-to-edge
// native scroll. Embla has no idea when virtual focus moves (gamepad/keyboard navigation never
// touches real DOM focus -- see lib/focus), so each slide is cloned here with an `onTileFocus`
// that reports its index back up.
//
// The focused tile grows via a *real* width change (CarouselGameTile swaps CAROUSEL_TILE_WIDTH
// for CAROUSEL_TILE_WIDTH_FOCUSED), the same mechanism the grid tiles already use for their own
// h-48/h-64 grow-on-focus, just on the other axis. That was a deliberate do-over of an earlier
// transform-based version (scale() on an absolutely-positioned inner box) that turned into a
// losing battle against overflow clipping somewhere in the ancestor chain, and which overlapped
// the next tile rather than genuinely moving it. Real width growth pushes later slides along via
// normal flex reflow instead -- no overlap, no clipping, and "grows from the left" falls out for
// free since a wider box's left edge doesn't move.
//
// One deliberate approximation: Embla is left believing every slide is always at its resting
// width (watchResize: false below), even though the focused one is actually wider. Embla's
// default behavior is to auto-correct for a slide resizing via its own ResizeObserver -- but
// that correction is an instant, unanimated reInit(), which was fighting the width's smooth CSS
// transition (a hard snap mid-grow) and is exactly what read as "jarring". Since only one slide
// is ever wider at a time, the positional error this approximation introduces is small and
// doesn't compound -- a good trade for a scroll that's actually allowed to animate.
export function Carousel({ focusKey, autoFocus, children }: CarouselProps) {
  // containScroll: "trimSnaps" -- without it, the asymmetric px-16 inset (matching space on both
  // sides so the last slide gets the same reveal the first one does) throws off Embla's scroll
  // range and the last slide falls short of the viewport, always partly hidden.
  const [viewportRef, emblaApi] = useEmblaCarousel({ align: "start", watchResize: false, containScroll: "trimSnaps" });
  const { ref, focusKey: resolvedFocusKey, focusSelf } = useFocusable({
    focusKey,
    trackChildren: true,
    saveLastFocusedChild: true,
  });

  useEffect(() => {
    if (autoFocus) focusSelf();
  }, [autoFocus, focusSelf]);

  const slides = Children.toArray(children);

  return (
    <FocusContext.Provider value={resolvedFocusKey}>
      {/* min-w-0 overrides flexbox's default `min-width: auto` on a flex item, which otherwise
          refuses to shrink below its content's intrinsic width -- since that content is every
          slide laid out at full width (Embla clips visually via the viewport's overflow-hidden,
          it doesn't constrain layout width), without this the whole row balloons out to fit all
          of them, and the *page* scrolls horizontally instead of the carousel. */}
      <div ref={ref} className="w-full min-w-0 space-y-2">
        <div ref={viewportRef} className="overflow-clip px-16 py-8">
          {/* Fixed height, sized to the focused tile (w-70, aspect-2/3) -- otherwise the row's own
              height is derived from whichever tile is currently tallest, which wobbles for the
              ~200ms both the outgoing and incoming focused tile are mid-transition at once. With
              items-end that wobble is entirely at the top edge, which read as a jump; a fixed
              height means the row never has to resize at all. */}
          <div className="flex h-112 items-end gap-8">
            {slides.map((slide, index) =>
              isValidElement(slide)
                ? cloneElement(slide as ReactElement<TileFocusProps>, {
                    onTileFocus: () => emblaApi?.scrollTo(index),
                  })
                : slide,
            )}
          </div>
        </div>
      </div>
    </FocusContext.Provider>
  );
}
