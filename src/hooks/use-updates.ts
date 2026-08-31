import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { useMutation } from "@tanstack/react-query";

// Tauri's updater plugin is a pull-based promise API (check() -> Update | null, then
// update.downloadAndInstall()), unlike the Electron MVP's push-based update:status channel
// (electron-updater's checking/available/downloading/downloaded events) -- there's no equivalent
// "currently in progress" state to mirror into a store here. A future Settings screen drives its
// own local progress state from downloadAndInstall's onEvent callback directly; this hook just
// exposes the two actions.
export function useCheckForUpdate() {
  return useMutation({
    mutationFn: (): Promise<Update | null> => check(),
  });
}

// The user consents once (choosing to install); downloading, installing, and relaunching happen
// without further input, same as the Electron original.
export function useDownloadAndInstallUpdate() {
  return useMutation({
    mutationFn: async (update: Update) => {
      await update.downloadAndInstall();
      await relaunch();
    },
  });
}
