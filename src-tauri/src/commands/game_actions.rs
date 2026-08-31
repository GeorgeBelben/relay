use sqlx::SqlitePool;
use tauri::State;

use crate::db::settings;
use crate::game_actions::{self, AlternateMatch};
use crate::ingestion::identify::steamgriddb::SteamGridDbClient;
use crate::ingestion::paths;

async fn require_steamgriddb_client(pool: &SqlitePool) -> Result<SteamGridDbClient, String> {
    let api_key = settings::get(pool, "steamgriddbApiKey").await.map_err(|e| e.to_string())?;
    let api_key = api_key.ok_or_else(|| "SteamGridDB API key not configured (Settings -> Metadata)".to_string())?;
    Ok(SteamGridDbClient::new(api_key))
}

#[tauri::command]
pub async fn search_alternate_matches(pool: State<'_, SqlitePool>, game_id: String) -> Result<Vec<AlternateMatch>, String> {
    let client = require_steamgriddb_client(pool.inner()).await?;
    game_actions::search_alternate_matches(&client, pool.inner(), &game_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_match(pool: State<'_, SqlitePool>, game_id: String, steamgriddb_id: i64, title: String) -> Result<(), String> {
    let client = require_steamgriddb_client(pool.inner()).await?;
    let http = reqwest::Client::new();
    game_actions::apply_match(&client, &http, pool.inner(), &paths::media_path(), &game_id, steamgriddb_id, &title)
        .await
        .map_err(|e| e.to_string())
}
