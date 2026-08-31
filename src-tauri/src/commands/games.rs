use sqlx::SqlitePool;
use tauri::State;

use crate::db::games::{self, Game, NewGame};

#[tauri::command]
pub async fn list_games(pool: State<'_, SqlitePool>) -> Result<Vec<Game>, String> {
    games::list(pool.inner()).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn get_game(pool: State<'_, SqlitePool>, id: String) -> Result<Option<Game>, String> {
    games::get(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn create_game(pool: State<'_, SqlitePool>, rom_id: String, title: String) -> Result<Game, String> {
    games::create(pool.inner(), NewGame { rom_id, title })
        .await
        .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn update_game(pool: State<'_, SqlitePool>, id: String, title: String) -> Result<Game, String> {
    games::update(pool.inner(), &id, &title).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn delete_game(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    games::delete(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}
