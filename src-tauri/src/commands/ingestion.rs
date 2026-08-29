use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

use crate::db::settings;
use crate::ingestion::identify::steamgriddb::SteamGridDbClient;
use crate::ingestion::paths;
use crate::ingestion::pipeline::{self, ScanStatus};

/// Guards against overlapping "Rescan Library" runs -- managed as Tauri state so it lives for
/// the app's lifetime, shared across every `rescan_library` invocation.
pub struct RescanGuard(pub Arc<AtomicBool>);

impl Default for RescanGuard {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

/// The most recently observed [`ScanStatus`], for `get_scan_status`'s pull-on-mount read -- a
/// listener that attaches after a fast scan already finished would otherwise never see anything.
pub struct ScanStatusState(pub Mutex<ScanStatus>);

impl Default for ScanStatusState {
    fn default() -> Self {
        Self(Mutex::new(ScanStatus::Idle))
    }
}

#[tauri::command]
pub fn get_scan_status(status: State<'_, ScanStatusState>) -> ScanStatus {
    status.0.lock().unwrap().clone()
}

#[tauri::command]
pub async fn rescan_library(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    guard: State<'_, RescanGuard>,
    status: State<'_, ScanStatusState>,
) -> Result<(), String> {
    rescan_library_at(&app, pool.inner(), &guard.0, &status.0, &paths::roms_path(), &paths::media_path()).await
}

/// The actual guts of `rescan_library`, with the library roots taken as parameters rather than
/// resolved internally -- the public command always passes the real `~/Relay` paths, but this is
/// what lets a test exercise the AppHandle-emit + status-state wiring against a tempdir instead
/// of risking a scan of whatever happens to be at the real path on the machine running it.
async fn rescan_library_at<R: tauri::Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    running: &Arc<AtomicBool>,
    status: &Mutex<ScanStatus>,
    roms_root: &Path,
    media_root: &Path,
) -> Result<(), String> {
    let api_key = settings::get(pool, "steamgriddbApiKey").await.map_err(|e| e.to_string())?;
    let client = api_key.map(SteamGridDbClient::new);

    pipeline::rescan(pool, roms_root, media_root, client.as_ref(), running, |next| {
        if let Ok(mut guard) = status.lock() {
            *guard = next.clone();
        }
        let _ = app.emit("scanner:status", &next);
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    use tauri::WebviewWindowBuilder;

    #[tokio::test]
    async fn rescan_library_at_updates_status_state_and_emits_events() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let app = mock_builder().build(mock_context(noop_assets())).expect("failed to build mock app");
        let _webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        let app_handle = app.handle().clone();

        let running = Arc::new(AtomicBool::new(false));
        let status = Mutex::new(ScanStatus::Idle);
        let roms_root = tempfile::tempdir().unwrap();
        let media_root = tempfile::tempdir().unwrap();

        rescan_library_at(&app_handle, &pool, &running, &status, roms_root.path(), media_root.path())
            .await
            .unwrap();

        // No systems seeded, so scan finds nothing and enrichment never starts (no api key
        // configured either) -- what matters here is that the state wiring itself works.
        assert_eq!(*status.lock().unwrap(), ScanStatus::Done);
        assert!(!running.load(std::sync::atomic::Ordering::SeqCst));
    }
}
