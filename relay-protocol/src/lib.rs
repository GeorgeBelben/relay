use serde::{Deserialize, Serialize};

/// Sent from a client (the UI or CLI) to the daemon.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Ping,
    LaunchGame {
        game_id: i64,
    },
    StopGame,
    GetState,
    GetLibrary,
    GetSettings {
        system: Option<String>,
    },
    SetSetting {
        system: Option<String>,
        key: String,
        value: String,
    },
    CreateProfile {
        name: String,
    },
    ListProfiles,
    SetActiveProfile {
        profile_id: i64,
    },
    GetActiveProfile,
    LinkRetroAchievements {
        username: String,
        web_api_key: String,
    },
    GetRaStats,
    GetControllers,
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
    /// A game was stopped by request (as opposed to exiting on its own).
    GameStopped,
    /// Reply to `Request::GetState`.
    State(DaemonState),
    Library(Vec<LibraryEntry>),
    Settings(Vec<(String, String)>),
    SettingSet,
    Profile(ProfileInfo),
    Profiles(Vec<ProfileInfo>),
    ActiveProfile(Option<ProfileInfo>),
    RaStats {
        points: i64,
        softcore_points: i64,
    },
    Controllers(Vec<ControllerInfo>),
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
    pub id: i64,
    pub system: String,
    pub system_display_name: String,
    pub file_name: String,
    pub rom_path: String,
    pub size_bytes: u64,
    pub play_time_seconds: u64,
    pub last_played_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub avatar_seed: String,
    pub ra_linked: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControllerType {
    Xbox,
    Playstation,
    Switch,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInfo {
    pub index: u32,
    pub name: String,
    pub controller_type: ControllerType,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}
