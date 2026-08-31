//! Thin wrapper over RetroAchievements' classic Web API -- https://api-docs.retroachievements.org.
//! Auth is a `y=<web API key>` query param (from retroachievements.org/settings, "Keys" tab), the
//! same one every third-party RA client (RetroArch, RALibretro, EmulationStation, ...) has used
//! for years. No OAuth exchange needed. Ported from the Electron MVP's
//! `lib/retroachievements/client.ts`.
//!
//! `getGameListWithHashes` (ROM-hash -> RA game ID matching, used by the ingestion pipeline to
//! auto-populate `games.retroachievements_game_id`) isn't ported here -- that's an ingestion/
//! matching concern, not profile-linking, and out of scope for this pass. Until it lands, no game
//! ever gets an RA game ID, so `get_game_info_and_user_progress` always has nothing to look up --
//! same "shows nothing until enrichment runs" shape as unenriched box art.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://retroachievements.org/API";
const BADGE_BASE_URL: &str = "https://i.retroachievements.org/Badge";

#[derive(Debug)]
pub enum RaError {
    Http { status: u16, url: String },
    Request(reqwest::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for RaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http { status, url } => write!(f, "RetroAchievements request failed: {status} ({url})"),
            Self::Request(e) => write!(f, "RetroAchievements request error: {e}"),
            Self::Parse(e) => write!(f, "RetroAchievements response parse error: {e}"),
        }
    }
}

impl std::error::Error for RaError {}

impl From<reqwest::Error> for RaError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

// RA's own "Game Beaten"/award ladder -- confirmed against api-docs.retroachievements.org's
// GetGameInfoAndUserProgress and GetUserCompletionProgress pages, both of which show
// HighestAwardKind: "beaten-hardcore" in their example responses; the other three values are
// confirmed via RA's own forum posts describing the feature. Cumulative, low to high -- reaching
// "completed"/"mastered" always implies the game was beaten first, so "has this game been beaten
// at all" is just `.is_some()`.
const HIGHEST_AWARD_KINDS: [&str; 4] = ["beaten-softcore", "beaten-hardcore", "completed", "mastered"];

fn parse_highest_award_kind(value: Option<String>) -> Option<String> {
    value.filter(|v| HIGHEST_AWARD_KINDS.contains(&v.as_str()))
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaAchievement {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub points: i64,
    pub badge_name: String,
    pub display_order: i64,
    // Present (a timestamp string) the moment the user has earned it in either softcore or
    // hardcore -- deliberately not distinguishing hardcore here, since RA's own most prominent
    // progress figure (num_awarded_to_user, below) doesn't either.
    pub unlocked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaGameProgress {
    pub game_id: i64,
    pub title: String,
    pub console_name: String,
    pub num_achievements: i64,
    pub num_awarded_to_user: i64,
    pub user_completion: String,
    pub highest_award_kind: Option<String>,
    pub achievements: Vec<RaAchievement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaRecentUnlock {
    pub game_title: String,
    pub title: String,
    pub points: i64,
    pub badge_url: String,
    pub unlocked_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaUserStats {
    pub points: i64,
    // RA returns this as a formatted string (e.g. "1,234"), not a number.
    pub rank: String,
    pub recent_unlocks: Vec<RaRecentUnlock>,
}

// The "_lock" suffix is RA's own convention for the greyed-out variant of a badge, used for
// achievements the user hasn't earned yet.
pub fn badge_url(badge_name: &str, unlocked: bool) -> String {
    format!("{BADGE_BASE_URL}/{badge_name}{}.png", if unlocked { "" } else { "_lock" })
}

#[derive(Debug, Deserialize)]
struct RawAchievement {
    #[serde(rename = "ID")]
    id: i64,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Points")]
    points: i64,
    #[serde(rename = "BadgeName")]
    badge_name: String,
    #[serde(rename = "DisplayOrder")]
    display_order: i64,
    #[serde(rename = "DateEarned", default)]
    date_earned: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawGameInfoAndUserProgress {
    #[serde(rename = "ID")]
    id: i64,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "ConsoleName")]
    console_name: String,
    #[serde(rename = "NumAchievements")]
    num_achievements: i64,
    #[serde(rename = "NumAwardedToUser")]
    num_awarded_to_user: i64,
    #[serde(rename = "UserCompletion")]
    user_completion: String,
    // Undocumented whether an unawarded game omits this, or sends null/"" -- treated identically
    // either way by parse_highest_award_kind, which rejects anything but the four known strings.
    #[serde(rename = "HighestAwardKind", default)]
    highest_award_kind: Option<String>,
    #[serde(rename = "Achievements", default)]
    achievements: HashMap<String, RawAchievement>,
}

#[derive(Debug, Deserialize)]
struct RawRecentAchievement {
    #[serde(rename = "GameTitle")]
    game_title: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Points")]
    points: i64,
    #[serde(rename = "BadgeName")]
    badge_name: String,
    #[serde(rename = "Date")]
    date: String,
}

#[derive(Debug, Deserialize)]
struct RawUserSummary {
    #[serde(rename = "TotalPoints")]
    total_points: i64,
    #[serde(rename = "Rank")]
    rank: String,
    #[serde(rename = "RecentAchievements", default)]
    recent_achievements: Option<serde_json::Value>,
}

// The classic API's own convention for "a collection of achievements" is an object keyed by
// achievement id (see RawGameInfoAndUserProgress.achievements above) rather than a plain array --
// unconfirmed live for GetUserSummary's RecentAchievements field specifically, so this accepts
// either shape; either way of returning "zero" (missing/empty) is treated as no recent unlocks.
fn recent_achievements_list(value: Option<serde_json::Value>) -> Result<Vec<RawRecentAchievement>, RaError> {
    match value {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(serde_json::Value::Array(items)) => {
            items.into_iter().map(|v| serde_json::from_value(v).map_err(RaError::Parse)).collect()
        }
        Some(serde_json::Value::Object(map)) => {
            map.into_values().map(|v| serde_json::from_value(v).map_err(RaError::Parse)).collect()
        }
        Some(_) => Ok(Vec::new()),
    }
}

pub struct RetroAchievementsClient {
    http: reqwest::Client,
    base_url: reqwest::Url,
    api_key: String,
}

impl RetroAchievementsClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Exposed so tests can point the client at a local mock server instead of the real API.
    pub fn with_base_url(api_key: impl Into<String>, base_url: &str) -> Self {
        Self { http: reqwest::Client::new(), base_url: reqwest::Url::parse(base_url).expect("invalid RA base URL"), api_key: api_key.into() }
    }

    async fn get(&self, path: &str, params: &[(&str, String)]) -> Result<serde_json::Value, RaError> {
        let mut url = self.base_url.join(path).expect("invalid RA API path");
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("y", &self.api_key);
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }

        let res = self.http.get(url.clone()).send().await?;
        if !res.status().is_success() {
            return Err(RaError::Http { status: res.status().as_u16(), url: url.to_string() });
        }
        Ok(res.json::<serde_json::Value>().await?)
    }

    /// Returns `None` for an unrecognized game ID or username -- the endpoint itself signals that
    /// by returning `[]` (an empty array, not an object) instead of an error, per its own source.
    pub async fn get_game_info_and_user_progress(&self, username: &str, game_id: i64) -> Result<Option<RaGameProgress>, RaError> {
        let value = self
            .get("API_GetGameInfoAndUserProgress.php", &[("u", username.to_string()), ("g", game_id.to_string())])
            .await?;

        if value.is_array() {
            return Ok(None);
        }
        let raw: RawGameInfoAndUserProgress = serde_json::from_value(value).map_err(RaError::Parse)?;
        if raw.id == 0 {
            return Ok(None);
        }

        let mut achievements: Vec<RaAchievement> = raw
            .achievements
            .into_values()
            .map(|a| RaAchievement {
                id: a.id,
                title: a.title,
                description: a.description,
                points: a.points,
                badge_name: a.badge_name,
                display_order: a.display_order,
                unlocked_at: a.date_earned,
            })
            .collect();
        achievements.sort_by_key(|a| a.display_order);

        Ok(Some(RaGameProgress {
            game_id: raw.id,
            title: raw.title,
            console_name: raw.console_name,
            num_achievements: raw.num_achievements,
            num_awarded_to_user: raw.num_awarded_to_user,
            user_completion: raw.user_completion,
            highest_award_kind: parse_highest_award_kind(raw.highest_award_kind),
            achievements,
        }))
    }

    // GetUserSummary's docs flag it as slow/over-fetching for a *per-render* call, which is exactly
    // why this only ever backs the ra_stats cache refresh (on link + on active-profile switch),
    // not per-screen. Its own `a` param bundles recent unlocks into the same response, avoiding a
    // second round trip.
    const RECENT_UNLOCKS_COUNT: u32 = 5;

    pub async fn get_user_stats(&self, username: &str) -> Result<RaUserStats, RaError> {
        let value = self
            .get("API_GetUserSummary.php", &[("u", username.to_string()), ("a", Self::RECENT_UNLOCKS_COUNT.to_string())])
            .await?;
        let raw: RawUserSummary = serde_json::from_value(value).map_err(RaError::Parse)?;

        let recent_unlocks = recent_achievements_list(raw.recent_achievements)?
            .into_iter()
            .map(|a| RaRecentUnlock {
                game_title: a.game_title,
                title: a.title,
                points: a.points,
                badge_url: badge_url(&a.badge_name, true), // only unlocked achievements ever appear here
                unlocked_at: a.date,
            })
            .collect();

        Ok(RaUserStats { points: raw.total_points, rank: raw.rank, recent_unlocks })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const RAW_GAME_INFO: &str = r#"{
        "ID": 1, "Title": "Chrono Trigger", "ConsoleName": "SNES",
        "NumAchievements": 2, "NumAwardedToUser": 2, "UserCompletion": "100.00%",
        "Achievements": {}
    }"#;

    fn with_field(json: &str, field: &str, value: serde_json::Value) -> serde_json::Value {
        let mut v: serde_json::Value = serde_json::from_str(json).unwrap();
        v[field] = value;
        v
    }

    #[tokio::test]
    async fn parses_a_recognized_highest_award_kind() {
        let server = MockServer::start().await;
        let body = with_field(RAW_GAME_INFO, "HighestAwardKind", "mastered".into());
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .and(query_param("y", "fake-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = client.get_game_info_and_user_progress("retrouser", 1).await.unwrap();
        assert_eq!(progress.unwrap().highest_award_kind.as_deref(), Some("mastered"));
    }

    #[tokio::test]
    async fn treats_a_missing_highest_award_kind_as_none() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(RAW_GAME_INFO, "application/json"))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = client.get_game_info_and_user_progress("retrouser", 1).await.unwrap();
        assert_eq!(progress.unwrap().highest_award_kind, None);
    }

    #[tokio::test]
    async fn falls_back_to_none_for_an_unrecognized_highest_award_kind() {
        let server = MockServer::start().await;
        let body = with_field(RAW_GAME_INFO, "HighestAwardKind", "some-future-kind".into());
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = client.get_game_info_and_user_progress("retrouser", 1).await.unwrap();
        assert_eq!(progress.unwrap().highest_award_kind, None);
    }

    #[tokio::test]
    async fn returns_none_for_an_unrecognized_game_or_username() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetGameInfoAndUserProgress.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let progress = client.get_game_info_and_user_progress("retrouser", 999).await.unwrap();
        assert_eq!(progress, None);
    }

    #[tokio::test]
    async fn maps_points_rank_and_a_recent_unlocks_array_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetUserSummary.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "TotalPoints": 4200,
                "Rank": "1,234",
                "RecentAchievements": [{ "GameTitle": "Chrono Trigger", "Title": "Time's Up", "Points": 10, "BadgeName": "12345", "Date": "2026-08-20 10:00:00" }],
            })))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let stats = client.get_user_stats("retrouser").await.unwrap();

        assert_eq!(stats.points, 4200);
        assert_eq!(stats.rank, "1,234");
        assert_eq!(
            stats.recent_unlocks,
            vec![RaRecentUnlock {
                game_title: "Chrono Trigger".into(),
                title: "Time's Up".into(),
                points: 10,
                badge_url: "https://i.retroachievements.org/Badge/12345.png".into(),
                unlocked_at: "2026-08-20 10:00:00".into(),
            }]
        );
    }

    #[tokio::test]
    async fn also_accepts_a_keyed_object_recent_unlocks_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetUserSummary.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "TotalPoints": 4200,
                "Rank": "1,234",
                "RecentAchievements": { "555": { "GameTitle": "Chrono Trigger", "Title": "Time's Up", "Points": 10, "BadgeName": "12345", "Date": "2026-08-20 10:00:00" } },
            })))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let stats = client.get_user_stats("retrouser").await.unwrap();

        assert_eq!(stats.recent_unlocks.len(), 1);
        assert_eq!(stats.recent_unlocks[0].title, "Time's Up");
    }

    #[tokio::test]
    async fn treats_a_missing_recent_achievements_field_as_no_recent_unlocks() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/API_GetUserSummary.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "TotalPoints": 0, "Rank": "N/A" })))
            .mount(&server)
            .await;

        let client = RetroAchievementsClient::with_base_url("fake-key", &server.uri());
        let stats = client.get_user_stats("retrouser").await.unwrap();
        assert_eq!(stats.recent_unlocks, vec![]);
    }

    #[test]
    fn badge_url_appends_lock_suffix_only_when_locked() {
        assert_eq!(badge_url("12345", true), "https://i.retroachievements.org/Badge/12345.png");
        assert_eq!(badge_url("12345", false), "https://i.retroachievements.org/Badge/12345_lock.png");
    }
}
