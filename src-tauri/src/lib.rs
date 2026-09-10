mod daemon;

#[tauri::command]
async fn check_daemon_connection() -> Result<String, String> {
    match daemon::ping_daemon().await {
        Ok(_) => Ok("connected".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn launch_game(rom_path: String) -> Result<String, String> {
    match daemon::launch_game(&rom_path).await? {
        relay_protocol::Response::GameLaunched { pid } => Ok(format!("launched, pid {pid}")),
        relay_protocol::Response::Error { message } => Err(message),
        other => Err(format!("unexpected response: {other:?}")),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_daemon_connection,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
