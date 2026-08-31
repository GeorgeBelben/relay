//! RA's *other* API surface -- a single non-RESTful endpoint (dorequest.php), distinct from the
//! REST-ish Web API in client.rs (different base path, different auth: u/p query params instead
//! of a Web API key). Ported from the Electron MVP's `lib/retroachievements/connectClient.ts` --
//! exists only for the documented third-party-standalone login flow
//! (api-docs.retroachievements.org/connect/standalone.html, r=login2) that mints a session token
//! from a one-time username+password exchange. Everything downstream (achievement unlock
//! reporting, hardcore mode, rich presence) is handled by RetroArch/PCSX2 themselves once that
//! token is injected into their own config -- not by this module calling the endpoint again
//! (config injection is separate, deferred work: see REL-115/REL-117).

use serde::Deserialize;

const DEFAULT_CONNECT_URL: &str = "https://retroachievements.org/dorequest.php";

// Documented format: "{Frontend/Standalone name}/{x.y.z Version} ({platform})".
fn user_agent() -> String {
    format!("Relay/{} (Linux)", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
pub enum ConnectError {
    Http(u16),
    Request(reqwest::Error),
    InvalidCredentials(String),
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(status) => write!(f, "RetroAchievements login failed: {status}"),
            Self::Request(e) => write!(f, "RetroAchievements login request error: {e}"),
            Self::InvalidCredentials(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ConnectError {}

impl From<reqwest::Error> for ConnectError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

#[derive(Debug, Deserialize)]
struct RawLoginResponse {
    #[serde(rename = "Success")]
    success: bool,
    #[serde(rename = "Token")]
    token: Option<String>,
    #[serde(rename = "Error")]
    error: Option<String>,
}

/// Throws on bad credentials or a network/API failure -- there's no "maybe" here the way there is
/// for e.g. a hash lookup; a failed login is always something the caller needs to surface to the
/// user, not silently swallow.
pub async fn login(http: &reqwest::Client, connect_url: &str, username: &str, password: &str) -> Result<String, ConnectError> {
    let mut url = reqwest::Url::parse(connect_url).expect("invalid RA connect URL");
    url.query_pairs_mut().append_pair("r", "login2").append_pair("u", username).append_pair("p", password);

    let res = http.get(url).header("User-Agent", user_agent()).send().await?;
    if !res.status().is_success() {
        return Err(ConnectError::Http(res.status().as_u16()));
    }

    let data: RawLoginResponse = res.json().await?;
    if !data.success {
        return Err(ConnectError::InvalidCredentials(
            data.error.unwrap_or_else(|| "RetroAchievements login failed: invalid username or password".to_string()),
        ));
    }
    data.token.ok_or_else(|| ConnectError::InvalidCredentials("RetroAchievements login succeeded but returned no token".to_string()))
}

/// Convenience wrapper for real call sites, pinned to the real endpoint.
pub async fn login_real(http: &reqwest::Client, username: &str, password: &str) -> Result<String, ConnectError> {
    login(http, DEFAULT_CONNECT_URL, username, password).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn hits_login2_with_u_p_and_a_real_user_agent_returning_the_token() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dorequest.php"))
            .and(query_param("r", "login2"))
            .and(query_param("u", "retrouser"))
            .and(query_param("p", "hunter2"))
            .and(header("User-Agent", user_agent().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "Success": true, "Token": "abc123" })))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let token = login(&http, &format!("{}/dorequest.php", server.uri()), "retrouser", "hunter2").await.unwrap();
        assert_eq!(token, "abc123");
    }

    #[tokio::test]
    async fn surfaces_ras_own_error_message_on_invalid_credentials() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dorequest.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "Success": false, "Error": "Credentials invalid" })))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let err = login(&http, &format!("{}/dorequest.php", server.uri()), "retrouser", "wrong").await.unwrap_err();
        assert!(matches!(err, ConnectError::InvalidCredentials(message) if message == "Credentials invalid"));
    }

    #[tokio::test]
    async fn errors_on_a_non_2xx_http_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/dorequest.php")).respond_with(ResponseTemplate::new(500)).mount(&server).await;

        let http = reqwest::Client::new();
        let err = login(&http, &format!("{}/dorequest.php", server.uri()), "retrouser", "hunter2").await.unwrap_err();
        assert!(matches!(err, ConnectError::Http(500)));
    }
}
