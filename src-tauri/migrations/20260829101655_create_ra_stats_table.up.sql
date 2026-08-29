CREATE TABLE ra_stats (
    profile_id TEXT PRIMARY KEY NOT NULL REFERENCES profiles(id),
    points INTEGER NOT NULL,
    rank TEXT NOT NULL,
    recent_unlocks_json TEXT NOT NULL,
    refreshed_at INTEGER NOT NULL
);
