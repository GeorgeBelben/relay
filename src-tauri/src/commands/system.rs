use tauri::{AppHandle, Manager};

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

/// `lib.rs`'s own `set_cursor_visible(false)` call at startup doesn't always take visual effect
/// until the OS actually processes a mouse-moved event over the window -- the default arrow can
/// sit visible for a moment after boot even though it's already been told to hide. Re-asserted
/// from the frontend the instant real movement is detected (see main.tsx) closes that gap.
#[tauri::command]
pub fn hide_cursor(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("main window must exist")?;
    window.set_cursor_visible(false).map_err(crate::logging::err_to_string)
}
