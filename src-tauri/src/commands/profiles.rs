use sqlx::SqlitePool;
use tauri::State;

use crate::db::profiles::{self, Profile};

#[tauri::command]
pub async fn list_profiles(pool: State<'_, SqlitePool>) -> Result<Vec<Profile>, String> {
    profiles::list(pool.inner()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_profile(pool: State<'_, SqlitePool>, id: String) -> Result<Option<Profile>, String> {
    profiles::get(pool.inner(), &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_profile(pool: State<'_, SqlitePool>, name: String) -> Result<Profile, String> {
    profiles::create(pool.inner(), &name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_profile(pool: State<'_, SqlitePool>, id: String, name: String) -> Result<Profile, String> {
    profiles::rename(pool.inner(), &id, &name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_profile(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    profiles::delete(pool.inner(), &id).await.map_err(|e| e.to_string())
}
