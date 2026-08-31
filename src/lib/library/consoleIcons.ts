import nes from "@/assets/console-logos/nintendo_nes.svg";
import snes from "@/assets/console-logos/nintendo_snes.svg";
import n64 from "@/assets/console-logos/nintendo_64.svg";
import gb from "@/assets/console-logos/nintendo_gameboy.svg";
import gbc from "@/assets/console-logos/nintendo_gameboy_color.svg";
import gba from "@/assets/console-logos/nintendo_gameboy_advance.svg";
import nds from "@/assets/console-logos/nintendo_ds.svg";
import gamecube from "@/assets/console-logos/nintendo_gamecube.svg";
import wii from "@/assets/console-logos/nintendo_wii.svg";
import psx from "@/assets/console-logos/playstation_tall.svg";
import ps2 from "@/assets/console-logos/playstation_ps2.svg";
import psp from "@/assets/console-logos/playstation_psp.svg";
import mastersystem from "@/assets/console-logos/sega_master_system.svg";
import gamegear from "@/assets/console-logos/sega_gamegear.svg";
import megadrive from "@/assets/console-logos/sega_megadrive.svg";
import saturn from "@/assets/console-logos/sega_saturn.svg";
import dreamcast from "@/assets/console-logos/sega_dreamcast.svg";
import xbox from "@/assets/console-logos/xbox_original.svg";
import mame from "@/assets/console-logos/mame.svg";
import ngpc from "@/assets/console-logos/snk_neogeo_pocket_color.svg";
import wonderswan from "@/assets/console-logos/bandai_wonderswan_color.svg";

// Keyed by system id (src-tauri/src/db/seed/systems.rs, or its Rust equivalent) -- matches the
// Electron MVP's own folder/system table.
const CONSOLE_ICON_URLS: Record<string, string> = {
  nes,
  snes,
  n64,
  gb,
  gbc,
  gba,
  nds,
  gamecube,
  wii,
  psx,
  ps2,
  psp,
  mastersystem,
  gamegear,
  megadrive,
  saturn,
  dreamcast,
  xbox,
  mame,
  ngpc,
  wonderswan,
};

export function getConsoleIconUrl(systemId: string): string | undefined {
  return CONSOLE_ICON_URLS[systemId];
}
