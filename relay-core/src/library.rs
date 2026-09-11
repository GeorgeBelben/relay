use crate::startup::roms_dir;
use crate::systems::ALL_SYSTEMS;
use relay_protocol::LibraryEntry;

/// Scans every known system's ROM folder and returns what it finds.
/// Does no caching or persistence yet — this re-reads the filesystem
/// every time it's called.
pub fn scan_library() -> Vec<LibraryEntry> {
    let mut entries = Vec::new();

    for system in ALL_SYSTEMS {
        let system_dir = roms_dir().join(system.id);

        let read_dir = match std::fs::read_dir(&system_dir) {
            Ok(rd) => rd,
            Err(e) => {
                eprintln!("couldn't read {}: {e}", system_dir.display());
                continue;
            }
        };

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("bad directory entry in {}: {e}", system.id);
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("couldn't read metadata for {}: {e}", path.display());
                    continue;
                }
            };

            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entries.push(LibraryEntry {
                system: system.id.to_string(),
                system_display_name: system.display_name.to_string(),
                file_name,
                rom_path: path.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
            })
        }
    }

    entries
}
