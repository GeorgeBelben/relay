use relay_protocol::{Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/relay.sock";

pub async fn ping_daemon() -> Result<Response, String> {
    let stream = UnixStream::connect(SOCKET_PATH)
        .await
        .map_err(|e| format!("couldn't connect {e}"))?;

    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut json = serde_json::to_string(&Request::Ping).unwrap();
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

pub async fn launch_game(rom_path: &str) -> Result<Response, String> {
    let stream = UnixStream::connect(SOCKET_PATH)
        .await
        .map_err(|e| format!("couldn't connect: {e}"))?;

    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let request = Request::LaunchGame {
        rom_path: rom_path.to_string(),
    };
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
