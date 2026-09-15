use crate::emulator::EmulatorBackend;
use crate::gamescope::gamescope_env;
use crate::startup::bios_dir;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};

const PCSX2_BIN: &str = "/usr/games/pcsx2-qt";

// EmulatorBackend implementation that shells out to pcsx2
pub struct Pcsx2Backend;

impl EmulatorBackend for Pcsx2Backend {
    fn launch(&self, rom_path: &str) -> Result<Child, String> {
        ensure_pcsx2_config("SCPH-70004.BIN");

        Command::new(PCSX2_BIN)
            .arg("-batch")
            .arg("-fullscreen")
            .arg(rom_path)
            .envs(gamescope_env())
            .spawn()
            .map_err(|e| format!("failed to launch Pcsx2: {e}"))
    }
}

fn pcsx2_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("could not determine a config directory for this user")
        .join("PCSX2/inis/PCSX2.ini")
}

/// Patches the handful of PCSX2.ini keys we care about, leaving everything
/// else PCSX2 generated for itself untouched.
///
/// Assumes the ini already exists (created by PCSX2's own first run / `-testconfig`)
/// — a fresh empty file wouldn't have the `[Folders]`/`[Filenames]`/`[UI]` sections
/// or their other default keys, so there's nothing safe to patch yet.
pub fn ensure_pcsx2_config(bios_filename: &str) {
    let config_path = pcsx2_config_path();

    let contents = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", config_path.display()));

    let bios_dir = bios_dir().to_string_lossy().to_string();

    let patched: Vec<String> = contents
        .lines()
        .map(|line| {
            if line.starts_with("Bios = ") {
                format!("Bios = {bios_dir}")
            } else if line.starts_with("BIOS = ") {
                format!("BIOS = {bios_filename}")
            } else if line.starts_with("SetupWizardIncomplete = ") {
                "SetupWizardIncomplete = false".to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    fs::write(&config_path, patched.join("\n") + "\n")
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", config_path.display()));
}
