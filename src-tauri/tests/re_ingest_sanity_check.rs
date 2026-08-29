//! REL-85: run the real ingestion pipeline against the actual dev ROM set at `~/Relay/roms` and
//! sanity-check the output. This is NOT part of the normal test suite -- it depends on real local
//! data that doesn't exist on CI (or a fresh clone), so it's `#[ignore]`d. Run explicitly with:
//!
//!     cargo test --test re_ingest_sanity_check -- --ignored --nocapture
//!
//! Compares against the Electron MVP's own dev DB
//! (~/Library/Application Support/relay/app.dev.db as of 2026-08-29): 3 real files (2 GBA, 1 PSX)
//! across otherwise-empty system folders.

use relay_lib::db::{games, roms, systems};
use relay_lib::ingestion::{paths, pipeline};

mod common;
use common::throwaway_pool;

// Straight from the MVP's own `systems` table (app.dev.db), not reconstructed from source --
// this is what actually produced the DB rows being compared against.
const SYSTEM_SEED: &[(&str, &str, &[&str])] = &[
    ("dreamcast", "Dreamcast", &["cdi", "chd", "gdi"]),
    ("gamecube", "GameCube", &["iso", "gcm", "rvz"]),
    ("gamegear", "Sega Game Gear", &["gg"]),
    ("gb", "Game Boy", &["gb"]),
    ("gba", "Game Boy Advance", &["gba"]),
    ("gbc", "Game Boy Color", &["gbc"]),
    ("mame", "Arcade (MAME)", &["zip", "chd"]),
    ("mastersystem", "Sega Master System", &["sms"]),
    ("megadrive", "Sega Genesis / Mega Drive", &["md", "bin", "gen"]),
    ("n64", "Nintendo 64", &["n64", "z64", "v64"]),
    ("nds", "Nintendo DS", &["nds"]),
    ("nes", "NES", &["nes"]),
    ("ngpc", "Neo Geo Pocket / Color", &["ngp", "ngc"]),
    ("ps2", "PlayStation 2", &["iso", "chd"]),
    ("psp", "PSP", &["iso", "cso"]),
    ("psx", "PlayStation", &["cue", "chd", "pbp"]),
    ("saturn", "Sega Saturn", &["cue", "chd"]),
    ("snes", "SNES", &["sfc", "smc"]),
    ("wii", "Wii", &["iso", "rvz", "wbfs"]),
    ("wonderswan", "WonderSwan / Color", &["ws", "wsc"]),
    ("xbox", "Xbox", &["iso", "xiso"]),
];

#[tokio::test]
#[ignore]
async fn scan_and_probe_matches_mvp_output_for_the_real_dev_rom_set() {
    let (pool, _dir) = throwaway_pool().await;

    for (id, name, extensions) in SYSTEM_SEED {
        systems::create(
            &pool,
            systems::NewSystem {
                id: id.to_string(),
                name: name.to_string(),
                extensions: serde_json::to_string(extensions).unwrap(),
                retroarch_core: None,
                standalone_binary: None,
            },
        )
        .await
        .unwrap();
    }

    let roms_root = paths::roms_path();
    println!("scanning {}", roms_root.display());
    let found = pipeline::scan_and_probe(&pool, &roms_root).await.unwrap();
    println!("found {found} rom(s)");

    let mut all_roms = roms::list(&pool).await.unwrap();
    all_roms.sort_by(|a, b| a.path.cmp(&b.path));
    let mut all_games = games::list(&pool).await.unwrap();
    all_games.sort_by(|a, b| a.title.cmp(&b.title));

    println!("\n--- roms ---");
    for rom in &all_roms {
        println!("{} | crc32={:?} size={:?} status={}", rom.path, rom.crc32, rom.size_bytes, rom.status);
    }
    println!("\n--- games ---");
    for game in &all_games {
        println!("{} (scanned_title={:?})", game.title, game.scanned_title);
    }

    // MVP comparison (app.dev.db, 2026-08-29): 3 roms/games, these exact crc32/size/status.
    assert_eq!(all_roms.len(), 3, "expected 3 roms, matching the MVP's dev DB");
    assert_eq!(all_games.len(), 3, "expected 3 games, matching the MVP's dev DB");

    let gba1 = all_roms.iter().find(|r| r.path.contains("Super Mario Advance 4")).expect("gba rom 1");
    assert_eq!(gba1.crc32.as_deref(), Some("37141f32"));
    assert_eq!(gba1.size_bytes, Some(8_388_608));
    assert_eq!(gba1.status, "ok");

    let gba2 = all_roms.iter().find(|r| r.path.contains("Mario & Luigi")).expect("gba rom 2");
    assert_eq!(gba2.crc32.as_deref(), Some("e718d850"));
    assert_eq!(gba2.size_bytes, Some(16_777_216));

    let psx = all_roms.iter().find(|r| r.path.contains("Crash Bandicoot")).expect("psx rom");
    assert_eq!(psx.crc32.as_deref(), Some("7420271e"));
    assert_eq!(psx.size_bytes, Some(87)); // the .cue playlist text file itself, not the .bin

    // Known, expected divergence: the MVP's titles ("Super Mario Advance 4: Super Mario Bros. 3",
    // matchConfidence 1.0 in its `games` table) come from a No-Intro DAT CRC32 lookup (REL-38 in
    // the MVP) that hasn't been ported here -- no Linear issue in this rewrite currently covers
    // it. This port's titles are the plain filename-derived fallback the MVP itself falls back to
    // on a DAT miss, which is what's actually being asserted below.
    let titles: Vec<&str> = all_games.iter().map(|g| g.title.as_str()).collect();
    assert!(titles.contains(&"Crash Bandicoot"), "titles: {titles:?}");
    assert!(titles.iter().any(|t| t.contains("Super Mario Advance 4")), "titles: {titles:?}");
    assert!(titles.iter().any(|t| t.contains("Mario & Luigi")), "titles: {titles:?}");
}
