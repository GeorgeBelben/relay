import { Header } from "@/components/header";
import { HintBar } from "@/components/hint-bar";
import { useFocusBridge } from "@/lib/focus/bridge";
import { useGamepadListeners } from "@/lib/gamepad/use-gamepad-listeners";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Toaster } from 'sonner';

export const Route = createRootRoute({
  component: () => {

    useGamepadListeners();
    useFocusBridge();

    return (
      <>
        <Header />
        <Outlet />
        <HintBar />
        <Toaster position="bottom-center" />
      </>
    );
  },
});
