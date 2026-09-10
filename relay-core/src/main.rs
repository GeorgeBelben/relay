use relay_protocol::{DaemonState, Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/relay.sock";

#[tokio::main]
async fn main() {
    // Remove any leftover socket file from the previous run - Unix sockets
    // fail to bind if the path already exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("failed to bind socket");

    println!("relay-core listening on {SOCKET_PATH}");

    loop {
        let (stream, _addr) = listener
            .accept()
            .await
            .expect("failed to accept connection");

        // Handle each connection on its own task so one client
        // can't block another
        tokio::spawn(async move { handle_client(stream).await });
    }
}

async fn handle_client(stream: tokio::net::UnixStream) {
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
            Request::GetState => Response::State(DaemonState { running_game: None }),
            Request::LaunchGame { rom_path } => {
                // We'll actually spawn emulators in a later step.
                println!("(pretending to launch {rom_path})");
                Response::GameLaunched { pid: 0 }
            }
        };

        let mut json = serde_json::to_string(&response).unwrap();
        json.push('\n');
        let _ = write_half.write_all(json.as_bytes()).await;
    }
}
