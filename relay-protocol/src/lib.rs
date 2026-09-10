use serde::{Deserialize, Serialize};

/// Sent from a client (the UI or CLI) to the daemon.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Simple liveness check.
    Ping,
    /// Ask the daemon to launch a game.
    LaunchGame { rom_path: String },
    /// Ask the daemon for a full snapshot of current state.
    GetState,
}

/// Sent from the daemon back to a client.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Reply to `Request::Ping`.
    Pong,
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
    Error {
        message: String,
    },
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
