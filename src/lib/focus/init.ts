import { GetBoundingClientRectAdapter, init } from "@noriginmedia/norigin-spatial-navigation-core";

// Must run once, before any `useFocusable` component mounts -- called from main.tsx, module
// scope, before the render call. shouldUseNativeEvents stays at its default (false): the
// library calls preventDefault/stopPropagation on the arrow/enter keydowns it owns natively,
// which is what stops the browser from also scrolling the page.
//
// layoutAdapter: the default adapter measures each element's position relative to its own
// immediate DOM parent, not a shared coordinate space -- fine for siblings under one container,
// but directional navigation between elements in different wrapper elements (e.g. two <section>s)
// silently fails to find a next candidate, since their coordinates aren't comparable. This
// adapter uses getBoundingClientRect() instead, which is always viewport-relative regardless of
// DOM nesting.
export function initFocusEngine() {
  init({ debug: false, visualDebug: false, layoutAdapter: GetBoundingClientRectAdapter });
}
