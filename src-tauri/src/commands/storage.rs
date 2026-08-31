use crate::ingestion::paths;
use crate::system::storage::{self, StorageUsage};

#[tauri::command]
pub async fn get_storage_usage() -> Result<StorageUsage, String> {
    storage::get_storage_usage(&paths::library_root()).await.map_err(crate::logging::err_to_string)
}
