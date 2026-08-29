// Disk usage breakdown for the settings UI (REL-109), ported from the Electron MVP's
// storage.service.ts / diskUsage.ts. `library_root` is always taken as a parameter rather than
// resolved internally (see ingestion::enrich's media_root precedent) so tests can point this at
// a tempdir instead of ever touching the real ~/Relay.

use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use serde::Serialize;

// Async recursion needs boxing -- a naive recursive `async fn` has an infinite-size future.
pub fn get_directory_size(dir: PathBuf) -> Pin<Box<dyn Future<Output = u64> + Send>> {
    Box::pin(async move {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            // A missing subdirectory (e.g. no BIOS files ever added) contributes 0 bytes rather
            // than erroring, matching the MVP's getDirectorySize.
            Err(_) => return 0,
        };

        let mut total = 0u64;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let Ok(metadata) = entry.metadata().await else { continue };
            if metadata.is_dir() {
                total += get_directory_size(entry.path()).await;
            } else {
                total += metadata.len();
            }
        }
        total
    })
}

// `library_root` may not exist yet on a fresh install (nothing creates ~/Relay itself up front --
// its subdirectories are created lazily by scan/enrich), which would make statvfs fail with
// ENOENT. Walk up to the nearest existing ancestor -- still the same filesystem/mount in practice.
fn nearest_existing_ancestor(path: &Path) -> PathBuf {
    let mut current = path;
    loop {
        if current.exists() {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return PathBuf::from("/"),
        }
    }
}

// Uses available-to-non-root bytes (`bavail`), not raw free (`bfree`) -- the app never runs as
// root, matching the MVP's own reasoning in diskUsage.ts.
pub fn get_disk_space(at_path: &Path) -> io::Result<(u64, u64)> {
    let existing = nearest_existing_ancestor(at_path);
    let stats = nix::sys::statvfs::statvfs(&existing).map_err(io::Error::from)?;
    // Field widths (blocks vs. fragment size) differ between platforms' statvfs -- cast both
    // explicitly rather than relying on inference to pick a consistent width.
    let fragment_size = stats.fragment_size() as u64;
    let total_bytes = stats.blocks() as u64 * fragment_size;
    let free_bytes = stats.blocks_available() as u64 * fragment_size;
    Ok((total_bytes, free_bytes))
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageUsage {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub games_bytes: u64,
    pub bios_bytes: u64,
    pub media_bytes: u64,
    pub saves_bytes: u64,
    pub system_bytes: u64,
}

pub async fn get_storage_usage(library_root: &Path) -> io::Result<StorageUsage> {
    let (total_bytes, free_bytes) = get_disk_space(library_root)?;

    let games_bytes = get_directory_size(library_root.join("roms")).await;
    let bios_bytes = get_directory_size(library_root.join("bios")).await;
    let media_bytes = get_directory_size(library_root.join("media")).await
        + get_directory_size(library_root.join("wallpapers")).await;
    let saves_bytes = get_directory_size(library_root.join("saves")).await
        + get_directory_size(library_root.join("savestates")).await
        + get_directory_size(library_root.join("screenshots")).await;

    // Whatever's left of "used" space once the categories above are subtracted out -- the OS,
    // other apps, etc. Clamped at 0 rather than going negative if the breakdown ever overcounts.
    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let known_categories_bytes = games_bytes + bios_bytes + media_bytes + saves_bytes;
    let system_bytes = used_bytes.saturating_sub(known_categories_bytes);

    Ok(StorageUsage { total_bytes, free_bytes, games_bytes, bios_bytes, media_bytes, saves_bytes, system_bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn directory_size_sums_files_across_nested_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(dir.path().join("a.bin"), vec![0u8; 100]).await.unwrap();
        tokio::fs::create_dir(dir.path().join("sub")).await.unwrap();
        tokio::fs::write(dir.path().join("sub").join("b.bin"), vec![0u8; 50]).await.unwrap();

        let size = get_directory_size(dir.path().to_path_buf()).await;
        assert_eq!(size, 150);
    }

    #[tokio::test]
    async fn directory_size_of_a_missing_directory_is_zero() {
        let size = get_directory_size(PathBuf::from("/does/not/exist/relay-test")).await;
        assert_eq!(size, 0);
    }

    #[test]
    fn disk_space_reports_plausible_nonzero_totals_for_a_real_path() {
        let (total, free) = get_disk_space(Path::new("/")).unwrap();
        assert!(total > 0);
        assert!(free <= total);
    }

    #[test]
    fn disk_space_falls_back_to_the_nearest_existing_ancestor() {
        let dir = tempfile::tempdir().unwrap();
        let missing_child = dir.path().join("not-created-yet").join("also-missing");

        let (total, _free) = get_disk_space(&missing_child).unwrap();
        assert!(total > 0);
    }

    #[tokio::test]
    async fn storage_usage_breaks_down_bytes_by_category_from_a_fabricated_library_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        tokio::fs::create_dir_all(root.join("roms")).await.unwrap();
        tokio::fs::write(root.join("roms").join("game.sfc"), vec![0u8; 200]).await.unwrap();
        tokio::fs::create_dir_all(root.join("media")).await.unwrap();
        tokio::fs::write(root.join("media").join("box.png"), vec![0u8; 30]).await.unwrap();
        tokio::fs::create_dir_all(root.join("saves")).await.unwrap();
        tokio::fs::write(root.join("saves").join("game.srm"), vec![0u8; 10]).await.unwrap();
        // bios/, wallpapers/, savestates/, screenshots/ deliberately left absent.

        let usage = get_storage_usage(root).await.unwrap();

        assert_eq!(usage.games_bytes, 200);
        assert_eq!(usage.bios_bytes, 0);
        assert_eq!(usage.media_bytes, 30);
        assert_eq!(usage.saves_bytes, 10);
        assert!(usage.total_bytes > 0);
        assert!(usage.free_bytes <= usage.total_bytes);
    }
}
