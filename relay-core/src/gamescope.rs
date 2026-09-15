use std::process::Command;

/// Finds gamescope's WAYLAND_DISPLAY/XDG_RUNTIME_DIR by reading its
/// environment directly out of /proc, so anything we spawn can target
/// the same Wayland display gamescope is compositing.
pub fn gamescope_env() -> Vec<(String, String)> {
    let output = Command::new("pgrep").arg("-x").arg("gamescope-wl").output();
    let Ok(output) = output else {
        return Vec::new();
    };
    let Ok(pid) = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
    else {
        return Vec::new();
    };

    let Ok(raw) = std::fs::read(format!("/proc/{pid}/environ")) else {
        return Vec::new();
    };

    raw.split(|&b| b == 0)
        .filter_map(|entry| {
            let s = String::from_utf8_lossy(entry);
            let (key, value) = s.split_once('=')?;
            if key == "WAYLAND_DISPLAY" || key == "XDG_RUNTIME_DIR" {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect()
}
