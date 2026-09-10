import { useEffect, useState } from "react";
import { getGamepads, guessControllerType } from "./gamepad";
import { useGamepadTypeStore } from "./use-gamepad-type";

function readConnectedIndexes(): Gamepad[] {
  return [...getGamepads()].filter((pad): pad is Gamepad => pad !== null);
}

export function useConnectedGamepads() {
  const [connectedGamepads, setConnectedGamepads] =
    useState<(Gamepad | null)[]>(readConnectedIndexes());

  useEffect(() => {
    const refresh = () => {
      useGamepadTypeStore
        .getState()
        .setLastGamepadType(guessControllerType(connectedGamepads[0]?.id ?? ""));
      setConnectedGamepads(readConnectedIndexes());
    };

    refresh();

    window.addEventListener("gamepadconnected", refresh);
    window.addEventListener("gamepaddisconnected", refresh);

    return () => {
      window.removeEventListener("gamepadconnected", refresh);
      window.removeEventListener("gamepaddisconnected", refresh);
    };
  }, []);

  return connectedGamepads;
}
