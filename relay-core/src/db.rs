use relay_protocol::LibraryEntry;
use rusqlite::{Connection, params};

use crate::library::ScannedRom;

pub struct Profile {
    pub id: i64,
    pub name: String,
    pub avatar_seed: String,
    pub ra_username: Option<String>,
    pub ra_web_api_key: Option<String>,
}

// Opens (creating if needed) the sqlite database
pub fn open() -> Connection {
    let path = crate::startup::data_dir().join("relay-core.db");
    let conn = Connection::open(&path).expect("failed to open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS library_entries (
            id INTEGER PRIMARY KEY,
            system TEXT NOT NULL,
            system_display_name TEXT NOT NULL,
            file_name TEXT NOT NULL,
            rom_path TEXT NOT NULL UNIQUE,
            size_bytes INTEGER NOT NULL,
            play_time_seconds INTEGER NOT NULL DEFAULT 0,
            last_played_at INTEGER
        )",
        [],
    )
    .expect("failed to create library_entries table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
        scope TEXT NOT NULL,
        key TEXT NOT NULL,
        value TEXT NOT NULL,
        PRIMARY KEY (scope, key)
    )",
        [],
    )
    .expect("failed to create settings table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS profiles (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        avatar_seed TEXT NOT NULL,
        ra_username TEXT,
        ra_web_api_key TEXT,
        created_at INTEGER NOT NULL
    )",
        [],
    )
    .expect("failed to create profiles table");

    conn
}

/// Inserts any scanned entries not already in the database. Existing
/// rows (matched by rom_path) are left untouched — this is what
/// preserves play_time_seconds/last_played_at across rescans.
pub fn upsert_scanned(conn: &Connection, scanned: &[ScannedRom]) {
    for entry in scanned {
        conn.execute(
            "INSERT INTO library_entries
                (system, system_display_name, file_name, rom_path, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(rom_path) DO NOTHING",
            params![
                entry.system,
                entry.system_display_name,
                entry.file_name,
                entry.rom_path,
                entry.size_bytes as i64,
            ],
        )
        .expect("failed to upsert library entry");
    }
}

/// Returns everything currently in the database, in the shape the
/// protocol expects.
pub fn get_all(conn: &Connection) -> Vec<LibraryEntry> {
    let mut stmt = conn
        .prepare(
            "SELECT id, system, system_display_name, file_name, rom_path,
                    size_bytes, play_time_seconds, last_played_at
             FROM library_entries
             ORDER BY system, file_name",
        )
        .expect("failed to prepare query");

    let rows = stmt
        .query_map([], |row| {
            Ok(LibraryEntry {
                id: row.get(0)?,
                system: row.get(1)?,
                system_display_name: row.get(2)?,
                file_name: row.get(3)?,
                rom_path: row.get(4)?,
                size_bytes: row.get::<_, i64>(5)? as u64,
                play_time_seconds: row.get::<_, i64>(6)? as u64,
                last_played_at: row.get(7)?,
            })
        })
        .expect("failed to run query");

    rows.filter_map(|r| r.ok()).collect()
}

pub fn get_by_id(conn: &Connection, id: i64) -> Option<LibraryEntry> {
    conn.query_row(
        "SELECT id, system, system_display_name, file_name, rom_path,
                size_bytes, play_time_seconds, last_played_at
         FROM library_entries
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(LibraryEntry {
                id: row.get(0)?,
                system: row.get(1)?,
                system_display_name: row.get(2)?,
                file_name: row.get(3)?,
                rom_path: row.get(4)?,
                size_bytes: row.get::<_, i64>(5)? as u64,
                play_time_seconds: row.get::<_, i64>(6)? as u64,
                last_played_at: row.get(7)?,
            })
        },
    )
    .ok()
}

/// All key/value pairs stored for one scope ("global" or "system:<id>").
/// Doesn't merge/resolve anything - that's settings.rs's job, this is
/// just the raw read.
pub fn get_settings_for_scope(conn: &Connection, scope: &str) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings WHERE scope = ?1")
        .expect("failed to prepare query");

    let rows = stmt
        .query_map(params![scope], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("failed to run query");

    rows.filter_map(|r| r.ok()).collect()
}

/// Sets a single key, overwriting any existing value for that scope+key.
pub fn set_setting(conn: &Connection, scope: &str, key: &str, value: &str) {
    conn.execute(
        "INSERT INTO settings (scope, key, value)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(scope, key) DO UPDATE SET value = excluded.value",
        params![scope, key, value],
    )
    .expect("failed to set setting");
}

/// Adds to a game's accumulated play time and updates when it was last
/// played. Called once, when a launched game's process actually exits.
pub fn record_play_session(conn: &Connection, game_id: i64, played_seconds: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "UPDATE library_entries
        SET play_time_seconds = play_time_seconds + ?1,
            last_played_at = ?2
        WHERE id = ?3",
        params![played_seconds as i64, now, game_id],
    )
    .expect("failed to record play session");
}

/// Creates a profile with a freshly-generated avatar seed, returning its id.
pub fn create_profile(conn: &Connection, name: &str) -> i64 {
    let avatar_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO profiles (name, avatar_seed, created_at) VALUES (?1, ?2, ?3)",
        params![name, avatar_seed, created_at],
    )
    .expect("failed to create profile");

    conn.last_insert_rowid()
}

pub fn list_profiles(conn: &Connection) -> Vec<Profile> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, avatar_seed, ra_username, ra_web_api_key FROM profiles ORDER BY id",
        )
        .expect("failed to prepare query");

    let rows = stmt
        .query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar_seed: row.get(2)?,
                ra_username: row.get(3)?,
                ra_web_api_key: row.get(4)?,
            })
        })
        .expect("failed to run query");

    rows.filter_map(|r| r.ok()).collect()
}

pub fn get_profile(conn: &Connection, id: i64) -> Option<Profile> {
    conn.query_row(
        "SELECT id, name, avatar_seed, ra_username, ra_web_api_key FROM profiles WHERE id = ?1",
        params![id],
        |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar_seed: row.get(2)?,
                ra_username: row.get(3)?,
                ra_web_api_key: row.get(4)?,
            })
        },
    )
    .ok()
}

pub fn link_retroachievements(
    conn: &Connection,
    profile_id: i64,
    username: &str,
    web_api_key: &str,
) {
    conn.execute(
        "UPDATE profiles SET ra_username = ?1, ra_web_api_key = ?2 WHERE id = ?3",
        params![username, web_api_key, profile_id],
    )
    .expect("failed to link retroachievements");
}
