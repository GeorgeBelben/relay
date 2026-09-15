use crate::systems::ALL_SYSTEMS;
use log::info;
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine a data directory for this user")
        .join("relay")
}

/// Ensures the directories relay-core needs actually exist, creating them
/// if this is a fresh install. Called once at startup, before the daemon
/// starts accepting connections — if we can't get this right, there's no
/// point continuing.
pub fn ensure_directories() {
    let data_dir = data_dir();

    create_dir(&data_dir);

    let library = library_root();
    for system in ALL_SYSTEMS {
        create_dir(&roms_dir().join(system.id));
    }

    create_dir(&bios_dir());
    create_dir(&saves_dir(None));
    create_dir(&states_dir(None));
    create_dir(&shaders_dir());
    create_dir(&screenshots_dir());

    info!("data dir ready: {}", data_dir.display());
    info!("library ready: {}", library.display());
}

fn create_dir(path: &PathBuf) {
    std::fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", path.display()))
}

pub fn library_root() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable was not set");
    PathBuf::from(home).join("Relay")
}

fn profile_root(profile_id: i64) -> PathBuf {
    library_root().join("profiles").join(profile_id.to_string())
}

pub fn ensure_profile_directories(profile_id: i64) {
    create_dir(&saves_dir(Some(profile_id)));
    create_dir(&states_dir(Some(profile_id)));
}

pub fn roms_dir() -> PathBuf {
    library_root().join("roms")
}

pub fn bios_dir() -> PathBuf {
    library_root().join("bios")
}

pub fn saves_dir(profile_id: Option<i64>) -> PathBuf {
    match profile_id {
        Some(id) => profile_root(id).join("saves"),
        None => library_root().join("saves"),
    }
}

pub fn states_dir(profile_id: Option<i64>) -> PathBuf {
    match profile_id {
        Some(id) => profile_root(id).join("states"),
        None => library_root().join("states"),
    }
}

pub fn shaders_dir() -> PathBuf {
    library_root().join("shaders")
}

pub fn screenshots_dir() -> PathBuf {
    library_root().join("screenshots")
}
