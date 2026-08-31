use std::path::PathBuf;

use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::db::profiles::{self, ProfileSummary};
use crate::db::ra_stats::{self, NewRaStats};
use crate::retroachievements::client::{RaRecentUnlock, RaUserStats, RetroAchievementsClient};
use crate::retroachievements::connect_client;

// Credential encryption key lives in the app data dir, alongside (but separate from) the SQLite
// DB itself -- see secrets.rs's own module doc for why a DB copy/backup shouldn't be enough to
// recover a linked account's credentials on its own.
fn secrets_key_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(crate::logging::err_to_string)?.join("secret.key"))
}

#[tauri::command]
pub async fn list_profiles(pool: State<'_, SqlitePool>) -> Result<Vec<ProfileSummary>, String> {
    let rows = profiles::list(pool.inner()).await.map_err(crate::logging::err_to_string)?;
    Ok(rows.into_iter().map(ProfileSummary::from).collect())
}

#[tauri::command]
pub async fn get_profile(pool: State<'_, SqlitePool>, id: String) -> Result<Option<ProfileSummary>, String> {
    let row = profiles::get(pool.inner(), &id).await.map_err(crate::logging::err_to_string)?;
    Ok(row.map(ProfileSummary::from))
}

#[tauri::command]
pub async fn create_profile(pool: State<'_, SqlitePool>, name: String) -> Result<ProfileSummary, String> {
    profiles::create(pool.inner(), &name).await.map(ProfileSummary::from).map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn rename_profile(pool: State<'_, SqlitePool>, id: String, name: String) -> Result<ProfileSummary, String> {
    profiles::rename(pool.inner(), &id, &name).await.map(ProfileSummary::from).map_err(crate::logging::err_to_string)
}

#[tauri::command]
pub async fn delete_profile(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    profiles::delete(pool.inner(), &id).await.map_err(crate::logging::err_to_string)
}

fn new_ra_stats(stats: RaUserStats) -> NewRaStats {
    NewRaStats { points: stats.points, rank: stats.rank, recent_unlocks_json: serde_json::to_string(&stats.recent_unlocks).unwrap_or_else(|_| "[]".to_string()) }
}

/// Validates against RA's own API before persisting -- a typo'd key fails immediately in the
/// Settings UI instead of silently saving something that'll only surface as a failure later, at
/// achievements-fetch time or on the next stats refresh. Also caches the stats it just fetched for
/// free, rather than making the profile detail screen wait for the next refresh cycle to show
/// them. Ported from the Electron MVP's `profiles.service.ts#linkWebApi`.
#[tauri::command]
pub async fn link_ra_web_api(app: AppHandle, pool: State<'_, SqlitePool>, profile_id: String, username: String, web_api_key: String) -> Result<(), String> {
    let client = RetroAchievementsClient::new(web_api_key.clone());
    let stats = client.get_user_stats(&username).await.map_err(crate::logging::err_to_string)?;

    let key_path = secrets_key_path(&app)?;
    profiles::set_web_api_link(pool.inner(), &key_path, &profile_id, &username, &web_api_key).await.map_err(crate::logging::err_to_string)?;
    ra_stats::upsert(pool.inner(), &profile_id, new_ra_stats(stats)).await.map_err(crate::logging::err_to_string)?;
    Ok(())
}

#[tauri::command]
pub async fn link_ra_connect_account(app: AppHandle, pool: State<'_, SqlitePool>, profile_id: String, username: String, password: String) -> Result<(), String> {
    let http = reqwest::Client::new();
    let token = connect_client::login_real(&http, &username, &password).await.map_err(crate::logging::err_to_string)?;

    let key_path = secrets_key_path(&app)?;
    profiles::set_connect_token(pool.inner(), &key_path, &profile_id, &username, &token).await.map_err(crate::logging::err_to_string)?;
    Ok(())
}

/// Clears every RA field at once -- the UI offers a single "Unlink RetroAchievements" action, not
/// separate ones per auth flow, since both point at the same account. Also clears the cached stats
/// -- unlike the Electron original, which left the last-fetched points/rank in place after
/// unlinking (nothing in that codebase explains this as deliberate, and showing stats for an
/// account that's no longer linked reads as a bug, not a "keep history" feature).
#[tauri::command]
pub async fn unlink_ra(pool: State<'_, SqlitePool>, profile_id: String) -> Result<(), String> {
    profiles::clear_ra_link(pool.inner(), &profile_id).await.map_err(crate::logging::err_to_string)?;
    ra_stats::delete(pool.inner(), &profile_id).await.map_err(crate::logging::err_to_string)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct RaStatsView {
    pub points: i64,
    pub rank: String,
    pub recent_unlocks: Vec<RaRecentUnlock>,
    pub refreshed_at: i64,
}

#[tauri::command]
pub async fn get_ra_stats(pool: State<'_, SqlitePool>, profile_id: String) -> Result<Option<RaStatsView>, String> {
    let Some(row) = ra_stats::get(pool.inner(), &profile_id).await.map_err(crate::logging::err_to_string)? else {
        return Ok(None);
    };
    let recent_unlocks: Vec<RaRecentUnlock> = serde_json::from_str(&row.recent_unlocks_json).map_err(crate::logging::err_to_string)?;
    Ok(Some(RaStatsView { points: row.points, rank: row.rank, recent_unlocks, refreshed_at: row.refreshed_at }))
}

/// Re-fetches and re-caches stats for a Web-API-linked profile -- used on app start / profile
/// switch (see the Electron MVP's `refreshActiveProfileStats`). A no-op, not an error, for a
/// profile with no Web API link -- there's nothing to refresh.
#[tauri::command]
pub async fn refresh_ra_stats(pool: State<'_, SqlitePool>, app: AppHandle, profile_id: String) -> Result<(), String> {
    let key_path = secrets_key_path(&app)?;
    let Some(creds) = profiles::get_ra_credentials(pool.inner(), &key_path, &profile_id).await.map_err(crate::logging::err_to_string)? else {
        return Ok(());
    };
    let Some(web_api_key) = creds.ra_web_api_key else {
        return Ok(());
    };

    let client = RetroAchievementsClient::new(web_api_key);
    let stats = client.get_user_stats(&creds.ra_username).await.map_err(crate::logging::err_to_string)?;
    ra_stats::upsert(pool.inner(), &profile_id, new_ra_stats(stats)).await.map_err(crate::logging::err_to_string)?;
    Ok(())
}
