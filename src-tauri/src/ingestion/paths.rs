use std::path::PathBuf;

// ~/Relay -- ported from the Electron MVP's libraryPaths.ts. Only the paths the ingestion
// pipeline actually needs so far (roms + downloaded media) are ported here; the remaining
// subdirectories (bios/wallpapers/saves/savestates/screenshots) are system::storage's concern
// (it just needs this one root, not a helper per subdirectory), not ingestion's.
pub fn library_root() -> PathBuf {
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
