use sqlx::SqlitePool;
use tauri::State;

use crate::db::roms::{self, NewRom, Rom};

#[tauri::command]
pub async fn list_roms(pool: State<'_, SqlitePool>) -> Result<Vec<Rom>, String> {
    roms::list(pool.inner()).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn get_rom(pool: State<'_, SqlitePool>, id: String) -> Result<Option<Rom>, String> {
    roms::get(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn create_rom(
    pool: State<'_, SqlitePool>,
    system_id: String,
    path: String,
    crc32: Option<String>,
    size_bytes: Option<i64>,
    discs: Option<String>,
) -> Result<Rom, String> {
    roms::create(pool.inner(), NewRom { system_id, path, crc32, size_bytes, discs })
        .await
        .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn update_rom(
    pool: State<'_, SqlitePool>,
    id: String,
    system_id: String,
    path: String,
    crc32: Option<String>,
    size_bytes: Option<i64>,
    discs: Option<String>,
) -> Result<Rom, String> {
    roms::update(pool.inner(), &id, NewRom { system_id, path, crc32, size_bytes, discs })
        .await
        .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn delete_rom(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    roms::delete(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}
