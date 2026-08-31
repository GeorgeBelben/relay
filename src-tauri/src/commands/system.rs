use tauri::AppHandle;

use crate::system::wallpaper;

/// There's no profile/auth system for OS-level login -- the real Linux account name is a more
/// honest placeholder for "currently logged in user" than a fake one, and it's free. Reads $USER
/// directly rather than querying the passwd database (the Electron original's `os.userInfo()`
/// did, via Node's libuv) -- this kiosk always runs as a single fixed service account, so the env
/// var is exactly as reliable here and avoids a new dependency for one string.
#[tauri::command]
pub fn get_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "user".to_string())
}

#[tauri::command]
pub async fn list_wallpapers() -> Vec<String> {
    wallpaper::list_wallpapers().await
}

/// The kiosk systemd unit runs this app as the only thing on screen, no desktop environment
/// underneath it -- quitting is what actually drops back to the console's login/CLI, not a
/// browser-style "close tab". Ported from the Electron MVP's `system.service.ts#quit`.
#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}
