import type { ControllerType } from "@/hooks/use-settings";
import type { InputMethod, NavAction, NavDirection } from "@/lib/input";

import xboxUp from "@/assets/input-icons/xbox/up.svg";
import xboxDown from "@/assets/input-icons/xbox/down.svg";
import xboxLeft from "@/assets/input-icons/xbox/left.svg";
import xboxRight from "@/assets/input-icons/xbox/right.svg";
import xboxConfirm from "@/assets/input-icons/xbox/confirm.svg";
import xboxBack from "@/assets/input-icons/xbox/back.svg";
import xboxMenu from "@/assets/input-icons/xbox/menu.svg";

import playstationUp from "@/assets/input-icons/playstation/up.svg";
import playstationDown from "@/assets/input-icons/playstation/down.svg";
import playstationLeft from "@/assets/input-icons/playstation/left.svg";
import playstationRight from "@/assets/input-icons/playstation/right.svg";
import playstationConfirm from "@/assets/input-icons/playstation/confirm.svg";
import playstationBack from "@/assets/input-icons/playstation/back.svg";
import playstationMenu from "@/assets/input-icons/playstation/menu.svg";

import switchUp from "@/assets/input-icons/switch/up.svg";
import switchDown from "@/assets/input-icons/switch/down.svg";
import switchLeft from "@/assets/input-icons/switch/left.svg";
import switchRight from "@/assets/input-icons/switch/right.svg";
import switchConfirm from "@/assets/input-icons/switch/confirm.svg";
import switchBack from "@/assets/input-icons/switch/back.svg";
import switchMenu from "@/assets/input-icons/switch/menu.svg";

import genericUp from "@/assets/input-icons/generic/up.svg";
import genericDown from "@/assets/input-icons/generic/down.svg";
import genericLeft from "@/assets/input-icons/generic/left.svg";
import genericRight from "@/assets/input-icons/generic/right.svg";
import genericConfirm from "@/assets/input-icons/generic/confirm.svg";
import genericBack from "@/assets/input-icons/generic/back.svg";
import genericMenu from "@/assets/input-icons/generic/menu.svg";

import keyboardUp from "@/assets/input-icons/keyboard/up.svg";
import keyboardDown from "@/assets/input-icons/keyboard/down.svg";
import keyboardLeft from "@/assets/input-icons/keyboard/left.svg";
import keyboardRight from "@/assets/input-icons/keyboard/right.svg";
import keyboardConfirm from "@/assets/input-icons/keyboard/confirm.svg";
import keyboardBack from "@/assets/input-icons/keyboard/back.svg";
import keyboardMenu from "@/assets/input-icons/keyboard/menu.svg";

// Excludes "home" -- see lib/hints/types.ts's Hint for why; there's no icon set for it since
// nothing ever renders it as a hint.
export type HintKey = NavDirection | Exclude<NavAction, "home">;

type IconSet = Record<HintKey, string>;

const CONTROLLER_ICON_SETS: Record<ControllerType, IconSet> = {
  xbox: {
    up: xboxUp,
    down: xboxDown,
    left: xboxLeft,
    right: xboxRight,
    confirm: xboxConfirm,
    back: xboxBack,
    menu: xboxMenu,
  },
  playstation: {
    up: playstationUp,
    down: playstationDown,
    left: playstationLeft,
    right: playstationRight,
    confirm: playstationConfirm,
    back: playstationBack,
    menu: playstationMenu,
  },
  switch: {
    up: switchUp,
    down: switchDown,
    left: switchLeft,
    right: switchRight,
    confirm: switchConfirm,
    back: switchBack,
    menu: switchMenu,
  },
  generic: {
    up: genericUp,
    down: genericDown,
    left: genericLeft,
    right: genericRight,
    confirm: genericConfirm,
    back: genericBack,
    menu: genericMenu,
  },
};

const KEYBOARD_ICON_SET: IconSet = {
  up: keyboardUp,
  down: keyboardDown,
  left: keyboardLeft,
  right: keyboardRight,
  confirm: keyboardConfirm,
  back: keyboardBack,
  menu: keyboardMenu,
};

export function getIconUrl(
  inputMethod: InputMethod,
  controllerType: ControllerType,
  key: HintKey,
): string {
  const set = inputMethod === "keyboard" ? KEYBOARD_ICON_SET : CONTROLLER_ICON_SETS[controllerType];
  return set[key];
}
