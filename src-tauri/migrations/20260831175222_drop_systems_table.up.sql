-- Systems (NES, SNES, etc.) are now a fixed set defined in Rust code (src-tauri/src/systems.rs,
-- REL-134), not user-editable data -- there's no longer a reason for them to be DB rows.
DROP TABLE systems;

-- roms.system_id keeps storing one of the fixed catalog's ids, just without a DB-level foreign
-- key to a table that no longer exists (SQLite foreign_keys enforcement was never turned on
-- anyway -- see db/profiles.rs's comment on the same point).
DROP TABLE roms;
CREATE TABLE roms (
    id TEXT PRIMARY KEY NOT NULL,
    system_id TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    crc32 TEXT,
    size_bytes INTEGER,
    discs TEXT,
    status TEXT NOT NULL DEFAULT 'ok',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
