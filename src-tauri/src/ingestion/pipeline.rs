use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use sqlx::SqlitePool;

use crate::db::{games, roms, systems};

use super::enrich;
use super::identify::no_intro::NoIntroDatLookup;
use super::identify::steamgriddb::SteamGridDbClient;
use super::probe;
use super::scan::{self, ScanTarget};
use super::title::title_from_filename;

// Mirrors the Electron MVP's shared/types.ts#ScanStatus exactly (field names included, via
// serde's internal tagging + kebab-case rename) so the frontend event contract is unchanged.
// `matching-achievements` is omitted -- no RetroAchievements pipeline exists in this rewrite yet.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ScanStatus {
    Idle,
    ScanningFiles,
    EnrichingArt { current: u32, total: u32 },
    Done,
    Error { message: String },
}

#[derive(Debug)]
pub enum PipelineError {
    Db(sqlx::Error),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<sqlx::Error> for PipelineError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

fn to_forward_slash(path: &Path) -> String {
    path.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/")
}

async fn upsert_rom_and_game(
    pool: &SqlitePool,
    system_id: &str,
    path: String,
    crc32: Option<String>,
    size_bytes: Option<i64>,
    discs: Option<String>,
    title: &str,
) -> Result<(), sqlx::Error> {
    let rom = roms::upsert(pool, roms::NewRom { system_id: system_id.to_string(), path, crc32, size_bytes, discs }).await?;
    games::upsert_for_rom(pool, &rom.id, title).await?;
    Ok(())
}

/// Scan stage (REL-80) + probe stage (REL-81) tied together: walks every known system's rom
/// folder, hashes what it finds, and upserts roms/games. Anything not found this pass is marked
/// missing, not deleted. Ported from the Electron MVP's `scanLibrary.ts` (its dev-seed branch is
/// deliberately not ported -- that's a dev-only convenience, not a pipeline concern).
pub async fn scan_and_probe(pool: &SqlitePool, roms_root: &Path, no_intro: &NoIntroDatLookup) -> Result<usize, sqlx::Error> {
    let all_systems = systems::list(pool).await?;
    let mut found_paths = Vec::new();

    for system in &all_systems {
        let system_folder = roms_root.join(&system.id);
        let extensions: Vec<String> = serde_json::from_str(&system.extensions).unwrap_or_default();
        let targets = scan::walk_system_folder(&system_folder, &extensions).await;

        for target in targets {
            match target {
                ScanTarget::Single { file_path, title } => {
                    let relative = to_forward_slash(file_path.strip_prefix(roms_root).unwrap_or(&file_path));
                    found_paths.push(relative.clone());

                    let (crc32, size_bytes) = match probe::probe_file(&file_path).await {
                        Ok(p) => (Some(p.crc32), Some(p.size_bytes)),
                        Err(_) => (None, None),
                    };

                    // An exact CRC32 match against No-Intro's own data beats guessing a title
                    // from whatever the filename happens to look like -- falls back to the
                    // filename-derived title exactly as before on any miss.
                    let title = match &crc32 {
                        Some(crc) => match no_intro.lookup(&system.id, crc).await {
                            Some(dat_title) => title_from_filename(&dat_title),
                            None => title,
                        },
                        None => title,
                    };

                    upsert_rom_and_game(pool, &system.id, relative, crc32, size_bytes, None, &title).await?;
                }
                ScanTarget::MultiDisc { m3u_path, disc_paths, title } => {
                    let relative = to_forward_slash(m3u_path.strip_prefix(roms_root).unwrap_or(&m3u_path));
                    found_paths.push(relative.clone());

                    let mut discs = Vec::new();
                    for disc_path in &disc_paths {
                        // A disc listed in the .m3u but missing on disk is skipped, not fatal --
                        // matches the MVP's warn-and-continue behavior.
                        if let Ok(p) = probe::probe_file(disc_path).await {
                            discs.push(serde_json::json!({
                                "path": to_forward_slash(disc_path.strip_prefix(roms_root).unwrap_or(disc_path)),
                                "crc32": p.crc32,
                                "sizeBytes": p.size_bytes,
                            }));
                        }
                    }
                    let discs_json = if discs.is_empty() { None } else { serde_json::to_string(&discs).ok() };

                    upsert_rom_and_game(pool, &system.id, relative, None, None, discs_json, &title).await?;
                }
            }
        }
    }

    roms::mark_missing(pool, &found_paths).await?;
    Ok(found_paths.len())
}

/// The full "Rescan Library" operation: scan+probe, then identify+enrich anything unenriched --
/// sequential, not fire-and-forget, so it's one honest operation with one status stream rather
/// than a second background process the UI can't reflect. `running` guards against overlapping
/// runs (a second "Rescan" press mid-run is a no-op, not an error) -- callers own the flag's
/// lifetime (the real app manages one alongside the DB pool; tests use a fresh local one, so
/// runs in different tests never see each other's state). `steamgriddb` is `None` when no API
/// key is configured yet, which skips enrichment entirely (matching the MVP's behavior) --
/// taking an already-constructed client (rather than building one from a key internally) is what
/// lets tests point it at a mock server instead of the real API; `no_intro` follows the same
/// pattern for scan-time title correction. `on_status` is called with each state transition --
/// production wires it to a Tauri event emit; tests just collect it.
#[allow(clippy::too_many_arguments)]
pub async fn rescan(
    pool: &SqlitePool,
    roms_root: &Path,
    media_root: &Path,
    no_intro: &NoIntroDatLookup,
    steamgriddb: Option<&SteamGridDbClient>,
    running: &AtomicBool,
    mut on_status: impl FnMut(ScanStatus),
) -> Result<(), PipelineError> {
    if running.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let result = run_rescan(pool, roms_root, media_root, no_intro, steamgriddb, &mut on_status).await;
    running.store(false, Ordering::SeqCst);

    if let Err(e) = &result {
        on_status(ScanStatus::Error { message: e.to_string() });
    }
    result
}

#[allow(clippy::too_many_arguments)]
async fn run_rescan(
    pool: &SqlitePool,
    roms_root: &Path,
    media_root: &Path,
    no_intro: &NoIntroDatLookup,
    steamgriddb: Option<&SteamGridDbClient>,
    on_status: &mut impl FnMut(ScanStatus),
) -> Result<(), PipelineError> {
    on_status(ScanStatus::ScanningFiles);
    scan_and_probe(pool, roms_root, no_intro).await?;

    if let Some(client) = steamgriddb {
        let http = reqwest::Client::new();
        let pending = games::list_unenriched(pool).await?;
        let total = pending.len() as u32;
        on_status(ScanStatus::EnrichingArt { current: 0, total });

        for (i, game) in pending.iter().enumerate() {
            if let Err(e) = enrich::enrich_one(client, &http, pool, game, media_root).await {
                eprintln!("enrich: failed to enrich \"{}\" ({}): {e}", game.title, game.id);
            }
            on_status(ScanStatus::EnrichingArt { current: (i + 1) as u32, total });
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    on_status(ScanStatus::Done);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::systems::{self, NewSystem};
    use std::fs;
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

    // Points at an unreachable address (nothing listens on 127.0.0.1:1) so a lookup fails fast
    // with no real network call and no cache file -- these tests don't care about DAT lookup
    // behavior (that's no_intro.rs's job), just that titles stay filename-derived without it.
    fn no_intro_stub() -> NoIntroDatLookup {
        NoIntroDatLookup::with_base_url(tempfile::tempdir().unwrap().keep(), "http://127.0.0.1:1/")
    }

    async fn seed_snes(pool: &SqlitePool) {
        systems::create(
            pool,
            NewSystem {
                id: "snes".into(),
                name: "SNES".into(),
                extensions: r#"["sfc"]"#.into(),
                retroarch_core: None,
                standalone_binary: None,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn scan_and_probe_creates_roms_and_games_then_marks_vanished_files_missing() {
        let (pool, _db_dir) = throwaway_pool().await;
        seed_snes(&pool).await;

        let roms_root = tempfile::tempdir().unwrap();
        let snes_dir = roms_root.path().join("snes");
        fs::create_dir(&snes_dir).unwrap();
        fs::write(snes_dir.join("Chrono Trigger (USA).sfc"), b"fake-rom-bytes").unwrap();
        let no_intro = no_intro_stub();

        let found = scan_and_probe(&pool, roms_root.path(), &no_intro).await.unwrap();
        assert_eq!(found, 1);

        let all_roms = roms::list(&pool).await.unwrap();
        assert_eq!(all_roms.len(), 1);
        assert_eq!(all_roms[0].status, "ok");
        assert!(all_roms[0].crc32.is_some());

        let all_games = games::list(&pool).await.unwrap();
        assert_eq!(all_games.len(), 1);
        assert_eq!(all_games[0].title, "Chrono Trigger");

        // Rescanning after the file's gone marks the rom missing rather than deleting it.
        fs::remove_file(snes_dir.join("Chrono Trigger (USA).sfc")).unwrap();
        let found_again = scan_and_probe(&pool, roms_root.path(), &no_intro).await.unwrap();
        assert_eq!(found_again, 0);

        let all_roms = roms::list(&pool).await.unwrap();
        assert_eq!(all_roms[0].status, "missing");
    }

    #[tokio::test]
    async fn rescan_without_api_key_skips_enrichment() {
        let (pool, _db_dir) = throwaway_pool().await;
        seed_snes(&pool).await;

        let roms_root = tempfile::tempdir().unwrap();
        fs::create_dir(roms_root.path().join("snes")).unwrap();
        fs::write(roms_root.path().join("snes/game.sfc"), b"data").unwrap();
        let media_root = tempfile::tempdir().unwrap();

        let no_intro = no_intro_stub();
        let mut statuses = Vec::new();
        let running = AtomicBool::new(false);
        rescan(&pool, roms_root.path(), media_root.path(), &no_intro, None, &running, |s| statuses.push(s))
            .await
            .unwrap();

        assert_eq!(statuses, vec![ScanStatus::ScanningFiles, ScanStatus::Done]);

        let game = &games::list(&pool).await.unwrap()[0];
        assert!(game.enriched_at.is_none());
    }

    #[tokio::test]
    async fn rescan_with_api_key_enriches_and_reports_progress() {
        let (pool, _db_dir) = throwaway_pool().await;
        seed_snes(&pool).await;

        let roms_root = tempfile::tempdir().unwrap();
        fs::create_dir(roms_root.path().join("snes")).unwrap();
        fs::write(roms_root.path().join("snes/Chrono Trigger.sfc"), b"data").unwrap();
        let media_root = tempfile::tempdir().unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/search/autocomplete/Chrono%20Trigger"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": 7, "name": "Chrono Trigger" }],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v2/grids/game/7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true, "data": [] })))
            .mount(&server)
            .await;

        let client = SteamGridDbClient::with_base_url("test-key", &format!("{}/api/v2", server.uri()));
        let no_intro = no_intro_stub();
        let mut statuses = Vec::new();
        let running = AtomicBool::new(false);
        rescan(&pool, roms_root.path(), media_root.path(), &no_intro, Some(&client), &running, |s| statuses.push(s))
            .await
            .unwrap();

        assert_eq!(statuses[0], ScanStatus::ScanningFiles);
        assert_eq!(statuses[1], ScanStatus::EnrichingArt { current: 0, total: 1 });
        assert_eq!(statuses[2], ScanStatus::EnrichingArt { current: 1, total: 1 });
        assert_eq!(statuses[3], ScanStatus::Done);

        let game = &games::list(&pool).await.unwrap()[0];
        assert_eq!(game.steamgriddb_id, Some(7));
    }

    #[tokio::test]
    async fn rescan_is_a_no_op_while_already_running() {
        let (pool, _db_dir) = throwaway_pool().await;
        seed_snes(&pool).await;

        let roms_root = tempfile::tempdir().unwrap();
        fs::create_dir(roms_root.path().join("snes")).unwrap();
        fs::write(roms_root.path().join("snes/game.sfc"), b"data").unwrap();
        let media_root = tempfile::tempdir().unwrap();

        let no_intro = no_intro_stub();
        let running = AtomicBool::new(true); // simulate an in-flight run
        let mut statuses = Vec::new();
        rescan(&pool, roms_root.path(), media_root.path(), &no_intro, None, &running, |s| statuses.push(s))
            .await
            .unwrap();

        assert!(statuses.is_empty());
        assert!(games::list(&pool).await.unwrap().is_empty());
    }
}
