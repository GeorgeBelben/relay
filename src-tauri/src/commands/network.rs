use crate::system::network::{self, WifiConnectError, WifiNetwork};

const NMCLI_BIN: &str = "nmcli";

#[tauri::command]
pub async fn list_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    network::list_wifi_networks(NMCLI_BIN).await
}

#[tauri::command]
pub async fn connect_to_wifi_network(ssid: String, password: Option<String>) -> Result<(), WifiConnectError> {
    network::connect_to_wifi_network(NMCLI_BIN, &ssid, password.as_deref()).await
}
