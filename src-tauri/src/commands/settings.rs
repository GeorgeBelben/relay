use sqlx::SqlitePool;
use tauri::State;

use crate::db::settings;

#[tauri::command]
pub async fn get_setting(pool: State<'_, SqlitePool>, key: String) -> Result<Option<String>, String> {
    settings::get(pool.inner(), &key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(pool: State<'_, SqlitePool>, key: String, value: String) -> Result<(), String> {
    settings::set(pool.inner(), &key, &value).await.map_err(|e| e.to_string())
}
