mod daemon;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn check_daemon_connection() -> Result<String, String> {
    match daemon::ping_daemon().await {
        Ok(_) => Ok("connected".to_string()),
        Err(e) => Err(e),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![check_daemon_connection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
