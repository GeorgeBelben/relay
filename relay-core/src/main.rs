use relay_protocol::{DaemonState, Request, Response, RunningGame};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

mod emulator;
mod retroarch;
mod startup;

use emulator::EmulatorBackend;

const SOCKET_PATH: &str = "/tmp/relay.sock";

/// Shared, mutable "what's running right now" state, accessible from
/// every client-handling task.
type SharedState = Arc<Mutex<Option<RunningGame>>>;

#[tokio::main]
async fn main() {
    startup::ensure_directories();

    // Remove any leftover socket file from the previous run - Unix sockets
    // fail to bind if the path already exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("failed to bind socket");

    println!("relay-core listening on {SOCKET_PATH}");

    let state: SharedState = Arc::new(Mutex::new(None));

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

async fn handle_client(stream: tokio::net::UnixStream, state: SharedState) {
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
                let running_game = state.lock().unwrap().clone();
                Response::State(DaemonState { running_game })
            }
            Request::LaunchGame { rom_path } => launch_game(rom_path, state.clone()),
        };

        let mut json = serde_json::to_string(&response).unwrap();
        json.push('\n');
        let _ = write_half.write_all(json.as_bytes()).await;
    }
}

fn launch_game(rom_path: String, state: SharedState) -> Response {
    let backend = retroarch::RetroArchBackend;

    let child = match backend.launch(&rom_path) {
        Ok(child) => child,
        Err(e) => {
            eprintln!("launch failed: {e}");
            return Response::Error { message: e };
        }
    };

    let pid = child.id();

    *state.lock().unwrap() = Some(RunningGame {
        pid,
        rom_path: rom_path.clone(),
    });

    // Watch for the process exiting, off the main async runtime, so this
    // doesn't block anything else the daemon is doing.
    let watch_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let mut child = child;
        let exit_status = child.wait();
        println!("game exited: {exit_status:?}");
        *watch_state.lock().unwrap() = None;
    });

    Response::GameLaunched { pid }
}
