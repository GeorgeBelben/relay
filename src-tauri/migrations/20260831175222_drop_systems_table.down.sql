CREATE TABLE systems (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    extensions TEXT NOT NULL,
    retroarch_core TEXT,
    standalone_binary TEXT
);

DROP TABLE roms;
CREATE TABLE roms (
    id TEXT PRIMARY KEY NOT NULL,
    system_id TEXT NOT NULL REFERENCES systems(id),
    path TEXT NOT NULL UNIQUE,
    crc32 TEXT,
    size_bytes INTEGER,
    discs TEXT,
    status TEXT NOT NULL DEFAULT 'ok',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
