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
}

#[tokio::main]
async fn main() {
    startup::ensure_directories();

    // Remove any leftover socket file from the previous run - Unix sockets
    // fail to bind if the path already exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("failed to bind socket");

    println!("relay-core listening on {SOCKET_PATH}");

    let state = AppState {
        running_game: Arc::new(Mutex::new(None)),
        db: Arc::new(Mutex::new(db::open())),
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
                println!("bad request: {e}");
                continue;
            }
        };

        println!("recieved: {request:?}");

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

    let settings = settings::Settings::resolve(&conn, &entry.system);

    drop(conn);

    let system =
        systems::find(&entry.system).expect("system for a library entry must exist in ALL_SYSTEMS");

    let backend: Box<dyn EmulatorBackend> = if system.retroarch_core.is_none() {
        Box::new(pcsx2::Pcsx2Backend)
    } else {
        Box::new(retroarch::RetroArchBackend)
    };

    let child = match backend.launch(&entry.rom_path, &settings) {
        Ok(child) => child,
        Err(e) => {
            eprintln!("launch failed: {e}");
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
        println!("game exited: {exit_status:?}");
        *watch_state.running_game.lock().unwrap() = None;
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
