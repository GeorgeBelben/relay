use std::path::Path;

use sqlx::SqlitePool;

use crate::db::time::now_unix;
use crate::db::{game_media, games};

use super::identify::matching::best_match;
use super::identify::steamgriddb::{SteamGridDbClient, SteamGridDbError};

// A title has to be a very close match to get auto-applied -- a wrong cover is worse than no
// cover, and titles this close together are effectively always the right game. Ported from the
// Electron MVP's metadata/enrichLibrary.ts.
const MATCH_THRESHOLD: f64 = 0.82;

#[derive(Debug, PartialEq)]
pub enum EnrichOutcome {
    Matched { steamgriddb_id: i64, confidence: f64, boxart_downloaded: bool },
    NoMatch,
}

#[derive(Debug)]
pub enum EnrichError {
    SteamGridDb(SteamGridDbError),
    Db(sqlx::Error),
    Download(reqwest::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for EnrichError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SteamGridDb(e) => write!(f, "SteamGridDB error: {e}"),
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Download(e) => write!(f, "box art download error: {e}"),
            Self::Io(e) => write!(f, "filesystem error: {e}"),
        }
    }
}

impl std::error::Error for EnrichError {}

impl From<SteamGridDbError> for EnrichError {
    fn from(e: SteamGridDbError) -> Self {
        Self::SteamGridDb(e)
    }
}
impl From<sqlx::Error> for EnrichError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}
impl From<reqwest::Error> for EnrichError {
    fn from(e: reqwest::Error) -> Self {
        Self::Download(e)
    }
}
impl From<std::io::Error> for EnrichError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Identifies and enriches exactly one game: searches SteamGridDB, applies the match threshold,
/// writes the outcome back to the DB, and -- on a match -- downloads box art under
/// `media_root/<system_id>/<game_id>/`. Ported from the Electron MVP's
/// `enrichLibrary.ts#enrichOne`; the multi-game loop with rate-limiting and progress reporting is
/// a separate concern (Tauri progress events) layered on top of this. `media_root` is threaded
/// through explicitly (rather than resolved internally) so tests can point it at a tempdir --
/// production call sites pass `ingestion::paths::media_path()`.
pub async fn enrich_one(
    client: &SteamGridDbClient,
    http: &reqwest::Client,
    pool: &SqlitePool,
    game: &games::UnenrichedGame,
    media_root: &Path,
) -> Result<EnrichOutcome, EnrichError> {
    let candidates = client.search_games(&game.title).await?;
    let matched = best_match(&game.title, &candidates, |c| c.name.as_str(), MATCH_THRESHOLD)
        .filter(|m| m.score >= MATCH_THRESHOLD);

    let Some(matched) = matched else {
        games::mark_no_match(pool, &game.id).await?;
        return Ok(EnrichOutcome::NoMatch);
    };

    let steamgriddb_id = matched.candidate.id;
    let matched_title = matched.candidate.name.clone();
    let score = matched.score;

    games::mark_matched(pool, &game.id, steamgriddb_id, &matched_title, score).await?;

    let boxart_downloaded = match client.get_boxart_url(steamgriddb_id).await? {
        Some(boxart_url) => {
            let dest_dir = media_root.join(&game.system_id).join(&game.id);
            let local_path = download_boxart(http, &boxart_url, &dest_dir, media_root).await?;
            game_media::create(
                pool,
                game_media::NewGameMedia {
                    game_id: game.id.clone(),
                    kind: "boxart".to_string(),
                    local_path,
                    source_url: Some(boxart_url),
                },
            )
            .await?;
            true
        }
        None => false,
    };

    Ok(EnrichOutcome::Matched { steamgriddb_id, confidence: score, boxart_downloaded })
}

/// Downloads `url` into `dest_dir` as `boxart-<unix-timestamp><ext>` and returns the path stored
/// in the database, relative to `media_root` and forward-slash-normalized -- it's read back into
/// a URL later, not passed straight to the filesystem. The timestamped filename matters: a
/// re-download that reused a fixed name would leave an `<img>`'s `src` unchanged across a swap,
/// and browsers only refetch an image when `src` actually changes.
async fn download_boxart(http: &reqwest::Client, url: &str, dest_dir: &Path, media_root: &Path) -> Result<String, EnrichError> {
    let res = http.get(url).send().await?.error_for_status()?;
    let bytes = res.bytes().await?;

    let ext = reqwest::Url::parse(url)
        .ok()
        .and_then(|u| Path::new(u.path()).extension().map(|e| e.to_string_lossy().into_owned()))
        .map(|e| format!(".{e}"))
        .unwrap_or_else(|| ".png".to_string());

    tokio::fs::create_dir_all(dest_dir).await?;
    let dest = dest_dir.join(format!("boxart-{}{ext}", now_unix()));
    tokio::fs::write(&dest, &bytes).await?;

    let relative = dest.strip_prefix(media_root).unwrap_or(&dest);
    Ok(relative.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::games as games_db;
    use crate::db::roms::{self, NewRom};
    use crate::db::systems::{self, NewSystem};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn throwaway_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, dir)
    }

    async fn seed_unenriched_game(pool: &SqlitePool, title: &str) -> games::UnenrichedGame {
        systems::create(
            pool,
            NewSystem {
                id: "snes".into(),
                name: "SNES".into(),
                extensions: "[\"sfc\"]".into(),
                retroarch_core: None,
                standalone_binary: None,
            },
        )
        .await
        .unwrap();
        let rom = roms::create(
            pool,
            NewRom { system_id: "snes".into(), path: "snes/game.sfc".into(), crc32: None, size_bytes: None, discs: None },
        )
        .await
        .unwrap();
        let game = games_db::create(pool, games_db::NewGame { rom_id: rom.id, title: title.into() }).await.unwrap();

        games::UnenrichedGame { id: game.id, title: game.title, system_id: "snes".into() }
    }

    #[tokio::test]
    async fn enrich_one_marks_no_match_when_nothing_clears_the_threshold() {
        let (pool, _dir) = throwaway_pool().await;
        let game = seed_unenriched_game(&pool, "Totally Unmatched Title").await;
        let media_root = tempfile::tempdir().unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/Totally%20Unmatched%20Title"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 1, "name": "Something Completely Different" }],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let http = reqwest::Client::new();

        let outcome = enrich_one(&client, &http, &pool, &game, media_root.path()).await.unwrap();
        assert_eq!(outcome, EnrichOutcome::NoMatch);

        let stored = games_db::get(&pool, &game.id).await.unwrap().unwrap();
        assert!(stored.enriched_at.is_some());
        assert!(stored.steamgriddb_id.is_none());
    }

    #[tokio::test]
    async fn enrich_one_marks_matched_and_downloads_boxart() {
        let (pool, _dir) = throwaway_pool().await;
        let game = seed_unenriched_game(&pool, "Chrono Trigger").await;
        let media_root = tempfile::tempdir().unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/Chrono%20Trigger"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 99, "name": "Chrono Trigger" }],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v2/grids/game/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 1, "url": format!("{}/images/boxart.png", server.uri()) }],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/images/boxart.png"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"fake-png-bytes".to_vec()))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let http = reqwest::Client::new();

        let outcome = enrich_one(&client, &http, &pool, &game, media_root.path()).await.unwrap();
        assert_eq!(
            outcome,
            EnrichOutcome::Matched { steamgriddb_id: 99, confidence: 1.0, boxart_downloaded: true }
        );

        let stored = games_db::get(&pool, &game.id).await.unwrap().unwrap();
        assert_eq!(stored.steamgriddb_id, Some(99));
        assert!(stored.enriched_at.is_some());

        let media = game_media::list_for_game(&pool, &game.id).await.unwrap();
        assert_eq!(media.len(), 1);
        assert!(media[0].local_path.starts_with("snes/"));
        assert!(media_root.path().join(&media[0].local_path).exists());
    }
}
