use log::{debug, error, info, warn};
use relay_protocol::ProfileInfo;
use relay_protocol::{DaemonState, Request, Response, RunningGame};
use rusqlite::Connection;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

mod db;
mod emulator;
mod gamescope;
mod library;
mod pcsx2;
mod retroarch;
mod settings;
mod startup;
mod systems;

use emulator::EmulatorBackend;

const SOCKET_PATH: &str = "/tmp/relay.sock";

#[derive(Clone)]
struct AppState {
    running_game: Arc<Mutex<Option<RunningGame>>>,
    db: Arc<Mutex<Connection>>,
    active_profile: Arc<Mutex<Option<i64>>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RaPoints {
    points: i64,
    softcore_points: i64,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    startup::ensure_directories();

    // Remove any leftover socket file from the previous run - Unix sockets
    // fail to bind if the path already exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("failed to bind socket");

    info!("relay-core listening on {SOCKET_PATH}");

    let state = AppState {
        running_game: Arc::new(Mutex::new(None)),
        db: Arc::new(Mutex::new(db::open())),
        active_profile: Arc::new(Mutex::new(None)),
    };

    loop {
        let (stream, _addr) = listener
            .accept()
            .await
            .expect("failed to accept connection");

        let state = state.clone();

        // Handle each connection on its own task so one client
        // can't block another
        tokio::spawn(async move { handle_client(stream, state).await });
    }
}

async fn handle_client(stream: tokio::net::UnixStream, state: AppState) {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();

        let bytes_read = reader.read_line(&mut line).await.unwrap_or(0);
        if bytes_read == 0 {
            // Client disconnected
            break;
        }

        let request: Request = match serde_json::from_str(line.trim()) {
            Ok(req) => req,
            Err(e) => {
                warn!("bad request: {e}");
                continue;
            }
        };

        debug!("recieved: {request:?}");

        let response = match request {
            Request::Ping => Response::Pong,
            Request::GetState => {
                let running_game = state.running_game.lock().unwrap().clone();
                Response::State(DaemonState { running_game })
            }
            Request::LaunchGame { game_id } => launch_game(game_id, state.clone()),
            Request::StopGame => stop_game(state.clone()),
            Request::GetLibrary => {
                let scanned = library::scan_library();
                let conn = state.db.lock().unwrap();
                db::upsert_scanned(&conn, &scanned);
                let entries = db::get_all(&conn);
                drop(conn);
                Response::Library(entries)
            }
            Request::GetSettings { system } => {
                let conn = state.db.lock().unwrap();
                let scope = settings::scope_for(system.as_deref());
                let entries = db::get_settings_for_scope(&conn, &scope);
                drop(conn);
                Response::Settings(entries)
            }
            Request::SetSetting { system, key, value } => {
                let conn = state.db.lock().unwrap();
                let scope = settings::scope_for(system.as_deref());
                db::set_setting(&conn, &scope, &key, &value);
                drop(conn);
                Response::SettingSet
            }
            Request::CreateProfile { name } => create_profile(name, state.clone()),
            Request::ListProfiles => list_profiles(state.clone()),
            Request::SetActiveProfile { profile_id } => {
                set_active_profile(profile_id, state.clone())
            }
            Request::GetActiveProfile => get_active_profile(state.clone()),
            Request::LinkRetroAchievements {
                username,
                web_api_key,
            } => link_retroachievements(username, web_api_key, state.clone()),
            Request::GetRaStats => get_ra_stats(state.clone()).await,
        };

        let mut json = serde_json::to_string(&response).unwrap();
        json.push('\n');
        let _ = write_half.write_all(json.as_bytes()).await;
    }
}

fn launch_game(game_id: i64, state: AppState) -> Response {
    let conn = state.db.lock().unwrap();
    let entry = db::get_by_id(&conn, game_id);

    let Some(entry) = entry else {
        return Response::Error {
            message: format!("no library entry with id {game_id}"),
        };
    };

    let profile_id = *state.active_profile.lock().unwrap();
    let settings = settings::Settings::resolve(&conn, &entry.system, profile_id);

    drop(conn);

    let system =
        systems::find(&entry.system).expect("system for a library entry must exist in ALL_SYSTEMS");

    let backend: Box<dyn EmulatorBackend> = if system.retroarch_core.is_none() {
        Box::new(pcsx2::Pcsx2Backend)
    } else {
        Box::new(retroarch::RetroArchBackend)
    };

    let launched_at = std::time::Instant::now();

    let child = match backend.launch(&entry.rom_path, &settings) {
        Ok(child) => child,
        Err(e) => {
            error!("launch failed: {e}");
            return Response::Error { message: e };
        }
    };

    let pid = child.id();

    *state.running_game.lock().unwrap() = Some(RunningGame {
        pid,
        rom_path: entry.rom_path.clone(),
    });

    // Watch for the process exiting, off the main async runtime, so this
    // doesn't block anything else the daemon is doing.
    let watch_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let mut child = child;
        let exit_status = child.wait();
        info!("game exited: {exit_status:?}");
        *watch_state.running_game.lock().unwrap() = None;

        let played_seconds = launched_at.elapsed().as_secs();
        let conn = watch_state.db.lock().unwrap();
        db::record_play_session(&conn, game_id, played_seconds);
    });

    Response::GameLaunched { pid }
}

fn stop_game(state: AppState) -> Response {
    let pid = state.running_game.lock().unwrap().as_ref().map(|g| g.pid);

    let Some(pid) = pid else {
        return Response::Error {
            message: "no game currently running".to_string(),
        };
    };

    match Command::new("kill").arg(pid.to_string()).status() {
        Ok(status) if status.success() => Response::GameStopped,
        Ok(status) => Response::Error {
            message: format!("kill exited with {status}"),
        },
        Err(e) => Response::Error {
            message: format!("failed to run kill: {e}"),
        },
    }
}

fn to_profile_info(profile: db::Profile) -> ProfileInfo {
    ProfileInfo {
        id: profile.id,
        name: profile.name,
        avatar_seed: profile.avatar_seed,
        ra_linked: profile.ra_username.is_some() && profile.ra_web_api_key.is_some(),
    }
}

fn create_profile(name: String, state: AppState) -> Response {
    let conn = state.db.lock().unwrap();
    let id = db::create_profile(&conn, &name);
    let profile = db::get_profile(&conn, id).expect("just-created profile must exist");
    drop(conn);
    startup::ensure_profile_directories(id);
    Response::Profile(to_profile_info(profile))
}

fn list_profiles(state: AppState) -> Response {
    let conn = state.db.lock().unwrap();
    let profiles = db::list_profiles(&conn);
    drop(conn);
    Response::Profiles(profiles.into_iter().map(to_profile_info).collect())
}

fn set_active_profile(profile_id: i64, state: AppState) -> Response {
    let conn = state.db.lock().unwrap();
    let profile = db::get_profile(&conn, profile_id);
    drop(conn);

    let Some(profile) = profile else {
        return Response::Error {
            message: format!("no profile with id {profile_id}"),
        };
    };

    *state.active_profile.lock().unwrap() = Some(profile_id);
    Response::ActiveProfile(Some(to_profile_info(profile)))
}

fn get_active_profile(state: AppState) -> Response {
    let profile_id = *state.active_profile.lock().unwrap();

    let Some(profile_id) = profile_id else {
        return Response::ActiveProfile(None);
    };

    let conn = state.db.lock().unwrap();
    let profile = db::get_profile(&conn, profile_id);
    drop(conn);

    Response::ActiveProfile(profile.map(to_profile_info))
}

fn link_retroachievements(username: String, web_api_key: String, state: AppState) -> Response {
    let profile_id = *state.active_profile.lock().unwrap();

    let Some(profile_id) = profile_id else {
        return Response::Error {
            message: "no active profile - use SetActiveProfile first".to_string(),
        };
    };

    let conn = state.db.lock().unwrap();
    db::link_retroachievements(&conn, profile_id, &username, &web_api_key);
    let profile = db::get_profile(&conn, profile_id).expect("active profile must exist");
    drop(conn);

    Response::Profile(to_profile_info(profile))
}

async fn get_ra_stats(state: AppState) -> Response {
    let profile_id = *state.active_profile.lock().unwrap();

    let Some(profile_id) = profile_id else {
        return Response::Error {
            message: "no active profile - use SetActiveProfile first".to_string(),
        };
    };

    let profile = {
        let conn = state.db.lock().unwrap();
        db::get_profile(&conn, profile_id)
    };

    let Some(profile) = profile else {
        return Response::Error {
            message: format!("no profile with id {profile_id}"),
        };
    };

    let (Some(username), Some(web_api_key)) = (profile.ra_username, profile.ra_web_api_key) else {
        return Response::Error {
            message: "active profile has no RetroAchievements account linked".to_string(),
        };
    };

    let response = reqwest::Client::new()
        .get("https://retroachievements.org/API/API_GetUserPoints.php")
        .query(&[("u", &username), ("y", &web_api_key)])
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            return Response::Error {
                message: format!("failed to reach RetroAchievements: {e}"),
            };
        }
    };

    match response.json::<RaPoints>().await {
        Ok(stats) => Response::RaStats {
            points: stats.points,
            softcore_points: stats.softcore_points,
        },
        Err(e) => Response::Error {
            message: format!("bad response from RetroAchievements: {e}"),
        },
    }
}
