use crate::emulator::EmulatorBackend;
use crate::gamescope::gamescope_x11_display;
use crate::settings::Settings;
use crate::startup::{bios_dir, saves_dir, screenshots_dir, states_dir};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};

const PCSX2_BIN: &str = "/usr/games/pcsx2-qt";

// EmulatorBackend implementation that shells out to pcsx2
pub struct Pcsx2Backend;

impl EmulatorBackend for Pcsx2Backend {
    fn launch(&self, rom_path: &str, settings: &Settings) -> Result<Child, String> {
        ensure_pcsx2_config(settings)?;

        let mut command = Command::new(PCSX2_BIN);
        command.arg("-fullscreen").arg("-nogui").arg(rom_path);

        if let Some(display) = gamescope_x11_display() {
            command
                .env("DISPLAY", display)
                .env("QT_QPA_PLATFORM", "xcb");
        }

        command
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
pub fn ensure_pcsx2_config(settings: &Settings) -> Result<(), String> {
    let bios_filename = settings
        .get_str("pcsx2.bios_filename")
        .ok_or("no pcsx2.bios_filename setting configured".to_string())?;

    let upscaler_multiplier = settings.get_int("pcsx2.upscale_multiplier").unwrap_or(1);

    let config_path = pcsx2_config_path();

    let contents = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", config_path.display()));

    let bios_dir = bios_dir().to_string_lossy().to_string();
    let saves_dir = saves_dir().to_string_lossy().to_string();
    let states_dir = states_dir().to_string_lossy().to_string();
    let screenshots_dir = screenshots_dir().to_string_lossy().to_string();

    let patched: Vec<String> = contents
        .lines()
        .map(|line| {
            if line.starts_with("Bios = ") {
                format!("Bios = {bios_dir}")
            } else if line.starts_with("BIOS = ") {
                format!("BIOS = {bios_filename}")
            } else if line.starts_with("SetupWizardIncomplete = ") {
                "SetupWizardIncomplete = false".to_string()
            } else if line.starts_with("upscale_multiplier = ") {
                format!("upscale_multiplier = {upscaler_multiplier}")
            } else if line.starts_with("MemoryCards = ") {
                format!("MemoryCards = {saves_dir}")
            } else if line.starts_with("Savestates = ") {
                format!("Savestates = {states_dir}")
            } else if line.starts_with("Snapshots = ") {
                format!("Snapshots = {screenshots_dir}")
            } else {
                line.to_string()
            }
        })
        .collect();

    fs::write(&config_path, patched.join("\n") + "\n")
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", config_path.display()));

    Ok(())
}
