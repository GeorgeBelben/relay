//! Thin wrapper over SteamGridDB's v2 API -- https://www.steamgriddb.com/api/v2. Auth is a
//! single bearer token (the user's own API key, from steamgriddb.com/profile/preferences/api),
//! no OAuth exchange needed. Ported from the Electron MVP's metadata/steamgriddbClient.ts.

use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SteamGridDbGame {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct SteamGridDbGrid {
    url: String,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    success: bool,
    data: T,
}

#[derive(Debug)]
pub enum SteamGridDbError {
    Http { status: u16, url: String },
    Unsuccessful { url: String },
    Request(reqwest::Error),
}

impl std::fmt::Display for SteamGridDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http { status, url } => write!(f, "SteamGridDB request failed: {status} ({url})"),
            Self::Unsuccessful { url } => write!(f, "SteamGridDB request unsuccessful ({url})"),
            Self::Request(e) => write!(f, "SteamGridDB request error: {e}"),
        }
    }
}

impl std::error::Error for SteamGridDbError {}

impl From<reqwest::Error> for SteamGridDbError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

#[derive(Clone)]
pub struct SteamGridDbClient {
    http: reqwest::Client,
    base_url: reqwest::Url,
    api_key: String,
}

impl SteamGridDbClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Exposed so tests can point the client at a local mock server instead of the real API.
    pub fn with_base_url(api_key: impl Into<String>, base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: reqwest::Url::parse(base_url).expect("invalid SteamGridDB base URL"),
            api_key: api_key.into(),
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(&self, url: reqwest::Url) -> Result<T, SteamGridDbError> {
        let res = self.http.get(url.clone()).bearer_auth(&self.api_key).send().await?;

        if !res.status().is_success() {
            return Err(SteamGridDbError::Http { status: res.status().as_u16(), url: url.to_string() });
        }

        let envelope: Envelope<T> = res.json().await?;
        if !envelope.success {
            return Err(SteamGridDbError::Unsuccessful { url: url.to_string() });
        }
        Ok(envelope.data)
    }

    pub async fn search_games(&self, title: &str) -> Result<Vec<SteamGridDbGame>, SteamGridDbError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().extend(["search", "autocomplete", title]);
        self.request(url).await
    }

    /// Portrait "grid" art (600x900) is the closest thing SteamGridDB has to traditional box art
    /// -- picks the first result at that dimension, or `None` if nobody's uploaded one.
    pub async fn get_boxart_url(&self, game_id: i64) -> Result<Option<String>, SteamGridDbError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().extend(["grids", "game", &game_id.to_string()]);
        url.query_pairs_mut().append_pair("dimensions", "600x900");

        let grids: Vec<SteamGridDbGrid> = self.request(url).await?;
        Ok(grids.into_iter().next().map(|g| g.url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_games_returns_parsed_candidates() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/Mario%20Kart%2064"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 42, "name": "Mario Kart 64" }],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let games = client.search_games("Mario Kart 64").await.unwrap();

        assert_eq!(games, vec![SteamGridDbGame { id: 42, name: "Mario Kart 64".into() }]);
    }

    #[tokio::test]
    async fn get_boxart_url_returns_first_grid_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/grids/game/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 1, "url": "https://example.com/boxart.png" }],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let url = client.get_boxart_url(42).await.unwrap();

        assert_eq!(url.as_deref(), Some("https://example.com/boxart.png"));
    }

    #[tokio::test]
    async fn get_boxart_url_returns_none_when_no_grids_exist() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/grids/game/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        assert_eq!(client.get_boxart_url(42).await.unwrap(), None);
    }

    #[tokio::test]
    async fn non_2xx_response_is_a_http_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/x"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("bad-key", &format!("{}/api/v2", server.uri()));
        let err = client.search_games("x").await.unwrap_err();

        assert!(matches!(err, SteamGridDbError::Http { status: 401, .. }));
    }

    #[tokio::test]
    async fn success_false_is_an_unsuccessful_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/x"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "data": [],
            })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let err = client.search_games("x").await.unwrap_err();

        assert!(matches!(err, SteamGridDbError::Unsuccessful { .. }));
    }
}
