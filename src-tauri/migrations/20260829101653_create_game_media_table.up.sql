CREATE TABLE game_media (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id),
    kind TEXT NOT NULL,
    local_path TEXT NOT NULL,
    source_url TEXT,
    created_at INTEGER NOT NULL,
    UNIQUE (game_id, kind)
);
