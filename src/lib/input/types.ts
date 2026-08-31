export type NavDirection = "up" | "down" | "left" | "right";
export type NavAction = "confirm" | "back" | "menu";

export type NavEvent = { type: "direction"; direction: NavDirection } | { type: "action"; action: NavAction };

export type InputMethod = "keyboard" | "gamepad";
