CREATE TABLE systems (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    extensions TEXT NOT NULL,
    retroarch_core TEXT,
    standalone_binary TEXT
);
