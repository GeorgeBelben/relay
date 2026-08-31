use serde::Serialize;

use crate::systems::{self, SystemDef};

/// Wire type mirroring `systems::SystemDef` -- Tauri's IPC needs an owned, `Serialize` value, not
/// the `'static` borrowed catalog entries themselves.
#[derive(Debug, Serialize)]
pub struct System {
    pub id: String,
    pub name: String,
    pub extensions: Vec<String>,
    pub retroarch_core: Option<String>,
    pub standalone_binary: Option<String>,
}

impl From<&SystemDef> for System {
    fn from(def: &SystemDef) -> Self {
        System {
            id: def.id.to_string(),
            name: def.name.to_string(),
            extensions: def.extensions.iter().map(|e| e.to_string()).collect(),
            retroarch_core: def.retroarch_core.map(str::to_string),
            standalone_binary: def.standalone_binary.map(str::to_string),
        }
    }
}

#[tauri::command]
pub fn list_systems() -> Vec<System> {
    systems::ALL.iter().map(System::from).collect()
}

#[tauri::command]
pub fn get_system(id: String) -> Option<System> {
    systems::get(&id).map(System::from)
}
