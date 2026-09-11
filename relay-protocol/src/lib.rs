use serde::{Deserialize, Serialize};

/// Sent from a client (the UI or CLI) to the daemon.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Ping,
    LaunchGame { rom_path: String },
    GetState,
    GetLibrary,
}

/// Sent from the daemon back to a client.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Reply to `Request::Ping`.
    Pong,
    Error {
        message: String,
    },
    /// A game was successfully launched.
    GameLaunched {
        pid: u32,
    },
    /// A previously-launched game process exited.
    GameExited {
        exit_code: Option<i32>,
    },
    /// Reply to `Request::GetState`.
    State(DaemonState),
    Library(Vec<LibraryEntry>),
}

/// A snapshot of what the daemon currently knows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub running_game: Option<RunningGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningGame {
    pub pid: u32,
    pub rom_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub system: String,
    pub system_display_name: String,
    pub file_name: String,
    pub rom_path: String,
    pub size_bytes: u64,
}
