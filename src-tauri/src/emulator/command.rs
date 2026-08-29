//! Pure, synchronous command-building -- no filesystem/process access, so it's unit-testable
//! without touching a real emulator install. Ported from the Electron MVP's
//! `launcher/buildLaunchCommand.ts`.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    pub command: String,
    pub args: Vec<String>,
}

/// Only needed on the RetroArch path -- a standalone emulator has no core to resolve and isn't
/// pointed at a generated config.
pub struct RetroarchOptions {
    pub cores_path: PathBuf,
    pub append_config_path: PathBuf,
}

/// `retroarch_core`/`standalone_binary` mirror `db::systems::System`'s two mutually-exclusive
/// fields directly -- kept as a separate, minimal struct here so this module doesn't depend on
/// `db` for a pure function that only ever reads two fields off it.
pub struct SystemLaunchConfig<'a> {
    pub retroarch_core: Option<&'a str>,
    pub standalone_binary: Option<&'a str>,
}

#[derive(Debug)]
pub enum BuildLaunchCommandError {
    MissingRetroarchOptions,
    NoEmulatorConfigured,
}

impl std::fmt::Display for BuildLaunchCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRetroarchOptions => {
                write!(f, "retroarchOptions is required to launch a RetroArch-core system")
            }
            Self::NoEmulatorConfigured => {
                write!(f, "System has neither a RetroArch core nor a standalone binary configured")
            }
        }
    }
}

impl std::error::Error for BuildLaunchCommandError {}

// Best-effort CLI conventions for the standalone emulators -- unlike the RetroArch path below (a
// single, well-documented `-L <core> <rom>` invocation), these haven't been checked against a
// real install on this rewrite. Confirm and adjust once this actually runs against real binaries
// on the device.
fn standalone_args(binary: &str, rom_path: &str) -> Vec<String> {
    match binary {
        // -b/--batch: quit when the game closes instead of returning to Dolphin's own UI.
        // -e/--exec: the disc image to boot straight into.
        "dolphin-emu" => vec!["-b".to_string(), "-e".to_string(), rom_path.to_string()],
        // xemu's CLI takes the disc image via -dvd_path rather than a bare positional argument.
        "xemu" => vec!["-dvd_path".to_string(), rom_path.to_string()],
        // PCSX2's Qt frontend accepts a bare positional path; deliberately not passing a
        // fullscreen flag here since the exact spelling (-fullscreen vs --fullscreen) isn't
        // confirmed.
        _ => vec![rom_path.to_string()],
    }
}

pub fn build_launch_command(
    system: &SystemLaunchConfig,
    rom_path: &str,
    retroarch_options: Option<&RetroarchOptions>,
) -> Result<LaunchCommand, BuildLaunchCommandError> {
    if let Some(binary) = system.standalone_binary {
        return Ok(LaunchCommand { command: binary.to_string(), args: standalone_args(binary, rom_path) });
    }

    if let Some(core) = system.retroarch_core {
        let options = retroarch_options.ok_or(BuildLaunchCommandError::MissingRetroarchOptions)?;
        let core_path = options.cores_path.join(format!("{core}_libretro.so"));
        return Ok(LaunchCommand {
            command: "retroarch".to_string(),
            args: vec![
                "-L".to_string(),
                path_to_string(&core_path),
                rom_path.to_string(),
                "--appendconfig".to_string(),
                path_to_string(&options.append_config_path),
            ],
        });
    }

    Err(BuildLaunchCommandError::NoEmulatorConfigured)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retroarch_resolves_the_core_against_the_cores_directory_and_appends_the_launch_config() {
        let system = SystemLaunchConfig { retroarch_core: Some("mesen"), standalone_binary: None };
        let options = RetroarchOptions {
            cores_path: PathBuf::from("/opt/retroarch/cores"),
            append_config_path: PathBuf::from("/tmp/relay-launch-abc.cfg"),
        };

        let result = build_launch_command(&system, "/roms/nes/Game.nes", Some(&options)).unwrap();

        assert_eq!(result.command, "retroarch");
        assert_eq!(
            result.args,
            vec![
                "-L".to_string(),
                "/opt/retroarch/cores/mesen_libretro.so".to_string(),
                "/roms/nes/Game.nes".to_string(),
                "--appendconfig".to_string(),
                "/tmp/relay-launch-abc.cfg".to_string(),
            ]
        );
    }

    #[test]
    fn retroarch_errors_if_retroarch_options_isnt_provided() {
        let system = SystemLaunchConfig { retroarch_core: Some("mesen"), standalone_binary: None };
        let err = build_launch_command(&system, "/roms/nes/Game.nes", None).unwrap_err();
        assert!(matches!(err, BuildLaunchCommandError::MissingRetroarchOptions));
    }

    #[test]
    fn dolphin_emu_uses_batch_mode_and_exec_flag() {
        let system = SystemLaunchConfig { retroarch_core: None, standalone_binary: Some("dolphin-emu") };
        let result = build_launch_command(&system, "/roms/wii/Game.iso", None).unwrap();
        assert_eq!(result.command, "dolphin-emu");
        assert_eq!(result.args, vec!["-b".to_string(), "-e".to_string(), "/roms/wii/Game.iso".to_string()]);
    }

    #[test]
    fn xemu_uses_dvd_path_flag() {
        let system = SystemLaunchConfig { retroarch_core: None, standalone_binary: Some("xemu") };
        let result = build_launch_command(&system, "/roms/xbox/Game.iso", None).unwrap();
        assert_eq!(result.command, "xemu");
        assert_eq!(result.args, vec!["-dvd_path".to_string(), "/roms/xbox/Game.iso".to_string()]);
    }

    #[test]
    fn pcsx2_uses_a_bare_positional_path() {
        let system = SystemLaunchConfig { retroarch_core: None, standalone_binary: Some("pcsx2") };
        let result = build_launch_command(&system, "/roms/ps2/Game.iso", None).unwrap();
        assert_eq!(result.command, "pcsx2");
        assert_eq!(result.args, vec!["/roms/ps2/Game.iso".to_string()]);
    }

    #[test]
    fn falls_back_to_a_bare_positional_path_for_an_unrecognized_standalone_binary() {
        let system = SystemLaunchConfig { retroarch_core: None, standalone_binary: Some("some-future-emulator") };
        let result = build_launch_command(&system, "/roms/x/Game.rom", None).unwrap();
        assert_eq!(result.command, "some-future-emulator");
        assert_eq!(result.args, vec!["/roms/x/Game.rom".to_string()]);
    }

    #[test]
    fn errors_when_a_system_has_neither_a_core_nor_a_standalone_binary_configured() {
        let system = SystemLaunchConfig { retroarch_core: None, standalone_binary: None };
        let err = build_launch_command(&system, "/roms/x/Game.rom", None).unwrap_err();
        assert!(matches!(err, BuildLaunchCommandError::NoEmulatorConfigured));
    }
}
