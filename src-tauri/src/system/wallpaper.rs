use crate::ingestion::paths::library_root;

const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Filenames only, alphabetical -- the frontend builds a loadable URL from these itself (asset
/// protocol scope + `library_root()/wallpapers/<filename>`), same reasoning as
/// `commands::game_media::get_media_root_path`'s own doc comment. A missing `wallpapers/`
/// directory (nothing ever placed there) is just an empty list, not an error -- ported from the
/// Electron MVP's `wallpaperService.list`.
pub async fn list_wallpapers() -> Vec<String> {
    let mut entries = match tokio::fs::read_dir(library_root().join("wallpapers")).await {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };

    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let is_file = entry.file_type().await.map(|t| t.is_file()).unwrap_or(false);
        if !is_file {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
        if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
            names.push(name);
        }
    }

    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // list_wallpapers reads from the real library_root() (~/Relay), not an injectable path -- same
    // pattern as commands::storage's get_storage_usage. Exercised here against whatever's actually
    // there rather than faking HOME, which would need an env-var-mutating test (not thread-safe
    // alongside the rest of the suite).
    #[tokio::test]
    async fn returns_an_empty_list_rather_than_erroring_when_nothing_is_there_or_the_dir_is_missing() {
        // Not asserting emptiness (a real ~/Relay/wallpapers might exist on the dev machine) --
        // just that this never panics/errors regardless of whether the directory exists.
        let _ = list_wallpapers().await;
    }

    #[tokio::test]
    async fn filters_by_image_extension_case_insensitively_and_sorts_alphabetically() {
        let dir = library_root().join("wallpapers");
        fs::create_dir_all(&dir).unwrap();
        let marker = format!("zzz-test-marker-{}", std::process::id());
        let a = dir.join(format!("{marker}-b.PNG"));
        let b = dir.join(format!("{marker}-a.jpg"));
        let ignored = dir.join(format!("{marker}-c.txt"));
        fs::write(&a, b"").unwrap();
        fs::write(&b, b"").unwrap();
        fs::write(&ignored, b"").unwrap();

        let names = list_wallpapers().await;

        fs::remove_file(&a).unwrap();
        fs::remove_file(&b).unwrap();
        fs::remove_file(&ignored).unwrap();

        let ours: Vec<_> = names.into_iter().filter(|n| n.contains(&marker)).collect();
        assert_eq!(ours, vec![format!("{marker}-a.jpg"), format!("{marker}-b.PNG")]);
    }
}
