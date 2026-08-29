-- Supersedes the Phase 1 thin-slice placeholder (title/file_path only) with the real,
-- relational schema ported from the Electron MVP's Drizzle schema.
DROP TABLE games;

CREATE TABLE games (
    id TEXT PRIMARY KEY NOT NULL,
    rom_id TEXT NOT NULL UNIQUE REFERENCES roms(id),
    title TEXT NOT NULL,
    scanned_title TEXT,
    steamgriddb_id INTEGER,
    match_confidence REAL,
    enriched_at INTEGER,
    retroachievements_game_id INTEGER,
    retroachievements_matched_at INTEGER,
    ra_highest_award_kind TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
