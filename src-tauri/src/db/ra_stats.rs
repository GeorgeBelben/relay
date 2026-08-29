use serde::Serialize;
use sqlx::SqlitePool;

use super::time::now_unix;

#[derive(Debug, Serialize)]
pub struct RaStats {
    pub profile_id: String,
    pub points: i64,
    pub rank: String,
    pub recent_unlocks_json: String,
    pub refreshed_at: i64,
}

pub struct NewRaStats {
    pub points: i64,
    pub rank: String,
    pub recent_unlocks_json: String,
}

pub async fn get(pool: &SqlitePool, profile_id: &str) -> Result<Option<RaStats>, sqlx::Error> {
    sqlx::query_as!(
        RaStats,
        r#"SELECT profile_id, points as "points!: i64", rank, recent_unlocks_json,
                  refreshed_at as "refreshed_at!: i64"
           FROM ra_stats WHERE profile_id = ?"#,
        profile_id
    )
    .fetch_optional(pool)
    .await
}

// A pure cache, wholly overwritten on every refresh -- see the schema's own migration comment.
pub async fn upsert(pool: &SqlitePool, profile_id: &str, stats: NewRaStats) -> Result<RaStats, sqlx::Error> {
    let now = now_unix();
    sqlx::query_as!(
        RaStats,
        r#"INSERT INTO ra_stats (profile_id, points, rank, recent_unlocks_json, refreshed_at)
           VALUES (?, ?, ?, ?, ?)
           ON CONFLICT (profile_id) DO UPDATE SET
             points = excluded.points,
             rank = excluded.rank,
             recent_unlocks_json = excluded.recent_unlocks_json,
             refreshed_at = excluded.refreshed_at
           RETURNING profile_id, points as "points!: i64", rank, recent_unlocks_json,
                     refreshed_at as "refreshed_at!: i64""#,
        profile_id,
        stats.points,
        stats.rank,
        stats.recent_unlocks_json,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, profile_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM ra_stats WHERE profile_id = ?", profile_id)
        .execute(pool)
        .await?;
    Ok(())
}
