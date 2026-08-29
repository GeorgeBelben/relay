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
