use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::db::{profiles, settings};
use crate::game_actions::{self, AlternateMatch, GameAchievementsProgress};
use crate::ingestion::identify::steamgriddb::SteamGridDbClient;
use crate::ingestion::paths;
use crate::retroachievements::client::RetroAchievementsClient;

async fn require_steamgriddb_client(pool: &SqlitePool) -> Result<SteamGridDbClient, String> {
    let api_key = settings::get(pool, "steamgriddbApiKey").await.map_err(crate::logging::err_to_string)?;
    let api_key = api_key.ok_or_else(|| "SteamGridDB API key not configured (Settings -> Metadata)".to_string())?;
    Ok(SteamGridDbClient::new(api_key))
}

fn secrets_key_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(crate::logging::err_to_string)?.join("secret.key"))
}

// Sourced from the active profile, not a single app-wide key -- an unlinked or no-active-profile
// state fails the same "not configured" way either way, since from this call site's perspective
// they're the same thing: no credentials to look achievements up with. Ported from the Electron
// MVP's `gameActions.service.ts#requireRetroAchievementsCredentials`.
async fn require_ra_credentials(app: &AppHandle, pool: &SqlitePool) -> Result<(RetroAchievementsClient, String), String> {
    let general = settings::get_general_settings(pool).await.map_err(crate::logging::err_to_string)?;
    let active_profile_id = general.active_profile_id.ok_or_else(|| "RetroAchievements not configured (Settings -> Profiles)".to_string())?;

    let key_path = secrets_key_path(app)?;
    let creds = profiles::get_ra_credentials(pool, &key_path, &active_profile_id)
        .await
        .map_err(crate::logging::err_to_string)?
        .ok_or_else(|| "RetroAchievements not configured (Settings -> Profiles)".to_string())?;
    let web_api_key = creds.ra_web_api_key.ok_or_else(|| "RetroAchievements not configured (Settings -> Profiles)".to_string())?;

    Ok((RetroAchievementsClient::new(web_api_key), creds.ra_username))
}

#[tauri::command]
pub async fn search_alternate_matches(pool: State<'_, SqlitePool>, game_id: String) -> Result<Vec<AlternateMatch>, String> {
    let client = require_steamgriddb_client(pool.inner()).await?;
    game_actions::search_alternate_matches(&client, pool.inner(), &game_id).await.map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn apply_match(pool: State<'_, SqlitePool>, game_id: String, steamgriddb_id: i64, title: String) -> Result<(), String> {
    let client = require_steamgriddb_client(pool.inner()).await?;
    let http = reqwest::Client::new();
    game_actions::apply_match(&client, &http, pool.inner(), &paths::media_path(), &game_id, steamgriddb_id, &title)
        .await
        .map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn get_achievements(app: AppHandle, pool: State<'_, SqlitePool>, game_id: String) -> Result<Option<GameAchievementsProgress>, String> {
    let (client, username) = require_ra_credentials(&app, pool.inner()).await?;
    game_actions::get_achievements(&client, pool.inner(), &username, &game_id).await.map_err(crate::logging::err_to_string)
}
