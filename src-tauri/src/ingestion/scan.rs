use std::path::{Path, PathBuf};

use super::title::title_from_filename;

/// A file (or multi-disc unit) found by [`walk_system_folder`], not yet hashed or identified --
/// that happens in later ingestion stages (probe, identify).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanTarget {
    Single {
        file_path: PathBuf,
        title: String,
    },
    MultiDisc {
        m3u_path: PathBuf,
        disc_paths: Vec<PathBuf>,
        title: String,
    },
}

// Filters out dotfiles -- most importantly macOS's AppleDouble sidecar files (`._Some Game.gb`),
// which get silently created alongside every real file when copying a ROM library onto a
// non-HFS+ filesystem (SMB, exFAT, a plain USB stick) from a Mac, and otherwise match the real
// file's own extension exactly, indexing as a second bogus, near-duplicate game per real one.
// `.DS_Store` and friends are caught the same way, on general "hidden file" principle.
fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Walks one system's rom folder (one level deep) and returns candidate scan targets.
///
/// A system folder contains either loose rom files matching `extensions` (compared
/// case-insensitively, without a leading dot -- e.g. `"nes"` not `".nes"`), or subfolders for
/// multi-disc games; a subfolder with no `.m3u` playlist inside it is skipped entirely, since
/// there's no reliable way to guess which loose files belong together. A missing or unreadable
/// system folder yields an empty list rather than an error -- ported from the Electron MVP's
/// `scanner/walk.ts`.
pub async fn walk_system_folder(system_folder: &Path, extensions: &[&str]) -> Vec<ScanTarget> {
    let Ok(mut entries) = tokio::fs::read_dir(system_folder).await else {
        return Vec::new();
    };

    let mut targets = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        let entry_path = entry.path();

        if is_hidden(&entry.file_name().to_string_lossy()) {
            continue;
        }

        if file_type.is_file() {
            let matches_extension = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext)));

            if matches_extension {
                let stem = entry_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                targets.push(ScanTarget::Single {
                    file_path: entry_path.clone(),
                    title: title_from_filename(stem),
                });
            }
            continue;
        }

        if file_type.is_dir() {
            if let Some(target) = read_multi_disc_folder(&entry_path).await {
                targets.push(target);
            }
        }
    }

    targets
}

async fn read_multi_disc_folder(folder_path: &Path) -> Option<ScanTarget> {
    let mut inner = tokio::fs::read_dir(folder_path).await.ok()?;

    let mut m3u_path = None;
    while let Ok(Some(entry)) = inner.next_entry().await {
        let is_file = matches!(entry.file_type().await, Ok(ft) if ft.is_file());
        if !is_file {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_hidden(&name) {
            continue;
        }
        if name.to_lowercase().ends_with(".m3u") {
            m3u_path = Some(entry.path());
            break;
        }
    }
    let m3u_path = m3u_path?;

    let playlist = tokio::fs::read_to_string(&m3u_path).await.ok()?;
    let disc_paths: Vec<PathBuf> = playlist
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| folder_path.join(line))
        .collect();

    let folder_name = folder_path.file_name()?.to_string_lossy().into_owned();
    Some(ScanTarget::MultiDisc {
        m3u_path,
        disc_paths,
        title: title_from_filename(&folder_name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn missing_folder_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        let targets = walk_system_folder(&missing, &["nes"]).await;
        assert!(targets.is_empty());
    }

    #[tokio::test]
    async fn skips_macos_appledouble_sidecar_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Pokemon - Red Version.gb"), "real-rom-bytes").unwrap();
        // The exact AppleDouble naming macOS produces copying onto a non-HFS+ filesystem --
        // same extension as the real file, so only the leading dot distinguishes it.
        fs::write(dir.path().join("._Pokemon - Red Version.gb"), "resource-fork-junk").unwrap();

        let targets = walk_system_folder(dir.path(), &["gb"]).await;

        assert_eq!(targets.len(), 1);
        match &targets[0] {
            ScanTarget::Single { file_path, .. } => {
                assert_eq!(file_path.file_name().unwrap(), "Pokemon - Red Version.gb");
            }
            other => panic!("expected Single, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn skips_an_appledouble_sidecar_for_a_multi_disc_m3u() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("Chrono Cross (USA)");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("Chrono Cross.m3u"), "Chrono Cross (Disc 1).cue\n").unwrap();
        // A hidden sidecar for the playlist itself -- must not be mistaken for the real m3u.
        fs::write(game_dir.join("._Chrono Cross.m3u"), "junk").unwrap();

        let targets = walk_system_folder(dir.path(), &["cue"]).await;

        assert_eq!(targets.len(), 1);
        match &targets[0] {
            ScanTarget::MultiDisc { m3u_path, .. } => {
                assert_eq!(m3u_path.file_name().unwrap(), "Chrono Cross.m3u");
            }
            other => panic!("expected MultiDisc, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn matches_files_by_extension_case_insensitively() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Super Mario Bros. (USA).NES"), "").unwrap();
        fs::write(dir.path().join("readme.txt"), "").unwrap();

        let targets = walk_system_folder(dir.path(), &["nes"]).await;

        assert_eq!(targets.len(), 1);
        match &targets[0] {
            ScanTarget::Single { file_path, title } => {
                assert_eq!(file_path.file_name().unwrap(), "Super Mario Bros. (USA).NES");
                assert_eq!(title, "Super Mario Bros.");
            }
            other => panic!("expected Single, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn multi_disc_folder_without_m3u_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("Some Game");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("Some Game (Disc 1).cue"), "").unwrap();

        let targets = walk_system_folder(dir.path(), &["cue"]).await;
        assert!(targets.is_empty());
    }

    #[tokio::test]
    async fn multi_disc_folder_with_m3u_parses_disc_list() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("Chrono Cross (USA)");
        fs::create_dir(&game_dir).unwrap();
        fs::write(
            game_dir.join("Chrono Cross.m3u"),
            "#EXTM3U\nChrono Cross (Disc 1).cue\n\nChrono Cross (Disc 2).cue\n",
        )
        .unwrap();

        let targets = walk_system_folder(dir.path(), &["cue"]).await;

        assert_eq!(targets.len(), 1);
        match &targets[0] {
            ScanTarget::MultiDisc { m3u_path, disc_paths, title } => {
                assert_eq!(m3u_path.file_name().unwrap(), "Chrono Cross.m3u");
                assert_eq!(title, "Chrono Cross");
                assert_eq!(
                    disc_paths,
                    &[
                        game_dir.join("Chrono Cross (Disc 1).cue"),
                        game_dir.join("Chrono Cross (Disc 2).cue"),
                    ]
                );
            }
            other => panic!("expected MultiDisc, got {other:?}"),
        }
    }
}
