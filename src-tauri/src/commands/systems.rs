use sqlx::SqlitePool;
use tauri::State;

use crate::db::systems::{self, NewSystem, System};

#[tauri::command]
pub async fn list_systems(pool: State<'_, SqlitePool>) -> Result<Vec<System>, String> {
    systems::list(pool.inner()).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn get_system(pool: State<'_, SqlitePool>, id: String) -> Result<Option<System>, String> {
    systems::get(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn create_system(
    pool: State<'_, SqlitePool>,
    id: String,
    name: String,
    extensions: String,
    retroarch_core: Option<String>,
    standalone_binary: Option<String>,
) -> Result<System, String> {
    systems::create(
        pool.inner(),
        NewSystem { id, name, extensions, retroarch_core, standalone_binary },
    )
    .await
    .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn update_system(
    pool: State<'_, SqlitePool>,
    id: String,
    name: String,
    extensions: String,
    retroarch_core: Option<String>,
    standalone_binary: Option<String>,
) -> Result<System, String> {
    systems::update(
        pool.inner(),
        &id,
        NewSystem { id: id.clone(), name, extensions, retroarch_core, standalone_binary },
    )
    .await
    .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn delete_system(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    systems::delete(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}
