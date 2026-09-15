use relay_protocol::LibraryEntry;
use rusqlite::{Connection, params};

use crate::library::ScannedRom;

// Opens (creating if needed) the sqlite database
pub fn open() -> Connection {
    let path = crate::startup::data_dir().join("relay.db");
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
