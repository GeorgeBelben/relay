use crate::system::bluetooth::{self, BluetoothDevice, BluetoothPairError};

const BLUETOOTHCTL_BIN: &str = "bluetoothctl";

#[tauri::command]
pub async fn scan_for_bluetooth_devices() -> Result<Vec<BluetoothDevice>, String> {
    bluetooth::scan_for_devices(BLUETOOTHCTL_BIN).await
}

#[tauri::command]
pub async fn list_paired_bluetooth_devices() -> Result<Vec<BluetoothDevice>, String> {
    bluetooth::list_paired_devices(BLUETOOTHCTL_BIN).await
}

#[tauri::command]
pub async fn pair_bluetooth_device(address: String) -> Result<(), BluetoothPairError> {
    bluetooth::pair_device(BLUETOOTHCTL_BIN, &address).await
}

#[tauri::command]
pub async fn remove_bluetooth_device(address: String) -> Result<(), String> {
    bluetooth::remove_device(BLUETOOTHCTL_BIN, &address).await
}
