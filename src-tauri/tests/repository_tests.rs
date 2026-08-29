use relay_lib::db::{game_media, games, profiles, ra_stats, roms, settings, systems};

mod common;
use common::throwaway_pool;

#[tokio::test]
async fn settings_get_and_set_round_trip() {
    let (pool, _dir) = throwaway_pool().await;

    assert_eq!(settings::get(&pool, "steamgriddbApiKey").await.unwrap(), None);

    settings::set(&pool, "steamgriddbApiKey", "abc123").await.unwrap();
    assert_eq!(
        settings::get(&pool, "steamgriddbApiKey").await.unwrap(),
        Some("abc123".to_string())
    );

    // set() on an existing key overwrites rather than erroring.
    settings::set(&pool, "steamgriddbApiKey", "def456").await.unwrap();
    assert_eq!(
        settings::get(&pool, "steamgriddbApiKey").await.unwrap(),
        Some("def456".to_string())
    );
}

#[tokio::test]
async fn systems_crud_round_trip() {
    let (pool, _dir) = throwaway_pool().await;

    let created = systems::create(
        &pool,
        systems::NewSystem {
            id: "nes".into(),
            name: "Nintendo Entertainment System".into(),
            extensions: r#"["nes"]"#.into(),
            retroarch_core: Some("mesen".into()),
            standalone_binary: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(created.id, "nes");

    let fetched = systems::get(&pool, "nes").await.unwrap().unwrap();
    assert_eq!(fetched.name, "Nintendo Entertainment System");

    let updated = systems::update(
        &pool,
        "nes",
        systems::NewSystem {
            id: "nes".into(),
            name: "NES".into(),
            extensions: r#"["nes"]"#.into(),
            retroarch_core: Some("mesen".into()),
            standalone_binary: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "NES");

    let all = systems::list(&pool).await.unwrap();
    assert_eq!(all.len(), 1);

    systems::delete(&pool, "nes").await.unwrap();
    assert!(systems::get(&pool, "nes").await.unwrap().is_none());
}

#[tokio::test]
async fn games_relational_chain() {
    let (pool, _dir) = throwaway_pool().await;

    systems::create(
        &pool,
        systems::NewSystem {
            id: "snes".into(),
            name: "Super Nintendo".into(),
            extensions: r#"["sfc","smc"]"#.into(),
            retroarch_core: Some("snes9x".into()),
            standalone_binary: None,
        },
    )
    .await
    .unwrap();

    let rom = roms::create(
        &pool,
        roms::NewRom {
            system_id: "snes".into(),
            path: "snes/Chrono Trigger.sfc".into(),
            crc32: Some("deadbeef".into()),
            size_bytes: Some(4_194_304),
            discs: None,
        },
    )
    .await
    .unwrap();

    let game = games::create(
        &pool,
        games::NewGame { rom_id: rom.id.clone(), title: "Chrono Trigger".into() },
    )
    .await
    .unwrap();
    assert_eq!(game.rom_id, rom.id);
    assert_eq!(game.scanned_title.as_deref(), Some("Chrono Trigger"));

    let media = game_media::create(
        &pool,
        game_media::NewGameMedia {
            game_id: game.id.clone(),
            kind: "boxart".into(),
            local_path: "media/chrono-trigger.png".into(),
            source_url: Some("https://example.com/chrono-trigger.png".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(media.game_id, game.id);

    let media_for_game = game_media::list_for_game(&pool, &game.id).await.unwrap();
    assert_eq!(media_for_game.len(), 1);

    let fetched_media = game_media::get(&pool, &media.id).await.unwrap().unwrap();
    assert_eq!(fetched_media.local_path, "media/chrono-trigger.png");

    let renamed = games::update(&pool, &game.id, "Chrono Trigger (Renamed)").await.unwrap();
    assert_eq!(renamed.title, "Chrono Trigger (Renamed)");

    // Deleting the media directly should leave the game untouched.
    game_media::delete(&pool, &media.id).await.unwrap();
    assert!(game_media::get(&pool, &media.id).await.unwrap().is_none());
    assert!(games::get(&pool, &game.id).await.unwrap().is_some());

    games::delete(&pool, &game.id).await.unwrap();
    assert!(games::get(&pool, &game.id).await.unwrap().is_none());
}

#[tokio::test]
async fn roms_crud_round_trip() {
    let (pool, _dir) = throwaway_pool().await;

    systems::create(
        &pool,
        systems::NewSystem {
            id: "genesis".into(),
            name: "Sega Genesis".into(),
            extensions: r#"["md","bin"]"#.into(),
            retroarch_core: Some("genesis_plus_gx".into()),
            standalone_binary: None,
        },
    )
    .await
    .unwrap();

    let rom = roms::create(
        &pool,
        roms::NewRom {
            system_id: "genesis".into(),
            path: "genesis/Sonic.md".into(),
            crc32: None,
            size_bytes: None,
            discs: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(rom.status, "ok");

    let updated = roms::update(
        &pool,
        &rom.id,
        roms::NewRom {
            system_id: "genesis".into(),
            path: "genesis/Sonic.md".into(),
            crc32: Some("cafef00d".into()),
            size_bytes: Some(524_288),
            discs: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.crc32.as_deref(), Some("cafef00d"));

    let all = roms::list(&pool).await.unwrap();
    assert_eq!(all.len(), 1);

    roms::delete(&pool, &rom.id).await.unwrap();
    assert!(roms::get(&pool, &rom.id).await.unwrap().is_none());
}

#[tokio::test]
async fn profiles_and_ra_stats_round_trip() {
    let (pool, _dir) = throwaway_pool().await;

    let profile = profiles::create(&pool, "George").await.unwrap();
    assert_eq!(profile.name, "George");
    assert!(profile.ra_username.is_none());

    let renamed = profiles::rename(&pool, &profile.id, "George B").await.unwrap();
    assert_eq!(renamed.name, "George B");

    assert!(ra_stats::get(&pool, &profile.id).await.unwrap().is_none());

    let stats = ra_stats::upsert(
        &pool,
        &profile.id,
        ra_stats::NewRaStats {
            points: 1234,
            rank: "1,234".into(),
            recent_unlocks_json: "[]".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(stats.points, 1234);

    // Upsert overwrites wholesale, as it's a pure cache.
    let refreshed = ra_stats::upsert(
        &pool,
        &profile.id,
        ra_stats::NewRaStats {
            points: 5678,
            rank: "5,678".into(),
            recent_unlocks_json: "[]".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(refreshed.points, 5678);

    // profiles::delete also removes ra_stats (no FK cascade is configured).
    profiles::delete(&pool, &profile.id).await.unwrap();
    assert!(profiles::get(&pool, &profile.id).await.unwrap().is_none());
    assert!(ra_stats::get(&pool, &profile.id).await.unwrap().is_none());
}
