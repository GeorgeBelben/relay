use serde::Serialize;
use sqlx::SqlitePool;

use super::time::now_unix;

#[derive(Debug, Serialize)]
pub struct GameMedia {
    pub id: String,
    pub game_id: String,
    pub kind: String,
    pub local_path: String,
    pub source_url: Option<String>,
    pub created_at: i64,
}

pub struct NewGameMedia {
    pub game_id: String,
    pub kind: String,
    pub local_path: String,
    pub source_url: Option<String>,
}

pub async fn list_for_game(pool: &SqlitePool, game_id: &str) -> Result<Vec<GameMedia>, sqlx::Error> {
    sqlx::query_as!(
        GameMedia,
        r#"SELECT id, game_id, kind, local_path, source_url, created_at as "created_at!: i64"
           FROM game_media WHERE game_id = ?"#,
        game_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<GameMedia>, sqlx::Error> {
    sqlx::query_as!(
        GameMedia,
        r#"SELECT id, game_id, kind, local_path, source_url, created_at as "created_at!: i64"
           FROM game_media WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, new: NewGameMedia) -> Result<GameMedia, sqlx::Error> {
    let id = nanoid::nanoid!();
    let now = now_unix();
    sqlx::query_as!(
        GameMedia,
        r#"INSERT INTO game_media (id, game_id, kind, local_path, source_url, created_at)
           VALUES (?, ?, ?, ?, ?, ?)
           RETURNING id, game_id, kind, local_path, source_url, created_at as "created_at!: i64""#,
        id,
        new.game_id,
        new.kind,
        new.local_path,
        new.source_url,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM game_media WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
