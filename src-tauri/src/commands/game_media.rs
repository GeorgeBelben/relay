use sqlx::SqlitePool;
use tauri::State;

use crate::db::game_media::{self, GameMedia};
use crate::ingestion::paths;

#[tauri::command]
pub async fn list_game_media(pool: State<'_, SqlitePool>, game_id: String) -> Result<Vec<GameMedia>, String> {
    game_media::list_for_game(pool.inner(), &game_id).await.map_err(crate::logging::err_to_string)
}

/// `game_media::local_path` is stored relative to this root (forward-slash-normalized, see
/// ingestion::enrich) -- the frontend needs it to reconstruct an absolute path before deciding how
/// to load the image (asset-protocol scope, a byte-serving command, etc -- a frontend-loading
/// decision, not something this command makes for it).
#[tauri::command]
pub fn get_media_root_path() -> String {
    paths::media_path().to_string_lossy().into_owned()
}
