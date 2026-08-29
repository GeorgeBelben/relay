pub mod commands;
pub mod db;
pub mod emulator;
pub mod ingestion;
pub mod system;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let data_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");

                let options = SqliteConnectOptions::new()
                    .filename(data_dir.join("relay.db"))
                    .create_if_missing(true);
                let pool = SqlitePoolOptions::new()
                    .connect_with(options)
                    .await
                    .expect("failed to connect to database");

                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("failed to run migrations");

                app_handle.manage(pool);
            });
            app.manage(commands::ingestion::RescanGuard::default());
            app.manage(commands::ingestion::ScanStatusState::default());
            app.manage(commands::emulator::LauncherState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::ingestion::rescan_library,
            commands::ingestion::get_scan_status,
            commands::emulator::launch_game,
            commands::emulator::kill_game,
            commands::emulator::get_launcher_status,
            commands::games::list_games,
            commands::games::get_game,
            commands::games::create_game,
            commands::games::update_game,
            commands::games::delete_game,
            commands::roms::list_roms,
            commands::roms::get_rom,
            commands::roms::create_rom,
            commands::roms::update_rom,
            commands::roms::delete_rom,
            commands::systems::list_systems,
            commands::systems::get_system,
            commands::systems::create_system,
            commands::systems::update_system,
            commands::systems::delete_system,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_general_settings,
            commands::settings::set_onboarding_completed,
            commands::settings::set_controller_type,
            commands::settings::set_active_profile_id,
            commands::settings::set_retroarch_cores_path,
            commands::settings::set_wallpaper,
            commands::settings::set_sound_volume,
            commands::settings::set_rumble_enabled,
            commands::storage::get_storage_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
