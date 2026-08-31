//! On-demand, user-initiated actions on a single game -- distinct from `ingestion::enrich`'s
//! automatic pass, which runs across the whole unenriched backlog during a scan. Ported from the
//! Electron MVP's `gameActions.service.ts`.

use std::path::Path;

use serde::Serialize;
use sqlx::SqlitePool;

use crate::db::{game_media, games, roms};
use crate::ingestion::enrich::{download_boxart, EnrichError};
use crate::ingestion::identify::steamgriddb::{SteamGridDbClient, SteamGridDbError};
use crate::retroachievements::client::{badge_url, RaError, RetroAchievementsClient};

// A manually-picked match is as confident as it gets -- distinct from enrich.rs's auto-match,
// which scores against a threshold since nobody's confirmed it by eye.
const MANUAL_MATCH_CONFIDENCE: f64 = 1.0;

// Alternate box art candidates aren't downloaded until one's actually picked (see apply_match) --
// this only needs enough of each to render a picker, so cap how many candidates get a grid
// lookup rather than firing one per search result.
const MAX_ALTERNATES: usize = 8;

#[derive(Debug, Serialize)]
pub struct AlternateMatch {
    pub steamgriddb_id: i64,
    pub title: String,
    pub boxart_url: Option<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct AchievementView {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub points: i64,
    pub badge_url: String,
    pub unlocked: bool,
}

/// Backs the drawer's "Achievements" view and the Home progress card. `badge_url` is already
/// resolved to the locked/unlocked variant server-side so the frontend never needs to know RA's
/// own badge-naming convention. Mirrors the Electron MVP's `RetroAchievementsProgress` shared type.
#[derive(Debug, PartialEq, Serialize)]
pub struct GameAchievementsProgress {
    pub game_id: i64,
    pub title: String,
    pub console_name: String,
    pub num_achievements: i64,
    pub num_awarded_to_user: i64,
    pub user_completion: String,
    pub highest_award_kind: Option<String>,
    pub achievements: Vec<AchievementView>,
}

#[derive(Debug)]
pub enum GameActionsError {
    GameNotFound,
    SteamGridDb(SteamGridDbError),
    RetroAchievements(RaError),
    Db(sqlx::Error),
    Download(reqwest::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for GameActionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameNotFound => write!(f, "game not found"),
            Self::SteamGridDb(e) => write!(f, "SteamGridDB error: {e}"),
            Self::RetroAchievements(e) => write!(f, "RetroAchievements error: {e}"),
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Download(e) => write!(f, "box art download error: {e}"),
            Self::Io(e) => write!(f, "filesystem error: {e}"),
        }
    }
}

impl std::error::Error for GameActionsError {}

impl From<sqlx::Error> for GameActionsError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}
impl From<SteamGridDbError> for GameActionsError {
    fn from(e: SteamGridDbError) -> Self {
        Self::SteamGridDb(e)
    }
}
impl From<RaError> for GameActionsError {
    fn from(e: RaError) -> Self {
        Self::RetroAchievements(e)
    }
}
impl From<EnrichError> for GameActionsError {
    fn from(e: EnrichError) -> Self {
        match e {
            EnrichError::SteamGridDb(e) => Self::SteamGridDb(e),
            EnrichError::Db(e) => Self::Db(e),
            EnrichError::Download(e) => Self::Download(e),
            EnrichError::Io(e) => Self::Io(e),
        }
    }
}

/// Re-searches SteamGridDB by the game's *filename-derived* title, not its current (possibly
/// wrong) matched title -- backs the drawer's "Change Box Art" picker, for when the automatic
/// match picked the wrong game. Searching by the current title would be self-defeating: once a
/// bad match has landed, that title IS the bad match's name, so re-running the search on it just
/// re-fetches the same (or similarly wrong) candidates. `scanned_title` tracks the actual filename
/// regardless of match status; falls back to `title` only for a game whose `scanned_title` predates
/// that column and hasn't had a rescan since.
pub async fn search_alternate_matches(client: &SteamGridDbClient, pool: &SqlitePool, game_id: &str) -> Result<Vec<AlternateMatch>, GameActionsError> {
    let game = games::get(pool, game_id).await?.ok_or(GameActionsError::GameNotFound)?;
    let search_title = game.scanned_title.as_deref().unwrap_or(&game.title);

    let candidates = client.search_games(search_title).await?;
    let candidates = candidates.into_iter().take(MAX_ALTERNATES);

    // One request per candidate for its preview art, in parallel -- fine here (a handful,
    // user-initiated, latency-sensitive), unlike enrich.rs's bulk pass which deliberately paces
    // itself. A candidate whose art lookup fails still shows up with boxart_url: None, same as
    // the Electron original's `.catch(() => null)`.
    let lookups = candidates.map(|candidate| {
        let client = client.clone();
        tokio::spawn(async move {
            let boxart_url = client.get_boxart_url(candidate.id).await.unwrap_or(None);
            AlternateMatch { steamgriddb_id: candidate.id, title: candidate.name, boxart_url }
        })
    });

    let mut matches = Vec::new();
    for lookup in lookups {
        matches.push(lookup.await.expect("boxart lookup task panicked"));
    }
    Ok(matches)
}

/// Downloads the chosen candidate's box art and applies it, overwriting whatever match/art this
/// game had before (see `ingestion::enrich::enrich_one` for the equivalent automatic-match path).
#[allow(clippy::too_many_arguments)]
pub async fn apply_match(
    client: &SteamGridDbClient,
    http: &reqwest::Client,
    pool: &SqlitePool,
    media_root: &Path,
    game_id: &str,
    steamgriddb_id: i64,
    title: &str,
) -> Result<(), GameActionsError> {
    let game = games::get(pool, game_id).await?.ok_or(GameActionsError::GameNotFound)?;
    let rom = roms::get(pool, &game.rom_id).await?.ok_or(GameActionsError::GameNotFound)?;

    games::mark_matched(pool, game_id, steamgriddb_id, title, MANUAL_MATCH_CONFIDENCE).await?;

    let Some(boxart_url) = client.get_boxart_url(steamgriddb_id).await? else {
        return Ok(());
    };

    let dest_dir = media_root.join(&rom.system_id).join(game_id);
    let local_path = download_boxart(http, &boxart_url, &dest_dir, media_root).await?;
    game_media::upsert_boxart(pool, game_id, &local_path, &boxart_url).await?;

    Ok(())
}

/// `None` means "this game isn't matched to a RetroAchievements entry" -- a normal, expected
/// outcome (unsupported system, no RA entry for this ROM, or the auto-match pass just hasn't run
/// yet -- see `retroachievements::client`'s own module doc for why nothing populates
/// `retroachievements_game_id` yet), not an error. Missing/invalid credentials *do* propagate as an
/// error, same as `search_alternate_matches`' missing-API-key case -- that's a real,
/// user-actionable configuration gap, not a per-game state.
pub async fn get_achievements(
    client: &RetroAchievementsClient,
    pool: &SqlitePool,
    username: &str,
    game_id: &str,
) -> Result<Option<GameAchievementsProgress>, GameActionsError> {
    let game = games::get(pool, game_id).await?.ok_or(GameActionsError::GameNotFound)?;
    let Some(ra_game_id) = game.retroachievements_game_id else { return Ok(None) };

    let Some(progress) = client.get_game_info_and_user_progress(username, ra_game_id).await? else {
        return Ok(None);
    };

    // Persists the "beaten" flag onto the game row so it's visible on a tile without being
    // focused (piggybacks on the fetch every tile focus already triggers). A failed write here
    // shouldn't fail the achievements view itself -- the fetch just succeeded and has real data
    // to show regardless of whether this cache update lands.
    let _ = games::update_highest_award_kind(pool, game_id, progress.highest_award_kind.as_deref()).await;

    let achievements = progress
        .achievements
        .into_iter()
        .map(|a| {
            let unlocked = a.unlocked_at.is_some();
            AchievementView { id: a.id, title: a.title, description: a.description, points: a.points, badge_url: badge_url(&a.badge_name, unlocked), unlocked }
        })
        .collect();

    Ok(Some(GameAchievementsProgress {
        game_id: progress.game_id,
        title: progress.title,
        console_name: progress.console_name,
        num_achievements: progress.num_achievements,
        num_awarded_to_user: progress.num_awarded_to_user,
        user_completion: progress.user_completion,
        highest_award_kind: progress.highest_award_kind,
        achievements,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    async fn seed_game(pool: &SqlitePool, title: &str) -> String {
        systems::create(
            pool,
            NewSystem { id: "snes".into(), name: "SNES".into(), extensions: "[\"sfc\"]".into(), retroarch_core: None, standalone_binary: None },
        )
        .await
        .unwrap();
        let rom = roms::create(pool, NewRom { system_id: "snes".into(), path: "snes/game.sfc".into(), crc32: None, size_bytes: None, discs: None })
            .await
            .unwrap();
        let game = games::create(pool, games::NewGame { rom_id: rom.id, title: title.into() }).await.unwrap();
        game.id
    }

    #[tokio::test]
    async fn search_alternate_matches_searches_by_scanned_title_not_current_title() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Chrono Trigger").await;
        // Simulate a bad automatic match: title has drifted from the filename-derived scanned_title.
        sqlx::query!("UPDATE games SET title = 'Wrong Game', scanned_title = 'chrono trigger (usa)' WHERE id = ?", game_id)
            .execute(&pool)
            .await
            .unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/chrono%20trigger%20(usa)"))
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
                "data": [{ "id": 1, "url": "https://example.com/boxart.png" }],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let matches = search_alternate_matches(&client, &pool, &game_id).await.unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].steamgriddb_id, 99);
        assert_eq!(matches[0].title, "Chrono Trigger");
        assert_eq!(matches[0].boxart_url.as_deref(), Some("https://example.com/boxart.png"));
    }

    #[tokio::test]
    async fn search_alternate_matches_caps_at_max_alternates_and_tolerates_a_failed_boxart_lookup() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Many Results").await;

        let candidates: Vec<_> = (0..10).map(|i| serde_json::json!({ "id": i, "name": format!("Result {i}") })).collect();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/Many%20Results"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true, "data": candidates })))
            .mount(&server)
            .await;
        // Every grid lookup 404s -- boxart_url should come back None for all of them, not an error.
        Mock::given(method("GET")).and(path_regex_grids()).respond_with(ResponseTemplate::new(404)).mount(&server).await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let matches = search_alternate_matches(&client, &pool, &game_id).await.unwrap();

        assert_eq!(matches.len(), MAX_ALTERNATES);
        assert!(matches.iter().all(|m| m.boxart_url.is_none()));
    }

    fn path_regex_grids() -> wiremock::matchers::PathRegexMatcher {
        wiremock::matchers::path_regex(r"^/api/v2/grids/game/\d+$")
    }

    #[tokio::test]
    async fn search_alternate_matches_returns_game_not_found_for_an_unknown_id() {
        let (pool, _dir) = throwaway_pool().await;
        let client = SteamGridDbClient::with_base_url("test-key", "http://localhost:1");

        let err = search_alternate_matches(&client, &pool, "nope").await.unwrap_err();
        assert!(matches!(err, GameActionsError::GameNotFound));
    }

    #[tokio::test]
    async fn apply_match_marks_matched_and_downloads_boxart() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Some Filename Title").await;
        let media_root = tempfile::tempdir().unwrap();

        let server = MockServer::start().await;
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

        apply_match(&client, &http, &pool, media_root.path(), &game_id, 99, "Chrono Trigger").await.unwrap();

        let stored = games::get(&pool, &game_id).await.unwrap().unwrap();
        assert_eq!(stored.steamgriddb_id, Some(99));
        assert_eq!(stored.title, "Chrono Trigger");
        assert_eq!(stored.match_confidence, Some(MANUAL_MATCH_CONFIDENCE));

        let media = game_media::list_for_game(&pool, &game_id).await.unwrap();
        assert_eq!(media.len(), 1);
        assert!(media[0].local_path.starts_with("snes/"));
    }

    #[tokio::test]
    async fn apply_match_replaces_rather_than_duplicates_an_existing_boxart_row() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Some Filename Title").await;
        let media_root = tempfile::tempdir().unwrap();

        let server = MockServer::start().await;
        for (steamgriddb_id, image_path) in [(1, "/images/first.png"), (2, "/images/second.png")] {
            Mock::given(method("GET"))
                .and(path(format!("/api/v2/grids/game/{steamgriddb_id}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": [{ "id": 1, "url": format!("{}{}", server.uri(), image_path) }],
                })))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path(image_path))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(b"fake-png-bytes".to_vec()))
                .mount(&server)
                .await;
        }

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let http = reqwest::Client::new();

        apply_match(&client, &http, &pool, media_root.path(), &game_id, 1, "First Match").await.unwrap();
        apply_match(&client, &http, &pool, media_root.path(), &game_id, 2, "Second Match").await.unwrap();

        let stored = games::get(&pool, &game_id).await.unwrap().unwrap();
        assert_eq!(stored.title, "Second Match");

        let media = game_media::list_for_game(&pool, &game_id).await.unwrap();
        assert_eq!(media.len(), 1, "re-applying a match should replace the boxart row, not add a second one");
        assert!(media[0].source_url.as_deref().unwrap().ends_with("second.png"));
    }

    #[tokio::test]
    async fn apply_match_returns_game_not_found_for_an_unknown_id() {
        let (pool, _dir) = throwaway_pool().await;
        let media_root = tempfile::tempdir().unwrap();
        let client = SteamGridDbClient::with_base_url("test-key", "http://localhost:1");
        let http = reqwest::Client::new();

        let err = apply_match(&client, &http, &pool, media_root.path(), "nope", 1, "Title").await.unwrap_err();
        assert!(matches!(err, GameActionsError::GameNotFound));
    }

    #[tokio::test]
    async fn get_achievements_returns_none_when_the_game_has_no_ra_match() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Unmatched Game").await;
        let client = RetroAchievementsClient::with_base_url("fake-key", "http://localhost:1");

        let progress = get_achievements(&client, &pool, "retrouser", &game_id).await.unwrap();
        assert_eq!(progress, None);
    }

    #[tokio::test]
    async fn get_achievements_returns_progress_and_persists_the_highest_award_kind() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Chrono Trigger").await;
        sqlx::query!("UPDATE games SET retroachievements_game_id = 99 WHERE id = ?", game_id).execute(&pool).await.unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ID": 99, "Title": "Chrono Trigger", "ConsoleName": "SNES",
                "NumAchievements": 1, "NumAwardedToUser": 1, "UserCompletion": "100.00%",
                "HighestAwardKind": "mastered",
                "Achievements": { "1": { "ID": 1, "Title": "Time's Up", "Description": "Beat the game", "Points": 10, "BadgeName": "12345", "DisplayOrder": 1, "DateEarned": "2026-08-20 10:00:00" } },
            })))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = get_achievements(&client, &pool, "retrouser", &game_id).await.unwrap().unwrap();

        assert_eq!(progress.game_id, 99);
        assert_eq!(progress.highest_award_kind.as_deref(), Some("mastered"));
        assert_eq!(progress.achievements.len(), 1);
        assert!(progress.achievements[0].unlocked);
        assert_eq!(progress.achievements[0].badge_url, "https://i.retroachievements.org/Badge/12345.png");

        let stored = games::get(&pool, &game_id).await.unwrap().unwrap();
        assert_eq!(stored.ra_highest_award_kind.as_deref(), Some("mastered"));
    }

    #[tokio::test]
    async fn get_achievements_returns_none_when_ra_doesnt_recognize_the_game_or_user() {
        let (pool, _dir) = throwaway_pool().await;
        let game_id = seed_game(&pool, "Chrono Trigger").await;
        sqlx::query!("UPDATE games SET retroachievements_game_id = 99 WHERE id = ?", game_id).execute(&pool).await.unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = get_achievements(&client, &pool, "retrouser", &game_id).await.unwrap();
        assert_eq!(progress, None);
    }

    #[tokio::test]
    async fn get_achievements_returns_game_not_found_for_an_unknown_id() {
        let (pool, _dir) = throwaway_pool().await;
        let client = RetroAchievementsClient::with_base_url("fake-key", "http://localhost:1");

        let err = get_achievements(&client, &pool, "retrouser", "nope").await.unwrap_err();
        assert!(matches!(err, GameActionsError::GameNotFound));
    }
}
