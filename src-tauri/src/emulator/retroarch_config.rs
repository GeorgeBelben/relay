//! Generates the per-launch `--appendconfig` file for RetroArch (REL-106), overlaying per-game
//! save/state/screenshot directories on top of whatever the user's own retroarch.cfg already has
//! (input bindings, video driver, etc, configured once through RetroArch's own menu) rather than
//! replacing it outright. Ported from the Electron MVP's launcher/retroarchConfig.ts, minus the
//! RetroAchievements cheevos_* keys -- no RA integration exists in this rewrite yet.

use std::path::Path;

// RetroArch's own network command interface, verified against the real source
// (github.com/libretro/RetroArch, command.h/command.c) rather than guessed:
// - network_cmd_enable defaults to false (config.def.h) -- forced on per-launch, since we don't
//   want to touch the user's own base retroarch.cfg just to get this one feature.
// - network_cmd_port defaults to 55355 (config.def.h) -- kept at the default rather than made
//   configurable; nothing about a single-device console needs a second RetroArch instance
//   competing for the port.
pub const NETWORK_CMD_PORT: u16 = 55355;

pub struct GameLaunchDirs<'a> {
    pub saves_dir: &'a Path,
    pub save_states_dir: &'a Path,
    pub screenshots_dir: &'a Path,
}

pub fn append_config_contents(dirs: &GameLaunchDirs) -> String {
    format!(
        "network_cmd_enable = \"true\"\nnetwork_cmd_port = \"{}\"\nsavefile_directory = \"{}\"\nsavestate_directory = \"{}\"\nscreenshot_directory = \"{}\"\n",
        NETWORK_CMD_PORT,
        dirs.saves_dir.to_string_lossy(),
        dirs.save_states_dir.to_string_lossy(),
        dirs.screenshots_dir.to_string_lossy(),
    )
}

/// Creates the per-game save/state/screenshot directories (RetroArch won't create them itself)
/// and writes the appendconfig file pointing at them.
pub async fn write_launch_config(config_path: &Path, dirs: &GameLaunchDirs<'_>) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dirs.saves_dir).await?;
    tokio::fs::create_dir_all(dirs.save_states_dir).await?;
    tokio::fs::create_dir_all(dirs.screenshots_dir).await?;
    tokio::fs::write(config_path, append_config_contents(dirs)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_config_contents_includes_network_cmd_and_all_three_directories() {
        let dirs = GameLaunchDirs {
            saves_dir: Path::new("/home/user/Relay/saves/nes/game-1"),
            save_states_dir: Path::new("/home/user/Relay/savestates/nes/game-1"),
            screenshots_dir: Path::new("/home/user/Relay/screenshots/nes/game-1"),
        };

        let contents = append_config_contents(&dirs);

        assert_eq!(
            contents,
            "network_cmd_enable = \"true\"\n\
             network_cmd_port = \"55355\"\n\
             savefile_directory = \"/home/user/Relay/saves/nes/game-1\"\n\
             savestate_directory = \"/home/user/Relay/savestates/nes/game-1\"\n\
             screenshot_directory = \"/home/user/Relay/screenshots/nes/game-1\"\n"
        );
    }

    #[tokio::test]
    async fn write_launch_config_creates_directories_and_writes_the_config_file() {
        let temp = tempfile::tempdir().unwrap();
        let saves_dir = temp.path().join("saves/nes/game-1");
        let save_states_dir = temp.path().join("savestates/nes/game-1");
        let screenshots_dir = temp.path().join("screenshots/nes/game-1");
        let config_path = temp.path().join("launch.cfg");

        let dirs = GameLaunchDirs { saves_dir: &saves_dir, save_states_dir: &save_states_dir, screenshots_dir: &screenshots_dir };
        write_launch_config(&config_path, &dirs).await.unwrap();

        assert!(saves_dir.is_dir());
        assert!(save_states_dir.is_dir());
        assert!(screenshots_dir.is_dir());

        let contents = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(contents.contains(&format!("savefile_directory = \"{}\"", saves_dir.to_string_lossy())));
    }
}
