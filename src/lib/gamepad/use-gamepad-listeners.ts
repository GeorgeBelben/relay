import { useEffect } from "react";
import { startGamepadListener } from "./gamepad";
import { createEmitter } from "../emitter";
import { NavEvent } from "./types";

export const navEvents = createEmitter<NavEvent>();

export function useGamepadListeners() {
  useEffect(() => {
    console.log("useGamepadListeners");
    const stopGamepad = startGamepadListener((event) => {
      console.log("gamepad event", event);
      navEvents.emit(event);

      //   soundEvents.emit(event);
      //   rumbleEvents.emit(event);
    });

    return () => {
      stopGamepad();
    };
  }, []);
}
