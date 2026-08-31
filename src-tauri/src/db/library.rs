use serde::Serialize;
use sqlx::SqlitePool;

/// Ready-to-render library data -- one row per playable game (its file still present at the last
/// scan), joined with its system and (if any) box art. `boxart_path` is forward-slash-relative to
/// the media root (see `commands::game_media::get_media_root_path`) -- resolving it into something
/// an `<img>` can load is a frontend-loading decision (asset-protocol scope vs. a byte-serving
/// command), not this module's job, same reasoning as `get_media_root_path`'s own doc comment.
/// Ported from the Electron MVP's `gamesRepository.listWithSystem`/`listAllByTitle`/`listRecentlyAdded`
/// + `libraryService.toLibraryGame`.
#[derive(Debug, Clone, Serialize)]
pub struct LibraryGame {
    pub id: String,
    pub title: String,
    pub system_id: String,
    pub system_name: String,
    pub boxart_path: Option<String>,
    // Cumulative ladder (see the games table's ra_highest_award_kind column) -- any non-null value
    // means the game was beaten at minimum, regardless of which tier it's actually reached.
    pub beaten: bool,
    pub added_at: i64,
}

#[derive(Debug, Serialize)]
pub struct LibraryShelf {
    pub system_id: String,
    pub system_name: String,
    pub games: Vec<LibraryGame>,
}

struct LibraryGameRow {
    id: String,
    title: String,
    system_id: String,
    system_name: String,
    boxart_path: Option<String>,
    ra_highest_award_kind: Option<String>,
    created_at: i64,
}

impl From<LibraryGameRow> for LibraryGame {
    fn from(row: LibraryGameRow) -> Self {
        LibraryGame {
            id: row.id,
            title: row.title,
            system_id: row.system_id,
            system_name: row.system_name,
            boxart_path: row.boxart_path,
            beaten: row.ra_highest_award_kind.is_some(),
            added_at: row.created_at,
        }
    }
}

/// One shelf per system with at least one scanned game, in the order the query already sorts
/// them (system name, then title) -- backs the Home screen.
pub async fn list_shelves(pool: &SqlitePool) -> Result<Vec<LibraryShelf>, sqlx::Error> {
    let rows = sqlx::query_as!(
        LibraryGameRow,
        r#"SELECT games.id, games.title,
                  systems.id as system_id, systems.name as system_name,
                  game_media.local_path as boxart_path,
                  games.ra_highest_award_kind,
                  games.created_at as "created_at!: i64"
           FROM games
           JOIN roms ON games.rom_id = roms.id
           JOIN systems ON roms.system_id = systems.id
           LEFT JOIN game_media ON game_media.game_id = games.id AND game_media.kind = 'boxart'
           WHERE roms.status = 'ok'
           ORDER BY systems.name, games.title"#
    )
    .fetch_all(pool)
    .await?;

    let mut shelves: Vec<LibraryShelf> = Vec::new();
    for row in rows {
        let game: LibraryGame = row.into();
        match shelves.iter_mut().find(|shelf| shelf.system_id == game.system_id) {
            Some(shelf) => shelf.games.push(game),
            None => shelves.push(LibraryShelf {
                system_id: game.system_id.clone(),
                system_name: game.system_name.clone(),
                games: vec![game],
            }),
        }
    }
    Ok(shelves)
}

/// Every playable game, alphabetical by title -- backs the "All Games" grid.
pub async fn list_all_games(pool: &SqlitePool) -> Result<Vec<LibraryGame>, sqlx::Error> {
    let rows = sqlx::query_as!(
        LibraryGameRow,
        r#"SELECT games.id, games.title,
                  systems.id as system_id, systems.name as system_name,
                  game_media.local_path as boxart_path,
                  games.ra_highest_award_kind,
                  games.created_at as "created_at!: i64"
           FROM games
           JOIN roms ON games.rom_id = roms.id
           JOIN systems ON roms.system_id = systems.id
           LEFT JOIN game_media ON game_media.game_id = games.id AND game_media.kind = 'boxart'
           WHERE roms.status = 'ok'
           ORDER BY games.title"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(LibraryGame::from).collect())
}

/// Newest-scanned games first -- backs the Home "Recently Added" fallback row (there's no real
/// play-session tracking yet, so this is what "recent" means for now).
pub async fn list_recently_added(pool: &SqlitePool, limit: i64) -> Result<Vec<LibraryGame>, sqlx::Error> {
    let rows = sqlx::query_as!(
        LibraryGameRow,
        r#"SELECT games.id, games.title,
                  systems.id as system_id, systems.name as system_name,
                  game_media.local_path as boxart_path,
                  games.ra_highest_award_kind,
                  games.created_at as "created_at!: i64"
           FROM games
           JOIN roms ON games.rom_id = roms.id
           JOIN systems ON roms.system_id = systems.id
           LEFT JOIN game_media ON game_media.game_id = games.id AND game_media.kind = 'boxart'
           WHERE roms.status = 'ok'
           ORDER BY games.created_at DESC
           LIMIT ?"#,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(LibraryGame::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn throwaway_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, dir)
    }

    async fn seed_system(pool: &SqlitePool, id: &str, name: &str) {
        sqlx::query!(
            "INSERT INTO systems (id, name, extensions) VALUES (?, ?, '[]')",
            id,
            name
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_rom(pool: &SqlitePool, id: &str, system_id: &str, path: &str, status: &str) {
        let now = crate::db::time::now_unix();
        sqlx::query!(
            "INSERT INTO roms (id, system_id, path, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            id,
            system_id,
            path,
            status,
            now,
            now,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_game(pool: &SqlitePool, id: &str, rom_id: &str, title: &str, created_at: i64) {
        sqlx::query!(
            "INSERT INTO games (id, rom_id, title, scanned_title, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            id,
            rom_id,
            title,
            title,
            created_at,
            created_at,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn list_shelves_groups_games_by_system_excluding_missing_roms() {
        let (pool, _dir) = throwaway_pool().await;
        seed_system(&pool, "nes", "NES").await;
        seed_system(&pool, "snes", "SNES").await;
        seed_rom(&pool, "r1", "nes", "mario.nes", "ok").await;
        seed_rom(&pool, "r2", "snes", "zelda.sfc", "ok").await;
        seed_rom(&pool, "r3", "nes", "missing.nes", "missing").await;
        seed_game(&pool, "g1", "r1", "Mario", 1).await;
        seed_game(&pool, "g2", "r2", "Zelda", 2).await;
        seed_game(&pool, "g3", "r3", "Missing Game", 3).await;

        let shelves = list_shelves(&pool).await.unwrap();

        assert_eq!(shelves.len(), 2);
        let nes_shelf = shelves.iter().find(|s| s.system_id == "nes").unwrap();
        assert_eq!(nes_shelf.games.len(), 1);
        assert_eq!(nes_shelf.games[0].title, "Mario");
        assert!(!nes_shelf.games[0].beaten);
        assert_eq!(nes_shelf.games[0].boxart_path, None);
    }

    #[tokio::test]
    async fn list_all_games_is_alphabetical_by_title() {
        let (pool, _dir) = throwaway_pool().await;
        seed_system(&pool, "nes", "NES").await;
        seed_rom(&pool, "r1", "nes", "z.nes", "ok").await;
        seed_rom(&pool, "r2", "nes", "a.nes", "ok").await;
        seed_game(&pool, "g1", "r1", "Zelda", 1).await;
        seed_game(&pool, "g2", "r2", "Adventure", 2).await;

        let games = list_all_games(&pool).await.unwrap();

        assert_eq!(games.iter().map(|g| g.title.as_str()).collect::<Vec<_>>(), vec!["Adventure", "Zelda"]);
    }

    #[tokio::test]
    async fn list_recently_added_is_newest_first_and_respects_limit() {
        let (pool, _dir) = throwaway_pool().await;
        seed_system(&pool, "nes", "NES").await;
        seed_rom(&pool, "r1", "nes", "a.nes", "ok").await;
        seed_rom(&pool, "r2", "nes", "b.nes", "ok").await;
        seed_rom(&pool, "r3", "nes", "c.nes", "ok").await;
        seed_game(&pool, "g1", "r1", "First", 1).await;
        seed_game(&pool, "g2", "r2", "Second", 2).await;
        seed_game(&pool, "g3", "r3", "Third", 3).await;

        let games = list_recently_added(&pool, 2).await.unwrap();

        assert_eq!(games.iter().map(|g| g.title.as_str()).collect::<Vec<_>>(), vec!["Third", "Second"]);
    }

    #[tokio::test]
    async fn beaten_is_true_once_ra_highest_award_kind_is_set() {
        let (pool, _dir) = throwaway_pool().await;
        seed_system(&pool, "nes", "NES").await;
        seed_rom(&pool, "r1", "nes", "a.nes", "ok").await;
        seed_game(&pool, "g1", "r1", "Beaten Game", 1).await;
        sqlx::query!("UPDATE games SET ra_highest_award_kind = 'beaten-softcore' WHERE id = 'g1'")
            .execute(&pool)
            .await
            .unwrap();

        let games = list_all_games(&pool).await.unwrap();

        assert!(games[0].beaten);
    }
}
