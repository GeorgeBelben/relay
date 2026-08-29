use sqlx::SqlitePool;
use tauri::State;

use crate::db::settings::{self, ControllerType, GeneralSettings};

#[tauri::command]
pub async fn get_setting(pool: State<'_, SqlitePool>, key: String) -> Result<Option<String>, String> {
    settings::get(pool.inner(), &key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(pool: State<'_, SqlitePool>, key: String, value: String) -> Result<(), String> {
    settings::set(pool.inner(), &key, &value).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_general_settings(pool: State<'_, SqlitePool>) -> Result<GeneralSettings, String> {
    settings::get_general_settings(pool.inner()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_onboarding_completed(pool: State<'_, SqlitePool>, completed: bool) -> Result<(), String> {
    settings::set_onboarding_completed(pool.inner(), completed).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_controller_type(pool: State<'_, SqlitePool>, controller_type: ControllerType) -> Result<(), String> {
    settings::set_controller_type(pool.inner(), controller_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_profile_id(pool: State<'_, SqlitePool>, profile_id: Option<String>) -> Result<(), String> {
    settings::set_active_profile_id(pool.inner(), profile_id.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_retroarch_cores_path(pool: State<'_, SqlitePool>, path: String) -> Result<(), String> {
    settings::set_retroarch_cores_path(pool.inner(), &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_wallpaper(pool: State<'_, SqlitePool>, wallpaper: Option<String>) -> Result<(), String> {
    settings::set_wallpaper(pool.inner(), wallpaper.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_sound_volume(pool: State<'_, SqlitePool>, volume: i64) -> Result<(), String> {
    settings::set_sound_volume(pool.inner(), volume).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_rumble_enabled(pool: State<'_, SqlitePool>, enabled: bool) -> Result<(), String> {
    settings::set_rumble_enabled(pool.inner(), enabled).await.map_err(|e| e.to_string())
}
