use crate::{emulator::EmulatorBackend, gamescope::gamescope_env, settings::Settings};
use std::process::{Child, Command};

const RETROARCH_BIN: &str = "retroarch";

// EmulatorBackend implementation that shells out to retroarch
pub struct RetroArchBackend;

impl EmulatorBackend for RetroArchBackend {
    fn launch(&self, rom_path: &str, settings: &Settings) -> Result<Child, String> {
        let core_path = settings
            .get_str("retroarch.core_path")
            .ok_or("no retroarch.core_path setting configured".to_string())?;

        Command::new(RETROARCH_BIN)
            .arg("-L")
            .arg(core_path)
            .arg(rom_path)
            .arg("--fullscreen")
            .envs(gamescope_env())
            .spawn()
            .map_err(|e| format!("failed to launch RetroArch: {e}"))
    }
}
