import type { OskTarget } from "./store";

// React tracks <input>/<textarea> values through a wrapped property setter, so a plain
// `el.value = x` is invisible to it -- the DOM updates but onChange never fires and a
// re-render would stomp the change right back. Going through the *native* setter (bypassing
// React's wrapper) and then dispatching a real "input" event is the standard workaround: React's
// delegated listener picks up the native event and its state update proceeds exactly as if the
// user had typed it.
function setNativeValue(el: OskTarget, value: string) {
  const prototype = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(prototype, "value")!.set!;
  setter.call(el, value);
  el.dispatchEvent(new Event("input", { bubbles: true }));
}

// Inserts `text` at the current cursor position (replacing the selection, if any), matching
// what a physical keyboard press does. Falls back to appending at the end when the element has
// no selection (a plain click-in-and-type scenario always has one, but a fresh programmatic
// focus doesn't always set one on every platform).
export function insertTextAtCursor(el: OskTarget, text: string) {
  const start = el.selectionStart ?? el.value.length;
  const end = el.selectionEnd ?? el.value.length;
  const nextValue = el.value.slice(0, start) + text + el.value.slice(end);
  setNativeValue(el, nextValue);
  const cursor = start + text.length;
  el.setSelectionRange(cursor, cursor);
}

// Deletes the selection if there is one, otherwise the single character before the cursor --
// same behavior as a physical Backspace.
export function deleteBeforeCursor(el: OskTarget) {
  const start = el.selectionStart ?? el.value.length;
  const end = el.selectionEnd ?? el.value.length;
  const deleteFrom = start === end ? Math.max(0, start - 1) : start;
  if (deleteFrom === end) return;
  const nextValue = el.value.slice(0, deleteFrom) + el.value.slice(end);
  setNativeValue(el, nextValue);
  el.setSelectionRange(deleteFrom, deleteFrom);
}
