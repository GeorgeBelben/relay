use sqlx::SqlitePool;
use tauri::State;

use crate::db::library::{self, LibraryGame, LibraryShelf};

#[tauri::command]
pub async fn list_library_shelves(pool: State<'_, SqlitePool>) -> Result<Vec<LibraryShelf>, String> {
    library::list_shelves(pool.inner()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_all_games_in_library(pool: State<'_, SqlitePool>) -> Result<Vec<LibraryGame>, String> {
    library::list_all_games(pool.inner()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_recently_added_games(pool: State<'_, SqlitePool>, limit: Option<i64>) -> Result<Vec<LibraryGame>, String> {
    library::list_recently_added(pool.inner(), limit.unwrap_or(10)).await.map_err(|e| e.to_string())
}
