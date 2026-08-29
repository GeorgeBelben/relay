use serde::Serialize;
use sqlx::SqlitePool;

use super::time::now_unix;

#[derive(Debug, Serialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub ra_username: Option<String>,
    pub ra_web_api_key_encrypted: Option<String>,
    pub ra_token_encrypted: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Profile>, sqlx::Error> {
    sqlx::query_as!(
        Profile,
        r#"SELECT id, name, ra_username, ra_web_api_key_encrypted, ra_token_encrypted,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM profiles ORDER BY created_at"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Profile>, sqlx::Error> {
    sqlx::query_as!(
        Profile,
        r#"SELECT id, name, ra_username, ra_web_api_key_encrypted, ra_token_encrypted,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM profiles WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, name: &str) -> Result<Profile, sqlx::Error> {
    let id = nanoid::nanoid!();
    let now = now_unix();
    sqlx::query_as!(
        Profile,
        r#"INSERT INTO profiles (id, name, created_at, updated_at)
           VALUES (?, ?, ?, ?)
           RETURNING id, name, ra_username, ra_web_api_key_encrypted, ra_token_encrypted,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        id,
        name,
        now,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn rename(pool: &SqlitePool, id: &str, name: &str) -> Result<Profile, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        Profile,
        r#"UPDATE profiles SET name = ?, updated_at = ?
           WHERE id = ?
           RETURNING id, name, ra_username, ra_web_api_key_encrypted, ra_token_encrypted,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        name,
        now,
        id,
    )
    .fetch_one(pool)
    .await
}

// No FK cascade configured (foreign_keys pragma isn't enabled), so ra_stats is removed
// explicitly rather than left orphaned.
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM ra_stats WHERE profile_id = ?", id)
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM profiles WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
