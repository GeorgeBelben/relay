use std::path::PathBuf;

// ~/Relay -- ported from the Electron MVP's libraryPaths.ts. Only the paths the ingestion
// pipeline actually needs so far (roms + downloaded media) are ported; saves/savestates/
// screenshots/wallpapers land when emulator launch (Phase 4) or later features need them.
fn library_root() -> PathBuf {
    dirs::home_dir().expect("could not resolve home directory").join("Relay")
}

pub fn roms_path() -> PathBuf {
    library_root().join("roms")
}

// Downloaded box art lives here, one subfolder per game -- see ingestion::enrich, which joins
// on `<system_id>/<game_id>` itself (accepting this root as a parameter rather than calling this
// function directly, so tests can point it at a tempdir instead). Not part of the documented
// roms/bios layout a user manages; this is app-owned output.
pub fn media_path() -> PathBuf {
    library_root().join("media")
}
