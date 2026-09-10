use std::path::PathBuf;

/// Every system we support a ROM subfolder for. Adjust freely — this is
/// just a list, not something wired into logic elsewhere yet.
const SUPPORTED_SYSTEMS: &[&str] = &[
    "nes", "snes", "genesis", "n64", "gb", "gba", "psx", "ps2", "gamecube", "wii", "psp",
];

/// Ensures the directories relay-core needs actually exist, creating them
/// if this is a fresh install. Called once at startup, before the daemon
/// starts accepting connections — if we can't get this right, there's no
/// point continuing.
pub fn ensure_directories() {
    let data_dir = dirs::data_dir()
        .expect("could not determine a data directory for this user")
        .join("relay");

    create_dir(&data_dir);

    let library = library_root();
    for system in SUPPORTED_SYSTEMS {
        create_dir(&roms_dir().join(system));
    }

    create_dir(&bios_dir());
    create_dir(&saves_dir());
    create_dir(&states_dir());
    create_dir(&shaders_dir());
    create_dir(&screenshots_dir());

    println!("data dir ready: {}", data_dir.display());
    println!("library ready: {}", library.display());
}

fn create_dir(path: &PathBuf) {
    std::fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", path.display()))
}

pub fn library_root() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable was not set");
    PathBuf::from(home).join("Relay")
}

pub fn roms_dir() -> PathBuf {
    library_root().join("roms")
}

pub fn bios_dir() -> PathBuf {
    library_root().join("bios")
}

pub fn saves_dir() -> PathBuf {
    library_root().join("saves")
}

pub fn states_dir() -> PathBuf {
    library_root().join("states")
}

pub fn shaders_dir() -> PathBuf {
    library_root().join("shaders")
}

pub fn screenshots_dir() -> PathBuf {
    library_root().join("screenshots")
}
