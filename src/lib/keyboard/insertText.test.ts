import { describe, expect, it } from "vitest";
import { deleteBeforeCursor, insertTextAtCursor } from "./insertText";

function makeInput(value: string, selectionStart: number, selectionEnd = selectionStart) {
  const el = document.createElement("input");
  el.value = value;
  el.setSelectionRange(selectionStart, selectionEnd);
  return el;
}

describe("insertTextAtCursor", () => {
  it("inserts at the cursor position", () => {
    const el = makeInput("helloworld", 5);
    insertTextAtCursor(el, " ");
    expect(el.value).toBe("hello world");
    expect(el.selectionStart).toBe(6);
    expect(el.selectionEnd).toBe(6);
  });

  it("appends when the cursor is at the end", () => {
    const el = makeInput("mario", 5);
    insertTextAtCursor(el, "!");
    expect(el.value).toBe("mario!");
  });

  it("replaces a selection rather than inserting alongside it", () => {
    const el = makeInput("mario kart", 0, 5);
    insertTextAtCursor(el, "zelda");
    expect(el.value).toBe("zelda kart");
    expect(el.selectionStart).toBe(5);
  });

  it("fires a real input event so a React onChange listener sees the change", () => {
    const el = makeInput("ab", 2);
    let seen: string | null = null;
    el.addEventListener("input", () => {
      seen = el.value;
    });
    insertTextAtCursor(el, "c");
    expect(seen).toBe("abc");
  });
});

describe("deleteBeforeCursor", () => {
  it("deletes the character before the cursor", () => {
    const el = makeInput("mario", 5);
    deleteBeforeCursor(el);
    expect(el.value).toBe("mari");
    expect(el.selectionStart).toBe(4);
  });

  it("deletes the selection instead of a single character when there is one", () => {
    const el = makeInput("mario kart", 0, 5);
    deleteBeforeCursor(el);
    expect(el.value).toBe(" kart");
    expect(el.selectionStart).toBe(0);
  });

  it("is a no-op at the start of the field", () => {
    const el = makeInput("mario", 0);
    deleteBeforeCursor(el);
    expect(el.value).toBe("mario");
  });
});
