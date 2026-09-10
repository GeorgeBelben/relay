use crate::emulator::EmulatorBackend;
use std::process::{Child, Command};

const RETROARCH_BIN: &str = "retroarch";
const SNES_CORE_PATH: &str = "/usr/lib/x86_64-linux-gnu/libretro/snes9x_libretro.so";

// EmulatorBackend implementation that shells out to retroarch
pub struct RetroArchBackend;

impl EmulatorBackend for RetroArchBackend {
    fn launch(&self, rom_path: &str) -> Result<Child, String> {
        Command::new(RETROARCH_BIN)
            .arg("-L")
            .arg(SNES_CORE_PATH)
            .arg(rom_path)
            .spawn()
            .map_err(|e| format!("failed to launch RetroArch: {e}"))
    }
}
