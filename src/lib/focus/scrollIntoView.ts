// Shared onFocus scroll behavior for every focusable row/tile in the app -- centers the focused
// element vertically in its scroll container (block: "center") rather than snapping it to
// whichever edge it happened to approach from, which is what scrollIntoView's own default
// ("nearest") does. With no mouse in the picture, focus movement is the only thing that can drive
// scrolling at all, so this runs from onFocus, not a click/hover handler.
export function scrollFocusedIntoView(el: Element | null): void {
  el?.scrollIntoView({ behavior: "smooth", inline: "nearest", block: "center" });
}
