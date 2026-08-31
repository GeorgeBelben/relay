use crate::system::datetime::{self, DateTimeStatus};

const TIMEDATECTL_BIN: &str = "timedatectl";

#[tauri::command]
pub async fn get_datetime_status() -> Result<DateTimeStatus, String> {
    datetime::get_status(TIMEDATECTL_BIN).await
}

#[tauri::command]
pub async fn list_timezones() -> Result<Vec<String>, String> {
    datetime::list_timezones(TIMEDATECTL_BIN).await
}

#[tauri::command]
pub async fn set_timezone(timezone: String) -> Result<(), String> {
    datetime::set_timezone(TIMEDATECTL_BIN, &timezone).await
}

#[tauri::command]
pub async fn set_ntp_enabled(enabled: bool) -> Result<(), String> {
    datetime::set_ntp_enabled(TIMEDATECTL_BIN, enabled).await
}

#[tauri::command]
pub async fn set_time(date_time: String) -> Result<(), String> {
    datetime::set_time(TIMEDATECTL_BIN, &date_time).await
}
