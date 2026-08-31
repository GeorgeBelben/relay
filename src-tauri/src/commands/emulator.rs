use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{games, roms, settings, systems};
use crate::emulator::command::{build_launch_command, RetroarchOptions, SystemLaunchConfig};
use crate::emulator::process::{self, LauncherStatus};
use crate::emulator::retroarch_config::{self, GameLaunchDirs};
use crate::ingestion::paths;

/// Shared launch state, managed as Tauri state so it lives for the app's lifetime -- one game (so
/// one child process) can run at a time, matching the MVP's single-window/single-device model.
pub struct LauncherState {
    pub running: Arc<AtomicBool>,
    pub active_pid: Arc<AtomicU32>,
    pub status: Arc<Mutex<LauncherStatus>>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            active_pid: Arc::new(AtomicU32::new(0)),
            status: Arc::new(Mutex::new(LauncherStatus::Idle)),
        }
    }
}

#[tauri::command]
pub fn get_launcher_status(state: State<'_, LauncherState>) -> LauncherStatus {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub fn kill_game(state: State<'_, LauncherState>) -> Result<(), String> {
    process::kill(&state.active_pid).map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn launch_game<R: tauri::Runtime>(
    app: AppHandle<R>,
    pool: State<'_, SqlitePool>,
    state: State<'_, LauncherState>,
    game_id: String,
) -> Result<(), String> {
    let game = games::get(pool.inner(), &game_id)
        .await
        .map_err(crate::logging::err_to_string)?
        .ok_or_else(|| format!("Game not found: {game_id}"))?;
    let rom = roms::get(pool.inner(), &game.rom_id)
        .await
        .map_err(crate::logging::err_to_string)?
        .ok_or_else(|| format!("Rom not found: {}", game.rom_id))?;
    let system = systems::get(pool.inner(), &rom.system_id)
        .await
        .map_err(crate::logging::err_to_string)?
        .ok_or_else(|| format!("System not found: {}", rom.system_id))?;

    let rom_path = paths::roms_path().join(&rom.path).to_string_lossy().into_owned();
    let system_config =
        SystemLaunchConfig { retroarch_core: system.retroarch_core.as_deref(), standalone_binary: system.standalone_binary.as_deref() };

    let retroarch_options = match &system.retroarch_core {
        Some(_) => Some(build_retroarch_options(&app, pool.inner(), &rom.system_id, &game_id, &paths::library_root()).await?),
        None => None,
    };

    let launch_command = build_launch_command(&system_config, &rom_path, retroarch_options.as_ref()).map_err(crate::logging::err_to_string)?;

    process::launch(
        &launch_command.command,
        &launch_command.args,
        &state.running,
        &state.active_pid,
        |next| {
            if let Ok(mut guard) = state.status.lock() {
                *guard = next.clone();
            }
            let _ = app.emit("launcher:status", &next);
        },
        |log_line| {
            let _ = app.emit("launcher:log", &log_line);
        },
    )
    .await
    .map_err(crate::logging::err_to_string)?;

    Ok(())
}

/// `cores_path` comes from Settings, falling back to the MVP's own default (a stock Ubuntu apt
/// install's libretro cores directory) rather than erroring when unset -- REL-108.
/// `append_config_path` overlays this game's save/state/screenshot directories on top of the
/// user's own retroarch.cfg -- REL-106; overwritten fresh on every launch, so no per-launch
/// cleanup is needed (unlike the MVP's per-launch tmp file, since this rewrite's `process::launch`
/// awaits the whole process lifecycle before returning, well past the point RetroArch reads it).
/// `library_root` is taken as a parameter rather than resolved internally via
/// `paths::library_root()` (same reasoning as ingestion::enrich's `media_root`), so tests can
/// point this at a tempdir instead of ever touching the real ~/Relay.
async fn build_retroarch_options<R: tauri::Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    system_id: &str,
    game_id: &str,
    library_root: &Path,
) -> Result<RetroarchOptions, String> {
    let cores_path = settings::get_general_settings(pool).await.map_err(crate::logging::err_to_string)?.retroarch_cores_path;

    let config_dir = app.path().app_data_dir().map_err(crate::logging::err_to_string)?.join("launch-configs");
    tokio::fs::create_dir_all(&config_dir).await.map_err(crate::logging::err_to_string)?;
    let append_config_path = config_dir.join(format!("{game_id}.cfg"));

    let saves_dir = library_root.join("saves").join(system_id).join(game_id);
    let save_states_dir = library_root.join("savestates").join(system_id).join(game_id);
    let screenshots_dir = library_root.join("screenshots").join(system_id).join(game_id);
    let dirs = GameLaunchDirs { saves_dir: &saves_dir, save_states_dir: &save_states_dir, screenshots_dir: &screenshots_dir };
    retroarch_config::write_launch_config(&append_config_path, &dirs).await.map_err(crate::logging::err_to_string)?;

    Ok(RetroarchOptions { cores_path: PathBuf::from(cores_path), append_config_path })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    use tauri::WebviewWindowBuilder;

    async fn throwaway_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn build_retroarch_options_creates_per_game_save_dirs_and_a_populated_appendconfig() {
        let (pool, _db_dir) = throwaway_pool().await;
        let app = mock_builder().build(mock_context(noop_assets())).expect("failed to build mock app");
        let _webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        let app_handle = app.handle().clone();
        let library_root = tempfile::tempdir().unwrap();

        let options = build_retroarch_options(&app_handle, &pool, "nes", "game-1", library_root.path()).await.unwrap();

        assert!(library_root.path().join("saves/nes/game-1").is_dir());
        assert!(library_root.path().join("savestates/nes/game-1").is_dir());
        assert!(library_root.path().join("screenshots/nes/game-1").is_dir());

        let contents = tokio::fs::read_to_string(&options.append_config_path).await.unwrap();
        assert!(contents.contains(&format!(
            "savefile_directory = \"{}\"",
            library_root.path().join("saves/nes/game-1").to_string_lossy()
        )));
        // No retroarchCoresPath setting configured -- falls back to REL-108's default.
        assert_eq!(options.cores_path, PathBuf::from("/usr/lib/x86_64-linux-gnu/libretro"));
    }
}
