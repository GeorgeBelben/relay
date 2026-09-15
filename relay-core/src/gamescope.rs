use std::os::unix::fs::FileTypeExt;

/// Finds gamescope's WAYLAND_DISPLAY/XDG_RUNTIME_DIR, so anything we spawn
/// can target the same Wayland display gamescope is compositing.
///
/// We can't read this out of gamescope's own environment (`/proc/pid/environ`
/// only reflects what a process was *exec'd* with, never anything it sets on
/// itself afterward via `setenv()` - which is exactly how compositors publish
/// WAYLAND_DISPLAY). Instead: `XDG_RUNTIME_DIR` is reliable from our *own*
/// environment (systemd sets it for us), and gamescope's socket is a real
/// file sitting in that directory - so we just look for it.
pub fn gamescope_env() -> Vec<(String, String)> {
    let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") else {
        return Vec::new();
    };

    let Ok(entries) = std::fs::read_dir(&runtime_dir) else {
        return Vec::new();
    };

    let display = entries.filter_map(|entry| entry.ok()).find_map(|entry| {
        let name = entry.file_name().into_string().ok()?;
        let is_gamescope_socket = name.starts_with("gamescope-")
            && !name.ends_with("-ei")
            && !name.ends_with(".lock");
        if !is_gamescope_socket {
            return None;
        }
        entry.file_type().ok()?.is_socket().then_some(name)
    });

    let Some(display) = display else {
        return Vec::new();
    };

    vec![
        ("WAYLAND_DISPLAY".to_string(), display),
        ("XDG_RUNTIME_DIR".to_string(), runtime_dir),
    ]
}
