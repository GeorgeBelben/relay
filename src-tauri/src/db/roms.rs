use serde::Serialize;
use sqlx::SqlitePool;

use super::time::now_unix;

#[derive(Debug, Serialize)]
pub struct Rom {
    pub id: String,
    pub system_id: String,
    pub path: String,
    pub crc32: Option<String>,
    pub size_bytes: Option<i64>,
    pub discs: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct NewRom {
    pub system_id: String,
    pub path: String,
    pub crc32: Option<String>,
    pub size_bytes: Option<i64>,
    pub discs: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Rom>, sqlx::Error> {
    sqlx::query_as!(
        Rom,
        r#"SELECT id, system_id, path, crc32, size_bytes, discs, status,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM roms ORDER BY path"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Rom>, sqlx::Error> {
    sqlx::query_as!(
        Rom,
        r#"SELECT id, system_id, path, crc32, size_bytes, discs, status,
                  created_at as "created_at!: i64", updated_at as "updated_at!: i64"
           FROM roms WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, new: NewRom) -> Result<Rom, sqlx::Error> {
    let id = nanoid::nanoid!();
    let now = now_unix();
    sqlx::query_as!(
        Rom,
        r#"INSERT INTO roms (id, system_id, path, crc32, size_bytes, discs, status, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, 'ok', ?, ?)
           RETURNING id, system_id, path, crc32, size_bytes, discs, status,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        id,
        new.system_id,
        new.path,
        new.crc32,
        new.size_bytes,
        new.discs,
        now,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &SqlitePool, id: &str, new: NewRom) -> Result<Rom, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        Rom,
        r#"UPDATE roms SET system_id = ?, path = ?, crc32 = ?, size_bytes = ?, discs = ?, updated_at = ?
           WHERE id = ?
           RETURNING id, system_id, path, crc32, size_bytes, discs, status,
                     created_at as "created_at!: i64", updated_at as "updated_at!: i64""#,
        new.system_id,
        new.path,
        new.crc32,
        new.size_bytes,
        new.discs,
        now,
        id,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM roms WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
