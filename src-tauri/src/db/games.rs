use serde::Serialize;
use sqlx::SqlitePool;

use super::time::now_unix;

#[derive(Debug, Serialize)]
pub struct Game {
    pub id: String,
    pub rom_id: String,
    pub title: String,
    pub scanned_title: Option<String>,
    pub steamgriddb_id: Option<i64>,
    pub match_confidence: Option<f64>,
    pub enriched_at: Option<i64>,
    pub retroachievements_game_id: Option<i64>,
    pub retroachievements_matched_at: Option<i64>,
    pub ra_highest_award_kind: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct NewGame {
    pub rom_id: String,
    pub title: String,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as!(
        Game,
        r#"SELECT id, rom_id, title, scanned_title,
                  steamgriddb_id as "steamgriddb_id: i64",
                  match_confidence,
                  enriched_at as "enriched_at: i64",
                  retroachievements_game_id as "retroachievements_game_id: i64",
                  retroachievements_matched_at as "retroachievements_matched_at: i64",
                  ra_highest_award_kind,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM games ORDER BY title"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Game>, sqlx::Error> {
    sqlx::query_as!(
        Game,
        r#"SELECT id, rom_id, title, scanned_title,
                  steamgriddb_id as "steamgriddb_id: i64",
                  match_confidence,
                  enriched_at as "enriched_at: i64",
                  retroachievements_game_id as "retroachievements_game_id: i64",
                  retroachievements_matched_at as "retroachievements_matched_at: i64",
                  ra_highest_award_kind,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM games WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, new: NewGame) -> Result<Game, sqlx::Error> {
    let id = nanoid::nanoid!();
    let now = now_unix();
    sqlx::query_as!(
        Game,
        r#"INSERT INTO games (id, rom_id, title, scanned_title, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?)
           RETURNING id, rom_id, title, scanned_title,
                     steamgriddb_id as "steamgriddb_id: i64",
                     match_confidence,
                     enriched_at as "enriched_at: i64",
                     retroachievements_game_id as "retroachievements_game_id: i64",
                     retroachievements_matched_at as "retroachievements_matched_at: i64",
                     ra_highest_award_kind,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        id,
        new.rom_id,
        new.title,
        new.title,
        now,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &SqlitePool, id: &str, title: &str) -> Result<Game, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        Game,
        r#"UPDATE games SET title = ?, updated_at = ?
           WHERE id = ?
           RETURNING id, rom_id, title, scanned_title,
                     steamgriddb_id as "steamgriddb_id: i64",
                     match_confidence,
                     enriched_at as "enriched_at: i64",
                     retroachievements_game_id as "retroachievements_game_id: i64",
                     retroachievements_matched_at as "retroachievements_matched_at: i64",
                     ra_highest_award_kind,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        title,
        now,
        id,
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, PartialEq)]
pub struct UnenrichedGame {
    pub id: String,
    pub title: String,
    pub system_id: String,
}

/// Games an enrichment pass hasn't touched yet, joined with their rom's system (needed to lay
/// out where a matched game's box art gets downloaded to). Excludes roms whose file has gone
/// missing since the last scan. Ported from the Electron MVP's `gamesRepository.listUnenriched`.
pub async fn list_unenriched(pool: &SqlitePool) -> Result<Vec<UnenrichedGame>, sqlx::Error> {
    sqlx::query_as!(
        UnenrichedGame,
        r#"SELECT games.id, games.title, roms.system_id
           FROM games
           JOIN roms ON games.rom_id = roms.id
           WHERE roms.status = 'ok' AND games.enriched_at IS NULL"#
    )
    .fetch_all(pool)
    .await
}

pub async fn mark_matched(
    pool: &SqlitePool,
    id: &str,
    steamgriddb_id: i64,
    title: &str,
    match_confidence: f64,
) -> Result<Game, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        Game,
        r#"UPDATE games SET steamgriddb_id = ?, title = ?, match_confidence = ?, enriched_at = ?, updated_at = ?
           WHERE id = ?
           RETURNING id, rom_id, title, scanned_title,
                     steamgriddb_id as "steamgriddb_id: i64",
                     match_confidence,
                     enriched_at as "enriched_at: i64",
                     retroachievements_game_id as "retroachievements_game_id: i64",
                     retroachievements_matched_at as "retroachievements_matched_at: i64",
                     ra_highest_award_kind,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        steamgriddb_id,
        title,
        match_confidence,
        now,
        now,
        id,
    )
    .fetch_one(pool)
    .await
}

// enrichedAt set with steamgriddbId left null means "we tried and SteamGridDB has nothing for
// this title" -- distinct from never having tried at all (enrichedAt still null), which is what
// makes list_unenriched() stop re-querying for a game that already confirmed no match.
pub async fn mark_no_match(pool: &SqlitePool, id: &str) -> Result<Game, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        Game,
        r#"UPDATE games SET enriched_at = ?, updated_at = ?
           WHERE id = ?
           RETURNING id, rom_id, title, scanned_title,
                     steamgriddb_id as "steamgriddb_id: i64",
                     match_confidence,
                     enriched_at as "enriched_at: i64",
                     retroachievements_game_id as "retroachievements_game_id: i64",
                     retroachievements_matched_at as "retroachievements_matched_at: i64",
                     ra_highest_award_kind,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        now,
        now,
        id,
    )
    .fetch_one(pool)
    .await
}

// No FK cascade configured, so game_media rows are removed explicitly first rather than
// left orphaned (mirrors profiles::delete's handling of ra_stats).
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM game_media WHERE game_id = ?", id)
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM games WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
