use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{games, roms, settings, systems};
use crate::emulator::command::{build_launch_command, RetroarchOptions, SystemLaunchConfig};
use crate::emulator::process::{self, LauncherStatus};
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
    process::kill(&state.active_pid).map_err(|e| e.to_string())
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
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Game not found: {game_id}"))?;
    let rom = roms::get(pool.inner(), &game.rom_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Rom not found: {}", game.rom_id))?;
    let system = systems::get(pool.inner(), &rom.system_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("System not found: {}", rom.system_id))?;

    let rom_path = paths::roms_path().join(&rom.path).to_string_lossy().into_owned();
    let system_config =
        SystemLaunchConfig { retroarch_core: system.retroarch_core.as_deref(), standalone_binary: system.standalone_binary.as_deref() };

    let retroarch_options = match &system.retroarch_core {
        Some(_) => Some(build_retroarch_options(&app, pool.inner(), &game_id).await?),
        None => None,
    };

    let launch_command = build_launch_command(&system_config, &rom_path, retroarch_options.as_ref()).map_err(|e| e.to_string())?;

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
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// `cores_path` comes from Settings (mirrors the MVP's `store.get("retroarchCoresPath")`), and
/// `append_config_path` is currently an empty placeholder file -- REL-106 will populate it with
/// real save/state/screenshot directory overrides; an empty file is a harmless no-op
/// `--appendconfig` target in the meantime, so a RetroArch launch itself isn't blocked on that.
async fn build_retroarch_options<R: tauri::Runtime>(app: &AppHandle<R>, pool: &SqlitePool, game_id: &str) -> Result<RetroarchOptions, String> {
    let cores_path = settings::get(pool, "retroarchCoresPath")
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No RetroArch cores path configured (Settings)".to_string())?;

    let config_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("launch-configs");
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
    let append_config_path = config_dir.join(format!("{game_id}.cfg"));
    if !append_config_path.exists() {
        tokio::fs::write(&append_config_path, b"").await.map_err(|e| e.to_string())?;
    }

    Ok(RetroarchOptions { cores_path: PathBuf::from(cores_path), append_config_path })
}
