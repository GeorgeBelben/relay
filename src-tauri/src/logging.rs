//! Ships WARN/ERROR-level `tracing` events to BetterStack (the same source the frontend's
//! `src/lib/better-stack.ts` already logs to, via `@logtail/browser`) -- deliberately not a
//! full log-shipping pipeline, just "if the app errors, know about it" for a kiosk device nobody's
//! watching a terminal on. INFO/DEBUG-level events still go to stdout (visible via
//! `journalctl -u relay`) through the ordinary `fmt` layer, but aren't shipped upstream.
//!
//! Delivery is fire-and-forget over a plain OS thread with its own blocking HTTP client, not
//! `tokio::spawn` -- this needs to work from the panic hook (see [`install_panic_hook`]), which
//! can run on any thread, including one with no active Tokio runtime. A logging failure must
//! never itself panic or block the caller.

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;

// Same BetterStack source as the frontend (src/lib/better-stack.ts) -- one combined log stream
// for the whole app, frontend and backend alike.
const BETTERSTACK_ENDPOINT: &str = "https://s2723695.eu-central-1a.betterstackdata.com";
const BETTERSTACK_SOURCE_TOKEN: &str = "Z7CHXeiRieHapLHYqY15GcAf";

#[derive(Default)]
struct EventFields {
    message: String,
    extra: serde_json::Map<String, serde_json::Value>,
}

impl Visit for EventFields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        if field.name() == "message" {
            self.message = rendered;
        } else {
            self.extra.insert(field.name().to_string(), serde_json::Value::String(rendered));
        }
    }
}

/// Pure JSON-shaping, split out from delivery so it's testable without any actual I/O.
fn build_payload(level: Level, target: &str, fields: EventFields) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "level": level.to_string(),
        "message": fields.message,
        "target": target,
    });
    if let Some(obj) = payload.as_object_mut() {
        obj.extend(fields.extra);
    }
    payload
}

/// The actual network call, split out from `ship` below so tests can invoke it directly and
/// synchronously (against a local mock server) rather than racing a spawned thread.
fn send_payload(endpoint: &str, token: &str, payload: &serde_json::Value) {
    let Ok(client) = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(5)).build() else {
        return;
    };
    // Best-effort: a log that fails to ship is a shame, not a second incident. Never
    // `.unwrap()`/propagate from here.
    let _ = client.post(endpoint).bearer_auth(token).json(payload).send();
}

fn ship(level: Level, target: &str, fields: EventFields) {
    let payload = build_payload(level, target, fields);
    std::thread::spawn(move || send_payload(BETTERSTACK_ENDPOINT, BETTERSTACK_SOURCE_TOKEN, &payload));
}

struct BetterStackLayer;

impl<S: Subscriber> Layer<S> for BetterStackLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if *event.metadata().level() > Level::WARN {
            return;
        }

        let mut fields = EventFields::default();
        event.record(&mut fields);
        ship(*event.metadata().level(), event.metadata().target(), fields);
    }
}

/// Call once, at the very start of `run()` -- before anything else logs.
pub fn init() {
    let _ = tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(BetterStackLayer).try_init();
}

/// Rust panics otherwise fail completely silently on this kiosk (no terminal, no crash dialog --
/// just a frozen or vanished window). Composes with the previous hook rather than replacing it,
/// so the panic's message/location/backtrace still print to stderr (visible via
/// `journalctl -u relay`) exactly as before; this only adds the BetterStack report on top.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "Rust panic");
        previous(info);
    }));
}

/// Drop-in replacement for the `.map_err(|e| e.to_string())` pattern used throughout
/// `commands/*.rs` -- logs the error (so a command failure is visible in BetterStack, not just
/// returned to the frontend and easy to miss) before converting it to the string every Tauri
/// command returns on its error path.
///
/// `#[track_caller]` matters here: every call site shares this one function, so without it every
/// logged event would show `logging.rs` as its origin -- useless for telling which command
/// actually failed. This captures the *caller's* file:line instead, the same mechanism
/// `Option::unwrap()`'s panic messages use for the same reason.
#[track_caller]
pub fn err_to_string<E: std::fmt::Display>(e: E) -> String {
    let message = e.to_string();
    let location = std::panic::Location::caller();
    tracing::error!(error = %message, location = %location, "command error");
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn err_to_string_returns_the_displayed_message() {
        assert_eq!(err_to_string("boom"), "boom");
        assert_eq!(err_to_string(std::io::Error::other("disk full")), "disk full");
    }

    #[test]
    fn build_payload_carries_level_target_message_and_extra_fields() {
        let mut extra = serde_json::Map::new();
        extra.insert("error".to_string(), serde_json::Value::String("disk full".to_string()));
        let fields = EventFields { message: "command error".to_string(), extra };

        let payload = build_payload(Level::ERROR, "relay_lib::commands::storage", fields);

        assert_eq!(payload["level"], "ERROR");
        assert_eq!(payload["message"], "command error");
        assert_eq!(payload["target"], "relay_lib::commands::storage");
        assert_eq!(payload["error"], "disk full");
    }

    #[tokio::test]
    async fn send_payload_posts_the_payload_with_bearer_auth() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(202))
            .expect(1)
            .mount(&server)
            .await;

        let payload = serde_json::json!({ "level": "ERROR", "message": "command error", "error": "disk full" });
        // send_payload uses a blocking client -- run it on a blocking-safe thread so it doesn't
        // starve this test's own async runtime.
        let uri = server.uri();
        tokio::task::spawn_blocking(move || send_payload(&uri, "test-token", &payload)).await.unwrap();

        server.verify().await;
    }
}
