use std::path::Path;

use serde::Serialize;
use sqlx::SqlitePool;

use crate::secrets::{decrypt_secret, encrypt_secret};

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

/// The redacted, IPC-safe view of a profile -- `ra_web_api_key_encrypted`/`ra_token_encrypted`
/// never cross to the frontend, only whether a link exists. Ported from the Electron MVP's
/// `profiles.service.ts#toProfile` ("Never includes raWebApiKeyEncrypted/raTokenEncrypted -- those
/// stay main-process-only"); the equivalent guarantee here is that `commands::profiles` only ever
/// returns this type, never [`Profile`] itself.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub id: String,
    pub name: String,
    pub ra_username: Option<String>,
    pub has_web_api_link: bool,
    pub has_connect_link: bool,
}

impl From<Profile> for ProfileSummary {
    fn from(p: Profile) -> Self {
        Self {
            id: p.id,
            name: p.name,
            ra_username: p.ra_username,
            has_web_api_link: p.ra_web_api_key_encrypted.is_some(),
            has_connect_link: p.ra_token_encrypted.is_some(),
        }
    }
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

pub struct RaCredentials {
    pub ra_username: String,
    pub ra_web_api_key: Option<String>,
    pub ra_token: Option<String>,
}

/// `None` credential fields decrypt to `None` too (missing) rather than erroring -- a profile with
/// no RA link at all is the common, expected case (linking is optional). Ported from the Electron
/// MVP's `profiles.repository.ts#getRaCredentials`.
pub async fn get_ra_credentials(pool: &SqlitePool, key_path: &Path, profile_id: &str) -> Result<Option<RaCredentials>, sqlx::Error> {
    let row = sqlx::query!("SELECT ra_username, ra_web_api_key_encrypted, ra_token_encrypted FROM profiles WHERE id = ?", profile_id)
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else { return Ok(None) };

    Ok(Some(RaCredentials {
        ra_username: row.ra_username.unwrap_or_default(),
        ra_web_api_key: decrypt_secret(key_path, row.ra_web_api_key_encrypted.as_deref()),
        ra_token: decrypt_secret(key_path, row.ra_token_encrypted.as_deref()),
    }))
}

#[derive(Debug)]
pub enum SetLinkError {
    Db(sqlx::Error),
    Encrypt(std::io::Error),
}

impl std::fmt::Display for SetLinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Encrypt(e) => write!(f, "credential encryption error: {e}"),
        }
    }
}

impl std::error::Error for SetLinkError {}

impl From<sqlx::Error> for SetLinkError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}
impl From<std::io::Error> for SetLinkError {
    fn from(e: std::io::Error) -> Self {
        Self::Encrypt(e)
    }
}

// Both link flows (Web API key vs. Connect token) write the same ra_username column -- they're
// always the same RA account in practice, and keeping one shared field is simpler than tracking
// two independently for what should never actually diverge.
pub async fn set_web_api_link(pool: &SqlitePool, key_path: &Path, profile_id: &str, username: &str, web_api_key: &str) -> Result<(), SetLinkError> {
    let encrypted = encrypt_secret(key_path, web_api_key)?;
    let now = now_unix();
    sqlx::query!(
        "UPDATE profiles SET ra_username = ?, ra_web_api_key_encrypted = ?, updated_at = ? WHERE id = ?",
        username,
        encrypted,
        now,
        profile_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_connect_token(pool: &SqlitePool, key_path: &Path, profile_id: &str, username: &str, token: &str) -> Result<(), SetLinkError> {
    let encrypted = encrypt_secret(key_path, token)?;
    let now = now_unix();
    sqlx::query!(
        "UPDATE profiles SET ra_username = ?, ra_token_encrypted = ?, updated_at = ? WHERE id = ?",
        username,
        encrypted,
        now,
        profile_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Clears every RA field at once -- the UI offers a single "Unlink RetroAchievements" action, not
// separate ones per auth flow, since both point at the same account.
pub async fn clear_ra_link(pool: &SqlitePool, profile_id: &str) -> Result<(), sqlx::Error> {
    let now = now_unix();
    sqlx::query!(
        "UPDATE profiles SET ra_username = NULL, ra_web_api_key_encrypted = NULL, ra_token_encrypted = NULL, updated_at = ? WHERE id = ?",
        now,
        profile_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn throwaway_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn a_profile_with_no_link_has_no_credentials() {
        let (pool, _dir) = throwaway_pool().await;
        let key_dir = tempfile::tempdir().unwrap();
        let key_path = key_dir.path().join("secret.key");

        let profile = create(&pool, "Player One").await.unwrap();
        let creds = get_ra_credentials(&pool, &key_path, &profile.id).await.unwrap().unwrap();

        assert_eq!(creds.ra_username, "");
        assert_eq!(creds.ra_web_api_key, None);
        assert_eq!(creds.ra_token, None);

        let summary: ProfileSummary = profile.into();
        assert!(!summary.has_web_api_link);
        assert!(!summary.has_connect_link);
    }

    #[tokio::test]
    async fn set_web_api_link_round_trips_through_get_ra_credentials() {
        let (pool, _dir) = throwaway_pool().await;
        let key_dir = tempfile::tempdir().unwrap();
        let key_path = key_dir.path().join("secret.key");

        let profile = create(&pool, "Player One").await.unwrap();
        set_web_api_link(&pool, &key_path, &profile.id, "retrouser", "my-web-api-key").await.unwrap();

        let creds = get_ra_credentials(&pool, &key_path, &profile.id).await.unwrap().unwrap();
        assert_eq!(creds.ra_username, "retrouser");
        assert_eq!(creds.ra_web_api_key.as_deref(), Some("my-web-api-key"));
        assert_eq!(creds.ra_token, None);

        let stored = get(&pool, &profile.id).await.unwrap().unwrap();
        assert_ne!(stored.ra_web_api_key_encrypted.as_deref(), Some("my-web-api-key"), "must not be stored in plaintext");

        let summary: ProfileSummary = get(&pool, &profile.id).await.unwrap().unwrap().into();
        assert!(summary.has_web_api_link);
        assert!(!summary.has_connect_link);
    }

    #[tokio::test]
    async fn set_connect_token_round_trips_through_get_ra_credentials() {
        let (pool, _dir) = throwaway_pool().await;
        let key_dir = tempfile::tempdir().unwrap();
        let key_path = key_dir.path().join("secret.key");

        let profile = create(&pool, "Player One").await.unwrap();
        set_connect_token(&pool, &key_path, &profile.id, "retrouser", "session-token").await.unwrap();

        let creds = get_ra_credentials(&pool, &key_path, &profile.id).await.unwrap().unwrap();
        assert_eq!(creds.ra_username, "retrouser");
        assert_eq!(creds.ra_token.as_deref(), Some("session-token"));
        assert_eq!(creds.ra_web_api_key, None);
    }

    #[tokio::test]
    async fn clear_ra_link_removes_every_ra_field_at_once() {
        let (pool, _dir) = throwaway_pool().await;
        let key_dir = tempfile::tempdir().unwrap();
        let key_path = key_dir.path().join("secret.key");

        let profile = create(&pool, "Player One").await.unwrap();
        set_web_api_link(&pool, &key_path, &profile.id, "retrouser", "key").await.unwrap();
        set_connect_token(&pool, &key_path, &profile.id, "retrouser", "token").await.unwrap();

        clear_ra_link(&pool, &profile.id).await.unwrap();

        let creds = get_ra_credentials(&pool, &key_path, &profile.id).await.unwrap().unwrap();
        assert_eq!(creds.ra_username, "");
        assert_eq!(creds.ra_web_api_key, None);
        assert_eq!(creds.ra_token, None);
    }

    #[tokio::test]
    async fn get_ra_credentials_returns_none_for_an_unknown_profile() {
        let (pool, _dir) = throwaway_pool().await;
        let key_dir = tempfile::tempdir().unwrap();
        let key_path = key_dir.path().join("secret.key");

        assert!(get_ra_credentials(&pool, &key_path, "nope").await.unwrap().is_none());
    }
}
