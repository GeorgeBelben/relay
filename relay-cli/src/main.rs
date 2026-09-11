use clap::{Parser, Subcommand};
use relay_protocol::{Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/relay.sock";

#[derive(Parser)]
#[command(name = "relay-cli", about = "Talk to the relay-core daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // Check if the daemon is alive
    Ping,
    // Ask the daemon what's currently running
    State,
    // Launch a rom by an absolute path
    Launch { rom_path: String },
    Library,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let request = match cli.command {
        Command::Ping => Request::Ping,
        Command::State => Request::GetState,
        Command::Launch { rom_path } => Request::LaunchGame { rom_path },
        Command::Library => Request::GetLibrary,
    };

    match send(request).await {
        Ok(response) => print_response(response),
        Err(e) => {
            eprintln!("error {e}");
            std::process::exit(1);
        }
    }
}

async fn send(request: Request) -> Result<Response, String> {
    let stream = UnixStream::connect(SOCKET_PATH)
        .await
        .map_err(|e| format!("couldn't connect to daemon: {e}"))?;

    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut json = serde_json::to_string(&request).unwrap();
    json.push('\n');
    write_half
        .write_all(json.as_bytes())
        .await
        .map_err(|e| format!("write failed: {e}"))?;

    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("read failed: {e}"))?;

    serde_json::from_str(line.trim()).map_err(|e| format!("bad response: {e}"))
}

fn print_response(response: Response) {
    match response {
        Response::Pong => println!("Pong"),
        Response::Error { message } => {
            eprintln!("daemon error: {message}");
            std::process::exit(1);
        }
        Response::GameLaunched { pid } => println!("launched, pid {pid}"),
        Response::GameExited { exit_code } => println!("game exited, code {exit_code:?}"),
        Response::State(state) => match state.running_game {
            Some(game) => println!("running: {} (pid {})", game.rom_path, game.pid),
            None => println!("nothing running"),
        },
        Response::Library(entries) => {
            if entries.is_empty() {
                println!("library is empty")
            } else {
                for entry in entries {
                    println!(
                        "[{}] {} ({} bytes)",
                        entry.system_display_name, entry.file_name, entry.size_bytes
                    );
                }
            }
        }
    }
}
