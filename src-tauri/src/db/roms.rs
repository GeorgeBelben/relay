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

/// Insert-or-update keyed on `path` (unique) -- a rescan always re-upserts every rom it finds,
/// so `create`'s plain INSERT would fail on the unique constraint the second time around.
/// Always (re)sets `status = 'ok'`, since finding the file again is what a rescan means. Ported
/// from the Electron MVP's `romsRepository.upsert`.
pub async fn upsert(pool: &SqlitePool, new: NewRom) -> Result<Rom, sqlx::Error> {
    let id = nanoid::nanoid!();
    let now = now_unix();
    sqlx::query_as!(
        Rom,
        r#"INSERT INTO roms (id, system_id, path, crc32, size_bytes, discs, status, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, 'ok', ?, ?)
           ON CONFLICT (path) DO UPDATE SET
             crc32 = excluded.crc32,
             size_bytes = excluded.size_bytes,
             discs = excluded.discs,
             status = 'ok',
             updated_at = excluded.updated_at
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

/// Anything not found in this scan gets marked missing rather than deleted, so play
/// history/associations aren't lost to a temporarily-unplugged drive or a file moved mid-copy.
/// Ported from the Electron MVP's `romsRepository.markMissing`.
pub async fn mark_missing(pool: &SqlitePool, found_paths: &[String]) -> Result<(), sqlx::Error> {
    if found_paths.is_empty() {
        sqlx::query!("UPDATE roms SET status = 'missing'").execute(pool).await?;
        return Ok(());
    }

    // sqlx's query! macro can't expand a dynamic IN (...) list, so this is built by hand. Every
    // value is still passed as a bound parameter below, never interpolated into the SQL text --
    // only the placeholder count (an integer, not attacker-controlled data) varies -- so this is
    // safe to assert past sqlx's dynamic-SQL audit lint.
    let placeholders = found_paths.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!("UPDATE roms SET status = 'missing' WHERE path NOT IN ({placeholders})");
    let mut q = sqlx::query(sqlx::AssertSqlSafe(query));
    for path in found_paths {
        q = q.bind(path);
    }
    q.execute(pool).await?;
    Ok(())
}
