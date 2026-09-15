use std::collections::HashMap;

use rusqlite::Connection;

const GLOBAL_SCOPE: &str = "global";

/// A resolved view of settings for one launch: system-scoped values
/// layered over global ones, already merged into a single lookup.
pub struct Settings {
    values: HashMap<String, String>,
    profile_id: Option<i64>,
}

impl Settings {
    /// Starts with global values, then overwrites with any system-scoped
    /// ones - so a system override always wins, and anything not
    /// overridden falls back to the global value.
    pub fn resolve(conn: &Connection, system_id: &str, profile_id: Option<i64>) -> Settings {
        let mut values: HashMap<String, String> =
            crate::db::get_settings_for_scope(conn, GLOBAL_SCOPE)
                .into_iter()
                .collect();

        let system_scope = scope_for(Some(system_id));
        for (key, value) in crate::db::get_settings_for_scope(conn, &system_scope) {
            values.insert(key, value);
        }

        Settings { values, profile_id }
    }

    pub fn profile_id(&self) -> Option<i64> {
        self.profile_id
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_str(key)?.parse().ok()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_str(key)?.parse().ok()
    }
}

pub fn scope_for(system: Option<&str>) -> String {
    match system {
        Some(id) => format!("system:{id}"),
        None => GLOBAL_SCOPE.to_string(),
    }
}
