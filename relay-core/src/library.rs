use crate::startup::{SUPPORTED_SYSTEMS, roms_dir};
use relay_protocol::LibraryEntry;

/// Scans every known system's ROM folder and returns what it finds.
/// Does no caching or persistence yet — this re-reads the filesystem
/// every time it's called.
pub fn scan_library() -> Vec<LibraryEntry> {
    let mut entries = Vec::new();

    for system in SUPPORTED_SYSTEMS {
        let system_dir = roms_dir().join(system);

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
                    eprintln!("bad directory entry in {system}: {e}");
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
                system: system.to_string(),
                file_name,
                rom_path: path.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
            })
        }
    }

    entries
}
