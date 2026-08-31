pub mod commands;
pub mod db;
pub mod emulator;
pub mod game_actions;
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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            // Kiosk has no mouse input; hide the cursor and only reveal the window once it's
            // hidden, so there's never a frame with GTK's default arrow visible (see main.css's
            // `cursor: none`, which can't take effect until the page's own CSS has loaded).
            let window = app.get_webview_window("main").expect("main window must exist");
            window.set_cursor_visible(false)?;
            window.show()?;

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
            commands::network::list_wifi_networks,
            commands::network::connect_to_wifi_network,
            commands::bluetooth::scan_for_bluetooth_devices,
            commands::bluetooth::list_paired_bluetooth_devices,
            commands::bluetooth::pair_bluetooth_device,
            commands::bluetooth::remove_bluetooth_device,
            commands::profiles::list_profiles,
            commands::profiles::get_profile,
            commands::profiles::create_profile,
            commands::profiles::rename_profile,
            commands::profiles::delete_profile,
            commands::game_media::list_game_media,
            commands::game_media::get_media_root_path,
            commands::library::list_library_shelves,
            commands::library::list_all_games_in_library,
            commands::library::list_recently_added_games,
            commands::game_actions::search_alternate_matches,
            commands::game_actions::apply_match,
            commands::system::get_username,
            commands::system::list_wallpapers,
            commands::system::quit,
            commands::datetime::get_datetime_status,
            commands::datetime::list_timezones,
            commands::datetime::set_timezone,
            commands::datetime::set_ntp_enabled,
            commands::datetime::set_time,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
